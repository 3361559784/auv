use std::collections::BTreeMap;
use std::path::PathBuf;
use std::process;
use std::time::{SystemTime, UNIX_EPOCH};

pub type AuvResult<T> = Result<T, String>;

#[derive(Clone, Debug)]
pub struct CommandSpec {
  pub id: &'static str,
  pub summary: &'static str,
  pub driver_id: &'static str,
  pub operation: &'static str,
}

#[derive(Clone, Debug, Default)]
pub struct ExecutionTarget {
  pub application_id: Option<String>,
}

#[derive(Clone, Debug, Default)]
pub struct InvokeRequest {
  pub command_id: String,
  pub target: ExecutionTarget,
  pub inputs: BTreeMap<String, String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RunStatus {
  Completed,
  Failed,
}

impl RunStatus {
  pub fn as_str(&self) -> &'static str {
    match self {
      Self::Completed => "completed",
      Self::Failed => "failed",
    }
  }
}

#[derive(Clone, Debug)]
pub struct InvokeResult {
  pub run_id: String,
  pub status: RunStatus,
  pub output_summary: String,
  pub artifact_paths: Vec<PathBuf>,
  pub failure_message: Option<String>,
}

#[derive(Clone, Debug)]
pub struct EventRecord {
  pub at_millis: u128,
  pub kind: String,
  pub message: String,
}

#[derive(Clone, Debug)]
pub struct ArtifactRecord {
  pub id: String,
  pub kind: String,
  pub path: PathBuf,
  pub note: Option<String>,
}

#[derive(Clone, Debug)]
pub struct RunRecord {
  pub run_id: String,
  pub command_id: String,
  pub driver_id: String,
  pub operation: String,
  pub target_application_id: Option<String>,
  pub runtime_version: String,
  pub started_at_millis: u128,
  pub finished_at_millis: Option<u128>,
  pub status: RunStatus,
  pub inputs: BTreeMap<String, String>,
  pub output_summary: String,
  pub events: Vec<EventRecord>,
  pub artifacts: Vec<ArtifactRecord>,
}

#[derive(Clone, Debug)]
pub struct DriverDescriptor {
  pub id: &'static str,
  pub summary: &'static str,
  pub capabilities: &'static [&'static str],
  pub donor_boundary: &'static str,
}

#[derive(Clone, Debug)]
pub struct DriverCall {
  pub operation: String,
  pub target: ExecutionTarget,
  pub inputs: BTreeMap<String, String>,
  pub working_directory: PathBuf,
}

#[derive(Clone, Debug)]
pub struct ProducedArtifact {
  pub kind: String,
  pub source_path: PathBuf,
  pub preferred_name: String,
  pub note: Option<String>,
}

#[derive(Clone, Debug)]
pub struct DriverResponse {
  pub summary: String,
  pub backend: Option<String>,
  pub notes: Vec<String>,
  pub artifacts: Vec<ProducedArtifact>,
}

pub fn now_millis() -> u128 {
  SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .unwrap_or_default()
    .as_millis()
}

pub fn new_run_id() -> String {
  format!("run_{}_{}", now_millis(), process::id())
}
