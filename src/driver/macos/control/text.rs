use super::super::*;
use super::common::activate_app_if_needed;

pub(crate) fn type_text(call: &DriverCall) -> AuvResult<DriverResponse> {
  let app = app_identifier(call).unwrap_or_default();
  let text = required_non_empty_string(call, "text")?;
  let replace_existing = optional_bool(call, "replace_existing")?.unwrap_or(false);
  let submit_key = optional_non_empty_string(call, "submit_key");
  let submit_settle_ms = optional_positive_u64(call, "submit_settle_ms")?.unwrap_or(0);

  activate_app_if_needed(&app)?;
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
    signals: std::collections::BTreeMap::new(),
    notes,
    artifacts: vec![artifact],
  })
}

pub(crate) fn paste_text_preserve_clipboard(call: &DriverCall) -> AuvResult<DriverResponse> {
  let app = app_identifier(call).unwrap_or_default();
  let text = required_non_empty_string(call, "text")?;
  let replace_existing = optional_bool(call, "replace_existing")?.unwrap_or(false);
  let submit_key = optional_non_empty_string(call, "submit_key");
  let submit_settle_ms = optional_positive_u64(call, "submit_settle_ms")?.unwrap_or(0);

  activate_app_if_needed(&app)?;
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
    signals: std::collections::BTreeMap::new(),
    notes,
    artifacts: vec![artifact],
  })
}

pub(crate) fn press_key(call: &DriverCall) -> AuvResult<DriverResponse> {
  let app = app_identifier(call).unwrap_or_default();
  let key = required_non_empty_string(call, "key")?;
  let settle_ms = optional_positive_u64(call, "settle_ms")?.unwrap_or(0);

  activate_app_if_needed(&app)?;
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
    signals: std::collections::BTreeMap::new(),
    notes: vec![
      format!("key={key}"),
      format!("settleMs={settle_ms}"),
      format!("app={app}"),
    ],
    artifacts: vec![artifact],
  })
}
