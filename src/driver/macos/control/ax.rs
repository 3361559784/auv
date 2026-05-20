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

  let press_report = run_swift_script(&build_ax_press_path_script(
    snapshot.pid,
    &matched.path,
    &matched.role,
    &action_name,
  ))?;
  let performed_action = report_value(&press_report, "performedAction=")
    .unwrap_or("")
    .to_string();
  let available_actions = report_value(&press_report, "availableActions=")
    .unwrap_or("")
    .to_string();

  let (center_x, center_y) = ax_node_center(matched);
  let report = render_ax_interaction_report("ax-press-button", &snapshot, matched, &query);
  let report = format!(
    "{report}performedAction={performed_action}\navailableActions={available_actions}\npressMechanism=ax-action\ncursorDisturbance=none\nactivatedApp={activate}\n"
  );
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
    backend: Some("macos.ax.perform-action".to_string()),
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
