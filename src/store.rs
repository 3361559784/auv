use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::driver::{copy_file, sanitized_artifact_name};
use crate::model::{ArtifactRecord, AuvResult, ProducedArtifact, RunRecord, now_millis};

pub struct LocalStore {
  root: PathBuf,
}

impl LocalStore {
  pub fn new(root: PathBuf) -> AuvResult<Self> {
    fs::create_dir_all(root.join("runs"))
      .map_err(|error| format!("failed to create run store root: {error}"))?;
    fs::create_dir_all(root.join("artifacts"))
      .map_err(|error| format!("failed to create artifact store root: {error}"))?;
    Ok(Self { root })
  }

  pub fn stage_artifact(
    &self,
    run_id: &str,
    index: usize,
    artifact: ProducedArtifact,
  ) -> AuvResult<ArtifactRecord> {
    let artifact_id = format!("artifact_{:02}", index + 1);
    let extension = artifact
      .source_path
      .extension()
      .and_then(|extension| extension.to_str())
      .unwrap_or("bin");
    let base_name = sanitized_artifact_name(
      artifact
        .preferred_name
        .trim_end_matches(&format!(".{extension}")),
    );
    let destination = self
      .root
      .join("artifacts")
      .join(run_id)
      .join(format!("{artifact_id}_{base_name}.{extension}"));

    copy_file(&artifact.source_path, &destination)?;
    if artifact.source_path != destination {
      let _ = fs::remove_file(&artifact.source_path);
    }

    Ok(ArtifactRecord {
      id: artifact_id,
      kind: artifact.kind,
      path: destination,
      note: artifact.note,
    })
  }

  pub fn persist_run(&self, run: &RunRecord) -> AuvResult<()> {
    let runs_root = self.root.join("runs");
    let run_directory = runs_root.join(&run.run_id);
    if run_directory.exists() {
      return Err(format!(
        "run directory {} already exists",
        run_directory.display()
      ));
    }

    let staging_directory = runs_root.join(format!(".{}-tmp-{}", run.run_id, now_millis()));
    fs::create_dir_all(&staging_directory).map_err(|error| {
      format!(
        "failed to create staging run directory {}: {error}",
        staging_directory.display()
      )
    })?;

    let write_result = write_run_snapshot(run, &staging_directory);
    if let Err(error) = write_result {
      let _ = fs::remove_dir_all(&staging_directory);
      return Err(error);
    }

    fs::rename(&staging_directory, &run_directory).map_err(|error| {
      let _ = fs::remove_dir_all(&staging_directory);
      format!(
        "failed to publish run directory {} from {}: {error}",
        run_directory.display(),
        staging_directory.display()
      )
    })?;

    Ok(())
  }

  pub fn render_inspection(&self, run_id: &str) -> AuvResult<String> {
    let inspection_path = self.root.join("runs").join(run_id).join("inspect.txt");

    fs::read_to_string(&inspection_path).map_err(|error| {
      format!(
        "failed to read inspect snapshot {}: {error}",
        inspection_path.display()
      )
    })
  }
}

fn write_run_snapshot(run: &RunRecord, directory: &Path) -> AuvResult<()> {
  write_snapshot_file(
    &directory.join("meta.txt"),
    render_meta(run),
    "run metadata",
  )?;
  write_snapshot_file(
    &directory.join("inputs.txt"),
    render_inputs(&run.inputs),
    "run inputs",
  )?;
  write_snapshot_file(
    &directory.join("events.log"),
    render_events(run),
    "run events",
  )?;
  write_snapshot_file(
    &directory.join("artifacts.txt"),
    render_artifacts(run),
    "artifact manifest",
  )?;
  write_snapshot_file(
    &directory.join("output.txt"),
    format!("{}\n", run.output_summary),
    "run output",
  )?;
  write_snapshot_file(
    &directory.join("inspect.txt"),
    render_inspection(run),
    "inspect snapshot",
  )?;
  Ok(())
}

fn write_snapshot_file(path: &Path, content: String, label: &str) -> AuvResult<()> {
  fs::write(path, content)
    .map_err(|error| format!("failed to write {} {}: {error}", label, path.display()))
}

fn render_meta(run: &RunRecord) -> String {
  let target = run
    .target_application_id
    .clone()
    .unwrap_or_else(|| "n/a".to_string());
  let finished = run
    .finished_at_millis
    .map(|value| value.to_string())
    .unwrap_or_else(|| "n/a".to_string());

  [
    format!("runId: {}", run.run_id),
    format!("status: {}", run.status.as_str()),
    format!("command: {}", run.command_id),
    format!("driver: {}", run.driver_id),
    format!("operation: {}", run.operation),
    format!("targetApplicationId: {target}"),
    format!("runtimeVersion: {}", run.runtime_version),
    format!("startedAtMillis: {}", run.started_at_millis),
    format!("finishedAtMillis: {finished}"),
  ]
  .join("\n")
    + "\n"
}

fn render_inputs(inputs: &BTreeMap<String, String>) -> String {
  if inputs.is_empty() {
    return "none\n".to_string();
  }

  let mut lines = Vec::new();
  for (key, value) in inputs {
    lines.push(format!("{key}={value}"));
  }
  lines.join("\n") + "\n"
}

fn render_events(run: &RunRecord) -> String {
  if run.events.is_empty() {
    return "none\n".to_string();
  }

  let mut lines = Vec::new();
  for event in &run.events {
    lines.push(format!(
      "{} {} {}",
      event.at_millis, event.kind, event.message
    ));
  }
  lines.join("\n") + "\n"
}

fn render_artifacts(run: &RunRecord) -> String {
  if run.artifacts.is_empty() {
    return "none\n".to_string();
  }

  let mut lines = Vec::new();
  for artifact in &run.artifacts {
    let note = artifact.note.clone().unwrap_or_else(|| "n/a".to_string());
    lines.push(format!(
      "{} kind={} path={} note={}",
      artifact.id,
      artifact.kind,
      artifact.path.display(),
      note
    ));
  }
  lines.join("\n") + "\n"
}

fn render_inspection(run: &RunRecord) -> String {
  let target = run
    .target_application_id
    .clone()
    .unwrap_or_else(|| "n/a".to_string());
  let finished = run
    .finished_at_millis
    .map(|value| value.to_string())
    .unwrap_or_else(|| "n/a".to_string());
  let sections = vec![
    format!("Run {}", run.run_id),
    format!("Status: {}", run.status.as_str()),
    format!("Command: {}", run.command_id),
    format!("Driver: {}", run.driver_id),
    format!("Operation: {}", run.operation),
    format!("Target: {target}"),
    format!("Runtime Version: {}", run.runtime_version),
    format!("Started At (ms): {}", run.started_at_millis),
    format!("Finished At (ms): {finished}"),
    String::new(),
    "Inputs".to_string(),
    render_block(&render_inputs(&run.inputs)),
    "Output".to_string(),
    render_block(&format!("{}\n", run.output_summary)),
    "Artifacts".to_string(),
    render_block(&render_artifacts(run)),
    "Events".to_string(),
    render_block(&render_events(run)),
  ];

  sections.join("\n")
}

fn render_block(raw: &str) -> String {
  raw
    .lines()
    .map(|line| format!("  {line}"))
    .collect::<Vec<_>>()
    .join("\n")
}
