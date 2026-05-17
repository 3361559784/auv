use super::control::{
  activate_app, click_point, click_screen_row, click_screen_text, click_window_point,
  focus_text_input, paste_text_preserve_clipboard, press_button, press_key, scroll_point,
  type_text,
};
use super::observe::{
  capture_screen, find_image_text, find_screen_rows, find_screen_text, identify_point,
  observe_window_tree, observe_windows, probe_coordinate_readiness, probe_displays,
  probe_permissions, project_screenshot_point, verify_ax_text, verify_now_playing_title,
  wait_for_screen_rows, wait_for_screen_text,
};
use super::{
  Driver, DriverCall, DriverDescriptor, DriverResponse, MacOsObserveDriver, descriptor,
  require_macos,
};
use crate::model::AuvResult;

impl Driver for MacOsObserveDriver {
  fn descriptor(&self) -> DriverDescriptor {
    descriptor::driver_descriptor()
  }

  fn invoke(&self, call: &DriverCall) -> AuvResult<DriverResponse> {
    invoke_operation(call)
  }
}

pub(crate) fn invoke_operation(call: &DriverCall) -> AuvResult<DriverResponse> {
  require_macos()?;

  match call.operation.as_str() {
    "capture_screen" => capture_screen(call),
    "probe_coordinate_readiness" => probe_coordinate_readiness(call),
    "probe_displays" => probe_displays(call),
    "project_screenshot_point" => project_screenshot_point(call),
    "identify_point" => identify_point(call),
    "observe_windows" => observe_windows(call),
    "observe_window_tree" => observe_window_tree(call),
    "find_screen_text" => find_screen_text(call),
    "wait_for_screen_text" => wait_for_screen_text(call),
    "find_screen_rows" => find_screen_rows(call),
    "wait_for_screen_rows" => wait_for_screen_rows(call),
    "find_image_text" => find_image_text(call),
    "probe_permissions" => probe_permissions(call),
    "verify_ax_text" => verify_ax_text(call),
    "verify_now_playing_title" => verify_now_playing_title(call),
    "activate_app" => activate_app(call),
    "focus_text_input" => focus_text_input(call),
    "press_button" => press_button(call),
    "type_text" => type_text(call),
    "paste_text_preserve_clipboard" => paste_text_preserve_clipboard(call),
    "press_key" => press_key(call),
    "click_point" => click_point(call),
    "click_window_point" => click_window_point(call),
    "click_screen_text" => click_screen_text(call),
    "click_screen_row" => click_screen_row(call),
    "scroll_point" => scroll_point(call),
    other => Err(format!(
      "driver macos.observe does not support operation {}",
      other
    )),
  }
}
