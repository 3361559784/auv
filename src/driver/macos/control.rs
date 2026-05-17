use std::thread;
use std::time::Duration;

use super::*;

pub(super) fn activate_app(call: &DriverCall) -> AuvResult<DriverResponse> {
  let app = app_identifier(call)
    .filter(|value| !value.is_empty())
    .ok_or_else(|| "missing target application id for activate_app".to_string())?;
  let settle_ms = optional_positive_u64(call, "settle_ms")?.unwrap_or(250);
  activate_target_app(&app)?;
  if settle_ms > 0 {
    thread::sleep(Duration::from_millis(settle_ms));
  }

  let artifact = build_text_artifact(
    "activate-app",
    "txt",
    &format!("activate-app-{}", sanitize_file_component(&app)),
    render_activate_app_report(&app, settle_ms),
    "Activated the target app through AppleScript before a foreground-dependent action.",
  )?;

  Ok(DriverResponse {
    summary: format!(
      "Activated {} and waited {} ms for the foreground app to settle.",
      app, settle_ms
    ),
    backend: Some("macos.osascript.activate-app".to_string()),
    notes: vec![format!("app={app}"), format!("settleMs={settle_ms}")],
    artifacts: vec![artifact],
  })
}

pub(super) fn focus_text_input(call: &DriverCall) -> AuvResult<DriverResponse> {
  let app = app_identifier(call).unwrap_or_default();
  let query = required_non_empty_string(call, "query")?;
  let reveal_shortcut = optional_non_empty_string(call, "reveal_shortcut");
  let reveal_settle_ms = optional_positive_u64(call, "reveal_settle_ms")?.unwrap_or(250);
  let max_depth = optional_i64(call, "max_depth")?.unwrap_or(6).clamp(1, 10);
  let max_children = optional_i64(call, "max_children")?
    .unwrap_or(16)
    .clamp(1, 50);
  if !app.is_empty() {
    activate_target_app(&app)?;
  }
  if let Some(shortcut) = reveal_shortcut.as_deref() {
    send_shortcut(shortcut)?;
    thread::sleep(Duration::from_millis(reveal_settle_ms));
  }
  let tree_report = run_swift_script(&build_observe_window_tree_script(
    &app,
    max_depth,
    max_children,
  ))?;
  let snapshot = parse_observed_ax_tree(&tree_report)?;
  let matched = find_best_ax_node(&snapshot, &query)
    .ok_or_else(|| no_matching_ax_node_error(&snapshot, &query, "text input-like"))?;
  let (center_x, center_y) = ax_node_center(matched);
  run_swift_script(&build_click_point_script(center_x, center_y, 0, 1))?;
  let report = render_ax_interaction_report("focus-text-input", &snapshot, matched, &query);
  let artifact = build_text_artifact(
    "focus-text-input",
    "txt",
    &format!("focus-text-input-{}", sanitize_file_component(&query)),
    report,
    "Focused a text input by matching the observed AX tree and clicking the resolved bounds.",
  )?;
  let mut notes = vec![
    format!("query={query}"),
    format!("matchedPath={}", matched.path),
    format!("matchedRole={}", matched.role),
    format!("matchedBounds={}", render_rect_compact(&matched.bounds)),
    format!("clickLogicalPoint={center_x:.3},{center_y:.3}"),
  ];
  if let Some(shortcut) = reveal_shortcut.as_deref() {
    notes.push(format!("revealShortcut={shortcut}"));
    notes.push(format!("revealSettleMs={reveal_settle_ms}"));
  }
  if !matched.description.is_empty() {
    notes.push(format!("matchedDescription={}", matched.description));
  }
  if !matched.placeholder.is_empty() {
    notes.push(format!("matchedPlaceholder={}", matched.placeholder));
  }
  if !matched.title.is_empty() {
    notes.push(format!("matchedTitle={}", matched.title));
  }

  Ok(DriverResponse {
    summary: if matched.title.is_empty() && matched.description.is_empty() {
      format!(
        "Focused text input in {} using query {} (role {}).",
        if snapshot.app_name.is_empty() {
          "target app"
        } else {
          &snapshot.app_name
        },
        query,
        matched.role
      )
    } else {
      format!(
        "Focused text input {} in {} using query {}.",
        if matched.title.is_empty() {
          matched.description.as_str()
        } else {
          matched.title.as_str()
        },
        if snapshot.app_name.is_empty() {
          "target app"
        } else {
          &snapshot.app_name
        },
        query
      )
    },
    backend: Some("macos.observe.ax-tree-click-focus".to_string()),
    notes,
    artifacts: vec![artifact],
  })
}

pub(super) fn press_button(call: &DriverCall) -> AuvResult<DriverResponse> {
  let app = app_identifier(call).unwrap_or_default();
  let query = required_non_empty_string(call, "query")?;
  let reveal_shortcut = optional_non_empty_string(call, "reveal_shortcut");
  let reveal_settle_ms = optional_positive_u64(call, "reveal_settle_ms")?.unwrap_or(250);
  let max_depth = optional_i64(call, "max_depth")?.unwrap_or(6).clamp(1, 10);
  let max_children = optional_i64(call, "max_children")?
    .unwrap_or(16)
    .clamp(1, 50);
  if !app.is_empty() {
    activate_target_app(&app)?;
  }
  if let Some(shortcut) = reveal_shortcut.as_deref() {
    send_shortcut(shortcut)?;
    thread::sleep(Duration::from_millis(reveal_settle_ms));
  }
  let tree_report = run_swift_script(&build_observe_window_tree_script(
    &app,
    max_depth,
    max_children,
  ))?;
  let snapshot = parse_observed_ax_tree(&tree_report)?;
  let matched = find_best_ax_node(&snapshot, &query)
    .ok_or_else(|| no_matching_ax_node_error(&snapshot, &query, "button-like"))?;
  let (center_x, center_y) = ax_node_center(matched);
  run_swift_script(&build_click_point_script(center_x, center_y, 0, 1))?;
  let report = render_ax_interaction_report("press-button", &snapshot, matched, &query);
  let artifact = build_text_artifact(
    "press-button",
    "txt",
    &format!("press-button-{}", sanitize_file_component(&query)),
    report,
    "Pressed a known control by matching the observed AX tree and clicking the resolved bounds.",
  )?;
  let mut notes = vec![
    format!("query={query}"),
    format!("matchedPath={}", matched.path),
    format!("matchedRole={}", matched.role),
    format!("matchedBounds={}", render_rect_compact(&matched.bounds)),
    format!("clickLogicalPoint={center_x:.3},{center_y:.3}"),
  ];
  if let Some(shortcut) = reveal_shortcut.as_deref() {
    notes.push(format!("revealShortcut={shortcut}"));
    notes.push(format!("revealSettleMs={reveal_settle_ms}"));
  }
  if !matched.description.is_empty() {
    notes.push(format!("matchedDescription={}", matched.description));
  }
  if !matched.help.is_empty() {
    notes.push(format!("matchedHelp={}", matched.help));
  }
  if !matched.title.is_empty() {
    notes.push(format!("matchedTitle={}", matched.title));
  }

  Ok(DriverResponse {
    summary: if matched.title.is_empty() && matched.description.is_empty() {
      format!(
        "Pressed button-like control in {} using query {} (role {}).",
        if snapshot.app_name.is_empty() {
          "target app"
        } else {
          &snapshot.app_name
        },
        query,
        matched.role
      )
    } else {
      format!(
        "Pressed {} in {} using query {}.",
        if matched.title.is_empty() {
          matched.description.as_str()
        } else {
          matched.title.as_str()
        },
        if snapshot.app_name.is_empty() {
          "target app"
        } else {
          &snapshot.app_name
        },
        query
      )
    },
    backend: Some("macos.observe.ax-tree-click-press".to_string()),
    notes,
    artifacts: vec![artifact],
  })
}

pub(super) fn type_text(call: &DriverCall) -> AuvResult<DriverResponse> {
  let app = app_identifier(call).unwrap_or_default();
  let text = required_non_empty_string(call, "text")?;
  let replace_existing = optional_bool(call, "replace_existing")?.unwrap_or(false);
  let submit_key = optional_non_empty_string(call, "submit_key");
  let submit_settle_ms = optional_positive_u64(call, "submit_settle_ms")?.unwrap_or(0);
  if !app.is_empty() {
    activate_target_app(&app)?;
  }
  type_text_via_system_events(
    &text,
    replace_existing,
    submit_key.as_deref(),
    submit_settle_ms,
  )?;

  let report = render_type_text_report(&app, &text, replace_existing, submit_key.as_deref());
  let artifact = build_text_artifact(
    "type-text",
    "txt",
    &format!("type-text-{}", sanitize_file_component(&text)),
    report,
    "Typed text into the active macOS control through System Events.",
  )?;

  let mut notes = vec![
    format!("text={text}"),
    format!("textLength={}", text.chars().count()),
    format!("replaceExisting={replace_existing}"),
  ];
  if !app.is_empty() {
    notes.push(format!("app={app}"));
  }
  if let Some(submit_key) = submit_key.as_deref() {
    notes.push(format!("submitKey={submit_key}"));
  }
  if submit_settle_ms > 0 {
    notes.push(format!("submitSettleMs={submit_settle_ms}"));
  }

  Ok(DriverResponse {
    summary: match submit_key.as_deref() {
      Some(submit_key) => format!(
        "Typed {} character(s) into {} and submitted with {}.",
        text.chars().count(),
        if app.is_empty() {
          "the active app"
        } else {
          &app
        },
        submit_key
      ),
      None => format!(
        "Typed {} character(s) into {}.",
        text.chars().count(),
        if app.is_empty() {
          "the active app"
        } else {
          &app
        }
      ),
    },
    backend: Some("macos.system-events.type-text".to_string()),
    notes,
    artifacts: vec![artifact],
  })
}

pub(super) fn paste_text_preserve_clipboard(call: &DriverCall) -> AuvResult<DriverResponse> {
  let app = app_identifier(call).unwrap_or_default();
  let text = required_non_empty_string(call, "text")?;
  let replace_existing = optional_bool(call, "replace_existing")?.unwrap_or(false);
  let submit_key = optional_non_empty_string(call, "submit_key");
  let submit_settle_ms = optional_positive_u64(call, "submit_settle_ms")?.unwrap_or(0);
  if !app.is_empty() {
    activate_target_app(&app)?;
  }
  paste_text_preserving_clipboard(
    &text,
    replace_existing,
    submit_key.as_deref(),
    submit_settle_ms,
  )?;

  let artifact = build_text_artifact(
    "paste-text-preserve-clipboard",
    "txt",
    &format!(
      "paste-text-preserve-clipboard-{}",
      sanitize_file_component(&text)
    ),
    [
      format!("pastedAt={}", now_millis()),
      format!("app={app}"),
      format!("text={text}"),
      format!("textLength={}", text.chars().count()),
      format!("replaceExisting={replace_existing}"),
      format!("submitKey={}", submit_key.as_deref().unwrap_or("n/a")),
      format!("submitSettleMs={submit_settle_ms}"),
      "clipboardRestored=true".to_string(),
    ]
    .join("\n"),
    "Pasted text through the macOS clipboard, then restored the prior clipboard snapshot.",
  )?;

  let mut notes = vec![
    format!("text={text}"),
    format!("textLength={}", text.chars().count()),
    format!("replaceExisting={replace_existing}"),
    "clipboardRestored=true".to_string(),
  ];
  if !app.is_empty() {
    notes.push(format!("app={app}"));
  }
  if let Some(submit_key) = submit_key.as_deref() {
    notes.push(format!("submitKey={submit_key}"));
  }
  if submit_settle_ms > 0 {
    notes.push(format!("submitSettleMs={submit_settle_ms}"));
  }

  Ok(DriverResponse {
    summary: match submit_key.as_deref() {
      Some(submit_key) => format!(
        "Pasted {} character(s) into {} and submitted with {} while restoring the clipboard.",
        text.chars().count(),
        if app.is_empty() {
          "the active app"
        } else {
          &app
        },
        submit_key
      ),
      None => format!(
        "Pasted {} character(s) into {} while restoring the clipboard.",
        text.chars().count(),
        if app.is_empty() {
          "the active app"
        } else {
          &app
        }
      ),
    },
    backend: Some("macos.system-events.paste-text-preserve-clipboard".to_string()),
    notes,
    artifacts: vec![artifact],
  })
}

pub(super) fn press_key(call: &DriverCall) -> AuvResult<DriverResponse> {
  let app = app_identifier(call).unwrap_or_default();
  let key = required_non_empty_string(call, "key")?;
  let settle_ms = optional_positive_u64(call, "settle_ms")?.unwrap_or(0);
  if !app.is_empty() {
    activate_target_app(&app)?;
  }
  send_key_input(&key, settle_ms)?;
  let artifact = build_text_artifact(
    "press-key",
    "txt",
    &format!("press-key-{}", sanitize_file_component(&key)),
    [
      format!("pressedAt={}", now_millis()),
      format!("app={app}"),
      format!("key={key}"),
      format!("settleMs={settle_ms}"),
    ]
    .join("\n"),
    "Pressed a keyboard key or shortcut through System Events.",
  )?;
  Ok(DriverResponse {
    summary: format!(
      "Pressed key {} in {}.",
      key,
      if app.is_empty() {
        "the active app"
      } else {
        &app
      }
    ),
    backend: Some("macos.system-events.press-key".to_string()),
    notes: vec![
      format!("key={key}"),
      format!("settleMs={settle_ms}"),
      format!("app={app}"),
    ],
    artifacts: vec![artifact],
  })
}

pub(super) fn click_window_point(call: &DriverCall) -> AuvResult<DriverResponse> {
  let app = app_identifier(call)
    .filter(|value| !value.is_empty())
    .ok_or_else(|| {
      "operation requires --target <application-id> or --app <application-id>".to_string()
    })?;
  activate_target_app(&app)?;
  let snapshot = super::observe::observe_windows_snapshot(32, "")?;
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
  let nested_call = DriverCall {
    operation: "click_point".to_string(),
    target: call.target.clone(),
    inputs: std::collections::BTreeMap::from([
      ("x".to_string(), format!("{logical_x:.3}")),
      ("y".to_string(), format!("{logical_y:.3}")),
      ("button".to_string(), button_label.clone()),
      ("click_count".to_string(), click_count.to_string()),
      ("app".to_string(), app.clone()),
    ]),
    working_directory: call.working_directory.clone(),
  };
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

pub(super) fn click_screen_text(call: &DriverCall) -> AuvResult<DriverResponse> {
  let query = required_non_empty_string(call, "query")?;
  let label = format!("screen-text-click-{}", sanitize_file_component(&query));
  let activated_app = maybe_activate_target_app_for_observation(call)?;
  let screenshot_path = capture_screenshot_file(&label)?;
  let dimensions = read_png_dimensions(&screenshot_path)?;
  let snapshot = enumerate_displays()?;
  let exact = optional_bool(call, "exact")?.unwrap_or(false);
  let case_sensitive = optional_bool(call, "case_sensitive")?.unwrap_or(false);
  let max_observations = optional_i64(call, "max_observations")?
    .unwrap_or(64)
    .clamp(1, 256);
  let match_index = optional_i64(call, "match_index")?.unwrap_or(0).max(0) as usize;
  let min_confidence = optional_f64(call, "min_confidence")?.unwrap_or(0.0);
  if !(0.0..=1.0).contains(&min_confidence) {
    return Err(format!(
      "invalid --min_confidence value {:.3}: expected a ratio within 0.0..=1.0",
      min_confidence
    ));
  }
  let region = parse_ocr_region_constraint(call, dimensions.width, dimensions.height)?;
  let ocr_report = run_swift_script(&build_ocr_find_text_script(
    screenshot_path.as_path(),
    &query,
    exact,
    case_sensitive,
    max_observations,
    region.as_ref(),
  ))?;
  let ocr_snapshot = parse_ocr_text_snapshot(&ocr_report)?;
  let filtered_matches = filter_ocr_matches(&ocr_snapshot.matches, min_confidence, region.as_ref());
  let matched = filtered_matches.get(match_index).copied().ok_or_else(|| {
    format!(
      "no filtered OCR text match at index {} for query {} (found {} after filtering from {})",
      match_index,
      query,
      filtered_matches.len(),
      ocr_snapshot.matches.len()
    )
  })?;
  let anchor_offset_x = optional_f64(call, "anchor_offset_x")?.unwrap_or(0.0);
  let anchor_offset_y = optional_f64(call, "anchor_offset_y")?.unwrap_or(0.0);
  let (match_center_x, match_center_y) = ocr_match_center(matched);
  let screenshot_center_x = match_center_x + anchor_offset_x;
  let screenshot_center_y = match_center_y + anchor_offset_y;
  let (logical_x, logical_y) =
    project_main_screenshot_point(&snapshot, screenshot_center_x, screenshot_center_y)?;
  let button_label = optional_string(call, "button").unwrap_or_else(|| "left".to_string());
  let click_count = optional_i64(call, "click_count")?.unwrap_or(1).clamp(1, 4);
  let settle_ms = optional_positive_u64(call, "settle_ms")?.unwrap_or(0);
  let nested_call = DriverCall {
    operation: "click_point".to_string(),
    target: call.target.clone(),
    inputs: std::collections::BTreeMap::from([
      ("x".to_string(), format!("{logical_x:.3}")),
      ("y".to_string(), format!("{logical_y:.3}")),
      ("button".to_string(), button_label.clone()),
      ("click_count".to_string(), click_count.to_string()),
      ("settle_ms".to_string(), settle_ms.to_string()),
    ]),
    working_directory: call.working_directory.clone(),
  };
  let _ = click_point(&nested_call)?;

  let report_artifact = build_text_artifact(
    "screen-text-click",
    "txt",
    &format!("screen-text-click-{}", sanitize_file_component(&query)),
    [
      format!("query={query}"),
      format!("matchIndex={match_index}"),
      format!("filteredMatchCount={}", filtered_matches.len()),
      format!("minConfidence={min_confidence:.3}"),
      format!("matchText={}", matched.text),
      format!("matchBounds={}", render_rect_compact(&matched.bounds)),
      format!("matchConfidence={:.3}", matched.confidence),
      format!("anchorOffset={anchor_offset_x:.3},{anchor_offset_y:.3}"),
      format!("screenshotCenter={screenshot_center_x:.3},{screenshot_center_y:.3}"),
      format!("logicalPoint={logical_x:.3},{logical_y:.3}"),
      format!("button={button_label}"),
      format!("clickCount={click_count}"),
      format!("settleMs={settle_ms}"),
    ]
    .join("\n"),
    "Clicked an OCR text anchor projected from screenshot pixels to logical coordinates.",
  )?;
  let screenshot_artifact = ProducedArtifact {
    kind: "screenshot".to_string(),
    source_path: screenshot_path,
    preferred_name: format!("{}.png", sanitize_file_component(&label)),
    note: Some("Screenshot captured for OCR click-anchor detection.".to_string()),
  };
  let mut notes = vec![
    format!("query={query}"),
    format!("matchIndex={match_index}"),
    format!("filteredMatchCount={}", filtered_matches.len()),
    format!("matchText={}", matched.text),
    format!("matchBounds={}", render_rect_compact(&matched.bounds)),
    format!("minConfidence={min_confidence:.3}"),
    format!("anchorOffset={anchor_offset_x:.3},{anchor_offset_y:.3}"),
    format!("screenshotCenter={screenshot_center_x:.3},{screenshot_center_y:.3}"),
    format!("logicalPoint={logical_x:.3},{logical_y:.3}"),
    format!("button={button_label}"),
    format!("clickCount={click_count}"),
    format!("settleMs={settle_ms}"),
  ];
  if let Some(app) = activated_app {
    notes.push(format!("activatedTargetBeforeCapture={app}"));
  }

  Ok(DriverResponse {
    summary: format!(
      "Clicked OCR text anchor {} for query {} at logical point ({logical_x:.3}, {logical_y:.3}).",
      matched.text, query
    ),
    backend: Some("macos.vision.click-screen-text".to_string()),
    notes,
    artifacts: vec![screenshot_artifact, report_artifact],
  })
}

pub(super) fn click_screen_row(call: &DriverCall) -> AuvResult<DriverResponse> {
  let label = optional_string(call, "label").unwrap_or_else(|| "screen-row-click".to_string());
  let activated_app = maybe_activate_target_app_for_observation(call)?;
  let screenshot_path = capture_screenshot_file(&label)?;
  let dimensions = read_png_dimensions(&screenshot_path)?;
  let snapshot = enumerate_displays()?;
  let min_confidence = optional_f64(call, "min_confidence")?.unwrap_or(0.0);
  if !(0.0..=1.0).contains(&min_confidence) {
    return Err(format!(
      "invalid --min_confidence value {:.3}: expected a ratio within 0.0..=1.0",
      min_confidence
    ));
  }
  let max_observations = optional_i64(call, "max_observations")?
    .unwrap_or(128)
    .clamp(1, 512);
  let region = parse_ocr_region_constraint(call, dimensions.width, dimensions.height)?;
  let row_index = optional_i64(call, "row_index")?.unwrap_or(1).clamp(1, 64) as usize - 1;
  let row_anchor_x_ratio = optional_f64(call, "row_anchor_x_ratio")?.unwrap_or(0.25);
  let row_anchor_y_ratio = optional_f64(call, "row_anchor_y_ratio")?.unwrap_or(0.5);
  let row_anchor_mode =
    optional_string(call, "row_anchor_mode").unwrap_or_else(|| "title_band".to_string());
  for (label, value) in [
    ("row_anchor_x_ratio", row_anchor_x_ratio),
    ("row_anchor_y_ratio", row_anchor_y_ratio),
  ] {
    if !(0.0..=1.0).contains(&value) {
      return Err(format!(
        "invalid --{} value {:.3}: expected a ratio within 0.0..=1.0",
        label, value
      ));
    }
  }
  match row_anchor_mode.as_str() {
    "title_band" | "row_ratio" => {}
    other => {
      return Err(format!(
        "invalid --row_anchor_mode value {}: expected title_band or row_ratio",
        other
      ));
    }
  }

  let detection = detect_screen_rows(
    screenshot_path.as_path(),
    min_confidence,
    max_observations,
    region.as_ref(),
  )?;
  let rows = detection.rows;
  let row = rows.get(row_index).ok_or_else(|| {
    format!(
      "no visible row at index {} (detected {} row(s) with strategy {})",
      row_index + 1,
      rows.len(),
      detection.strategy
    )
  })?;

  let screenshot_center_x = match row_anchor_mode.as_str() {
    "row_ratio" => row.bounds.x as f64 + (row.bounds.width as f64 * row_anchor_x_ratio),
    "title_band" => {
      let region_left = region
        .as_ref()
        .map(|value| value.x as f64 + (value.width as f64 * 0.16))
        .unwrap_or(row.bounds.x as f64 + (row.bounds.width as f64 * 0.16));
      let cover_offset = row.bounds.x as f64 + (row.bounds.height as f64 * 1.05) + 18.0;
      cover_offset
        .max(region_left)
        .min((row.bounds.x + row.bounds.width - 24).max(row.bounds.x) as f64)
    }
    _ => unreachable!(),
  };
  let screenshot_center_y = row.bounds.y as f64 + (row.bounds.height as f64 * row_anchor_y_ratio);
  let (logical_x, logical_y) =
    project_main_screenshot_point(&snapshot, screenshot_center_x, screenshot_center_y)?;
  let button_label = optional_string(call, "button").unwrap_or_else(|| "left".to_string());
  let click_count = optional_i64(call, "click_count")?.unwrap_or(1).clamp(1, 4);
  let settle_ms = optional_positive_u64(call, "settle_ms")?.unwrap_or(0);

  let nested_call = DriverCall {
    operation: "click_point".to_string(),
    target: call.target.clone(),
    inputs: std::collections::BTreeMap::from([
      ("x".to_string(), format!("{logical_x:.3}")),
      ("y".to_string(), format!("{logical_y:.3}")),
      ("button".to_string(), button_label.clone()),
      ("click_count".to_string(), click_count.to_string()),
      ("settle_ms".to_string(), settle_ms.to_string()),
    ]),
    working_directory: call.working_directory.clone(),
  };
  let _ = click_point(&nested_call)?;

  let report_artifact = build_text_artifact(
    "screen-row-click",
    "txt",
    &format!("screen-row-click-{}", sanitize_file_component(&label)),
    [
      format!("rowStrategy={}", detection.strategy),
      format!("rowIndex={}", row_index + 1),
      format!("detectedRowCount={}", rows.len()),
      format!("matchCount={}", detection.raw_match_count),
      format!("filteredMatchCount={}", detection.filtered_match_count),
      format!("minConfidence={min_confidence:.3}"),
      format!("rowBounds={}", render_rect_compact(&row.bounds)),
      format!("rowSource={}", row.source),
      format!("rowText={}", row.text_fragments.join(" | ")),
      format!("rowAnchorMode={row_anchor_mode}"),
      format!("rowAnchorRatio={row_anchor_x_ratio:.3},{row_anchor_y_ratio:.3}"),
      format!("screenshotCenter={screenshot_center_x:.3},{screenshot_center_y:.3}"),
      format!("logicalPoint={logical_x:.3},{logical_y:.3}"),
      format!("button={button_label}"),
      format!("clickCount={click_count}"),
      format!("settleMs={settle_ms}"),
    ]
    .join("\n"),
    "Detected a visible row (OCR first, then visual-band fallback), projected a row-derived anchor point, and clicked it.",
  )?;
  let screenshot_artifact = ProducedArtifact {
    kind: "screenshot".to_string(),
    source_path: screenshot_path,
    preferred_name: format!("{}.png", sanitize_file_component(&label)),
    note: Some("Screenshot captured for visible OCR row detection before row click.".to_string()),
  };
  let mut notes = vec![
    format!("rowStrategy={}", detection.strategy),
    format!("rowIndex={}", row_index + 1),
    format!("detectedRowCount={}", rows.len()),
    format!("rowSource={}", row.source),
    format!("rowBounds={}", render_rect_compact(&row.bounds)),
    format!("rowText={}", row.text_fragments.join(" | ")),
    format!("rowAnchorMode={row_anchor_mode}"),
    format!("rowAnchorRatio={row_anchor_x_ratio:.3},{row_anchor_y_ratio:.3}"),
    format!("logicalPoint={logical_x:.3},{logical_y:.3}"),
    format!("settleMs={settle_ms}"),
  ];
  if let Some(app) = activated_app {
    notes.push(format!("activatedTargetBeforeCapture={app}"));
  }
  if let Some(region) = region.as_ref() {
    notes.push(render_ocr_region_note(region));
  }

  Ok(DriverResponse {
    summary: format!(
      "Clicked visible row {} with strategy {} at logical point ({logical_x:.3}, {logical_y:.3}).",
      row_index + 1,
      detection.strategy
    ),
    backend: Some(format!(
      "macos.vision.click-screen-row.{}",
      detection.strategy
    )),
    notes,
    artifacts: vec![screenshot_artifact, report_artifact],
  })
}

pub(super) fn click_point(call: &DriverCall) -> AuvResult<DriverResponse> {
  let x = required_f64(call, "x")?;
  let y = required_f64(call, "y")?;
  let click_count = optional_i64(call, "click_count")?.unwrap_or(1).clamp(1, 4);
  let settle_ms = optional_positive_u64(call, "settle_ms")?.unwrap_or(0);
  let (button_name, button_code) = parse_mouse_button(call)?;
  let snapshot = enumerate_displays()?;
  let resolution = resolve_display_point(&snapshot, x, y)
    .ok_or_else(|| format!("logical point ({x:.3}, {y:.3}) is outside all connected displays"))?;
  if let Some(app) = app_identifier(call) {
    if !app.is_empty() {
      activate_target_app(&app)?;
    }
  }
  run_swift_script(&build_click_point_script(x, y, button_code, click_count))?;
  if settle_ms > 0 {
    thread::sleep(Duration::from_millis(settle_ms));
  }
  let report = [
    format!("capturedAt={}", snapshot.captured_at),
    format!("globalLogicalPoint={x:.3},{y:.3}"),
    format!(
      "backingPixelPoint={},{}",
      resolution.backing_pixel_x, resolution.backing_pixel_y
    ),
    format!("displayId={}", resolution.display.display_id),
    format!("button={button_name}"),
    format!("clickCount={click_count}"),
    format!("settleMs={settle_ms}"),
    "coordinateSpace=global-logical".to_string(),
    "cursorAfter=target".to_string(),
  ]
  .join("\n")
    + "\n";
  let artifact = build_text_artifact(
    "click-point",
    "txt",
    &format!(
      "click-point-{}-{}",
      sanitize_file_component(&format!("{x:.3}")),
      sanitize_file_component(&format!("{y:.3}"))
    ),
    report,
    "Clicked a macOS logical point through Quartz and recorded its coordinate contract.",
  )?;

  Ok(DriverResponse {
    summary: format!(
      "Clicked {} at global logical point ({x:.3}, {y:.3}) on display #{}.",
      button_name, resolution.display.display_id
    ),
    backend: Some("macos.swift.quartz-click".to_string()),
    notes: vec![
      "coordinateSpace=global-logical".to_string(),
      format!("button={button_name}"),
      format!("clickCount={click_count}"),
      format!("settleMs={settle_ms}"),
      format!(
        "backingPixelPoint={},{}",
        resolution.backing_pixel_x, resolution.backing_pixel_y
      ),
      render_display_note(&resolution.display),
      "cursorAfter=target".to_string(),
    ],
    artifacts: vec![artifact],
  })
}

pub(super) fn scroll_point(call: &DriverCall) -> AuvResult<DriverResponse> {
  let x = required_f64(call, "x")?;
  let y = required_f64(call, "y")?;
  let (delta_x, delta_y, normalized_scroll) = resolve_scroll_deltas(call)?;
  let snapshot = enumerate_displays()?;
  let resolution = resolve_display_point(&snapshot, x, y)
    .ok_or_else(|| format!("logical point ({x:.3}, {y:.3}) is outside all connected displays"))?;
  if let Some(app) = app_identifier(call) {
    if !app.is_empty() {
      activate_target_app(&app)?;
    }
  }
  run_swift_script(&build_scroll_point_script(x, y, delta_x, delta_y))?;
  let report = [
    format!("capturedAt={}", snapshot.captured_at),
    format!("globalLogicalPoint={x:.3},{y:.3}"),
    format!(
      "backingPixelPoint={},{}",
      resolution.backing_pixel_x, resolution.backing_pixel_y
    ),
    format!("displayId={}", resolution.display.display_id),
    format!("deltaX={delta_x:.0}"),
    format!("deltaY={delta_y:.0}"),
    format!("normalizedScroll={normalized_scroll}"),
    "coordinateSpace=global-logical".to_string(),
    "cursorAfter=target".to_string(),
  ]
  .join("\n")
    + "\n";
  let artifact = build_text_artifact(
    "scroll-point",
    "txt",
    &format!(
      "scroll-point-{}-{}",
      sanitize_file_component(&format!("{x:.3}")),
      sanitize_file_component(&format!("{y:.3}"))
    ),
    report,
    "Scrolled at a macOS logical point through Quartz and recorded its coordinate contract.",
  )?;

  Ok(DriverResponse {
    summary: format!(
      "Scrolled at global logical point ({x:.3}, {y:.3}) on display #{} with {}.",
      resolution.display.display_id, normalized_scroll
    ),
    backend: Some("macos.swift.quartz-scroll".to_string()),
    notes: vec![
      "coordinateSpace=global-logical".to_string(),
      format!("deltaX={delta_x:.0}"),
      format!("deltaY={delta_y:.0}"),
      format!("normalizedScroll={normalized_scroll}"),
      format!(
        "backingPixelPoint={},{}",
        resolution.backing_pixel_x, resolution.backing_pixel_y
      ),
      render_display_note(&resolution.display),
      "cursorAfter=target".to_string(),
    ],
    artifacts: vec![artifact],
  })
}
