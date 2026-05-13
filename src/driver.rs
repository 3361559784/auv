use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

use crate::model::{
  AuvResult,
  DriverCall,
  DriverDescriptor,
  DriverResponse,
  ProducedArtifact,
  now_millis,
};

pub trait Driver {
  fn descriptor(&self) -> DriverDescriptor;
  fn invoke(&self, call: &DriverCall) -> AuvResult<DriverResponse>;
}

pub struct DriverRegistry {
  drivers: HashMap<String, Box<dyn Driver>>,
}

impl DriverRegistry {
  pub fn new(drivers: Vec<Box<dyn Driver>>) -> Self {
    let mut registry = HashMap::new();
    for driver in drivers {
      let descriptor = driver.descriptor();
      registry.insert(descriptor.id.to_string(), driver);
    }
    Self { drivers: registry }
  }

  pub fn get(&self, driver_id: &str) -> Option<&dyn Driver> {
    self.drivers.get(driver_id).map(Box::as_ref)
  }

  pub fn descriptors(&self) -> Vec<DriverDescriptor> {
    let mut descriptors = self
      .drivers
      .values()
      .map(|driver| driver.descriptor())
      .collect::<Vec<_>>();
    descriptors.sort_by(|left, right| left.id.cmp(right.id));
    descriptors
  }
}

pub fn default_driver_registry() -> DriverRegistry {
  DriverRegistry::new(vec![
    Box::new(FixtureObserveDriver),
    Box::new(MacOsScreenshotDriver),
  ])
}

struct FixtureObserveDriver;

impl Driver for FixtureObserveDriver {
  fn descriptor(&self) -> DriverDescriptor {
    DriverDescriptor {
      id: "fixture.observe",
      summary: "Non-UI fixture driver that proves invoke -> run -> inspect without platform side effects.",
      capabilities: &["observe.fixture"],
      donor_boundary: "AUV-native fixture driver; useful for validating the shared execution substrate before real app drivers land.",
    }
  }

  fn invoke(&self, call: &DriverCall) -> AuvResult<DriverResponse> {
    if call.operation != "observe_fixture_scene" {
      return Err(format!(
        "driver fixture.observe does not support operation {}",
        call.operation
      ));
    }

    let target = call
      .target
      .application_id
      .clone()
      .unwrap_or_else(|| "fixture://default".to_string());
    let label = call
      .inputs
      .get("label")
      .cloned()
      .unwrap_or_else(|| "fixture-observation".to_string());

    Ok(DriverResponse {
      summary: format!(
        "Observed deterministic fixture scene for target {} with label {}.",
        target, label
      ),
      backend: Some("fixture.static".to_string()),
      notes: vec![
        "This command does not touch the real desktop.".to_string(),
        "Use it to verify that implicit run creation and inspect output stay stable.".to_string(),
      ],
      artifacts: Vec::new(),
    })
  }
}

struct MacOsScreenshotDriver;

impl Driver for MacOsScreenshotDriver {
  fn descriptor(&self) -> DriverDescriptor {
    DriverDescriptor {
      id: "macos.screenshot",
      summary: "Capture screenshots on macOS through the shared driver protocol.",
      capabilities: &["observe.screenshot", "artifact.image"],
      donor_boundary: "Borrow the driver layer idea from AIRI desktop, but keep MCP/server orchestration, approval, and workflow shells out of AUV core.",
    }
  }

  fn invoke(&self, call: &DriverCall) -> AuvResult<DriverResponse> {
    if call.operation != "capture_screen" {
      return Err(format!(
        "driver macos.screenshot does not support operation {}",
        call.operation
      ));
    }

    if env::consts::OS != "macos" {
      return Err("macos.screenshot is only available on macOS".to_string());
    }

    let label = sanitize_file_component(
      call.inputs
        .get("label")
        .map(String::as_str)
        .unwrap_or("desktop"),
    );
    let temporary_path = screenshot_temp_path(&label);
    let output = Command::new("screencapture")
      .arg("-x")
      .arg(&temporary_path)
      .output()
      .map_err(|error| format!("failed to spawn screencapture: {error}"))?;

    if !output.status.success() {
      let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
      return Err(format!(
        "screencapture exited with status {}: {}",
        output.status,
        if stderr.is_empty() { "no stderr output" } else { &stderr }
      ));
    }

    if !temporary_path.exists() {
      return Err(format!(
        "screencapture reported success but no image was created at {}",
        temporary_path.display()
      ));
    }

    Ok(DriverResponse {
      summary: "Captured one desktop screenshot through the shared AUV runtime.".to_string(),
      backend: Some("macos.screencapture".to_string()),
      notes: vec![
        format!(
          "Temporary screenshot created at {} before artifact ingestion.",
          temporary_path.display()
        ),
        "This is intentionally a driver-level primitive; it does not pull in AIRI MCP tools, action executors, or approval queues.".to_string(),
      ],
      artifacts: vec![ProducedArtifact {
        kind: "screenshot".to_string(),
        source_path: temporary_path,
        preferred_name: format!("{label}.png"),
        note: Some("Phase-1 screenshot artifact captured through the macOS first-party driver.".to_string()),
      }],
    })
  }
}

fn screenshot_temp_path(label: &str) -> PathBuf {
  env::temp_dir().join(format!("auv-{}-{}.png", label, now_millis()))
}

fn sanitize_file_component(raw: &str) -> String {
  let sanitized = raw
    .chars()
    .map(|character| match character {
      'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' => character,
      _ => '-',
    })
    .collect::<String>()
    .trim_matches('-')
    .to_string();

  if sanitized.is_empty() {
    "artifact".to_string()
  }
  else {
    sanitized
  }
}

pub fn copy_file(source: &PathBuf, destination: &PathBuf) -> AuvResult<()> {
  if let Some(parent) = destination.parent() {
    fs::create_dir_all(parent)
      .map_err(|error| format!("failed to create artifact directory {}: {error}", parent.display()))?;
  }

  fs::copy(source, destination).map_err(|error| {
    format!(
      "failed to copy artifact from {} to {}: {error}",
      source.display(),
      destination.display()
    )
  })?;

  Ok(())
}

pub fn sanitized_artifact_name(raw: &str) -> String {
  sanitize_file_component(raw)
}
