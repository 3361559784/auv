//! Two-source operation summary read path and join policy (API-P7).
//!
//! API-P3 showed `GetOperation` is a two-source projection:
//! - `OperationResult` (persisted) owns `operation_id`, `status`,
//!   `known_limits`, and evidence artifact refs.
//! - the `InvokeResult`-sourced summary (via `OperationSummarySource`) owns
//!   `output_summary`, `signals`, and `failure_message`.
//!
//! This module joins them explicitly. Per API-P4 (`GetOperation` flow), the
//! persisted record is the required skeleton and the runtime summary is layered
//! on when available. When the runtime summary is absent, the join records it as
//! `None` rather than fabricating empty strings as authoritative data (API-P4:
//! "It must not silently fabricate empty strings").

use std::collections::BTreeMap;

use auv_cli_invoke::{OperationSummarySource, RunStatus};
use auv_tracing_driver::store::LocalStore;

use crate::contract::{ArtifactRef, OperationResult, OperationStatus};
use crate::model::AuvResult;
use crate::run_read;

/// Outcome of loading and joining a `GetOperation` summary for one run.
#[derive(Clone, Debug, PartialEq)]
pub enum JoinedOperationSummaryLoad {
  /// Persisted skeleton found and joined with the supplied runtime source.
  Found(JoinedOperationSummary),
  /// The run directory does not exist in the store.
  RunNotFound,
  /// The run exists but recorded no `operation-result` JSON artifact.
  NoPersistedOperationResult,
}

/// The `InvokeResult`-sourced half of the summary projection, captured as owned
/// data for one operation.
#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeOperationSummary {
  pub output_summary: String,
  pub signals: BTreeMap<String, String>,
  pub failure_message: Option<String>,
}

impl RuntimeOperationSummary {
  fn from_source(source: &dyn OperationSummarySource) -> Self {
    Self {
      output_summary: source.output_summary().to_string(),
      signals: source.signals().clone(),
      failure_message: source.failure_message().map(str::to_string),
    }
  }
}

/// Explicit two-source join of a `GetOperation` summary view.
///
/// `runtime` is `None` when no `InvokeResult`-sourced summary was available for
/// the run (for example, not cached). Callers must treat that as "runtime
/// summary unknown", not as empty output.
#[derive(Clone, Debug, PartialEq)]
pub struct JoinedOperationSummary {
  // OperationResult-sourced (persisted, required skeleton).
  pub run_id: String,
  pub operation_id: String,
  pub status: OperationStatus,
  pub known_limits: Vec<String>,
  pub artifacts: Vec<ArtifactRef>,
  // InvokeResult-sourced (runtime return value, may be absent).
  pub runtime: Option<RuntimeOperationSummary>,
}

fn runtime_status_matches_persisted(persisted: OperationStatus, runtime: RunStatus) -> bool {
  matches!(
    (persisted, runtime),
    (OperationStatus::Completed, RunStatus::Completed)
      | (OperationStatus::Failed, RunStatus::Failed)
  )
}

/// Join a persisted `OperationResult` with an optional runtime summary source.
///
/// Pure join policy: the persisted record provides the required fields; the
/// runtime summary is layered on when present, otherwise `runtime` is `None`.
/// When both halves are present but disagree on completion status, a
/// `auv.api.session.runtime_status_mismatch` known_limit is appended so callers
/// do not silently return contradictory status and failure_message fields.
pub fn join_operation_summary(
  operation: &OperationResult,
  runtime: Option<&dyn OperationSummarySource>,
) -> JoinedOperationSummary {
  // NOTICE(api-p3-od2): `operation_id` here is the persisted domain label from
  // `OperationResult` (e.g. "music.search.results"), which is NOT the invoke
  // `command_id`. The proto `operation_id` mapping rule (command_id vs domain
  // label vs widened ref) is API-P3 open decision 2 / API-P4 gate 3 and is
  // resolved at the proto mapper (API-P8), not in this read path.
  let mut known_limits = operation.known_limits.clone();
  if let Some(source) = runtime {
    if !runtime_status_matches_persisted(operation.status, source.status()) {
      known_limits.push("auv.api.session.runtime_status_mismatch".to_string());
    }
  }
  JoinedOperationSummary {
    run_id: operation.run_id.as_str().to_string(),
    operation_id: operation.operation_id.clone(),
    status: operation.status,
    known_limits,
    artifacts: operation.evidence_artifacts.clone(),
    runtime: runtime.map(RuntimeOperationSummary::from_source),
  }
}

/// Load and join the `GetOperation` summary for a run.
///
/// Reads the persisted `OperationResult` (storage-side read path via
/// [`run_read::read_operation_result`]) and joins it with the runtime summary
/// source. When `runtime_override` is absent, falls back to the persisted
/// `operation-summary` artifact (API-P11). Distinguishes a missing run from a
/// run that exists but recorded no `OperationResult`.
pub fn load_joined_operation_summary(
  store: &LocalStore,
  run_id: &str,
  runtime_override: Option<&dyn OperationSummarySource>,
) -> AuvResult<JoinedOperationSummaryLoad> {
  let run_dir = store.run_dir(run_id)?;
  if !run_dir.join("run.json").exists() {
    return Ok(JoinedOperationSummaryLoad::RunNotFound);
  }
  let Some(operation) = run_read::read_operation_result(store, run_id)? else {
    return Ok(JoinedOperationSummaryLoad::NoPersistedOperationResult);
  };
  let stored_summary = if runtime_override.is_none() {
    run_read::read_operation_summary(store, run_id)?
  } else {
    None
  };
  let runtime: Option<&dyn OperationSummarySource> = match runtime_override {
    Some(source) => Some(source),
    None => stored_summary
      .as_ref()
      .map(|summary| summary as &dyn OperationSummarySource),
  };
  Ok(JoinedOperationSummaryLoad::Found(join_operation_summary(
    &operation, runtime,
  )))
}

#[cfg(test)]
mod tests {
  use std::collections::BTreeMap;
  use std::fs;

  use auv_cli_invoke::{
    InvokeResult, OperationSummary, OperationSummaryRecord, OperationSummarySource, RunStatus,
  };
  use auv_tracing_driver::store::LocalStore;
  use auv_tracing_driver::trace::SpanId;

  use super::{
    JoinedOperationSummary, JoinedOperationSummaryLoad, join_operation_summary,
    load_joined_operation_summary,
  };
  use crate::api::session_service::test_fixtures::{
    SessionRunFixture, music_runtime_summary, music_search_operation,
    persist_operation_result_and_summary_run, persist_operation_result_run, unique_temp_dir,
    write_minimal_run,
  };
  use crate::contract::{OPERATION_SUMMARY_API_VERSION, OperationStatus};

  fn sample_operation(run_id: &str) -> crate::contract::OperationResult {
    music_search_operation(run_id)
  }

  fn runtime_summary(run_id: &str) -> OperationSummary {
    music_runtime_summary(run_id)
  }

  #[test]
  fn join_includes_runtime_summary_when_present() {
    let operation = sample_operation("run-join");
    let summary = runtime_summary("run-join");

    let joined = join_operation_summary(&operation, Some(&summary));

    assert_eq!(joined.run_id, "run-join");
    assert_eq!(joined.operation_id, "music.search.results");
    assert_eq!(joined.status, OperationStatus::Completed);
    assert_eq!(joined.known_limits, vec!["semantic_shaping_synthetic"]);
    let runtime = joined.runtime.expect("runtime summary should be present");
    assert_eq!(runtime.output_summary, "did the thing");
    assert_eq!(
      runtime.signals.get("now_playing").map(String::as_str),
      Some("track-x")
    );
    assert_eq!(runtime.failure_message, None);
  }

  #[test]
  fn join_marks_runtime_absent_without_fabricating() {
    let operation = sample_operation("run-join-missing");

    let joined = join_operation_summary(&operation, None);

    // Persisted skeleton still present.
    assert_eq!(joined.operation_id, "music.search.results");
    assert_eq!(joined.known_limits, vec!["semantic_shaping_synthetic"]);
    // Runtime summary explicitly absent, not fabricated as empty strings.
    assert!(joined.runtime.is_none());
  }

  #[test]
  fn join_preserves_persisted_known_limits_and_status_on_failure() {
    let mut operation = sample_operation("run-join-failed");
    operation.status = OperationStatus::Failed;
    operation.known_limits = vec!["dispatch_failed".to_string()];

    let joined = join_operation_summary(&operation, None);

    assert_eq!(joined.status, OperationStatus::Failed);
    assert_eq!(joined.known_limits, vec!["dispatch_failed"]);
  }

  #[test]
  fn join_flags_runtime_status_mismatch() {
    let operation = sample_operation("run-mismatch");
    let summary = OperationSummary::capture(&InvokeResult {
      run_id: "run-mismatch".to_string(),
      producer_span_id: SpanId::new("0000000000000001"),
      status: RunStatus::Failed,
      output_summary: "runtime failed".to_string(),
      signals: BTreeMap::new(),
      artifacts: Vec::new(),
      artifact_paths: Vec::new(),
      failure_message: Some("boom".to_string()),
    });

    let joined = join_operation_summary(&operation, Some(&summary));

    assert_eq!(joined.status, OperationStatus::Completed);
    assert!(
      joined
        .known_limits
        .iter()
        .any(|limit| limit == "auv.api.session.runtime_status_mismatch")
    );
    let runtime = joined.runtime.expect("runtime summary should be present");
    assert_eq!(runtime.failure_message.as_deref(), Some("boom"));
  }

  #[test]
  fn load_joined_operation_summary_returns_run_not_found_for_missing_run() {
    let root = unique_temp_dir("session-summary-missing-run");
    let store = LocalStore::new(root.clone()).expect("store should initialize");

    let loaded =
      load_joined_operation_summary(&store, "missing-run", None).expect("load should succeed");
    assert_eq!(loaded, JoinedOperationSummaryLoad::RunNotFound);

    let _ = fs::remove_dir_all(root);
  }

  #[test]
  fn load_joined_operation_summary_returns_no_persisted_result_when_run_lacks_artifact() {
    let root = unique_temp_dir("session-summary-no-op-result");
    let store = LocalStore::new(root.clone()).expect("store should initialize");
    write_minimal_run(&store, "run-no-op-result");

    let loaded =
      load_joined_operation_summary(&store, "run-no-op-result", None).expect("load should succeed");
    assert_eq!(
      loaded,
      JoinedOperationSummaryLoad::NoPersistedOperationResult
    );

    let _ = fs::remove_dir_all(root);
  }

  #[test]
  fn load_joined_operation_summary_joins_persisted_and_runtime_halves() {
    let operation = sample_operation("run-happy");
    let SessionRunFixture { root, store } =
      persist_operation_result_run("session-summary-load", "run-happy", &operation);
    let summary = runtime_summary("run-happy");

    let loaded = load_joined_operation_summary(&store, "run-happy", Some(&summary))
      .expect("load should succeed");
    let JoinedOperationSummaryLoad::Found(joined) = loaded else {
      panic!("expected joined summary, got {loaded:?}");
    };

    assert_eq!(joined.run_id, "run-happy");
    assert_eq!(joined.operation_id, "music.search.results");
    assert_eq!(joined.status, OperationStatus::Completed);
    let runtime = joined.runtime.expect("runtime summary should be present");
    assert_eq!(runtime.output_summary, "did the thing");
    assert_eq!(
      runtime.signals.get("now_playing").map(String::as_str),
      Some("track-x")
    );

    let _ = fs::remove_dir_all(root);
  }

  #[test]
  fn load_joined_operation_summary_loads_persisted_runtime_when_cache_absent() {
    let operation = sample_operation("run-stored-runtime");
    let summary = runtime_summary("run-stored-runtime");
    let SessionRunFixture { root, store } = persist_operation_result_and_summary_run(
      "session-summary-stored-runtime",
      "run-stored-runtime",
      &operation,
      &summary,
    );

    let loaded = load_joined_operation_summary(&store, "run-stored-runtime", None)
      .expect("load should succeed");
    let JoinedOperationSummaryLoad::Found(joined) = loaded else {
      panic!("expected joined summary, got {loaded:?}");
    };

    let runtime = joined.runtime.expect("runtime summary should be present");
    assert_eq!(runtime.output_summary, "did the thing");
    assert_eq!(
      runtime.signals.get("now_playing").map(String::as_str),
      Some("track-x")
    );

    let _ = fs::remove_dir_all(root);
  }

  #[test]
  fn operation_summary_record_deserializes_without_api_version_field() {
    let json = r#"{
      "run_id": "run-legacy",
      "status": "completed",
      "output_summary": "legacy",
      "signals": {},
      "failure_message": null
    }"#;
    let record: OperationSummaryRecord = serde_json::from_str(json).expect("deserialize");
    assert_eq!(record.api_version, OPERATION_SUMMARY_API_VERSION);
    assert_eq!(record.run_id, "run-legacy");
    assert_eq!(record.output_summary, "legacy");
    let restored = OperationSummary::from_record(record);
    assert_eq!(restored.output_summary(), "legacy");
  }

  fn _assert_send_sync<T: Send + Sync>() {}
  #[test]
  fn joined_summary_is_send_sync() {
    _assert_send_sync::<JoinedOperationSummary>();
  }
}
