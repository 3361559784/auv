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

use auv_cli_invoke::OperationSummarySource;
use auv_tracing_driver::store::LocalStore;

use crate::contract::{ArtifactRef, OperationResult, OperationStatus};
use crate::model::AuvResult;
use crate::run_read;

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

/// Join a persisted `OperationResult` with an optional runtime summary source.
///
/// Pure join policy: the persisted record provides the required fields; the
/// runtime summary is layered on when present, otherwise `runtime` is `None`.
pub fn join_operation_summary(
  operation: &OperationResult,
  runtime: Option<&dyn OperationSummarySource>,
) -> JoinedOperationSummary {
  // NOTICE(api-p3-od2): `operation_id` here is the persisted domain label from
  // `OperationResult` (e.g. "music.search.results"), which is NOT the invoke
  // `command_id`. The proto `operation_id` mapping rule (command_id vs domain
  // label vs widened ref) is API-P3 open decision 2 / API-P4 gate 3 and is
  // resolved at the proto mapper (API-P8), not in this read path.
  JoinedOperationSummary {
    run_id: operation.run_id.as_str().to_string(),
    operation_id: operation.operation_id.clone(),
    status: operation.status,
    known_limits: operation.known_limits.clone(),
    artifacts: operation.evidence_artifacts.clone(),
    runtime: runtime.map(RuntimeOperationSummary::from_source),
  }
}

/// Load and join the `GetOperation` summary for a run.
///
/// Reads the persisted `OperationResult` (storage-side read path via
/// [`run_read::read_operation_result`]) and joins it with the supplied runtime
/// summary source. Returns `Ok(None)` when the run recorded no `OperationResult`
/// (the persisted skeleton is required; without it there is nothing
/// authoritative to return).
pub fn load_joined_operation_summary(
  store: &LocalStore,
  run_id: &str,
  runtime: Option<&dyn OperationSummarySource>,
) -> AuvResult<Option<JoinedOperationSummary>> {
  let Some(operation) = run_read::read_operation_result(store, run_id)? else {
    return Ok(None);
  };
  Ok(Some(join_operation_summary(&operation, runtime)))
}

#[cfg(test)]
mod tests {
  use std::collections::BTreeMap;

  use auv_cli_invoke::{InvokeResult, OperationSummary, RunStatus};
  use auv_tracing_driver::trace::{RunId, SpanId};

  use super::{JoinedOperationSummary, join_operation_summary};
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

  fn _assert_send_sync<T: Send + Sync>() {}
  #[test]
  fn joined_summary_is_send_sync() {
    _assert_send_sync::<JoinedOperationSummary>();
  }
}
