use std::collections::BTreeMap;
use std::thread;
use std::time::Duration;

use serde_json::json;

use super::super::capture::xcap_backend;
use super::super::support::{
  artifacts::{build_text_artifact, sanitize_file_component, screenshot_temp_path},
  call::{app_identifier, optional_non_empty_string, optional_positive_u64},
  geometry::render_rect_compact,
  typed_capture::{TypedWindowCaptureObservation, capture_window_with_typed_session},
};
use super::super::{DriverCall, DriverResponse};
use crate::model::{AuvResult, ProducedArtifact};

pub(crate) fn teach_click(call: &DriverCall) -> AuvResult<DriverResponse> {
  let app = app_identifier(call)
    .filter(|value| !value.is_empty())
    .ok_or_else(|| {
      "operation requires --target <application-id> or --app <application-id>".to_string()
    })?;
  let timeout_ms = optional_positive_u64(call, "timeout_ms")?.unwrap_or(10_000);
  let first_after_ms = optional_positive_u64(call, "first_after_ms")?.unwrap_or(150);
  let second_after_ms = optional_positive_u64(call, "second_after_ms")?.unwrap_or(250);
  if second_after_ms < first_after_ms {
    return Err(
      "invalid --second_after_ms: expected a value greater than or equal to --first_after_ms"
        .to_string(),
    );
  }
  let prompt = optional_non_empty_string(call, "prompt")
    .unwrap_or_else(|| "Click Ready, then click the UI target you want AUV to learn.".to_string());
  let label_base = format!("teach-click-{}", sanitize_file_component(&app));

  let before = capture_window_with_typed_session(call, &format!("{label_base}-before"))?;
  let before_artifact = screenshot_artifact(&before, &format!("{label_base}-before"), "before")?;

  let taught = auv_driver_macos::native::pointer::teach_next_click(&prompt, timeout_ms)?;
  let bounds = &before.candidate.window_ref.bounds;
  let window_x = taught.x - bounds.x as f64;
  let window_y = taught.y - bounds.y as f64;
  let inside_window = window_x >= 0.0
    && window_y >= 0.0
    && window_x <= bounds.width as f64
    && window_y <= bounds.height as f64;

  thread::sleep(Duration::from_millis(first_after_ms));
  let after_first =
    capture_window_with_typed_session(call, &format!("{label_base}-after-{first_after_ms}ms"))?;
  let after_first_artifact = screenshot_artifact(
    &after_first,
    &format!("{label_base}-after-{first_after_ms}ms"),
    "first after-click frame",
  )?;

  thread::sleep(Duration::from_millis(second_after_ms - first_after_ms));
  let after_second =
    capture_window_with_typed_session(call, &format!("{label_base}-after-{second_after_ms}ms"))?;
  let after_second_artifact = screenshot_artifact(
    &after_second,
    &format!("{label_base}-after-{second_after_ms}ms"),
    "second after-click frame",
  )?;

  let report_json = build_text_artifact(
    "teach-click-report",
    "json",
    &label_base,
    serde_json::to_string_pretty(&json!({
      "app": app,
      "window_ref": format!("window_{}", before.candidate.window_ref.window_number),
      "native_window_id": before.candidate.native_window_id,
      "owner_pid": before.candidate.window_ref.owner_pid,
      "owner_bundle_id": before.candidate.window_ref.owner_bundle_id,
      "window_title": before.candidate.window_ref.title,
      "window_bounds": {
        "x": bounds.x,
        "y": bounds.y,
        "width": bounds.width,
        "height": bounds.height
      },
      "click": {
        "global_logical": {
          "x": taught.x,
          "y": taught.y
        },
        "window_local": {
          "x": window_x,
          "y": window_y
        },
        "button_code": taught.button_code,
        "captured_at_unix_ms": taught.captured_at_unix_ms,
        "inside_window": inside_window
      },
      "timing": {
        "timeout_ms": timeout_ms,
        "first_after_ms": first_after_ms,
        "second_after_ms": second_after_ms
      }
    }))
    .map_err(|error| format!("failed to encode teach-click report JSON: {error}"))?,
    "Machine-readable taught click coordinates and capture timing.",
  )?;
  let report_text = build_text_artifact(
    "teach-click-report",
    "txt",
    &format!("{label_base}-report"),
    [
      format!("app={app}"),
      format!(
        "windowRef=window_{}",
        before.candidate.window_ref.window_number
      ),
      format!(
        "nativeWindowId={}",
        before.candidate.native_window_id.as_deref().unwrap_or("")
      ),
      format!("ownerPid={}", before.candidate.window_ref.owner_pid),
      format!(
        "ownerBundleId={}",
        before.candidate.window_ref.owner_bundle_id
      ),
      format!("windowTitle={}", before.candidate.window_ref.title),
      format!("windowBounds={}", render_rect_compact(bounds)),
      format!("clickGlobalLogical={:.3},{:.3}", taught.x, taught.y),
      format!("clickWindowLocal={window_x:.3},{window_y:.3}"),
      format!("clickButtonCode={}", taught.button_code),
      format!("clickCapturedAtUnixMs={}", taught.captured_at_unix_ms),
      format!("insideWindow={inside_window}"),
      format!("timeoutMs={timeout_ms}"),
      format!("firstAfterMs={first_after_ms}"),
      format!("secondAfterMs={second_after_ms}"),
    ]
    .join("\n"),
    "Human-readable taught click coordinates and capture timing.",
  )?;

  let mut signals = BTreeMap::new();
  signals.insert("input.learning.mode".to_string(), "teach-click".to_string());
  signals.insert(
    "input.click.inside_window".to_string(),
    inside_window.to_string(),
  );

  Ok(DriverResponse {
    summary: format!(
      "Captured taught click for {} at window-local ({window_x:.3}, {window_y:.3}); insideWindow={inside_window}.",
      app
    ),
    backend: Some("macos.desktop.teach-click".to_string()),
    signals,
    notes: vec![
      format!("app={app}"),
      format!(
        "windowRef=window_{}",
        before.candidate.window_ref.window_number
      ),
      format!("windowBounds={}", render_rect_compact(bounds)),
      format!("clickGlobalLogical={:.3},{:.3}", taught.x, taught.y),
      format!("clickWindowLocal={window_x:.3},{window_y:.3}"),
      format!("insideWindow={inside_window}"),
      format!("firstAfterMs={first_after_ms}"),
      format!("secondAfterMs={second_after_ms}"),
    ],
    artifacts: vec![
      before_artifact,
      after_first_artifact,
      after_second_artifact,
      report_json,
      report_text,
    ],
  })
}

fn screenshot_artifact(
  observation: &TypedWindowCaptureObservation,
  label: &str,
  note_label: &str,
) -> AuvResult<ProducedArtifact> {
  let screenshot_path = screenshot_temp_path(label);
  xcap_backend::save_rgba_image(observation.capture.image.clone(), &screenshot_path)?;
  Ok(ProducedArtifact {
    kind: "screenshot".to_string(),
    source_path: screenshot_path,
    preferred_name: format!("{}.png", sanitize_file_component(label)),
    note: Some(format!("Teach-click {note_label} window screenshot.")),
  })
}
