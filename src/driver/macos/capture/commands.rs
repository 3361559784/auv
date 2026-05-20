use std::collections::{HashMap, HashSet};

use super::artifact::{render_capture_contract_json, render_capture_contract_text};
use super::types::{
  CaptureBackend, CaptureContract, CaptureSource, CoordinateSpace, DisplayDescriptor, Rect,
  Scale2D, WindowDescriptor, capture_error,
};
use super::xcap_backend;
use crate::driver::macos::{
  DriverCall, DriverResponse, build_text_artifact, maybe_activate_target_app_for_observation,
  optional_bool, optional_i64, optional_string, required_f64, sanitize_file_component,
  screenshot_temp_path,
};
use crate::model::{AuvResult, ProducedArtifact, now_millis};

pub(crate) fn capture_display(call: &DriverCall) -> AuvResult<DriverResponse> {
  let label = optional_string(call, "label").unwrap_or_else(|| "display-capture".to_string());
  let display_ref = optional_string(call, "display_ref");
  let display_id = optional_string(call, "display_id");
  let has_display_selector = display_ref.is_some() || display_id.is_some();
  let main = optional_bool(call, "main")?.unwrap_or(!has_display_selector);
  let activated_app = maybe_activate_target_app_for_observation(call)?;

  let monitors = xcap::Monitor::all().map_err(|error| {
    format!(
      "{}: failed to enumerate displays before capture: {error}",
      capture_error::BACKEND_FAILED
    )
  })?;
  let displays = xcap_backend::descriptors_from_monitors(&monitors)?;
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

  let monitor = monitors.get(display_index).ok_or_else(|| {
    format!(
      "{}: display {} disappeared before capture",
      capture_error::STALE_DISPLAY_REF,
      descriptor.display_ref
    )
  })?;
  let image = monitor.capture_image().map_err(|error| {
    format!(
      "{}: failed to capture {} through xcap: {error}",
      capture_error::BACKEND_FAILED,
      descriptor.display_ref
    )
  })?;
  let screenshot_path = screenshot_temp_path(&label);
  let screenshot_pixel_size = xcap_backend::save_rgba_image(image, &screenshot_path)?;
  let (pixel_to_logical_scale, logical_to_pixel_scale) =
    xcap_backend::scale_from_logical_and_physical(
      &descriptor.global_logical_bounds,
      &screenshot_pixel_size,
    )?;

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
    pixel_to_logical_scale,
    logical_to_pixel_scale,
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

  let monitors = xcap::Monitor::all().map_err(|error| {
    format!(
      "{}: failed to enumerate displays before capture: {error}",
      capture_error::BACKEND_FAILED
    )
  })?;
  let displays = xcap_backend::descriptors_from_monitors(&monitors)?;
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
  let monitor = monitors.get(resolved.display_index).ok_or_else(|| {
    format!(
      "{}: display {} disappeared before region capture",
      capture_error::STALE_DISPLAY_REF,
      descriptor.display_ref
    )
  })?;
  let capture_x = integral_capture_dimension("x", resolved.display_local_logical.x)?;
  let capture_y = integral_capture_dimension("y", resolved.display_local_logical.y)?;
  let capture_width =
    integral_positive_capture_dimension("width", resolved.display_local_logical.width)?;
  let capture_height =
    integral_positive_capture_dimension("height", resolved.display_local_logical.height)?;

  let image = monitor
    .capture_region(capture_x, capture_y, capture_width, capture_height)
    .map_err(xcap_backend::map_xcap_capture_error)?;
  let screenshot_path = screenshot_temp_path(&label);
  let screenshot_pixel_size = xcap_backend::save_rgba_image(image, &screenshot_path)?;
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
  let window_id = optional_string(call, "window_id").map(|value| value.trim().to_string());
  let window_title = optional_string(call, "window_title").map(|value| value.trim().to_string());
  let window_index = optional_window_index(call, "window_index")?;
  let prefer_main_window = optional_bool(call, "prefer_main_window")?.unwrap_or(true);
  let include_shadow = optional_bool(call, "include_shadow")?.unwrap_or(false);
  if include_shadow {
    return Err(format!(
      "{}: xcap macOS window capture does not expose include_shadow=true",
      capture_error::UNSUPPORTED_BACKEND
    ));
  }
  let target_app = call
    .target
    .application_id
    .clone()
    .map(|value| value.trim().to_string())
    .filter(|value| !value.is_empty());
  let activated_app = maybe_activate_target_app_for_observation(call)?;

  let displays = xcap_backend::list_displays()?;
  let windows = xcap_backend::list_windows(&displays)?;
  let selected = if let Some(window_id) = window_id.as_deref() {
    select_window_by_id(&windows, window_id)?
  } else {
    let matches = windows
      .iter()
      .filter(|window| {
        window_matches_selectors(window, target_app.as_deref(), window_title.as_deref())
      })
      .collect::<Vec<_>>();

    select_window_match(&matches, window_index, prefer_main_window)?
  };

  let displays = xcap_backend::list_displays()?;
  let (xcap_window, selected) = find_fresh_xcap_window(&selected, &displays)?;
  let display_ref = selected.display_ref.clone().ok_or_else(|| {
    format!(
      "{}: refreshed window is not fully contained by one display",
      capture_error::STALE_WINDOW_REF
    )
  })?;
  let display = displays
    .iter()
    .find(|display| display.display_ref == display_ref)
    .ok_or_else(|| {
      format!(
        "{}: refreshed window display {} is missing from the display list",
        capture_error::STALE_DISPLAY_REF,
        display_ref
      )
    })?;
  let image = xcap_window
    .capture_image()
    .map_err(xcap_backend::map_xcap_capture_error)?;
  let screenshot_path = screenshot_temp_path(&label);
  let screenshot_pixel_size = xcap_backend::save_rgba_image(image, &screenshot_path)?;
  let pixel_to_logical_scale = Scale2D {
    x: selected.global_logical_bounds.width / screenshot_pixel_size.width,
    y: selected.global_logical_bounds.height / screenshot_pixel_size.height,
  };
  let logical_to_pixel_scale = Scale2D {
    x: screenshot_pixel_size.width / selected.global_logical_bounds.width,
    y: screenshot_pixel_size.height / selected.global_logical_bounds.height,
  };

  let contract = CaptureContract {
    coordinate_contract_version: 1,
    capture_source: CaptureSource::Window {
      window_ref: selected.window_ref.clone(),
      display_ref: display_ref.clone(),
      native_window_id: selected.native_window_id.clone(),
      native_display_id: display.native_display_id.clone(),
    },
    capture_backend: CaptureBackend::XcapMacos,
    include_shadow,
    source_global_logical_bounds: selected.global_logical_bounds.clone(),
    source_physical_pixel_bounds: Rect {
      x: (selected.global_logical_bounds.x - display.global_logical_bounds.x)
        * display.logical_to_pixel_scale.x,
      y: (selected.global_logical_bounds.y - display.global_logical_bounds.y)
        * display.logical_to_pixel_scale.y,
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
    note: Some("Window screenshot captured through xcap.".to_string()),
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
    format!("windowRef={}", selected.window_ref),
    format!("displayRef={display_ref}"),
    format!("nativeWindowId={}", selected.native_window_id),
    format!("includeShadow={include_shadow}"),
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
      "Captured {} on {} through xcap ({:.0}x{:.0} pixels).",
      selected.window_ref, display_ref, screenshot_pixel_size.width, screenshot_pixel_size.height
    ),
    backend: Some("xcap.macos".to_string()),
    signals: std::collections::BTreeMap::new(),
    notes,
    artifacts: vec![screenshot_artifact, contract_json, contract_text],
  })
}

pub(crate) fn list_displays(_call: &DriverCall) -> AuvResult<DriverResponse> {
  let displays = xcap_backend::list_displays()?;
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

fn integral_capture_dimension(name: &str, value: f64) -> AuvResult<u32> {
  if value.fract() != 0.0 {
    return Err(format!(
      "{}: region {} must be an integer in backend capture units",
      capture_error::REGION_OUT_OF_BOUNDS,
      name
    ));
  }
  if value < 0.0 || value > u32::MAX as f64 {
    return Err(format!(
      "{}: region {} is outside the capture backend range",
      capture_error::REGION_OUT_OF_BOUNDS,
      name
    ));
  }
  Ok(value as u32)
}

fn integral_positive_capture_dimension(name: &str, value: f64) -> AuvResult<u32> {
  let integral = integral_capture_dimension(name, value)?;
  if integral == 0 {
    return Err(format!(
      "{}: region {} must be positive",
      capture_error::REGION_OUT_OF_BOUNDS,
      name
    ));
  }
  Ok(integral)
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

fn window_matches_selectors(
  window: &WindowDescriptor,
  target_app: Option<&str>,
  window_title: Option<&str>,
) -> bool {
  if let Some(target_app) = target_app
    && !target_app.is_empty()
    && window.app_name != target_app
    && window
      .owner_bundle_id
      .as_deref()
      .map(|bundle_id| bundle_id != target_app)
      .unwrap_or(true)
    && window.native_window_id != target_app
  {
    return false;
  }
  if let Some(window_title) = window_title
    && !window_title.is_empty()
    && !window.title.contains(window_title)
  {
    return false;
  }
  true
}

fn select_window_by_id(
  windows: &[WindowDescriptor],
  window_id: &str,
) -> AuvResult<WindowDescriptor> {
  if window_id.trim().is_empty() {
    return Err(format!(
      "{}: window_id must not be empty",
      capture_error::WINDOW_NOT_FOUND
    ));
  }
  let matches = windows
    .iter()
    .filter(|window| window.native_window_id == window_id)
    .collect::<Vec<_>>();
  match matches.as_slice() {
    [window] => Ok((*window).clone()),
    [] => Err(format!(
      "{}: no xcap window matched window_id {}",
      capture_error::WINDOW_NOT_FOUND,
      window_id
    )),
    _ => Err(format!(
      "{}: window_id {} matched {} xcap windows",
      capture_error::AMBIGUOUS_WINDOW_SELECTOR,
      window_id,
      matches.len()
    )),
  }
}

fn optional_window_index(call: &DriverCall, key: &str) -> AuvResult<Option<usize>> {
  match optional_i64(call, key)? {
    Some(value) if value < 0 => Err(format!(
      "invalid --{} value {}: expected a non-negative integer",
      key, value
    )),
    Some(value) => Ok(Some(value as usize)),
    None => Ok(None),
  }
}

fn select_window_match(
  matches: &[&WindowDescriptor],
  window_index: Option<usize>,
  prefer_main_window: bool,
) -> AuvResult<WindowDescriptor> {
  if matches.is_empty() {
    return Err(format!(
      "{}: no xcap window matched the provided selector",
      capture_error::WINDOW_NOT_FOUND
    ));
  }

  if let Some(window_index) = window_index {
    return matches
      .get(window_index)
      .map(|window| (*window).clone())
      .ok_or_else(|| {
        format!(
          "{}: window_index {} is outside {} matched xcap window(s)",
          capture_error::WINDOW_NOT_FOUND,
          window_index,
          matches.len()
        )
      });
  }

  if matches.len() == 1 {
    return Ok(matches[0].clone());
  }

  if prefer_main_window {
    let mut scored = matches
      .iter()
      .map(|window| (main_window_score(window), *window))
      .collect::<Vec<_>>();
    scored.sort_by(|(left, _), (right, _)| right.cmp(left));
    if scored[0].0 != scored[1].0 {
      return Ok(scored[0].1.clone());
    }
  }

  Err(format!(
    "{}: selector matched {} xcap windows",
    capture_error::AMBIGUOUS_WINDOW_SELECTOR,
    matches.len()
  ))
}

fn main_window_score(window: &WindowDescriptor) -> (bool, i32, u64) {
  let area = (window.global_logical_bounds.width.max(0.0)
    * window.global_logical_bounds.height.max(0.0))
  .round()
  .max(0.0) as u64;
  (
    window.is_focused.unwrap_or(false),
    window.z_order.unwrap_or(i32::MIN),
    area,
  )
}

fn find_fresh_xcap_window(
  selected: &WindowDescriptor,
  displays: &[DisplayDescriptor],
) -> AuvResult<(xcap::Window, WindowDescriptor)> {
  let windows = xcap::Window::all().map_err(|error| {
    format!(
      "{}: failed to re-enumerate windows before capture: {error}",
      capture_error::BACKEND_FAILED
    )
  })?;
  for window in &windows {
    let Ok(id) = window.id() else {
      continue;
    };
    if id.to_string() == selected.native_window_id {
      let pids = [window.pid().map_err(|error| {
        format!(
          "{}: failed to read refreshed window pid: {error}",
          capture_error::STALE_WINDOW_REF
        )
      })?]
      .into_iter()
      .collect::<HashSet<_>>();
      let bundle_ids = xcap_backend::bundle_ids_by_pid(&pids).unwrap_or_else(|_| HashMap::new());
      let refreshed = xcap_backend::descriptor_from_window(
        selected
          .window_ref
          .strip_prefix("window_")
          .and_then(|value| value.parse::<usize>().ok())
          .unwrap_or(0),
        window,
        displays,
        &bundle_ids,
      )
      .map_err(|error| {
        format!(
          "{}: failed to refresh selected window descriptor: {error}",
          capture_error::STALE_WINDOW_REF
        )
      })?;
      return Ok((window.clone(), refreshed));
    }
  }

  Err(format!(
    "{}: selected window {} disappeared before capture",
    capture_error::STALE_WINDOW_REF,
    selected.window_ref
  ))
}

#[cfg(test)]
mod window_selector_tests {
  use super::*;

  fn window(owner_bundle_id: Option<&str>) -> WindowDescriptor {
    WindowDescriptor {
      window_ref: "window_0".to_string(),
      title: "Main".to_string(),
      app_name: "NetEaseMusic".to_string(),
      owner_bundle_id: owner_bundle_id.map(str::to_string),
      owner_pid: Some(42),
      z_order: Some(10),
      is_focused: Some(false),
      is_minimized: Some(false),
      global_logical_bounds: Rect {
        x: 0.0,
        y: 0.0,
        width: 100.0,
        height: 100.0,
      },
      display_ref: Some("display_0".to_string()),
      native_window_id: "9001".to_string(),
      capture_backend: CaptureBackend::XcapMacos,
    }
  }

  #[test]
  fn window_selector_matches_bundle_id_target() {
    assert!(window_matches_selectors(
      &window(Some("com.netease.163music")),
      Some("com.netease.163music"),
      None
    ));
  }

  #[test]
  fn window_selector_rejects_wrong_bundle_id_target() {
    assert!(!window_matches_selectors(
      &window(Some("com.netease.163music")),
      Some("com.tencent.QQMusicMac"),
      None
    ));
  }

  #[test]
  fn window_selector_still_matches_app_name_target() {
    assert!(window_matches_selectors(
      &window(Some("com.netease.163music")),
      Some("NetEaseMusic"),
      None
    ));
  }

  #[test]
  fn window_id_selector_ignores_other_selector_inputs() {
    let first = window(Some("com.netease.163music"));
    let mut second = window(Some("com.tencent.QQMusicMac"));
    second.native_window_id = "9002".to_string();
    second.app_name = "QQMusic".to_string();

    let selected = select_window_by_id(&[first, second], "9002").unwrap();

    assert_eq!(selected.app_name, "QQMusic");
  }

  #[test]
  fn window_selector_uses_window_index_to_disambiguate() {
    let first = window(Some("com.netease.163music"));
    let mut second = window(Some("com.netease.163music"));
    second.native_window_id = "9002".to_string();

    let selected = select_window_match(&[&first, &second], Some(1), true).unwrap();

    assert_eq!(selected.native_window_id, "9002");
  }

  #[test]
  fn window_selector_prefers_focused_main_window() {
    let first = window(Some("com.netease.163music"));
    let mut second = window(Some("com.netease.163music"));
    second.native_window_id = "9002".to_string();
    second.is_focused = Some(true);

    let selected = select_window_match(&[&first, &second], None, true).unwrap();

    assert_eq!(selected.native_window_id, "9002");
  }

  #[test]
  fn window_selector_fails_ambiguous_without_preference() {
    let first = window(Some("com.netease.163music"));
    let mut second = window(Some("com.netease.163music"));
    second.native_window_id = "9002".to_string();
    second.z_order = first.z_order;

    let error = select_window_match(&[&first, &second], None, false).unwrap_err();

    assert!(error.contains(capture_error::AMBIGUOUS_WINDOW_SELECTOR));
  }
}
