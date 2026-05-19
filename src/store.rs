use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::driver::{copy_file, sanitized_artifact_name};
use crate::model::{AuvResult, ProducedArtifact};
use crate::trace::{
  ARTIFACT_API_VERSION, ArtifactId, ArtifactRecordV1Alpha1, EVENT_API_VERSION, EventId,
  EventRecordV1Alpha1, RUN_API_VERSION, RunId, RunRecordV1Alpha1, SPAN_API_VERSION, SpanId,
  SpanRecordV1Alpha1,
};

pub struct CanonicalRun {
  pub run: RunRecordV1Alpha1,
  pub spans: Vec<SpanRecordV1Alpha1>,
  pub events: Vec<EventRecordV1Alpha1>,
  pub artifacts: Vec<ArtifactRecordV1Alpha1>,
}

pub struct LocalStore {
  root: PathBuf,
}

impl LocalStore {
  pub fn new(root: PathBuf) -> AuvResult<Self> {
    fs::create_dir_all(root.join("runs"))
      .map_err(|error| format!("failed to create run store root: {error}"))?;
    Ok(Self { root })
  }

  pub fn root(&self) -> &Path {
    &self.root
  }

  pub fn run_dir(&self, run_id: impl AsRef<str>) -> PathBuf {
    self.root.join("runs").join(run_id.as_ref())
  }

  pub fn write_run_snapshot(&self, snapshot: &CanonicalRun) -> AuvResult<()> {
    let run_directory = self.run_dir(&snapshot.run.run_id);
    fs::create_dir_all(run_directory.join("artifacts")).map_err(|error| {
      format!(
        "failed to create canonical run directory {}: {error}",
        run_directory.display()
      )
    })?;
    write_json_atomic(
      &run_directory.join("run.json"),
      &snapshot.run,
      "run metadata",
    )?;
    write_jsonl_atomic(
      &run_directory.join("spans.jsonl"),
      &snapshot.spans,
      "span records",
    )?;
    write_jsonl_atomic(
      &run_directory.join("events.jsonl"),
      &snapshot.events,
      "event records",
    )?;
    write_jsonl_atomic(
      &run_directory.join("artifacts.jsonl"),
      &snapshot.artifacts,
      "artifact records",
    )?;
    Ok(())
  }

  pub fn stage_artifact(
    &self,
    run_id: &RunId,
    index: usize,
    artifact: ProducedArtifact,
    span_id: &SpanId,
    event_id: Option<EventId>,
  ) -> AuvResult<ArtifactRecordV1Alpha1> {
    self.stage_artifact_file(
      run_id,
      index,
      span_id,
      event_id,
      artifact.kind,
      artifact.source_path,
      artifact.preferred_name,
      artifact.note,
    )
  }

  pub fn stage_artifact_file(
    &self,
    run_id: &RunId,
    index: usize,
    span_id: &SpanId,
    event_id: Option<EventId>,
    role: String,
    source_path: PathBuf,
    preferred_name: String,
    summary: Option<String>,
  ) -> AuvResult<ArtifactRecordV1Alpha1> {
    let artifact_id = ArtifactId::new(format!("artifact_{:04}", index + 1));
    let extension = source_path
      .extension()
      .and_then(|extension| extension.to_str())
      .unwrap_or("bin");
    let base_name =
      sanitized_artifact_name(preferred_name.trim_end_matches(&format!(".{extension}")));
    let relative_path =
      PathBuf::from("artifacts").join(format!("{}_{base_name}.{extension}", artifact_id.as_str()));
    let destination = self.run_dir(run_id).join(&relative_path);

    copy_file(&source_path, &destination)?;
    if source_path != destination {
      let _ = fs::remove_file(&source_path);
    }

    Ok(ArtifactRecordV1Alpha1 {
      api_version: ARTIFACT_API_VERSION.to_string(),
      artifact_id,
      span_id: span_id.clone(),
      event_id,
      role,
      mime_type: mime_type_for_extension(extension).to_string(),
      path: relative_path.to_string_lossy().into_owned(),
      sha256: None,
      attributes: Default::default(),
      summary,
    })
  }

  pub fn read_run(&self, run_id: &str) -> AuvResult<CanonicalRun> {
    let run_directory = self.run_dir(run_id);
    let run: RunRecordV1Alpha1 = read_json(&run_directory.join("run.json"))?;
    if run.api_version != RUN_API_VERSION {
      return Err(format!("unsupported_run_format: {}", run.api_version));
    }
    let spans: Vec<SpanRecordV1Alpha1> = read_jsonl(&run_directory.join("spans.jsonl"))?;
    let events: Vec<EventRecordV1Alpha1> = read_jsonl(&run_directory.join("events.jsonl"))?;
    let artifacts: Vec<ArtifactRecordV1Alpha1> =
      read_jsonl(&run_directory.join("artifacts.jsonl"))?;

    for span in &spans {
      if span.api_version != SPAN_API_VERSION {
        return Err(format!("invalid_run_format: {}", span.api_version));
      }
    }
    for event in &events {
      if event.api_version != EVENT_API_VERSION {
        return Err(format!("invalid_run_format: {}", event.api_version));
      }
    }
    for artifact in &artifacts {
      if artifact.api_version != ARTIFACT_API_VERSION {
        return Err(format!("invalid_run_format: {}", artifact.api_version));
      }
    }

    Ok(CanonicalRun {
      run,
      spans,
      events,
      artifacts,
    })
  }

  pub fn list_runs(&self) -> AuvResult<Vec<RunRecordV1Alpha1>> {
    let runs_root = self.root.join("runs");
    let mut runs = Vec::new();
    for entry in fs::read_dir(&runs_root)
      .map_err(|error| format!("failed to read runs root {}: {error}", runs_root.display()))?
    {
      let entry = entry.map_err(|error| format!("failed to enumerate runs: {error}"))?;
      if !entry.path().is_dir() {
        continue;
      }
      let run_path = entry.path().join("run.json");
      if !run_path.exists() {
        continue;
      }
      let run: RunRecordV1Alpha1 = read_json(&run_path)?;
      if run.api_version == RUN_API_VERSION {
        runs.push(run);
      }
    }
    runs.sort_by_key(|run| run.started_at_millis);
    Ok(runs)
  }
}

fn write_json_atomic<T: serde::Serialize>(path: &Path, value: &T, label: &str) -> AuvResult<()> {
  let tmp = path.with_extension("tmp");
  let bytes = serde_json::to_vec_pretty(value)
    .map_err(|error| format!("failed to encode {label} {}: {error}", path.display()))?;
  fs::write(&tmp, bytes)
    .map_err(|error| format!("failed to write {label} {}: {error}", tmp.display()))?;
  fs::rename(&tmp, path)
    .map_err(|error| format!("failed to publish {label} {}: {error}", path.display()))
}

fn write_jsonl_atomic<T: serde::Serialize>(
  path: &Path,
  values: &[T],
  label: &str,
) -> AuvResult<()> {
  let tmp = path.with_extension("tmp");
  let mut file = fs::File::create(&tmp)
    .map_err(|error| format!("failed to create {label} {}: {error}", tmp.display()))?;
  for value in values {
    serde_json::to_writer(&mut file, value)
      .map_err(|error| format!("failed to encode {label} {}: {error}", tmp.display()))?;
    file
      .write_all(b"\n")
      .map_err(|error| format!("failed to write {label} {}: {error}", tmp.display()))?;
  }
  drop(file);
  fs::rename(&tmp, path)
    .map_err(|error| format!("failed to publish {label} {}: {error}", path.display()))
}

fn read_json<T: serde::de::DeserializeOwned>(path: &Path) -> AuvResult<T> {
  let raw = fs::read_to_string(path)
    .map_err(|error| format!("failed to read {}: {error}", path.display()))?;
  serde_json::from_str(&raw).map_err(|error| format!("failed to parse {}: {error}", path.display()))
}

fn read_jsonl<T: serde::de::DeserializeOwned>(path: &Path) -> AuvResult<Vec<T>> {
  let raw = fs::read_to_string(path)
    .map_err(|error| format!("failed to read {}: {error}", path.display()))?;
  let mut records = Vec::new();
  for (index, line) in raw.lines().enumerate() {
    if line.trim().is_empty() {
      continue;
    }
    let record = serde_json::from_str(line).map_err(|error| {
      format!(
        "failed to parse {} line {}: {error}",
        path.display(),
        index + 1
      )
    })?;
    records.push(record);
  }
  Ok(records)
}

fn mime_type_for_extension(extension: &str) -> &'static str {
  match extension {
    "json" => "application/json",
    "png" => "image/png",
    "txt" | "log" | "md" => "text/plain",
    _ => "application/octet-stream",
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::trace::{
    ArtifactRecordV1Alpha1, EventRecordV1Alpha1, RUN_API_VERSION, RunRecordV1Alpha1, RunType,
    SpanRecordV1Alpha1, TraceState, TraceStatusCode,
  };
  use std::collections::BTreeMap;
  use std::env;

  #[test]
  fn local_store_persists_canonical_run_files() {
    let root = temp_dir("store-canonical");
    let store = LocalStore::new(root.clone()).expect("should initialize");
    let run = dummy_run("run_store_test");
    let span = dummy_span(&run.root_span_id);
    let event = dummy_event(&span.span_id);
    let artifact = dummy_artifact(&span.span_id);

    store
      .write_run_snapshot(&CanonicalRun {
        run,
        spans: vec![span],
        events: vec![event],
        artifacts: vec![artifact],
      })
      .expect("should persist canonical run");

    let run_dir = root.join("runs").join("run_store_test");
    assert!(run_dir.join("run.json").exists());
    assert!(run_dir.join("spans.jsonl").exists());
    assert!(run_dir.join("events.jsonl").exists());
    assert!(run_dir.join("artifacts.jsonl").exists());
    assert!(!run_dir.join("inspect.txt").exists());
    assert!(!run_dir.join("meta.txt").exists());

    let loaded = store.read_run("run_store_test").expect("run should read");
    assert_eq!(loaded.run.api_version, RUN_API_VERSION);
    assert_eq!(loaded.spans.len(), 1);
    assert_eq!(loaded.events.len(), 1);
    assert_eq!(loaded.artifacts.len(), 1);

    let _ = fs::remove_dir_all(root);
  }

  #[test]
  fn local_store_ignores_directories_without_run_json() {
    let root = temp_dir("store-list");
    let store = LocalStore::new(root.clone()).expect("should initialize");
    fs::create_dir_all(root.join("runs").join("old_run_without_run_json")).expect("old run dir");

    let runs = store.list_runs().expect("runs should list");
    assert!(runs.is_empty());

    let _ = fs::remove_dir_all(root);
  }

  fn temp_dir(label: &str) -> PathBuf {
    let path = env::temp_dir().join(format!("auv-{}-{}", label, crate::model::now_millis()));
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).expect("temp dir should be creatable");
    path
  }

  fn dummy_run(run_id: &str) -> RunRecordV1Alpha1 {
    let root_span_id = crate::trace::SpanId::new("0000000000000001");
    RunRecordV1Alpha1 {
      api_version: RUN_API_VERSION.to_string(),
      run_id: crate::trace::RunId::new(run_id),
      trace_id: crate::trace::TraceId::new("00000000000000000000000000000001"),
      run_type: RunType::Command,
      state: TraceState::Ended,
      status_code: TraceStatusCode::Ok,
      started_at_millis: 100,
      finished_at_millis: Some(200),
      root_span_id,
      attributes: BTreeMap::new(),
      summary: Some("ok".to_string()),
      failure: None,
    }
  }

  fn dummy_span(span_id: &crate::trace::SpanId) -> SpanRecordV1Alpha1 {
    SpanRecordV1Alpha1 {
      api_version: crate::trace::SPAN_API_VERSION.to_string(),
      span_id: span_id.clone(),
      parent_span_id: None,
      name: "auv.command".to_string(),
      state: TraceState::Ended,
      status_code: TraceStatusCode::Ok,
      started_at_millis: 100,
      finished_at_millis: Some(200),
      attributes: BTreeMap::new(),
      summary: Some("ok".to_string()),
      failure: None,
    }
  }

  fn dummy_event(span_id: &crate::trace::SpanId) -> EventRecordV1Alpha1 {
    EventRecordV1Alpha1 {
      api_version: crate::trace::EVENT_API_VERSION.to_string(),
      event_id: crate::trace::EventId::new("event_1"),
      span_id: span_id.clone(),
      name: "command.resolved".to_string(),
      timestamp_millis: 100,
      attributes: BTreeMap::new(),
      message: Some("resolved".to_string()),
      artifact_ids: Vec::new(),
    }
  }

  fn dummy_artifact(span_id: &crate::trace::SpanId) -> ArtifactRecordV1Alpha1 {
    ArtifactRecordV1Alpha1 {
      api_version: crate::trace::ARTIFACT_API_VERSION.to_string(),
      artifact_id: crate::trace::ArtifactId::new("artifact_0001"),
      span_id: span_id.clone(),
      event_id: None,
      role: "driver.output".to_string(),
      mime_type: "text/plain".to_string(),
      path: "artifacts/artifact_0001_output.txt".to_string(),
      sha256: None,
      attributes: BTreeMap::new(),
      summary: Some("output".to_string()),
    }
  }
}
