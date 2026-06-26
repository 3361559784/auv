use std::collections::BTreeSet;
use std::fs;
use std::io::BufReader;
use std::path::{Path, PathBuf};

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::training_result::{
  TrainingResultArtifactRecord, TrainingResultManifest, TrainingResultStatus,
};

pub type TrainingResultArtifactFetchResult<T> = Result<T, String>;

pub const TRAINING_RESULT_ARTIFACT_FETCH_MANIFEST_SCHEMA_VERSION: u32 = 1;
pub const TRAINING_RESULT_ARTIFACT_FETCH_INSPECT_REPORT_SCHEMA_VERSION: u32 = 1;
const STATUS_SNAPSHOT_FILE: &str = "job_status.json";
const RESULT_CONFIG_FILE: &str = "config.yml";
const RESULT_MODELS_DIR: &str = "nerfstudio_models";
const NORMALIZED_RESULT_ROOT_DIR: &str = "normalized-result";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TrainingResultArtifactFetchInputs {
  pub training_result_manifest_path: PathBuf,
  pub output_dir: PathBuf,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrainingResultArtifactFetchOutput {
  pub output_dir: PathBuf,
  pub manifest_path: PathBuf,
  pub inspect_report_path: PathBuf,
  pub normalized_result_dir: PathBuf,
  pub manifest: TrainingResultArtifactFetchManifest,
  pub inspect_report: TrainingResultArtifactFetchInspectReport,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrainingResultArtifactFetchManifest {
  pub schema_version: u32,
  pub generated_at_millis: u64,
  pub source_training_result_manifest_path: String,
  pub source_training_job_manifest_path: String,
  pub source_training_launch_plan_path: String,
  pub source_training_package_manifest_path: String,
  pub source_scene_packet_manifest_path: String,
  pub source_bundle_manifest_paths: Vec<String>,
  pub source_run_ids: Vec<String>,
  pub trainer_backend: String,
  pub job_backend: String,
  pub source_job_status: TrainingResultStatus,
  pub source_result_status: TrainingResultStatus,
  #[serde(default)]
  pub source_result_status_reason: Option<String>,
  pub source_result_dir: String,
  pub normalized_result_dir: String,
  pub normalized_artifacts: Vec<TrainingResultNormalizedArtifactRecord>,
  pub known_limits: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrainingResultArtifactFetchInspectReport {
  pub schema_version: u32,
  pub generated_at_millis: u64,
  pub training_result_artifact_fetch_manifest_path: String,
  pub source_training_result_manifest_path: String,
  pub source_training_job_manifest_path: String,
  pub source_training_launch_plan_path: String,
  pub source_scene_packet_manifest_path: String,
  pub source_bundle_manifest_paths: Vec<String>,
  pub source_run_ids: Vec<String>,
  pub trainer_backend: String,
  pub job_backend: String,
  pub source_job_status: TrainingResultStatus,
  pub source_result_status: TrainingResultStatus,
  #[serde(default)]
  pub source_result_status_reason: Option<String>,
  pub fetch_status: TrainingResultArtifactFetchStatus,
  #[serde(default)]
  pub fetch_reason: Option<TrainingResultArtifactFetchReason>,
  pub source_result_dir: String,
  pub normalized_result_dir: String,
  pub source_result_dir_exists: bool,
  pub required_artifacts_present: bool,
  pub normalized_artifact_count: usize,
  pub warnings: Vec<String>,
  pub known_limits: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrainingResultNormalizedArtifactRecord {
  pub kind: TrainingResultNormalizedArtifactKind,
  pub relative_path: String,
  pub absolute_path: String,
  pub readable: bool,
  #[serde(default)]
  pub byte_size: Option<u64>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrainingResultNormalizedArtifactKind {
  Config,
  ModelsDirectory,
  StatusSnapshot,
}

impl TrainingResultNormalizedArtifactKind {
  pub fn as_str(self) -> &'static str {
    match self {
      Self::Config => "config",
      Self::ModelsDirectory => "models_directory",
      Self::StatusSnapshot => "status_snapshot",
    }
  }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrainingResultArtifactFetchStatus {
  Blocked,
  Failed,
  Succeeded,
}

impl TrainingResultArtifactFetchStatus {
  pub fn as_str(self) -> &'static str {
    match self {
      Self::Blocked => "blocked",
      Self::Failed => "failed",
      Self::Succeeded => "succeeded",
    }
  }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrainingResultArtifactFetchReason {
  SourceResultBlocked,
  SourceResultArtifactsMissing,
  SourceResultDirectoryMissing,
  CopyFailed,
}

impl TrainingResultArtifactFetchReason {
  pub fn as_str(self) -> &'static str {
    match self {
      Self::SourceResultBlocked => "source_result_blocked",
      Self::SourceResultArtifactsMissing => "source_result_artifacts_missing",
      Self::SourceResultDirectoryMissing => "source_result_directory_missing",
      Self::CopyFailed => "copy_failed",
    }
  }
}

pub fn fetch_3dgs_training_result_artifacts(
  inputs: TrainingResultArtifactFetchInputs,
) -> TrainingResultArtifactFetchResult<TrainingResultArtifactFetchOutput> {
  let result_manifest = read_json_file::<TrainingResultManifest>(
    &inputs.training_result_manifest_path,
    "MC-7 D7 training result manifest",
  )?;

  let mut warnings = BTreeSet::new();
  let mut known_limits = BTreeSet::new();
  known_limits.extend(result_manifest.known_limits.iter().cloned());
  known_limits.insert(
    "MC-7 D11 fetches and normalizes trainer result artifacts only; it does not grade model quality or claim downstream splat usability"
      .to_string(),
  );

  let source_result_dir = PathBuf::from(&result_manifest.result_dir);
  let source_result_dir_exists = source_result_dir.is_dir();
  let normalized_result_dir = inputs.output_dir.join(NORMALIZED_RESULT_ROOT_DIR);
  let generated_at_millis = auv_tracing_driver::now_millis();
  let manifest_path = inputs
    .output_dir
    .join("minecraft-3dgs-training-result-artifact-manifest.json");
  let inspect_report_path = inputs
    .output_dir
    .join("minecraft-3dgs-training-result-artifact-inspect.json");

  let source_result_status_reason = result_manifest
    .status
    .ne(&TrainingResultStatus::Succeeded)
    .then_some("result_not_succeeded".to_string())
    .or_else(|| infer_source_result_reason(&result_manifest));

  let required_artifacts_present = has_required_source_artifacts(&result_manifest.result_artifacts);

  let (fetch_status, fetch_reason, normalized_artifacts) = if result_manifest.status
    != TrainingResultStatus::Succeeded
  {
    warnings.insert(
      "source training result is not succeeded; D11 records blocked fetch evidence instead of claiming normalized artifacts"
        .to_string(),
    );
    (
      TrainingResultArtifactFetchStatus::Blocked,
      Some(TrainingResultArtifactFetchReason::SourceResultBlocked),
      Vec::new(),
    )
  } else if !source_result_dir_exists {
    warnings.insert(
      "source result directory is missing; D11 cannot fetch normalized artifacts".to_string(),
    );
    (
      TrainingResultArtifactFetchStatus::Failed,
      Some(TrainingResultArtifactFetchReason::SourceResultDirectoryMissing),
      Vec::new(),
    )
  } else if !required_artifacts_present {
    warnings.insert(
      "source result manifest does not expose the required config/models artifacts for normalization"
        .to_string(),
    );
    (
      TrainingResultArtifactFetchStatus::Failed,
      Some(TrainingResultArtifactFetchReason::SourceResultArtifactsMissing),
      Vec::new(),
    )
  } else {
    match normalize_result_artifacts(&source_result_dir, &normalized_result_dir) {
      Ok(artifacts) => (
        TrainingResultArtifactFetchStatus::Succeeded,
        None,
        artifacts,
      ),
      Err(error) => {
        warnings.insert(error);
        (
          TrainingResultArtifactFetchStatus::Failed,
          Some(TrainingResultArtifactFetchReason::CopyFailed),
          Vec::new(),
        )
      }
    }
  };

  let manifest = TrainingResultArtifactFetchManifest {
    schema_version: TRAINING_RESULT_ARTIFACT_FETCH_MANIFEST_SCHEMA_VERSION,
    generated_at_millis,
    source_training_result_manifest_path: inputs
      .training_result_manifest_path
      .to_string_lossy()
      .into_owned(),
    source_training_job_manifest_path: result_manifest.source_training_job_manifest_path.clone(),
    source_training_launch_plan_path: result_manifest.source_training_launch_plan_path.clone(),
    source_training_package_manifest_path: result_manifest
      .source_training_package_manifest_path
      .clone(),
    source_scene_packet_manifest_path: result_manifest.source_scene_packet_manifest_path.clone(),
    source_bundle_manifest_paths: result_manifest.source_bundle_manifest_paths.clone(),
    source_run_ids: result_manifest.source_run_ids.clone(),
    trainer_backend: result_manifest.trainer_backend.clone(),
    job_backend: result_manifest.job_backend.clone(),
    source_job_status: map_training_status(result_manifest.source_job_status),
    source_result_status: result_manifest.status,
    source_result_status_reason: source_result_status_reason.clone(),
    source_result_dir: result_manifest.result_dir.clone(),
    normalized_result_dir: normalized_result_dir.to_string_lossy().into_owned(),
    normalized_artifacts: normalized_artifacts.clone(),
    known_limits: known_limits.iter().cloned().collect(),
  };
  write_json(
    &manifest_path,
    &manifest,
    "MC-7 D11 training result artifact fetch manifest",
  )?;

  let inspect_report = TrainingResultArtifactFetchInspectReport {
    schema_version: TRAINING_RESULT_ARTIFACT_FETCH_INSPECT_REPORT_SCHEMA_VERSION,
    generated_at_millis,
    training_result_artifact_fetch_manifest_path: manifest_path.to_string_lossy().into_owned(),
    source_training_result_manifest_path: inputs
      .training_result_manifest_path
      .to_string_lossy()
      .into_owned(),
    source_training_job_manifest_path: result_manifest.source_training_job_manifest_path.clone(),
    source_training_launch_plan_path: result_manifest.source_training_launch_plan_path.clone(),
    source_scene_packet_manifest_path: result_manifest.source_scene_packet_manifest_path.clone(),
    source_bundle_manifest_paths: result_manifest.source_bundle_manifest_paths.clone(),
    source_run_ids: result_manifest.source_run_ids.clone(),
    trainer_backend: result_manifest.trainer_backend.clone(),
    job_backend: result_manifest.job_backend.clone(),
    source_job_status: map_training_status(result_manifest.source_job_status),
    source_result_status: result_manifest.status,
    source_result_status_reason,
    fetch_status,
    fetch_reason,
    source_result_dir: result_manifest.result_dir.clone(),
    normalized_result_dir: normalized_result_dir.to_string_lossy().into_owned(),
    source_result_dir_exists,
    required_artifacts_present,
    normalized_artifact_count: normalized_artifacts.len(),
    warnings: warnings.iter().cloned().collect(),
    known_limits: known_limits.iter().cloned().collect(),
  };
  write_json(
    &inspect_report_path,
    &inspect_report,
    "MC-7 D11 training result artifact fetch inspect JSON",
  )?;

  Ok(TrainingResultArtifactFetchOutput {
    output_dir: inputs.output_dir,
    manifest_path,
    inspect_report_path,
    normalized_result_dir,
    manifest,
    inspect_report,
  })
}

fn infer_source_result_reason(manifest: &TrainingResultManifest) -> Option<String> {
  if !PathBuf::from(&manifest.result_dir).is_dir() {
    return Some("result_directory_missing".to_string());
  }
  if !has_required_source_artifacts(&manifest.result_artifacts) {
    return Some("result_artifacts_missing".to_string());
  }
  None
}

fn has_required_source_artifacts(artifacts: &[TrainingResultArtifactRecord]) -> bool {
  let has_config = artifacts
    .iter()
    .any(|artifact| artifact.relative_path == RESULT_CONFIG_FILE && artifact.readable);
  let has_models = artifacts
    .iter()
    .any(|artifact| artifact.relative_path == RESULT_MODELS_DIR && artifact.readable);
  has_config && has_models
}

fn normalize_result_artifacts(
  source_result_dir: &Path,
  normalized_result_dir: &Path,
) -> TrainingResultArtifactFetchResult<Vec<TrainingResultNormalizedArtifactRecord>> {
  fs::create_dir_all(normalized_result_dir).map_err(|error| {
    format!(
      "failed to create MC-7 D11 normalized result directory {}: {error}",
      normalized_result_dir.display()
    )
  })?;

  let mut artifacts = Vec::new();

  let source_config = source_result_dir.join(RESULT_CONFIG_FILE);
  let target_config = normalized_result_dir.join(RESULT_CONFIG_FILE);
  copy_file(&source_config, &target_config)?;
  artifacts.push(normalized_artifact_record(
    normalized_result_dir,
    &target_config,
    TrainingResultNormalizedArtifactKind::Config,
  ));

  let source_models_dir = source_result_dir.join(RESULT_MODELS_DIR);
  let target_models_dir = normalized_result_dir.join(RESULT_MODELS_DIR);
  copy_directory_recursive(&source_models_dir, &target_models_dir)?;
  artifacts.push(normalized_artifact_record(
    normalized_result_dir,
    &target_models_dir,
    TrainingResultNormalizedArtifactKind::ModelsDirectory,
  ));

  let source_status = source_result_dir.join(STATUS_SNAPSHOT_FILE);
  if source_status.exists() {
    let target_status = normalized_result_dir.join(STATUS_SNAPSHOT_FILE);
    copy_file(&source_status, &target_status)?;
    artifacts.push(normalized_artifact_record(
      normalized_result_dir,
      &target_status,
      TrainingResultNormalizedArtifactKind::StatusSnapshot,
    ));
  }

  Ok(artifacts)
}

fn normalized_artifact_record(
  normalized_result_dir: &Path,
  path: &Path,
  kind: TrainingResultNormalizedArtifactKind,
) -> TrainingResultNormalizedArtifactRecord {
  let metadata = fs::metadata(path).ok();
  let relative_path = path
    .strip_prefix(normalized_result_dir)
    .map(|value| value.to_string_lossy().into_owned())
    .unwrap_or_else(|_| path.to_string_lossy().into_owned());
  TrainingResultNormalizedArtifactRecord {
    kind,
    relative_path,
    absolute_path: path.to_string_lossy().into_owned(),
    readable: metadata.is_some(),
    byte_size: metadata.and_then(|metadata| metadata.is_file().then_some(metadata.len())),
  }
}

fn copy_file(source: &Path, target: &Path) -> TrainingResultArtifactFetchResult<()> {
  if !source.is_file() {
    return Err(format!(
      "required source file {} is missing or unreadable",
      source.display()
    ));
  }
  if let Some(parent) = target.parent() {
    fs::create_dir_all(parent).map_err(|error| {
      format!(
        "failed to create normalized artifact directory {}: {error}",
        parent.display()
      )
    })?;
  }
  fs::copy(source, target).map_err(|error| {
    format!(
      "failed to copy source file {} into normalized artifact {}: {error}",
      source.display(),
      target.display()
    )
  })?;
  Ok(())
}

fn copy_directory_recursive(source: &Path, target: &Path) -> TrainingResultArtifactFetchResult<()> {
  if !source.is_dir() {
    return Err(format!(
      "required source directory {} is missing or unreadable",
      source.display()
    ));
  }
  fs::create_dir_all(target).map_err(|error| {
    format!(
      "failed to create normalized directory {}: {error}",
      target.display()
    )
  })?;
  for entry in fs::read_dir(source).map_err(|error| {
    format!(
      "failed to read source result directory {}: {error}",
      source.display()
    )
  })? {
    let entry = entry.map_err(|error| {
      format!(
        "failed to enumerate source result directory {}: {error}",
        source.display()
      )
    })?;
    let source_path = entry.path();
    let target_path = target.join(entry.file_name());
    let file_type = entry.file_type().map_err(|error| {
      format!(
        "failed to read source artifact type {}: {error}",
        source_path.display()
      )
    })?;
    if file_type.is_dir() {
      copy_directory_recursive(&source_path, &target_path)?;
    } else if file_type.is_file() {
      copy_file(&source_path, &target_path)?;
    }
  }
  Ok(())
}

fn map_training_status(
  status: crate::training_job::TrainingLaunchJobStatus,
) -> TrainingResultStatus {
  match status {
    crate::training_job::TrainingLaunchJobStatus::Queued => TrainingResultStatus::Queued,
    crate::training_job::TrainingLaunchJobStatus::Submitted => TrainingResultStatus::Submitted,
    crate::training_job::TrainingLaunchJobStatus::Blocked => TrainingResultStatus::Blocked,
    crate::training_job::TrainingLaunchJobStatus::Failed => TrainingResultStatus::Failed,
    crate::training_job::TrainingLaunchJobStatus::Succeeded => TrainingResultStatus::Succeeded,
  }
}

fn write_json(
  path: &Path,
  value: &impl Serialize,
  label: &str,
) -> TrainingResultArtifactFetchResult<()> {
  if let Some(parent) = path.parent() {
    fs::create_dir_all(parent).map_err(|error| {
      format!(
        "failed to create {label} directory {}: {error}",
        parent.display()
      )
    })?;
  }
  let json = serde_json::to_string_pretty(value)
    .map(|mut json| {
      json.push('\n');
      json
    })
    .map_err(|error| format!("failed to serialize {label}: {error}"))?;
  fs::write(path, json.as_bytes())
    .map_err(|error| format!("failed to write {label} {}: {error}", path.display()))
}

fn read_json_file<T: DeserializeOwned>(
  path: &Path,
  label: &str,
) -> TrainingResultArtifactFetchResult<T> {
  let file = fs::File::open(path)
    .map_err(|error| format!("failed to open {label} {}: {error}", path.display()))?;
  serde_json::from_reader(BufReader::new(file))
    .map_err(|error| format!("failed to parse {label} {}: {error}", path.display()))
}

#[cfg(test)]
mod tests {
  use super::*;
  use tempfile::TempDir;

  fn write_training_result_manifest_fixture(
    temp: &TempDir,
    source_result_status: TrainingResultStatus,
    create_result_dir: bool,
    include_config: bool,
    include_models: bool,
    include_status_snapshot: bool,
  ) -> PathBuf {
    let result_dir = temp.path().join("trainer-output/nerfstudio-splatfacto");
    if create_result_dir {
      fs::create_dir_all(&result_dir).expect("result dir");
    }
    let mut result_artifacts = Vec::new();
    if include_config {
      fs::write(
        result_dir.join(RESULT_CONFIG_FILE),
        b"trainer: splatfacto\n",
      )
      .expect("config");
      result_artifacts.push(TrainingResultArtifactRecord {
        relative_path: RESULT_CONFIG_FILE.to_string(),
        absolute_path: result_dir.join(RESULT_CONFIG_FILE).display().to_string(),
        readable: true,
        byte_size: Some(21),
      });
    }
    if include_models {
      fs::create_dir_all(result_dir.join(RESULT_MODELS_DIR)).expect("models dir");
      fs::write(
        result_dir.join(RESULT_MODELS_DIR).join("step-000001.ckpt"),
        b"checkpoint",
      )
      .expect("checkpoint");
      result_artifacts.push(TrainingResultArtifactRecord {
        relative_path: RESULT_MODELS_DIR.to_string(),
        absolute_path: result_dir.join(RESULT_MODELS_DIR).display().to_string(),
        readable: true,
        byte_size: None,
      });
    }
    if include_status_snapshot {
      fs::write(
        result_dir.join(STATUS_SNAPSHOT_FILE),
        br#"{"status":"succeeded"}"#,
      )
      .expect("status snapshot");
      result_artifacts.push(TrainingResultArtifactRecord {
        relative_path: STATUS_SNAPSHOT_FILE.to_string(),
        absolute_path: result_dir.join(STATUS_SNAPSHOT_FILE).display().to_string(),
        readable: true,
        byte_size: Some(22),
      });
    }

    let manifest = TrainingResultManifest {
      schema_version: 1,
      generated_at_millis: 1,
      source_training_job_manifest_path: "/tmp/job.json".to_string(),
      source_training_launch_plan_path: "/tmp/launch-plan.json".to_string(),
      source_training_package_manifest_path: "/tmp/package.json".to_string(),
      source_scene_packet_manifest_path: "/tmp/scene.json".to_string(),
      source_bundle_manifest_paths: vec!["/tmp/bundle.json".to_string()],
      source_run_ids: vec!["run-1".to_string()],
      trainer_backend: "nerfstudio.splatfacto".to_string(),
      job_backend: "remote".to_string(),
      job_submission_endpoint: "https://jobs.example.test/v1".to_string(),
      source_job_status: crate::training_job::TrainingLaunchJobStatus::Submitted,
      status: source_result_status,
      job_id: "job-123".to_string(),
      job_url: Some("https://jobs.example.test/jobs/job-123".to_string()),
      result_dir: result_dir.display().to_string(),
      exported_frame_count: 2,
      skipped_frame_count: 0,
      result_artifacts,
      known_limits: vec!["limit-a".to_string()],
    };
    let manifest_path = temp.path().join("minecraft-3dgs-training-result.json");
    fs::write(
      &manifest_path,
      serde_json::to_vec_pretty(&manifest).expect("serialize manifest"),
    )
    .expect("write manifest");
    manifest_path
  }

  #[test]
  fn fetch_training_result_artifacts_happy_path_writes_normalized_outputs() {
    let temp = tempfile::tempdir().expect("temp dir");
    let manifest_path = write_training_result_manifest_fixture(
      &temp,
      TrainingResultStatus::Succeeded,
      true,
      true,
      true,
      true,
    );

    let output = fetch_3dgs_training_result_artifacts(TrainingResultArtifactFetchInputs {
      training_result_manifest_path: manifest_path,
      output_dir: temp.path().join("normalized"),
    })
    .expect("fetch should succeed");

    assert_eq!(
      output.inspect_report.fetch_status,
      TrainingResultArtifactFetchStatus::Succeeded
    );
    assert!(output.manifest_path.is_file());
    assert!(output.inspect_report_path.is_file());
    assert!(
      output
        .normalized_result_dir
        .join(RESULT_CONFIG_FILE)
        .is_file()
    );
    assert!(
      output
        .normalized_result_dir
        .join(RESULT_MODELS_DIR)
        .is_dir()
    );
    assert_eq!(output.manifest.normalized_artifacts.len(), 3);
  }

  #[test]
  fn fetch_training_result_artifacts_blocks_when_source_result_not_succeeded() {
    let temp = tempfile::tempdir().expect("temp dir");
    let manifest_path = write_training_result_manifest_fixture(
      &temp,
      TrainingResultStatus::Blocked,
      true,
      true,
      true,
      false,
    );

    let output = fetch_3dgs_training_result_artifacts(TrainingResultArtifactFetchInputs {
      training_result_manifest_path: manifest_path,
      output_dir: temp.path().join("normalized"),
    })
    .expect("blocked fetch should still write outputs");

    assert_eq!(
      output.inspect_report.fetch_status,
      TrainingResultArtifactFetchStatus::Blocked
    );
    assert_eq!(
      output.inspect_report.fetch_reason,
      Some(TrainingResultArtifactFetchReason::SourceResultBlocked)
    );
    assert!(output.manifest.normalized_artifacts.is_empty());
  }

  #[test]
  fn fetch_training_result_artifacts_fails_when_required_artifacts_missing() {
    let temp = tempfile::tempdir().expect("temp dir");
    let manifest_path = write_training_result_manifest_fixture(
      &temp,
      TrainingResultStatus::Succeeded,
      true,
      true,
      false,
      false,
    );

    let output = fetch_3dgs_training_result_artifacts(TrainingResultArtifactFetchInputs {
      training_result_manifest_path: manifest_path,
      output_dir: temp.path().join("normalized"),
    })
    .expect("failed fetch should still write outputs");

    assert_eq!(
      output.inspect_report.fetch_status,
      TrainingResultArtifactFetchStatus::Failed
    );
    assert_eq!(
      output.inspect_report.fetch_reason,
      Some(TrainingResultArtifactFetchReason::SourceResultArtifactsMissing)
    );
    assert!(output.manifest.normalized_artifacts.is_empty());
  }
}
