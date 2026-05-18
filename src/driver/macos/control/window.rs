use super::super::*;
use super::common::build_click_point_call;
use super::pointer::click_point;

pub(crate) fn click_window_point(call: &DriverCall) -> AuvResult<DriverResponse> {
  let app = app_identifier(call)
    .filter(|value| !value.is_empty())
    .ok_or_else(|| {
      "operation requires --target <application-id> or --app <application-id>".to_string()
    })?;
  let selector = parse_app_selector(&app)?;
  activate_target_app(&app)?;

  let snapshot = super::super::observe::observe_windows_snapshot(128, "")?;
  let resolved_app = resolve_app_ref(&snapshot, &selector)?;
  let window = resolve_window_ref(&snapshot, &resolved_app)?;

  let (logical_x, logical_y, coordinate_summary) = resolve_window_point(call, &window)?;
  let button_label = optional_string(call, "button").unwrap_or_else(|| "left".to_string());
  let click_count = optional_i64(call, "click_count")?.unwrap_or(1).clamp(1, 4);
  let nested_call = build_click_point_call(
    &call.target,
    call.working_directory.as_path(),
    logical_x,
    logical_y,
    &button_label,
    click_count,
    None,
    Some(&app),
  );
  let _ = click_point(&nested_call)?;

  let artifact = build_text_artifact(
    "click-window-point",
    "txt",
    &format!("click-window-point-{}", sanitize_file_component(&app)),
    [
      format!("app={app}"),
      format!("appSelector={}", resolved_app.selector.raw),
      format!("matchStrategy={}", resolved_app.match_strategy),
      format!(
        "resolvedAppBundleId={}",
        resolved_app
          .resolved_bundle_id
          .clone()
          .unwrap_or_else(|| "n/a".to_string())
      ),
      format!("resolvedAppName={}", resolved_app.resolved_app_name),
      format!("windowRef={}", window.window_number),
      format!("windowTitle={}", window.title),
      format!("windowBounds={}", render_rect_compact(&window.bounds)),
      format!("ownerBundleId={}", window.owner_bundle_id),
      format!("ownerPid={}", window.owner_pid),
      format!("resolvedLogicalPoint={logical_x:.3},{logical_y:.3}"),
      coordinate_summary.clone(),
      format!("button={button_label}"),
      format!("clickCount={click_count}"),
    ]
    .join("\n"),
    "Clicked a point relative to a resolved macOS app window.",
  )?;
  let mut notes = vec![
    format!("app={app}"),
    format!("appSelector={}", resolved_app.selector.raw),
    format!("matchStrategy={}", resolved_app.match_strategy),
    format!(
      "resolvedAppBundleId={}",
      resolved_app
        .resolved_bundle_id
        .clone()
        .unwrap_or_else(|| "n/a".to_string())
    ),
    format!("windowRef={}", window.window_number),
    format!("windowBounds={}", render_rect_compact(&window.bounds)),
    format!("logicalPoint={logical_x:.3},{logical_y:.3}"),
    coordinate_summary,
  ];
  if !window.owner_bundle_id.is_empty() {
    notes.push(format!("ownerBundleId={}", window.owner_bundle_id));
  }
  if !window.title.is_empty() {
    notes.push(format!("windowTitle={}", window.title));
  }

  Ok(DriverResponse {
    summary: format!(
      "Clicked {} window-relative point in {} at global logical point ({logical_x:.3}, {logical_y:.3}).",
      button_label, app
    ),
    backend: Some("macos.observe.window-relative-click".to_string()),
    notes,
    artifacts: vec![artifact],
  })
}
