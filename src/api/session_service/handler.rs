//! Session API handler skeleton (API-P8).
//!
//! Transport-agnostic: this is NOT a gRPC server and implements no tonic
//! service trait. API-P4 defers the tonic/axum transport decision; this skeleton
//! only wires the proto request/response shapes to the internal seams:
//! session-aware invoke (API-P5), the operation summary cache (API-P6), and the
//! two-source `GetOperation` join (API-P7). Binding a transport is a later
//! owner-named slice (see the `mod.rs` TODO).
//!
//! NOTICE(api-p5-invoke-persist): `Invoke` records a run and caches the runtime
//! summary (API-P6) but does not yet persist an `OperationResult` artifact.
//! That write path belongs to `Runtime::record_operation`; until it is wired
//! through this handler, `GetOperation` returns `PersistedOperationRequired`
//! for runs created only via `Invoke`.

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use auv_api_proto::v1::session as proto;
use auv_cli_invoke::{OperationSummary, OperationSummaryCache, default_registry};
use auv_tracing_driver::store::LocalStore;
use auv_tracing_driver::{MemoryRunRecorder, RunRecordingBackend, SessionId};

use crate::api::session_service::SessionApiError;
use crate::api::session_service::mapper;
use crate::api::session_service::registry::SessionRegistry;
use crate::api::session_service::summary::{
  JoinedOperationSummaryLoad, load_joined_operation_summary,
};

/// Process-local session API handler over one store path.
///
/// Holds the lightweight session registry (API-P4 responsibility A) and the
/// in-memory summary cache (API-P6). Each invoke opens a fresh recording backend
/// over the store path, mirroring the existing `mcp` invoke surface.
pub struct SessionApiHandler {
  store_root: PathBuf,
  registry: Mutex<SessionRegistry>,
  summaries: Mutex<OperationSummaryCache>,
}

impl SessionApiHandler {
  pub fn new(store_root: PathBuf) -> Self {
    Self {
      store_root,
      registry: Mutex::new(SessionRegistry::new()),
      summaries: Mutex::new(OperationSummaryCache::new()),
    }
  }

  fn open_store(&self) -> Result<LocalStore, SessionApiError> {
    LocalStore::new(self.store_root.clone()).map_err(SessionApiError::Storage)
  }

  /// `CreateSession`: allocate + register lightweight session metadata, return a
  /// `SessionRef`.
  pub fn create_session(
    &self,
    _request: proto::CreateSessionRequest,
  ) -> Result<proto::CreateSessionResponse, SessionApiError> {
    // TODO(api-p8-event): emit a `session_created` SessionEvent once the event
    // projector (API-P4 responsibility D) has a source. No event bus is wired in
    // this skeleton, so creation is silent.
    let session_id = self
      .registry
      .lock()
      .expect("session registry mutex poisoned")
      .create();
    Ok(proto::CreateSessionResponse {
      session: Some(proto::SessionRef {
        session_id: session_id.as_str().to_string(),
      }),
    })
  }

  /// `Invoke`: validate the session, decode the payload, run the session-aware
  /// recorded invoke (API-P5), record the summary (API-P6), and map the result.
  pub fn invoke(
    &self,
    request: proto::InvokeRequest,
  ) -> Result<proto::InvokeResponse, SessionApiError> {
    let session = request
      .session
      .ok_or(SessionApiError::MissingField("session"))?;
    let session_id = session.session_id;
    if !self
      .registry
      .lock()
      .expect("session registry mutex poisoned")
      .contains(&session_id)
    {
      return Err(SessionApiError::UnknownSession(session_id));
    }

    let command_id = request.command_id;
    let host_request = mapper::decode_invoke_payload(command_id.clone(), &request.json_payload)?;

    let store = self.open_store()?;
    let recording = RunRecordingBackend::new(store, Arc::new(MemoryRunRecorder::new()));
    let registry = default_registry();
    let result = auv_cli_invoke::invoke_recorded_with_session(
      &recording,
      &registry,
      host_request,
      SessionId::new(session_id),
    )
    .map_err(SessionApiError::InvokeExecution)?;

    self
      .summaries
      .lock()
      .expect("summary cache mutex poisoned")
      .record(OperationSummary::capture(&result));

    Ok(mapper::invoke_result_to_response(&command_id, &result))
  }

  /// `GetOperation`: read the persisted record + cached runtime summary and
  /// return the explicit two-source join (API-P7).
  pub fn get_operation(
    &self,
    request: proto::GetOperationRequest,
  ) -> Result<proto::GetOperationResponse, SessionApiError> {
    let operation = request
      .operation
      .ok_or(SessionApiError::MissingField("operation"))?;
    let run_id = operation.run_id;

    let runtime_summary = {
      let summaries = self.summaries.lock().expect("summary cache mutex poisoned");
      summaries.get(&run_id).cloned()
    };

    let store = self.open_store()?;
    let runtime = runtime_summary
      .as_ref()
      .map(|summary| summary as &dyn auv_cli_invoke::OperationSummarySource);
    match load_joined_operation_summary(&store, &run_id, runtime)
      .map_err(SessionApiError::Storage)?
    {
      JoinedOperationSummaryLoad::Found(joined) => {
        Ok(mapper::joined_to_get_operation_response(&joined))
      }
      JoinedOperationSummaryLoad::RunNotFound => Err(SessionApiError::RunNotFound(run_id)),
      JoinedOperationSummaryLoad::NoPersistedOperationResult => {
        Err(SessionApiError::PersistedOperationRequired(run_id))
      }
    }
  }

  /// `StreamSessionEvents`: validates the session, then refuses.
  ///
  /// API-P4 responsibility D (event projector) has no internal event source yet
  /// (gate 4). Rather than emit a fabricated/empty stream, the skeleton returns
  /// `NotWired` so callers see the gap explicitly.
  pub fn stream_session_events(
    &self,
    request: proto::StreamSessionEventsRequest,
  ) -> Result<Vec<proto::SessionEvent>, SessionApiError> {
    let session = request
      .session
      .ok_or(SessionApiError::MissingField("session"))?;
    if !self
      .registry
      .lock()
      .expect("session registry mutex poisoned")
      .contains(&session.session_id)
    {
      return Err(SessionApiError::UnknownSession(session.session_id));
    }
    Err(SessionApiError::NotWired {
      gate: "event projector (API-P4 responsibility D)",
    })
  }
}

#[cfg(test)]
mod tests {
  use std::sync::atomic::{AtomicU64, Ordering};

  use auv_api_proto::v1::session as proto;
  use auv_tracing_driver::now_millis;

  use super::SessionApiHandler;
  use crate::api::session_service::SessionApiError;

  static DIR_COUNTER: AtomicU64 = AtomicU64::new(0);

  fn handler() -> SessionApiHandler {
    let unique = DIR_COUNTER.fetch_add(1, Ordering::Relaxed);
    let root = std::env::temp_dir().join(format!("auv-session-api-{}-{}", now_millis(), unique));
    SessionApiHandler::new(root)
  }

  #[test]
  fn create_session_allocates_and_registers_session() {
    let handler = handler();
    let response = handler
      .create_session(proto::CreateSessionRequest {
        client_label: String::new(),
      })
      .expect("create_session");
    let session = response.session.expect("session ref");
    assert!(!session.session_id.is_empty());
    assert!(
      handler
        .registry
        .lock()
        .unwrap()
        .contains(&session.session_id)
    );
  }

  #[test]
  fn invoke_rejects_unknown_session() {
    let handler = handler();
    let error = handler
      .invoke(proto::InvokeRequest {
        session: Some(proto::SessionRef {
          session_id: "ghost".to_string(),
        }),
        command_id: "fixture.observe".to_string(),
        json_payload: Vec::new(),
      })
      .expect_err("unknown session should fail");
    assert_eq!(error, SessionApiError::UnknownSession("ghost".to_string()));
  }

  #[test]
  fn invoke_runs_fixture_command_for_known_session() {
    let handler = handler();
    let session = handler
      .create_session(proto::CreateSessionRequest {
        client_label: String::new(),
      })
      .expect("create_session")
      .session
      .expect("session ref");
    let response = handler
      .invoke(proto::InvokeRequest {
        session: Some(session),
        command_id: "fixture.observe".to_string(),
        json_payload: Vec::new(),
      })
      .expect("invoke fixture.observe");
    assert_eq!(response.status, "completed");
    let operation = response.operation.expect("operation ref");
    assert!(!operation.run_id.is_empty());
    assert_eq!(operation.operation_id, "fixture.observe");
  }

  #[test]
  fn get_operation_without_persisted_record_requires_persisted_operation_result() {
    // The invoke path records a run and caches the runtime summary (API-P6) but
    // does NOT write a persisted OperationResult (that is a higher-level
    // Runtime::record_operation concern). API-P7's join requires the persisted
    // skeleton, so GetOperation reports PersistedOperationRequired even though
    // the runtime summary is cached for this run.
    let handler = handler();
    let session = handler
      .create_session(proto::CreateSessionRequest {
        client_label: String::new(),
      })
      .expect("create_session")
      .session
      .expect("session ref");
    let invoked = handler
      .invoke(proto::InvokeRequest {
        session: Some(session),
        command_id: "fixture.observe".to_string(),
        json_payload: Vec::new(),
      })
      .expect("invoke fixture.observe");
    let run_id = invoked.operation.expect("operation ref").run_id;

    let error = handler
      .get_operation(proto::GetOperationRequest {
        operation: Some(proto::OperationRef {
          run_id,
          operation_id: String::new(),
        }),
      })
      .expect_err("missing persisted operation result should fail");
    assert!(matches!(
      error,
      SessionApiError::PersistedOperationRequired(_)
    ));
  }

  #[test]
  fn get_operation_returns_joined_response_when_persisted_and_cached() {
    use std::collections::BTreeMap;
    use std::fs;

    use crate::contract::{
      OPERATION_RESULT_API_VERSION, OperationOutput, OperationResult, OperationStatus,
    };
    use auv_cli_invoke::{InvokeResult, OperationSummary, RunStatus};
    use auv_tracing_driver::artifact::ArtifactFileSource;
    use auv_tracing_driver::store::CanonicalRun;
    use auv_tracing_driver::trace::{
      RUN_API_VERSION, RunId, RunRecordV1Alpha1, RunType, SPAN_API_VERSION, SpanId,
      SpanRecordV1Alpha1, TraceId, TraceState, TraceStatusCode,
    };

    let unique = DIR_COUNTER.fetch_add(1, Ordering::Relaxed);
    let root = std::env::temp_dir().join(format!(
      "auv-session-api-joined-{}-{}",
      now_millis(),
      unique
    ));
    let handler = SessionApiHandler::new(root.clone());

    let run_id = "run-get-op-happy";
    let store = handler.open_store().expect("open store");
    let root_span_id = SpanId::new("0000000000000001");
    let run = RunRecordV1Alpha1 {
      api_version: RUN_API_VERSION.to_string(),
      run_id: RunId::new(run_id),
      trace_id: TraceId::new("00000000000000000000000000000001"),
      run_type: RunType::Execute,
      state: TraceState::Ended,
      status_code: TraceStatusCode::Ok,
      started_at_millis: 100,
      finished_at_millis: Some(200),
      root_span_id: root_span_id.clone(),
      attributes: BTreeMap::new(),
      summary: Some("done".to_string()),
      failure: None,
    };
    let span = SpanRecordV1Alpha1 {
      api_version: SPAN_API_VERSION.to_string(),
      span_id: root_span_id.clone(),
      parent_span_id: None,
      name: "auv.run.read".to_string(),
      state: TraceState::Ended,
      status_code: TraceStatusCode::Ok,
      started_at_millis: 100,
      finished_at_millis: Some(200),
      attributes: BTreeMap::new(),
      summary: None,
      failure: None,
    };
    let operation = OperationResult {
      api_version: OPERATION_RESULT_API_VERSION.to_string(),
      run_id: RunId::new(run_id),
      status: OperationStatus::Completed,
      operation_id: "music.search.results".to_string(),
      evidence_artifacts: Vec::new(),
      output: OperationOutput::Acknowledged { message: None },
      verifications: Vec::new(),
      freshness_basis: None,
      known_limits: Vec::new(),
    };
    let source_path = root.join("source-operation-result.json");
    let rendered = serde_json::to_string_pretty(&operation).expect("serialize") + "\n";
    fs::write(&source_path, rendered).expect("write artifact source");
    let artifact = store
      .stage_artifact_file(
        &run.run_id,
        0,
        &span.span_id,
        None,
        ArtifactFileSource {
          role: "operation-result".to_string(),
          source_path,
          preferred_name: "operation-result.json".to_string(),
          summary: None,
        },
      )
      .expect("stage artifact");
    store
      .write_run_snapshot(&CanonicalRun {
        run,
        spans: vec![span],
        events: Vec::new(),
        artifacts: vec![artifact],
      })
      .expect("persist run");

    let mut signals = BTreeMap::new();
    signals.insert("now_playing".to_string(), "track-x".to_string());
    handler
      .summaries
      .lock()
      .expect("summary cache mutex poisoned")
      .record(OperationSummary::capture(&InvokeResult {
        run_id: run_id.to_string(),
        producer_span_id: SpanId::new("0000000000000001"),
        status: RunStatus::Completed,
        output_summary: "did the thing".to_string(),
        signals,
        artifacts: Vec::new(),
        artifact_paths: Vec::new(),
        failure_message: None,
      }));

    let response = handler
      .get_operation(proto::GetOperationRequest {
        operation: Some(proto::OperationRef {
          run_id: run_id.to_string(),
          operation_id: String::new(),
        }),
      })
      .expect("get_operation should succeed");

    assert_eq!(response.status, "completed");
    assert_eq!(response.output_summary, "did the thing");
    assert_eq!(
      response.signals.get("now_playing").map(String::as_str),
      Some("track-x")
    );
    let operation_ref = response.operation.expect("operation ref");
    assert_eq!(operation_ref.run_id, run_id);
    assert_eq!(operation_ref.operation_id, "music.search.results");

    let _ = fs::remove_dir_all(root);
  }

  #[test]
  fn stream_session_events_is_not_wired() {
    let handler = handler();
    let session = handler
      .create_session(proto::CreateSessionRequest {
        client_label: String::new(),
      })
      .expect("create_session")
      .session
      .expect("session ref");
    let error = handler
      .stream_session_events(proto::StreamSessionEventsRequest {
        session: Some(session),
      })
      .expect_err("stream should be not wired");
    assert!(matches!(error, SessionApiError::NotWired { .. }));
  }
}
