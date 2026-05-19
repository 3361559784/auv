use std::net::SocketAddr;
use std::sync::Arc;

use axum::body::Body;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Path, State};
use axum::http::header::CONTENT_TYPE;
use axum::http::{HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use tokio::net::TcpListener;
use tokio::sync::broadcast;

use crate::model::AuvResult;
use crate::recording::BroadcastRunEventSink;
use crate::store::LocalStore;

pub const DEFAULT_INSPECT_HOST: &str = "127.0.0.1";
pub const DEFAULT_INSPECT_PORT: u16 = 7319;

#[derive(Clone)]
struct InspectServerState {
  store: Arc<LocalStore>,
  event_sink: Arc<BroadcastRunEventSink>,
}

#[derive(Clone, Debug)]
pub struct InspectServeConfig {
  pub host: String,
  pub port: u16,
}

impl Default for InspectServeConfig {
  fn default() -> Self {
    Self {
      host: DEFAULT_INSPECT_HOST.to_string(),
      port: DEFAULT_INSPECT_PORT,
    }
  }
}

pub fn router(store: LocalStore, event_sink: Arc<BroadcastRunEventSink>) -> Router {
  let state = InspectServerState {
    store: Arc::new(store),
    event_sink,
  };
  Router::new()
    .route("/runs", get(list_runs))
    .route("/runs/{run_id}", get(get_run))
    .route("/runs/{run_id}/spans", get(get_spans))
    .route("/runs/{run_id}/events", get(get_events))
    .route("/runs/{run_id}/artifacts", get(get_artifacts))
    .route("/runs/{run_id}/artifacts/{artifact_id}", get(get_artifact))
    .route("/runs/{run_id}/stream", get(stream_run))
    .with_state(state)
}

pub async fn serve(
  store: LocalStore,
  event_sink: Arc<BroadcastRunEventSink>,
  config: InspectServeConfig,
) -> AuvResult<SocketAddr> {
  let address = format!("{}:{}", config.host, config.port)
    .parse::<SocketAddr>()
    .map_err(|error| format!("invalid inspect server address: {error}"))?;
  let listener = TcpListener::bind(address)
    .await
    .map_err(|error| format!("failed to bind inspect server {address}: {error}"))?;
  let local_address = listener
    .local_addr()
    .map_err(|error| format!("failed to read inspect server address: {error}"))?;
  axum::serve(listener, router(store, event_sink))
    .await
    .map_err(|error| format!("inspect server failed: {error}"))?;
  Ok(local_address)
}

async fn list_runs(State(state): State<InspectServerState>) -> Result<Response, InspectHttpError> {
  let runs = state
    .store
    .list_runs()
    .map_err(InspectHttpError::from_store)?;
  Ok(Json(runs).into_response())
}

async fn get_run(
  State(state): State<InspectServerState>,
  Path(run_id): Path<String>,
) -> Result<Response, InspectHttpError> {
  let run = state
    .store
    .read_run(&run_id)
    .map_err(InspectHttpError::from_store)?;
  Ok(Json(run).into_response())
}

async fn get_spans(
  State(state): State<InspectServerState>,
  Path(run_id): Path<String>,
) -> Result<Response, InspectHttpError> {
  let run = state
    .store
    .read_run(&run_id)
    .map_err(InspectHttpError::from_store)?;
  Ok(Json(run.spans).into_response())
}

async fn get_events(
  State(state): State<InspectServerState>,
  Path(run_id): Path<String>,
) -> Result<Response, InspectHttpError> {
  let run = state
    .store
    .read_run(&run_id)
    .map_err(InspectHttpError::from_store)?;
  Ok(Json(run.events).into_response())
}

async fn get_artifacts(
  State(state): State<InspectServerState>,
  Path(run_id): Path<String>,
) -> Result<Response, InspectHttpError> {
  let run = state
    .store
    .read_run(&run_id)
    .map_err(InspectHttpError::from_store)?;
  Ok(Json(run.artifacts).into_response())
}

async fn get_artifact(
  State(state): State<InspectServerState>,
  Path((run_id, artifact_id)): Path<(String, String)>,
) -> Result<Response, InspectHttpError> {
  let (artifact, path) = state
    .store
    .artifact_file(&run_id, &artifact_id)
    .map_err(InspectHttpError::from_store)?;
  let bytes = tokio::fs::read(&path)
    .await
    .map_err(|error| InspectHttpError::not_found(format!("failed to read artifact: {error}")))?;
  let mut response = Body::from(bytes).into_response();
  let content_type = HeaderValue::from_str(&artifact.mime_type)
    .unwrap_or_else(|_| HeaderValue::from_static("application/octet-stream"));
  response.headers_mut().insert(CONTENT_TYPE, content_type);
  Ok(response)
}

async fn stream_run(
  State(state): State<InspectServerState>,
  Path(run_id): Path<String>,
  websocket: WebSocketUpgrade,
) -> Result<Response, InspectHttpError> {
  state
    .store
    .run_dir(&run_id)
    .map_err(InspectHttpError::from_store)?;
  Ok(
    websocket
      .on_upgrade(move |socket| stream_run_events(socket, state.event_sink, run_id))
      .into_response(),
  )
}

async fn stream_run_events(
  mut socket: WebSocket,
  event_sink: Arc<BroadcastRunEventSink>,
  run_id: String,
) {
  let mut receiver = event_sink.subscribe();
  loop {
    match receiver.recv().await {
      Ok(event) if event.run_id().as_str() == run_id => {
        let Ok(payload) = serde_json::to_string(&event) else {
          continue;
        };
        if socket.send(Message::Text(payload.into())).await.is_err() {
          break;
        }
      }
      Ok(_) => {}
      Err(broadcast::error::RecvError::Lagged(_)) => {}
      Err(broadcast::error::RecvError::Closed) => break,
    }
  }
}

#[derive(Debug)]
struct InspectHttpError {
  status: StatusCode,
  message: String,
}

impl InspectHttpError {
  fn from_store(error: String) -> Self {
    let status = if error.contains("invalid run id") {
      StatusCode::BAD_REQUEST
    } else if error.contains("failed to read") || error.contains("not found") {
      StatusCode::NOT_FOUND
    } else {
      StatusCode::INTERNAL_SERVER_ERROR
    };
    Self {
      status,
      message: error,
    }
  }

  fn not_found(message: String) -> Self {
    Self {
      status: StatusCode::NOT_FOUND,
      message,
    }
  }
}

impl IntoResponse for InspectHttpError {
  fn into_response(self) -> Response {
    (
      self.status,
      Json(serde_json::json!({
        "error": self.message,
      })),
    )
      .into_response()
  }
}

#[cfg(test)]
mod tests {
  use std::collections::BTreeMap;
  use std::fs;
  use std::sync::Arc;

  use axum::body::{Body, to_bytes};
  use axum::http::{Request, StatusCode};
  use tower::ServiceExt;

  use super::router;
  use crate::model::now_millis;
  use crate::recording::BroadcastRunEventSink;
  use crate::store::{CanonicalRun, LocalStore};
  use crate::trace::{
    ARTIFACT_API_VERSION, ArtifactId, ArtifactRecordV1Alpha1, EVENT_API_VERSION, EventId,
    EventRecordV1Alpha1, RUN_API_VERSION, RunId, RunRecordV1Alpha1, RunType, SPAN_API_VERSION,
    SpanId, SpanRecordV1Alpha1, TraceId, TraceState, TraceStatusCode,
  };

  #[tokio::test]
  async fn routes_return_canonical_records_and_artifact_bytes() {
    let root = temp_dir("inspect-server-routes");
    let store = LocalStore::new(root.clone()).expect("store should initialize");
    let run_id = RunId::new("run_inspect_server_test");
    let span_id = SpanId::new("0000000000000001");
    let artifact_id = ArtifactId::new("artifact_server_test");
    store
      .write_run_snapshot(&CanonicalRun {
        run: RunRecordV1Alpha1 {
          api_version: RUN_API_VERSION.to_string(),
          run_id: run_id.clone(),
          trace_id: TraceId::new("00000000000000000000000000000001"),
          run_type: RunType::Command,
          state: TraceState::Ended,
          status_code: TraceStatusCode::Ok,
          started_at_millis: 100,
          finished_at_millis: Some(101),
          root_span_id: span_id.clone(),
          attributes: BTreeMap::new(),
          summary: Some("done".to_string()),
          failure: None,
        },
        spans: vec![SpanRecordV1Alpha1 {
          api_version: SPAN_API_VERSION.to_string(),
          span_id: span_id.clone(),
          parent_span_id: None,
          name: "auv.inspect.server".to_string(),
          state: TraceState::Ended,
          status_code: TraceStatusCode::Ok,
          started_at_millis: 100,
          finished_at_millis: Some(101),
          attributes: BTreeMap::new(),
          summary: None,
          failure: None,
        }],
        events: vec![EventRecordV1Alpha1 {
          api_version: EVENT_API_VERSION.to_string(),
          event_id: EventId::new("event_server_test"),
          span_id: span_id.clone(),
          name: "inspect.event".to_string(),
          timestamp_millis: 100,
          attributes: BTreeMap::new(),
          message: None,
          artifact_ids: vec![artifact_id.clone()],
        }],
        artifacts: vec![ArtifactRecordV1Alpha1 {
          api_version: ARTIFACT_API_VERSION.to_string(),
          artifact_id: artifact_id.clone(),
          span_id,
          event_id: None,
          role: "driver.output".to_string(),
          mime_type: "text/plain".to_string(),
          path: "artifacts/artifact_server_test.txt".to_string(),
          sha256: None,
          attributes: BTreeMap::new(),
          summary: None,
        }],
      })
      .expect("run should persist");
    let artifact_path = root
      .join("runs")
      .join(run_id.as_str())
      .join("artifacts")
      .join("artifact_server_test.txt");
    fs::write(&artifact_path, "artifact body").expect("artifact should write");

    let app = router(store, Arc::new(BroadcastRunEventSink::new(16)));
    let spans_response = app
      .clone()
      .oneshot(
        Request::builder()
          .uri("/runs/run_inspect_server_test/spans")
          .body(Body::empty())
          .expect("request should build"),
      )
      .await
      .expect("route should respond");
    assert_eq!(spans_response.status(), StatusCode::OK);
    let spans_body = to_bytes(spans_response.into_body(), usize::MAX)
      .await
      .expect("body should read");
    let spans: serde_json::Value =
      serde_json::from_slice(&spans_body).expect("spans should be json");
    assert_eq!(spans[0]["name"], "auv.inspect.server");

    let artifact_response = app
      .oneshot(
        Request::builder()
          .uri("/runs/run_inspect_server_test/artifacts/artifact_server_test")
          .body(Body::empty())
          .expect("request should build"),
      )
      .await
      .expect("route should respond");
    assert_eq!(artifact_response.status(), StatusCode::OK);
    assert_eq!(
      artifact_response
        .headers()
        .get("content-type")
        .and_then(|value| value.to_str().ok()),
      Some("text/plain")
    );
    let artifact_body = to_bytes(artifact_response.into_body(), usize::MAX)
      .await
      .expect("body should read");
    assert_eq!(&artifact_body[..], b"artifact body");

    let _ = fs::remove_dir_all(root);
  }

  fn temp_dir(label: &str) -> std::path::PathBuf {
    let path = std::env::temp_dir().join(format!("auv-{}-{}", label, now_millis()));
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).expect("temp dir should be creatable");
    path
  }
}
