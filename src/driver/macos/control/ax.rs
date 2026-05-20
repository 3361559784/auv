use std::thread;
use std::time::Duration;

use super::super::overlay::{OverlayWrapperOutcome, with_overlay_cursor};
use super::super::*;
use super::common::{
  DEFAULT_CLICK_INTERVAL_MS, activate_app_if_needed, build_ax_click_notes,
  send_reveal_shortcut_if_needed,
};

pub(crate) fn focus_text_input(call: &DriverCall) -> AuvResult<DriverResponse> {
  let app = app_identifier(call).unwrap_or_default();
  let query = required_non_empty_string(call, "query")?;
  let reveal_shortcut = optional_non_empty_string(call, "reveal_shortcut");
  let reveal_settle_ms = optional_positive_u64(call, "reveal_settle_ms")?.unwrap_or(250);
  let max_depth = optional_i64(call, "max_depth")?.unwrap_or(6).clamp(1, 10);
  let max_children = optional_i64(call, "max_children")?
    .unwrap_or(16)
    .clamp(1, 50);

  activate_app_if_needed(&app)?;
  send_reveal_shortcut_if_needed(reveal_shortcut.as_deref(), reveal_settle_ms)?;

  let tree_report = run_swift_script(&build_observe_window_tree_script(
    &app,
    max_depth,
    max_children,
  ))?;
  let snapshot = parse_observed_ax_tree(&tree_report)?;
  let matched = find_best_ax_node(&snapshot, &query)
    .ok_or_else(|| no_matching_ax_node_error(&snapshot, &query, "text input-like"))?;
  let (center_x, center_y) = ax_node_center(matched);
  run_swift_script(&build_click_point_script(
    center_x,
    center_y,
    0,
    1,
    DEFAULT_CLICK_INTERVAL_MS,
  ))?;

  let report = render_ax_interaction_report("focus-text-input", &snapshot, matched, &query);
  let artifact = build_text_artifact(
    "focus-text-input",
    "txt",
    &format!("focus-text-input-{}", sanitize_file_component(&query)),
    report,
    "Focused a text input by matching the observed AX tree and clicking the resolved bounds.",
  )?;
  let mut notes = build_ax_click_notes(&query, matched, center_x, center_y);
  if let Some(shortcut) = reveal_shortcut.as_deref() {
    notes.push(format!("revealShortcut={shortcut}"));
    notes.push(format!("revealSettleMs={reveal_settle_ms}"));
  }
  if !matched.placeholder.is_empty() {
    notes.push(format!("matchedPlaceholder={}", matched.placeholder));
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
    signals: std::collections::BTreeMap::new(),
    notes,
    artifacts: vec![artifact],
  })
}

pub(crate) fn ax_press_button(call: &DriverCall) -> AuvResult<DriverResponse> {
  let app = app_identifier(call).unwrap_or_default();
  let query = required_non_empty_string(call, "query")?;
  let reveal_shortcut = optional_non_empty_string(call, "reveal_shortcut");
  let reveal_settle_ms = optional_positive_u64(call, "reveal_settle_ms")?.unwrap_or(250);
  let max_depth = optional_i64(call, "max_depth")?.unwrap_or(6).clamp(1, 10);
  let max_children = optional_i64(call, "max_children")?
    .unwrap_or(16)
    .clamp(1, 50);
  let activate = optional_bool(call, "activate")?.unwrap_or(true);
  let action_name =
    optional_non_empty_string(call, "action").unwrap_or_else(|| "AXPress".to_string());
  let overlay = optional_bool(call, "overlay")?.unwrap_or(false);
  let overlay_label = optional_non_empty_string(call, "label").unwrap_or_else(|| "AUV".to_string());
  let preview_ms =
    optional_positive_u64(call, "preview_ms")?.unwrap_or(if overlay { 250 } else { 0 });
  let settle_ms = optional_positive_u64(call, "settle_ms")?.unwrap_or(0);

  if activate {
    activate_app_if_needed(&app)?;
  }
  send_reveal_shortcut_if_needed(reveal_shortcut.as_deref(), reveal_settle_ms)?;

  let tree_report = run_swift_script(&build_observe_window_tree_script(
    &app,
    max_depth,
    max_children,
  ))?;
  let snapshot = parse_observed_ax_tree(&tree_report)?;
  if snapshot.pid <= 0 {
    return Err(format!(
      "observe_window_tree did not return a valid pid for app {:?} (got {}); cannot dispatch AX action",
      snapshot.app_name, snapshot.pid
    ));
  }
  let matched = find_best_ax_node(&snapshot, &query)
    .ok_or_else(|| no_matching_ax_node_error(&snapshot, &query, "AX-pressable"))?;
  let (center_x, center_y) = ax_node_center(matched);

  let press_script =
    build_ax_press_path_script(snapshot.pid, &matched.path, &matched.role, &action_name);
  let (press_report, overlay_outcome): (String, Option<OverlayWrapperOutcome>) = if overlay {
    let (report, outcome) = with_overlay_cursor(center_x, center_y, &overlay_label, || {
      if preview_ms > 0 {
        thread::sleep(Duration::from_millis(preview_ms));
      }
      let report = run_swift_script(&press_script)?;
      if settle_ms > 0 {
        thread::sleep(Duration::from_millis(settle_ms));
      }
      Ok(report)
    })?;
    (report, Some(outcome))
  } else {
    (run_swift_script(&press_script)?, None)
  };
  let performed_action = report_value(&press_report, "performedAction=")
    .unwrap_or("")
    .to_string();
  let available_actions = report_value(&press_report, "availableActions=")
    .unwrap_or("")
    .to_string();

  let report = render_ax_interaction_report("ax-press-button", &snapshot, matched, &query);
  let mut report = format!(
    "{report}performedAction={performed_action}\navailableActions={available_actions}\npressMechanism=ax-action\ncursorDisturbance=none\nactivatedApp={activate}\noverlayPresentation={}\n",
    if overlay { "visual-only" } else { "off" },
  );
  if let Some(outcome) = &overlay_outcome {
    report.push_str(&format!("overlayShowEvent={}\n", outcome.show_event));
    report.push_str(&format!("overlayHideEvent={}\n", outcome.hide_event));
    report.push_str(&format!("daemonPid={}\n", outcome.daemon_pid));
    report.push_str(&format!("previewMs={preview_ms}\n"));
    report.push_str(&format!("settleMs={settle_ms}\n"));
    report.push_str(&format!("overlayLabel={overlay_label}\n"));
  }
  let artifact = build_text_artifact(
    "ax-press-button",
    "txt",
    &format!("ax-press-button-{}", sanitize_file_component(&query)),
    report,
    "Pressed a control via AXUIElementPerformAction; the real cursor is not moved.",
  )?;

  let mut notes = build_ax_click_notes(&query, matched, center_x, center_y);
  notes.push("pressMechanism=ax-action".to_string());
  notes.push("cursorDisturbance=none".to_string());
  notes.push(format!("performedAction={performed_action}"));
  if !available_actions.is_empty() {
    notes.push(format!("availableActions={available_actions}"));
  }
  notes.push(format!("activatedApp={activate}"));
  if let Some(outcome) = &overlay_outcome {
    notes.push("overlayPresentation=visual-only".to_string());
    notes.push(format!("overlayShowEvent={}", outcome.show_event));
    notes.push(format!("overlayHideEvent={}", outcome.hide_event));
    notes.push(format!("daemonPid={}", outcome.daemon_pid));
    notes.push(format!("previewMs={preview_ms}"));
    notes.push(format!("settleMs={settle_ms}"));
    notes.push(format!("overlayLabel={overlay_label}"));
  }
  if let Some(shortcut) = reveal_shortcut.as_deref() {
    notes.push(format!("revealShortcut={shortcut}"));
    notes.push(format!("revealSettleMs={reveal_settle_ms}"));
  }
  if !matched.help.is_empty() {
    notes.push(format!("matchedHelp={}", matched.help));
  }

  let mut signals = std::collections::BTreeMap::new();
  signals.insert("pressMechanism".to_string(), "ax-action".to_string());
  signals.insert("cursorDisturbance".to_string(), "none".to_string());
  signals.insert("performedAction".to_string(), performed_action.clone());
  if !available_actions.is_empty() {
    signals.insert("availableActions".to_string(), available_actions);
  }
  if let Some(outcome) = &overlay_outcome {
    signals.insert(
      "overlayEvent".to_string(),
      format!("{}+{}", outcome.show_event, outcome.hide_event),
    );
    signals.insert("daemonPid".to_string(), outcome.daemon_pid.to_string());
  }

  let backend = if overlay {
    "macos.ax.perform-action+overlay-daemon"
  } else {
    "macos.ax.perform-action"
  };

  Ok(DriverResponse {
    summary: if matched.title.is_empty() && matched.description.is_empty() {
      format!(
        "Pressed button-like control in {} via AXUIElementPerformAction using query {} (role {}).",
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
        "Pressed {} in {} via AXUIElementPerformAction using query {}.",
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
    backend: Some(backend.to_string()),
    signals,
    notes,
    artifacts: vec![artifact],
  })
}

pub(crate) fn press_button(call: &DriverCall) -> AuvResult<DriverResponse> {
  let app = app_identifier(call).unwrap_or_default();
  let query = required_non_empty_string(call, "query")?;
  let reveal_shortcut = optional_non_empty_string(call, "reveal_shortcut");
  let reveal_settle_ms = optional_positive_u64(call, "reveal_settle_ms")?.unwrap_or(250);
  let max_depth = optional_i64(call, "max_depth")?.unwrap_or(6).clamp(1, 10);
  let max_children = optional_i64(call, "max_children")?
    .unwrap_or(16)
    .clamp(1, 50);

  activate_app_if_needed(&app)?;
  send_reveal_shortcut_if_needed(reveal_shortcut.as_deref(), reveal_settle_ms)?;

  let tree_report = run_swift_script(&build_observe_window_tree_script(
    &app,
    max_depth,
    max_children,
  ))?;
  let snapshot = parse_observed_ax_tree(&tree_report)?;
  let matched = find_best_ax_node(&snapshot, &query)
    .ok_or_else(|| no_matching_ax_node_error(&snapshot, &query, "button-like"))?;
  let (center_x, center_y) = ax_node_center(matched);
  run_swift_script(&build_click_point_script(
    center_x,
    center_y,
    0,
    1,
    DEFAULT_CLICK_INTERVAL_MS,
  ))?;

  let report = render_ax_interaction_report("press-button", &snapshot, matched, &query);
  let artifact = build_text_artifact(
    "press-button",
    "txt",
    &format!("press-button-{}", sanitize_file_component(&query)),
    report,
    "Pressed a known control by matching the observed AX tree and clicking the resolved bounds.",
  )?;
  let mut notes = build_ax_click_notes(&query, matched, center_x, center_y);
  if let Some(shortcut) = reveal_shortcut.as_deref() {
    notes.push(format!("revealShortcut={shortcut}"));
    notes.push(format!("revealSettleMs={reveal_settle_ms}"));
  }
  if !matched.help.is_empty() {
    notes.push(format!("matchedHelp={}", matched.help));
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
    signals: std::collections::BTreeMap::new(),
    notes,
    artifacts: vec![artifact],
  })
}
