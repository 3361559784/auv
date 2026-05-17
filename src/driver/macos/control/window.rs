use super::super::*;
use super::common::build_click_point_call;
use super::pointer::click_point;

pub(crate) fn click_window_point(call: &DriverCall) -> AuvResult<DriverResponse> {
  let app = app_identifier(call)
    .filter(|value| !value.is_empty())
    .ok_or_else(|| {
      "operation requires --target <application-id> or --app <application-id>".to_string()
    })?;
  activate_target_app(&app)?;

  let snapshot = super::super::observe::observe_windows_snapshot(32, "")?;
  let mut candidate_windows = snapshot
    .windows
    .iter()
    .filter(|window| {
      app_contains_window(&app, &window.app_name)
        || (!snapshot.frontmost_app_name.is_empty()
          && snapshot.frontmost_app_name == window.app_name)
    })
    .collect::<Vec<_>>();
  candidate_windows.sort_by(|left, right| {
    let left_key = (left.layer != 0, -window_area(left));
    let right_key = (right.layer != 0, -window_area(right));
    left_key.cmp(&right_key)
  });
  let window = candidate_windows
    .into_iter()
    .next()
    .or_else(|| snapshot.windows.first())
    .ok_or_else(|| format!("could not find a visible window for app {}", app))?;

  let (logical_x, logical_y, coordinate_summary) = resolve_window_point(call, window)?;
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
      format!("windowTitle={}", window.title),
      format!("windowBounds={}", render_rect_compact(&window.bounds)),
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
    format!("windowBounds={}", render_rect_compact(&window.bounds)),
    format!("logicalPoint={logical_x:.3},{logical_y:.3}"),
    coordinate_summary,
  ];
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
