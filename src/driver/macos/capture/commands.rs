// File: src/driver/macos/capture/commands.rs
use super::artifact::{render_capture_contract_json, render_capture_contract_text};
use super::types::{
  CaptureBackend, CaptureContract, CaptureSource, CoordinateSpace, DisplayDescriptor, Rect,
  Scale2D, Size, capture_error,
};
use super::xcap_backend;
use crate::driver::macos::support::{
  artifacts::{build_text_artifact, sanitize_file_component, screenshot_temp_path},
  call::{optional_bool, optional_string, required_f64},
  display::maybe_activate_target_app_for_observation,
  typed_capture::capture_window_with_typed_session,
};
use crate::driver::macos::{DriverCall, DriverResponse};
use crate::model::{AuvResult, ProducedArtifact, now_millis};

pub(crate) fn capture_display(call: &DriverCall) -> AuvResult<DriverResponse> {
  let label = optional_string(call, "label").unwrap_or_else(|| "display-capture".to_string());
  let display_ref = optional_string(call, "display_ref");
  let display_id = optional_string(call, "display_id");
  let has_display_selector = display_ref.is_some() || display_id.is_some();
  let main = optional_bool(call, "main")?.unwrap_or(!has_display_selector);
  let activated_app = maybe_activate_target_app_for_observation(call)?;
  let displays = typed_display_descriptors()?;
  let display_index = xcap_backend::resolve_display_index(
    &displays,
    display_ref.as_deref(),
    display_id.as_deref(),
    main,
  )?;
  let descriptor = displays
    .get(display_index)
    .ok_or_else(|| {
      format!(
        "{}: resolved display index {} is missing from the display descriptor list",
        capture_error::STALE_DISPLAY_REF,
        display_index
      )
    })?
    .clone();
  let observation = crate::driver::macos::typed::session::capture_display_bridge(Some(
    descriptor.native_display_id.clone(),
  ))?;
  let screenshot_pixel_size = typed_capture_size(&observation.capture)?;
  let screenshot_path = screenshot_temp_path(&label);
  xcap_backend::save_rgba_image(observation.capture.image, &screenshot_path)?;

  let contract = CaptureContract {
    coordinate_contract_version: 1,
    capture_source: CaptureSource::Display {
      display_ref: descriptor.display_ref.clone(),
      native_display_id: descriptor.native_display_id.clone(),
    },
    capture_backend: CaptureBackend::XcapMacos,
    include_shadow: false,
    source_global_logical_bounds: descriptor.global_logical_bounds.clone(),
    source_physical_pixel_bounds: Rect {
      x: 0.0,
      y: 0.0,
      width: screenshot_pixel_size.width,
      height: screenshot_pixel_size.height,
    },
    screenshot_pixel_size: screenshot_pixel_size.clone(),
    pixel_to_logical_scale: descriptor.pixel_to_logical_scale.clone(),
    logical_to_pixel_scale: descriptor.logical_to_pixel_scale.clone(),
    captured_at_unix_ms: now_millis(),
  };

  let screenshot_artifact = ProducedArtifact {
    kind: "screenshot".to_string(),
    source_path: screenshot_path,
    preferred_name: format!("{}.png", sanitize_file_component(&label)),
    note: Some("Display screenshot captured through xcap.".to_string()),
  };
  let contract_json = build_text_artifact(
    "capture-contract",
    "json",
    &format!("{}-capture-contract", sanitize_file_component(&label)),
    render_capture_contract_json(&contract)?,
    "Machine-readable capture coordinate contract.",
  )?;
  let contract_text = build_text_artifact(
    "capture-contract-report",
    "txt",
    &format!("{}-capture-contract", sanitize_file_component(&label)),
    render_capture_contract_text(&contract),
    "Human-readable capture coordinate contract.",
  )?;

  let mut notes = vec![
    format!("displayRef={}", descriptor.display_ref),
    format!("nativeDisplayId={}", descriptor.native_display_id),
    format!(
      "screenshotPixels={:.0}x{:.0}",
      screenshot_pixel_size.width, screenshot_pixel_size.height
    ),
  ];
  if let Some(app) = activated_app {
    notes.push(format!("activatedTargetBeforeCapture={app}"));
  }

  Ok(DriverResponse {
    summary: format!(
      "Captured {} through xcap ({:.0}x{:.0} pixels).",
      descriptor.display_ref, screenshot_pixel_size.width, screenshot_pixel_size.height
    ),
    backend: Some("xcap.macos".to_string()),
    signals: std::collections::BTreeMap::new(),
    notes,
    artifacts: vec![screenshot_artifact, contract_json, contract_text],
  })
}

pub(crate) fn capture_region(call: &DriverCall) -> AuvResult<DriverResponse> {
  let label = optional_string(call, "label").unwrap_or_else(|| "region-capture".to_string());
  let display_ref = optional_string(call, "display_ref");
  let display_id = optional_string(call, "display_id");
  let coordinate_space = parse_coordinate_space(call)?;
  let input = Rect {
    x: required_f64(call, "x")?,
    y: required_f64(call, "y")?,
    width: required_f64(call, "width")?,
    height: required_f64(call, "height")?,
  };
  let activated_app = maybe_activate_target_app_for_observation(call)?;

  let displays = typed_display_descriptors()?;
  let resolved = xcap_backend::resolve_region(
    &displays,
    input,
    coordinate_space.clone(),
    display_ref.as_deref(),
    display_id.as_deref(),
  )?;
  let descriptor = displays
    .get(resolved.display_index)
    .ok_or_else(|| {
      format!(
        "{}: resolved display index {} is missing from the display descriptor list",
        capture_error::STALE_DISPLAY_REF,
        resolved.display_index
      )
    })?
    .clone();
  let observation = crate::driver::macos::typed::session::capture_region_bridge(
    Some(descriptor.display_ref.clone()),
    auv_driver::Rect::new(
      resolved.source_global_logical_bounds.x,
      resolved.source_global_logical_bounds.y,
      resolved.source_global_logical_bounds.width,
      resolved.source_global_logical_bounds.height,
    ),
  )?;
  let screenshot_path = screenshot_temp_path(&label);
  let screenshot_pixel_size = typed_capture_size(&observation.capture)?;
  xcap_backend::save_rgba_image(observation.capture.image, &screenshot_path)?;
  let pixel_to_logical_scale = Scale2D {
    x: resolved.source_global_logical_bounds.width / screenshot_pixel_size.width,
    y: resolved.source_global_logical_bounds.height / screenshot_pixel_size.height,
  };
  let logical_to_pixel_scale = Scale2D {
    x: screenshot_pixel_size.width / resolved.source_global_logical_bounds.width,
    y: screenshot_pixel_size.height / resolved.source_global_logical_bounds.height,
  };

  let contract = CaptureContract {
    coordinate_contract_version: 1,
    capture_source: CaptureSource::Region {
      display_ref: descriptor.display_ref.clone(),
      native_display_id: descriptor.native_display_id.clone(),
      input_space: coordinate_space,
    },
    capture_backend: CaptureBackend::XcapMacos,
    include_shadow: false,
    source_global_logical_bounds: resolved.source_global_logical_bounds.clone(),
    source_physical_pixel_bounds: Rect {
      x: resolved.display_local_logical.x * descriptor.logical_to_pixel_scale.x,
      y: resolved.display_local_logical.y * descriptor.logical_to_pixel_scale.y,
      width: screenshot_pixel_size.width,
      height: screenshot_pixel_size.height,
    },
    screenshot_pixel_size: screenshot_pixel_size.clone(),
    pixel_to_logical_scale,
    logical_to_pixel_scale,
    captured_at_unix_ms: now_millis(),
  };

  let screenshot_artifact = ProducedArtifact {
    kind: "screenshot".to_string(),
    source_path: screenshot_path,
    preferred_name: format!("{}.png", sanitize_file_component(&label)),
    note: Some("Region screenshot captured through xcap.".to_string()),
  };
  let contract_json = build_text_artifact(
    "capture-contract",
    "json",
    &format!("{}-capture-contract", sanitize_file_component(&label)),
    render_capture_contract_json(&contract)?,
    "Machine-readable capture coordinate contract.",
  )?;
  let contract_text = build_text_artifact(
    "capture-contract-report",
    "txt",
    &format!("{}-capture-contract", sanitize_file_component(&label)),
    render_capture_contract_text(&contract),
    "Human-readable capture coordinate contract.",
  )?;

  let mut notes = vec![
    format!("displayRef={}", descriptor.display_ref),
    format!(
      "sourceGlobalLogicalBounds={:.3},{:.3},{:.3},{:.3}",
      resolved.source_global_logical_bounds.x,
      resolved.source_global_logical_bounds.y,
      resolved.source_global_logical_bounds.width,
      resolved.source_global_logical_bounds.height
    ),
    format!(
      "screenshotPixels={:.0}x{:.0}",
      screenshot_pixel_size.width, screenshot_pixel_size.height
    ),
  ];
  if let Some(app) = activated_app {
    notes.push(format!("activatedTargetBeforeCapture={app}"));
  }

  Ok(DriverResponse {
    summary: format!(
      "Captured region on {} through xcap ({:.0}x{:.0} pixels).",
      descriptor.display_ref, screenshot_pixel_size.width, screenshot_pixel_size.height
    ),
    backend: Some("xcap.macos".to_string()),
    signals: std::collections::BTreeMap::new(),
    notes,
    artifacts: vec![screenshot_artifact, contract_json, contract_text],
  })
}

pub(crate) fn capture_window(call: &DriverCall) -> AuvResult<DriverResponse> {
  let label = optional_string(call, "label").unwrap_or_else(|| "window-capture".to_string());
  let include_shadow = optional_bool(call, "include_shadow")?.unwrap_or(false);
  if include_shadow {
    return Err(format!(
      "{}: typed macOS window capture does not expose include_shadow=true",
      capture_error::UNSUPPORTED_BACKEND
    ));
  }
  let observation = capture_window_with_typed_session(call, &label)?;
  let screenshot_path = screenshot_temp_path(&label);
  let screenshot_pixel_size =
    xcap_backend::save_rgba_image(observation.capture.image.clone(), &screenshot_path)?;
  let contract = observation.contract;

  let screenshot_artifact = ProducedArtifact {
    kind: "screenshot".to_string(),
    source_path: screenshot_path,
    preferred_name: format!("{}.png", sanitize_file_component(&label)),
    note: Some(format!(
      "Window screenshot captured through {}.",
      observation.capture.backend
    )),
  };
  let contract_json = build_text_artifact(
    "capture-contract",
    "json",
    &format!("{}-capture-contract", sanitize_file_component(&label)),
    render_capture_contract_json(&contract)?,
    "Machine-readable capture coordinate contract.",
  )?;
  let contract_text = build_text_artifact(
    "capture-contract-report",
    "txt",
    &format!("{}-capture-contract", sanitize_file_component(&label)),
    render_capture_contract_text(&contract),
    "Human-readable capture coordinate contract.",
  )?;

  let mut notes = vec![
    format!(
      "windowRef=window_{}",
      observation.candidate.window_ref.window_number
    ),
    format!("displayRef={}", observation.display_ref),
    format!(
      "nativeWindowId={}",
      observation
        .candidate
        .native_window_id
        .as_deref()
        .unwrap_or_default()
    ),
    format!("candidateIndex={}", observation.candidate.candidate_index),
    format!("selectionReason={}", observation.candidate.selection_reason),
    format!(
      "isFullyContainedInDisplay={}",
      observation.candidate.is_fully_contained_in_display
    ),
    format!("includeShadow={include_shadow}"),
    format!("captureBackend={}", observation.capture.backend),
    format!(
      "screenshotPixels={:.0}x{:.0}",
      screenshot_pixel_size.width, screenshot_pixel_size.height
    ),
  ];
  if let Some(reason) = observation.capture.fallback_reason {
    notes.push(format!("fallbackReason={reason}"));
  }

  Ok(DriverResponse {
    summary: format!(
      "Captured window_{} on {} through {} ({:.0}x{:.0} pixels).",
      observation.candidate.window_ref.window_number,
      observation.display_ref,
      observation.capture.backend,
      screenshot_pixel_size.width,
      screenshot_pixel_size.height
    ),
    backend: Some(observation.capture.backend),
    signals: std::collections::BTreeMap::new(),
    notes,
    artifacts: vec![screenshot_artifact, contract_json, contract_text],
  })
}

pub(crate) fn list_displays(_call: &DriverCall) -> AuvResult<DriverResponse> {
  let displays = typed_display_descriptors()?;
  let main_display = displays
    .iter()
    .find(|display| display.is_main)
    .or_else(|| displays.first())
    .ok_or_else(|| {
      format!(
        "{}: no displays were reported by the capture backend",
        capture_error::DISPLAY_NOT_FOUND
      )
    })?;
  let mut rendered = serde_json::to_string_pretty(&displays).map_err(|error| {
    format!(
      "{}: failed to encode display list JSON: {error}",
      capture_error::BACKEND_FAILED
    )
  })?;
  rendered.push('\n');

  let artifact = build_text_artifact(
    "display-list",
    "json",
    "display-list",
    rendered,
    "Machine-readable xcap display list normalized into AUV display descriptors.",
  )?;

  let notes = displays
    .iter()
    .take(5)
    .map(render_display_note)
    .collect::<Vec<_>>();

  Ok(DriverResponse {
    summary: format!(
      "Listed {} display(s); main display is {} at {:.0}x{:.0} logical / {:.0}x{:.0} pixels.",
      displays.len(),
      main_display.display_ref,
      main_display.global_logical_bounds.width,
      main_display.global_logical_bounds.height,
      main_display.physical_pixel_size.width,
      main_display.physical_pixel_size.height
    ),
    backend: Some("xcap.macos".to_string()),
    signals: std::collections::BTreeMap::new(),
    notes,
    artifacts: vec![artifact],
  })
}

fn typed_display_descriptors() -> AuvResult<Vec<DisplayDescriptor>> {
  // TODO(remove-legacy-capture-descriptor-adapter): this converts typed
  // `auv-driver` display records back into legacy capture descriptors only
  // while root command handlers still produce old capture-contract artifacts.
  // Delete it when capture commands are invoked through typed runtime APIs.
  let typed = crate::driver::macos::typed::session::list_displays_bridge()?;
  typed
    .displays
    .iter()
    .map(|display| {
      let physical_pixel_size = Size {
        width: display.frame.size.width * display.scale_factor,
        height: display.frame.size.height * display.scale_factor,
      };
      display_descriptor_from_typed(display, &physical_pixel_size)
    })
    .collect()
}

fn typed_capture_size(capture: &auv_driver::Capture) -> AuvResult<Size> {
  Ok(Size {
    width: capture.image.width() as f64,
    height: capture.image.height() as f64,
  })
}

fn display_descriptor_from_typed(
  display: &auv_driver::Display,
  physical_pixel_size: &Size,
) -> AuvResult<DisplayDescriptor> {
  let global_logical_bounds = Rect {
    x: display.frame.origin.x,
    y: display.frame.origin.y,
    width: display.frame.size.width,
    height: display.frame.size.height,
  };
  let (pixel_to_logical_scale, logical_to_pixel_scale) =
    xcap_backend::scale_from_logical_and_physical(&global_logical_bounds, physical_pixel_size)?;
  Ok(DisplayDescriptor {
    display_ref: display
      .name
      .clone()
      .unwrap_or_else(|| format!("display_{}", display.id)),
    is_main: display.is_primary,
    is_builtin: display.is_builtin.unwrap_or(false),
    global_logical_bounds: global_logical_bounds.clone(),
    visible_logical_bounds: global_logical_bounds,
    physical_pixel_size: physical_pixel_size.clone(),
    scale_factor: display.scale_factor,
    pixel_to_logical_scale,
    logical_to_pixel_scale,
    native_display_id: display.id.clone(),
    capture_backend: CaptureBackend::XcapMacos,
  })
}

fn parse_coordinate_space(call: &DriverCall) -> AuvResult<CoordinateSpace> {
  match optional_string(call, "coordinate_space")
    .unwrap_or_else(|| "global_logical".to_string())
    .trim()
  {
    "global_logical" => Ok(CoordinateSpace::GlobalLogical),
    "display_logical" => Ok(CoordinateSpace::DisplayLogical),
    "display_physical" => Ok(CoordinateSpace::DisplayPhysical),
    other => Err(format!(
      "{}: unsupported coordinate_space {}; expected global_logical, display_logical, or display_physical",
      capture_error::REGION_OUT_OF_BOUNDS,
      other
    )),
  }
}

fn render_display_note(display: &DisplayDescriptor) -> String {
  format!(
    "{} native_id={} main={} bounds={:.0},{:.0},{:.0}x{:.0} logical pixels={:.0}x{:.0}",
    display.display_ref,
    display.native_display_id,
    display.is_main,
    display.global_logical_bounds.x,
    display.global_logical_bounds.y,
    display.global_logical_bounds.width,
    display.global_logical_bounds.height,
    display.physical_pixel_size.width,
    display.physical_pixel_size.height
  )
}
