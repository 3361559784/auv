#[cfg(target_os = "macos")]
use super::ffi::ffi::{
  NativeActionResponse, click_point as native_click_point, scroll_point as native_scroll_point,
};
use crate::model::AuvResult;

#[cfg(target_os = "macos")]
pub(crate) fn click_point(
  x: f64,
  y: f64,
  button_code: i32,
  click_count: i64,
  click_interval_ms: u64,
) -> AuvResult<()> {
  action_result(
    "click_point",
    native_click_point(x, y, button_code, click_count, click_interval_ms),
  )
}

#[cfg(not(target_os = "macos"))]
pub(crate) fn click_point(
  _x: f64,
  _y: f64,
  _button_code: i32,
  _click_count: i64,
  _click_interval_ms: u64,
) -> AuvResult<()> {
  Err("macOS native pointer click is unsupported on this target".to_string())
}

#[cfg(target_os = "macos")]
pub(crate) fn scroll_point(x: f64, y: f64, delta_x: f64, delta_y: f64) -> AuvResult<()> {
  action_result("scroll_point", native_scroll_point(x, y, delta_x, delta_y))
}

#[cfg(not(target_os = "macos"))]
pub(crate) fn scroll_point(_x: f64, _y: f64, _delta_x: f64, _delta_y: f64) -> AuvResult<()> {
  Err("macOS native pointer scroll is unsupported on this target".to_string())
}

#[cfg(target_os = "macos")]
fn action_result(operation: &str, response: NativeActionResponse) -> AuvResult<()> {
  super::error::native_result(
    operation,
    response.ok.then_some(()),
    response.error_message,
    response.recovery_hint,
  )
}

#[cfg(test)]
mod tests {
  use super::*;

  #[cfg(target_os = "macos")]
  #[test]
  fn action_result_includes_operation_name() {
    let error = action_result(
      "click_point",
      NativeActionResponse {
        ok: false,
        error_message: Some("event creation failed".to_string()),
        recovery_hint: Some("grant Accessibility permission".to_string()),
      },
    )
    .unwrap_err();

    assert!(error.contains("click_point"));
    assert!(error.contains("event creation failed"));
  }
}
