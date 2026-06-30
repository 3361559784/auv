use std::time::{SystemTime, UNIX_EPOCH};

use auv_view::memory::{
  ARTIFACT_DIR_BRIDGE_RUN_ID, MemoryReadConfig, MemoryWriteInput, ReacquireConfig,
  ReacquireDriverAdapter, ReacquireOutcome, ReacquireTarget, StaleReason, ViewMemory,
  ViewMemoryScopeSnapshot, memory_file_path, outcome_label, parse_memory_file, reacquire,
  strategy_name, try_build_memory, write_memory_file,
};
use auv_view::{VIEW_IR_SCHEMA_VERSION, ViewBounds};
use serde::{Deserialize, Serialize};

use crate::{PlaylistSelectTarget, PlaylistSidebarScan};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PlaylistReacquireSummary {
  pub outcome: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub strategy_used: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub stale_reason: Option<String>,
  pub observation_count: usize,
  pub skipped_rescan_replay: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub enum PlaylistReacquireAttempt {
  Hit {
    bounds: ViewBounds,
    summary: PlaylistReacquireSummary,
  },
  Stale {
    summary: PlaylistReacquireSummary,
  },
  Miss {
    summary: PlaylistReacquireSummary,
  },
}

pub const PLAYLIST_SIDEBAR_SCOPE_ID: &str = "playlist_sidebar";
pub const PLAYLIST_SCAN_CACHE_FILE_NAME: &str = "playlist-scan-cache.json";

pub fn enabled() -> bool {
  enabled_with_env(std::env::var("AUV_NETEASE_VIEW_MEMORY").ok().as_deref())
}

pub(crate) fn enabled_with_env(value: Option<&str>) -> bool {
  matches!(value, Some("1"))
}

pub fn system_time_millis() -> u64 {
  SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .map(|duration| duration.as_millis() as u64)
    .unwrap_or(0)
}

pub fn write_from_scan(inputs: &crate::Inputs, scan: &PlaylistSidebarScan) -> Result<(), String> {
  if !enabled() {
    return Ok(());
  }

  let reconstruction = scan.reconstruction();
  let sidebar_bounds = scan
    .sidebar_region()
    .bounds
    .unwrap_or_else(|| ViewBounds::new(0.0, 0.0, 240.0, 400.0));
  let baseline_width = sidebar_bounds.width.round().max(1.0) as u32;
  let memory = try_build_memory(
    MemoryWriteInput {
      app_bundle_id: &inputs.app_id,
      scope_id: PLAYLIST_SIDEBAR_SCOPE_ID,
      root: &reconstruction.root,
      scope_snapshot: ViewMemoryScopeSnapshot {
        region_id: PLAYLIST_SIDEBAR_SCOPE_ID.to_string(),
        region_bounds_window_local: sidebar_bounds,
        baseline_width,
        schema_version_view_ir: VIEW_IR_SCHEMA_VERSION.to_string(),
      },
      source_reconstruction_ref: PLAYLIST_SCAN_CACHE_FILE_NAME.to_string(),
      source_run_id: ARTIFACT_DIR_BRIDGE_RUN_ID.to_string(),
      last_reconstructed_at_millis: system_time_millis(),
      clean: scan.diagnostics().is_empty(),
    },
    reconstruction,
  )
  .ok_or_else(|| "scan did not produce writable ViewMemory".to_string())?;

  let path = memory_file_path(&inputs.artifact_dir, PLAYLIST_SIDEBAR_SCOPE_ID);
  write_memory_file(&path, &memory)
}

pub fn load_memory_raw(inputs: &crate::Inputs) -> Option<ViewMemory> {
  if !enabled() {
    return None;
  }
  let path = memory_file_path(&inputs.artifact_dir, PLAYLIST_SIDEBAR_SCOPE_ID);
  parse_memory_file(&path)
}

pub fn try_reacquire_playlist_target(
  memory: &ViewMemory,
  target: &PlaylistSelectTarget,
  adapter: &mut dyn ReacquireDriverAdapter,
  read_config: &MemoryReadConfig,
  current_baseline_width: Option<u32>,
) -> PlaylistReacquireAttempt {
  let reacquire_target = ReacquireTarget::LabelWithSection {
    label: target.label.clone(),
    section_hint: Some(target.section_kind.domain_kind().to_string()),
  };
  let outcome = reacquire(
    memory,
    reacquire_target,
    adapter,
    &ReacquireConfig {
      max_scroll_attempts: 5,
      memory_read: Some(read_config.clone()),
      current_baseline_width,
    },
  );
  summary_from_outcome(outcome)
}

fn summary_from_outcome(outcome: ReacquireOutcome) -> PlaylistReacquireAttempt {
  let outcome_label_str = outcome_label(&outcome).to_string();
  match outcome {
    ReacquireOutcome::Reacquired {
      node,
      strategy_used,
      observation_count,
      ..
    } => PlaylistReacquireAttempt::Hit {
      bounds: node.bounds,
      summary: PlaylistReacquireSummary {
        outcome: outcome_label_str,
        strategy_used: Some(strategy_name(strategy_used).to_string()),
        stale_reason: None,
        observation_count,
        skipped_rescan_replay: true,
      },
    },
    ReacquireOutcome::Stale {
      reason,
      observation_count,
      ..
    } => PlaylistReacquireAttempt::Stale {
      summary: PlaylistReacquireSummary {
        outcome: outcome_label_str,
        strategy_used: None,
        stale_reason: Some(stale_reason_wire(reason).to_string()),
        observation_count,
        skipped_rescan_replay: false,
      },
    },
    ReacquireOutcome::NotFound {
      observation_count, ..
    } => PlaylistReacquireAttempt::Miss {
      summary: PlaylistReacquireSummary {
        outcome: outcome_label_str,
        strategy_used: None,
        stale_reason: None,
        observation_count,
        skipped_rescan_replay: false,
      },
    },
  }
}

fn stale_reason_wire(reason: StaleReason) -> &'static str {
  match reason {
    StaleReason::MemoryRejectedAtFreshness => "memory_rejected_at_freshness",
    StaleReason::SchemaMismatch => "schema_mismatch",
    StaleReason::BaselineMismatchHard => "baseline_mismatch_hard",
    StaleReason::RegionGoneAtReacquisition => "region_gone_at_reacquisition",
    StaleReason::ObservationFailedAtReacquisition => "observation_failed_at_reacquisition",
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::SidebarSectionKind;
  use auv_view::memory::{
    ReacquireCandidate, ReacquireObservation, VIEW_MEMORY_SCHEMA_VERSION, ViewMemoryScopeSnapshot,
  };
  use auv_view::{ParserDiagnostic, ViewBounds};

  struct FakeAdapter {
    observations: Vec<ReacquireObservation>,
    cursor: usize,
  }

  impl ReacquireDriverAdapter for FakeAdapter {
    fn observe_viewport(&mut self) -> Result<ReacquireObservation, ParserDiagnostic> {
      self
        .observations
        .get(self.cursor)
        .cloned()
        .map(|observation| {
          self.cursor += 1;
          observation
        })
        .ok_or_else(|| ParserDiagnostic {
          code: "no_observation".into(),
          message: "fake adapter exhausted".into(),
          node_id: None,
        })
    }

    fn scroll_down(&mut self) -> Result<(), ParserDiagnostic> {
      Ok(())
    }

    fn scroll_up(&mut self) -> Result<(), ParserDiagnostic> {
      Ok(())
    }
  }

  fn sample_memory() -> ViewMemory {
    ViewMemory {
      schema_version: VIEW_MEMORY_SCHEMA_VERSION.to_string(),
      memory_id: "com.netease.163music:playlist_sidebar".into(),
      app_bundle_id: "com.netease.163music".into(),
      scope_id: PLAYLIST_SIDEBAR_SCOPE_ID.into(),
      last_reconstructed_at_millis: 1_719_744_000_000,
      source_run_id: ARTIFACT_DIR_BRIDGE_RUN_ID.into(),
      source_reconstruction_ref: PLAYLIST_SCAN_CACHE_FILE_NAME.into(),
      anchors: Vec::new(),
      landmarks: Vec::new(),
      node_snapshots: Default::default(),
      scope_snapshot: ViewMemoryScopeSnapshot {
        region_id: PLAYLIST_SIDEBAR_SCOPE_ID.into(),
        region_bounds_window_local: ViewBounds::new(0.0, 220.0, 346.0, 720.0),
        baseline_width: 346,
        schema_version_view_ir: VIEW_IR_SCHEMA_VERSION.to_string(),
      },
      diagnostics: Vec::new(),
    }
  }

  fn road_trip_target() -> PlaylistSelectTarget {
    PlaylistSelectTarget {
      label: "Road Trip".into(),
      section_id: "section.favorite_playlists".into(),
      section_kind: SidebarSectionKind::FavoritePlaylists,
      item_id: "item.road-trip".into(),
      anchor_id: None,
      candidate_id: Some("item.road-trip".into()),
      observation_index: Some(0),
      bounds: Some(ViewBounds::new(32.0, 106.0, 120.0, 20.0)),
    }
  }

  #[test]
  fn enabled_with_env_requires_exact_value() {
    assert!(!enabled_with_env(None));
    assert!(!enabled_with_env(Some("0")));
    assert!(!enabled_with_env(Some("true")));
    assert!(enabled_with_env(Some("1")));
  }

  #[test]
  fn playlist_select_uses_reacquire_when_memory_hit() {
    let memory = sample_memory();
    let target = road_trip_target();
    let mut adapter = FakeAdapter {
      observations: vec![ReacquireObservation {
        fingerprint: "favorite".into(),
        candidates: vec![ReacquireCandidate {
          node_id: Some("item.road-trip".into()),
          label: "Road Trip".into(),
          section_hint: Some("netease.favorite_playlists".into()),
          bounds: ViewBounds::new(32.0, 106.0, 120.0, 20.0),
        }],
      }],
      cursor: 0,
    };

    let attempt = try_reacquire_playlist_target(
      &memory,
      &target,
      &mut adapter,
      &MemoryReadConfig {
        now_millis: memory.last_reconstructed_at_millis,
        ..Default::default()
      },
      Some(memory.scope_snapshot.baseline_width),
    );

    match attempt {
      PlaylistReacquireAttempt::Hit { summary, .. } => {
        assert!(summary.skipped_rescan_replay);
        assert_eq!(
          summary.strategy_used.as_deref(),
          Some("label_current_viewport")
        );
      }
      other => panic!("expected reacquire hit, got {other:?}"),
    }
  }

  #[test]
  fn playlist_select_falls_back_on_stale_memory() {
    let mut memory = sample_memory();
    memory.last_reconstructed_at_millis = 1_000;
    let target = road_trip_target();
    let mut adapter = FakeAdapter {
      observations: vec![],
      cursor: 0,
    };

    let attempt = try_reacquire_playlist_target(
      &memory,
      &target,
      &mut adapter,
      &MemoryReadConfig {
        now_millis: 1_000 + auv_view::memory::DEFAULT_MEMORY_TTL_MILLIS + 1,
        ..Default::default()
      },
      Some(memory.scope_snapshot.baseline_width),
    );

    match attempt {
      PlaylistReacquireAttempt::Stale { summary } => {
        assert!(!summary.skipped_rescan_replay);
        assert_eq!(
          summary.stale_reason.as_deref(),
          Some("memory_rejected_at_freshness")
        );
      }
      other => panic!("expected stale memory, got {other:?}"),
    }
  }
}
