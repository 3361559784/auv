use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use crate::driver::{copy_file, sanitized_artifact_name};
use crate::model::{ArtifactRecord, AuvResult, ProducedArtifact, RunRecord};

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
    let run_directory = self.root.join("runs").join(&run.run_id);
    fs::create_dir_all(&run_directory).map_err(|error| {
      format!(
        "failed to create run directory {}: {error}",
        run_directory.display()
      )
    })?;

    fs::write(run_directory.join("meta.txt"), render_meta(run))
      .map_err(|error| format!("failed to write run metadata: {error}"))?;
    fs::write(run_directory.join("inputs.txt"), render_inputs(&run.inputs))
      .map_err(|error| format!("failed to write run inputs: {error}"))?;
    fs::write(run_directory.join("events.log"), render_events(run))
      .map_err(|error| format!("failed to write run events: {error}"))?;
    fs::write(run_directory.join("artifacts.txt"), render_artifacts(run))
      .map_err(|error| format!("failed to write artifact manifest: {error}"))?;
    fs::write(run_directory.join("output.txt"), format!("{}\n", run.output_summary))
      .map_err(|error| format!("failed to write run output: {error}"))?;
    fs::write(run_directory.join("inspect.txt"), render_inspection(run))
      .map_err(|error| format!("failed to write inspect snapshot: {error}"))?;

    Ok(())
  }

  pub fn render_inspection(&self, run_id: &str) -> AuvResult<String> {
    let inspection_path = self
      .root
      .join("runs")
      .join(run_id)
      .join("inspect.txt");

    fs::read_to_string(&inspection_path).map_err(|error| {
      format!(
        "failed to read inspect snapshot {}: {error}",
        inspection_path.display()
      )
    })
  }
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
