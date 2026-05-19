use std::collections::BTreeMap;
use std::process;
use std::sync::atomic::{AtomicU64, Ordering};

use serde::{Deserialize, Serialize};

use crate::model::now_millis;

pub const RUN_API_VERSION: &str = "auv.run.v1alpha1";
pub const SPAN_API_VERSION: &str = "auv.span.v1alpha1";
pub const EVENT_API_VERSION: &str = "auv.event.v1alpha1";
pub const ARTIFACT_API_VERSION: &str = "auv.artifact.v1alpha1";

static TRACE_COUNTER: AtomicU64 = AtomicU64::new(0);
static RUN_COUNTER: AtomicU64 = AtomicU64::new(0);
static SPAN_COUNTER: AtomicU64 = AtomicU64::new(0);
static EVENT_COUNTER: AtomicU64 = AtomicU64::new(0);

macro_rules! id_type {
  ($name:ident) => {
    #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
    #[serde(transparent)]
    pub struct $name(String);

    impl $name {
      pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
      }

      pub fn as_str(&self) -> &str {
        &self.0
      }
    }

    impl std::fmt::Display for $name {
      fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
      }
    }

    impl AsRef<str> for $name {
      fn as_ref(&self) -> &str {
        self.as_str()
      }
    }
  };
}

id_type!(RunId);
id_type!(TraceId);
id_type!(SpanId);
id_type!(EventId);
id_type!(ArtifactId);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RunType {
  Command,
  Execute,
  Probe,
  Analyze,
  Distill,
  Validate,
}

impl RunType {
  pub fn as_str(&self) -> &'static str {
    match self {
      Self::Command => "command",
      Self::Execute => "execute",
      Self::Probe => "probe",
      Self::Analyze => "analyze",
      Self::Distill => "distill",
      Self::Validate => "validate",
    }
  }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TraceState {
  Running,
  Ended,
}

impl TraceState {
  pub fn as_str(&self) -> &'static str {
    match self {
      Self::Running => "running",
      Self::Ended => "ended",
    }
  }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TraceStatusCode {
  Unset,
  Ok,
  Error,
}

impl TraceStatusCode {
  pub fn as_str(&self) -> &'static str {
    match self {
      Self::Unset => "unset",
      Self::Ok => "ok",
      Self::Error => "error",
    }
  }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TraceFailure {
  pub message: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RunRecordV1Alpha1 {
  pub api_version: String,
  pub run_id: RunId,
  pub trace_id: TraceId,
  pub run_type: RunType,
  pub state: TraceState,
  pub status_code: TraceStatusCode,
  pub started_at_millis: u128,
  pub finished_at_millis: Option<u128>,
  pub root_span_id: SpanId,
  pub attributes: BTreeMap<String, serde_json::Value>,
  pub summary: Option<String>,
  pub failure: Option<TraceFailure>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SpanRecordV1Alpha1 {
  pub api_version: String,
  pub span_id: SpanId,
  pub parent_span_id: Option<SpanId>,
  pub name: String,
  pub state: TraceState,
  pub status_code: TraceStatusCode,
  pub started_at_millis: u128,
  pub finished_at_millis: Option<u128>,
  pub attributes: BTreeMap<String, serde_json::Value>,
  pub summary: Option<String>,
  pub failure: Option<TraceFailure>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EventRecordV1Alpha1 {
  pub api_version: String,
  pub event_id: EventId,
  pub span_id: SpanId,
  pub name: String,
  pub timestamp_millis: u128,
  pub attributes: BTreeMap<String, serde_json::Value>,
  pub message: Option<String>,
  pub artifact_ids: Vec<ArtifactId>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ArtifactRecordV1Alpha1 {
  pub api_version: String,
  pub artifact_id: ArtifactId,
  pub span_id: SpanId,
  pub event_id: Option<EventId>,
  pub role: String,
  pub mime_type: String,
  pub path: String,
  pub sha256: Option<String>,
  pub attributes: BTreeMap<String, serde_json::Value>,
  pub summary: Option<String>,
}

pub fn new_run_id() -> RunId {
  let sequence = RUN_COUNTER.fetch_add(1, Ordering::Relaxed);
  RunId::new(format!(
    "run_{}_{}_{}",
    now_millis(),
    process::id(),
    sequence
  ))
}

pub fn new_trace_id() -> TraceId {
  let sequence = TRACE_COUNTER.fetch_add(1, Ordering::Relaxed);
  TraceId::new(format!("{:016x}{:016x}", now_millis() as u64, sequence))
}

pub fn new_span_id() -> SpanId {
  let sequence = SPAN_COUNTER.fetch_add(1, Ordering::Relaxed);
  SpanId::new(format!("{:016x}", sequence + 1))
}

pub fn new_event_id() -> EventId {
  let sequence = EVENT_COUNTER.fetch_add(1, Ordering::Relaxed);
  EventId::new(format!("event_{}_{}", now_millis(), sequence))
}

pub fn string_attr(value: impl Into<String>) -> serde_json::Value {
  serde_json::Value::String(value.into())
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn api_versions_are_v1alpha1() {
    assert_eq!(RUN_API_VERSION, "auv.run.v1alpha1");
    assert_eq!(SPAN_API_VERSION, "auv.span.v1alpha1");
    assert_eq!(EVENT_API_VERSION, "auv.event.v1alpha1");
    assert_eq!(ARTIFACT_API_VERSION, "auv.artifact.v1alpha1");
  }

  #[test]
  fn generated_ids_are_prefixed_and_distinct() {
    let first_run = new_run_id();
    let second_run = new_run_id();
    let trace_id = new_trace_id();
    let span_id = new_span_id();
    let event_id = new_event_id();

    assert!(first_run.as_str().starts_with("run_"));
    assert_ne!(first_run, second_run);
    assert_eq!(trace_id.as_str().len(), 32);
    assert_eq!(span_id.as_str().len(), 16);
    assert!(event_id.as_str().starts_with("event_"));
  }

  #[test]
  fn status_codes_match_otel_words() {
    assert_eq!(TraceStatusCode::Unset.as_str(), "unset");
    assert_eq!(TraceStatusCode::Ok.as_str(), "ok");
    assert_eq!(TraceStatusCode::Error.as_str(), "error");
  }
}
