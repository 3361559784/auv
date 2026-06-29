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
/// [`run_read::read_operation_result`]) and joins it with the supplied runtime
/// summary source. Distinguishes a missing run from a run that exists but
/// recorded no `OperationResult`.
pub fn load_joined_operation_summary(
  store: &LocalStore,
  run_id: &str,
  runtime: Option<&dyn OperationSummarySource>,
) -> AuvResult<JoinedOperationSummaryLoad> {
  let run_dir = store.run_dir(run_id)?;
  if !run_dir.join("run.json").exists() {
    return Ok(JoinedOperationSummaryLoad::RunNotFound);
  }
  let Some(operation) = run_read::read_operation_result(store, run_id)? else {
    return Ok(JoinedOperationSummaryLoad::NoPersistedOperationResult);
  };
  Ok(JoinedOperationSummaryLoad::Found(join_operation_summary(
    &operation, runtime,
  )))
}

#[cfg(test)]
mod tests {
  use std::collections::BTreeMap;
  use std::fs;
  use std::path::{Path, PathBuf};

  use auv_cli_invoke::{InvokeResult, OperationSummary, RunStatus};
  use auv_tracing_driver::artifact::ArtifactFileSource;
  use auv_tracing_driver::store::{CanonicalRun, LocalStore};
  use auv_tracing_driver::trace::{
    RUN_API_VERSION, RunId, RunRecordV1Alpha1, RunType, SPAN_API_VERSION, SpanId,
    SpanRecordV1Alpha1, TraceId, TraceState, TraceStatusCode,
  };
  use serde::Serialize;

  use super::{
    JoinedOperationSummary, JoinedOperationSummaryLoad, join_operation_summary,
    load_joined_operation_summary,
  };
  use crate::contract::{
    OPERATION_RESULT_API_VERSION, OperationOutput, OperationResult, OperationStatus,
  };

  fn sample_operation(run_id: &str) -> OperationResult {
    OperationResult {
      api_version: OPERATION_RESULT_API_VERSION.to_string(),
      run_id: RunId::new(run_id),
      status: OperationStatus::Completed,
      operation_id: "music.search.results".to_string(),
      evidence_artifacts: Vec::new(),
      output: OperationOutput::Acknowledged { message: None },
      verifications: Vec::new(),
      freshness_basis: None,
      known_limits: vec!["semantic_shaping_synthetic".to_string()],
    }
  }

  fn runtime_summary(run_id: &str) -> OperationSummary {
    let mut signals = BTreeMap::new();
    signals.insert("now_playing".to_string(), "track-x".to_string());
    OperationSummary::capture(&InvokeResult {
      run_id: run_id.to_string(),
      producer_span_id: SpanId::new("0000000000000001"),
      status: RunStatus::Completed,
      output_summary: "did the thing".to_string(),
      signals,
      artifacts: Vec::new(),
      artifact_paths: Vec::new(),
      failure_message: None,
    })
  }

  fn temp_dir(label: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!("auv-{label}-{}", crate::model::now_millis()));
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).expect("temp dir should be creatable");
    path
  }

  fn dummy_run(run_id: &str) -> RunRecordV1Alpha1 {
    let root_span_id = SpanId::new("0000000000000001");
    RunRecordV1Alpha1 {
      api_version: RUN_API_VERSION.to_string(),
      run_id: RunId::new(run_id),
      trace_id: TraceId::new("00000000000000000000000000000001"),
      run_type: RunType::Execute,
      state: TraceState::Ended,
      status_code: TraceStatusCode::Ok,
      started_at_millis: 100,
      finished_at_millis: Some(200),
      root_span_id,
      attributes: BTreeMap::new(),
      summary: Some("done".to_string()),
      failure: None,
    }
  }

  fn dummy_span(span_id: &SpanId) -> SpanRecordV1Alpha1 {
    SpanRecordV1Alpha1 {
      api_version: SPAN_API_VERSION.to_string(),
      span_id: span_id.clone(),
      parent_span_id: None,
      name: "auv.run.read".to_string(),
      state: TraceState::Ended,
      status_code: TraceStatusCode::Ok,
      started_at_millis: 100,
      finished_at_millis: Some(200),
      attributes: BTreeMap::new(),
      summary: None,
      failure: None,
    }
  }

  fn stage_json_artifact<T: Serialize>(
    store: &LocalStore,
    root: &Path,
    run_id: &RunId,
    span_id: &SpanId,
    index: usize,
    role: &str,
    preferred_name: &str,
    value: &T,
  ) -> auv_tracing_driver::trace::ArtifactRecordV1Alpha1 {
    let source_path = root.join(format!("source-{index}-{preferred_name}"));
    let rendered =
      serde_json::to_string_pretty(value).expect("artifact json should serialize") + "\n";
    fs::write(&source_path, rendered).expect("artifact source should write");
    store
      .stage_artifact_file(
        run_id,
        index,
        span_id,
        None,
        ArtifactFileSource {
          role: role.to_string(),
          source_path,
          preferred_name: preferred_name.to_string(),
          summary: None,
        },
      )
      .expect("artifact should stage")
  }

  fn persist_run_with_operation_result(
    run_id: &str,
    operation: &OperationResult,
  ) -> (PathBuf, LocalStore) {
    let root = temp_dir("session-summary-load");
    let store = LocalStore::new(root.clone()).expect("store should initialize");
    let run = dummy_run(run_id);
    let span = dummy_span(&run.root_span_id);
    let artifact = stage_json_artifact(
      &store,
      &root,
      &run.run_id,
      &span.span_id,
      0,
      "operation-result",
      "operation-result.json",
      operation,
    );
    store
      .write_run_snapshot(&CanonicalRun {
        run,
        spans: vec![span],
        events: Vec::new(),
        artifacts: vec![artifact],
      })
      .expect("run snapshot should persist");
    (root, store)
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
    let root = temp_dir("session-summary-missing-run");
    let store = LocalStore::new(root.clone()).expect("store should initialize");

    let loaded =
      load_joined_operation_summary(&store, "missing-run", None).expect("load should succeed");
    assert_eq!(loaded, JoinedOperationSummaryLoad::RunNotFound);

    let _ = fs::remove_dir_all(root);
  }

  #[test]
  fn load_joined_operation_summary_returns_no_persisted_result_when_run_lacks_artifact() {
    let root = temp_dir("session-summary-no-op-result");
    let store = LocalStore::new(root.clone()).expect("store should initialize");
    let run = dummy_run("run-no-op-result");
    let span = dummy_span(&run.root_span_id);
    store
      .write_run_snapshot(&CanonicalRun {
        run,
        spans: vec![span],
        events: Vec::new(),
        artifacts: Vec::new(),
      })
      .expect("run snapshot should persist");

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
    let (_root, store) = persist_run_with_operation_result("run-happy", &operation);
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

    let _ = fs::remove_dir_all(_root);
  }

  fn _assert_send_sync<T: Send + Sync>() {}
  #[test]
  fn joined_summary_is_send_sync() {
    _assert_send_sync::<JoinedOperationSummary>();
  }
}
