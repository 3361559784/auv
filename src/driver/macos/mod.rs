use std::env;
use std::fs;
use std::io::{ErrorKind, Read};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread;
use std::time::Duration;

use super::Driver;
use crate::model::{
  AuvResult, DriverCall, DriverDescriptor, DriverResponse, ProducedArtifact, now_millis,
};

mod control;
mod observe;
mod support;
#[cfg(test)]
mod tests;

use self::control::{
  activate_app, click_point, click_screen_row, click_screen_text, click_window_point,
  focus_text_input, paste_text_preserve_clipboard, press_button, press_key, scroll_point,
  type_text,
};
use self::observe::{
  capture_screen, find_image_text, find_screen_rows, find_screen_text, identify_point,
  observe_window_tree, observe_windows, probe_coordinate_readiness, probe_displays,
  probe_permissions, project_screenshot_point, verify_now_playing_title, wait_for_screen_rows,
  wait_for_screen_text,
};
use self::support::*;
pub(crate) use self::support::{copy_file, sanitized_artifact_name};

const PROBE_ACCESSIBILITY_SCRIPT: &str = include_str!("scripts/probe_accessibility.swift");
const PROBE_SCREEN_RECORDING_SCRIPT: &str = include_str!("scripts/probe_screen_recording.swift");
const ENUMERATE_DISPLAYS_SCRIPT: &str = include_str!("scripts/enumerate_displays.swift");
const OBSERVE_WINDOWS_SCRIPT_TEMPLATE: &str = include_str!("scripts/observe_windows.swift");
const OBSERVE_WINDOW_TREE_SCRIPT_TEMPLATE: &str = include_str!("scripts/observe_window_tree.swift");
const OCR_FIND_TEXT_SCRIPT_TEMPLATE: &str = include_str!("scripts/ocr_find_text.swift");
const FIND_VISUAL_ROWS_SCRIPT_TEMPLATE: &str = include_str!("scripts/find_visual_rows.swift");
const CLICK_POINT_SCRIPT_TEMPLATE: &str = include_str!("scripts/click_point.swift");
const SCROLL_POINT_SCRIPT_TEMPLATE: &str = include_str!("scripts/scroll_point.swift");
const CAPTURE_CLIPBOARD_SCRIPT: &str = include_str!("scripts/capture_clipboard.swift");
const RESTORE_CLIPBOARD_SCRIPT_TEMPLATE: &str = include_str!("scripts/restore_clipboard.swift");
const SET_CLIPBOARD_TEXT_SCRIPT_TEMPLATE: &str = include_str!("scripts/set_clipboard_text.swift");

const XCRUN_BINARY: &str = "/usr/bin/xcrun";
const OSASCRIPT_BINARY: &str = "/usr/bin/osascript";
const SCREEN_CAPTURE_BINARY: &str = "/usr/sbin/screencapture";

pub(crate) struct MacOsObserveDriver;

impl Driver for MacOsObserveDriver {
  fn descriptor(&self) -> DriverDescriptor {
    DriverDescriptor {
      id: "macos.observe",
      summary: "Observation-first desktop donor primitives extracted into the shared AUV driver protocol.",
      capabilities: &[
        "observe.screenshot",
        "observe.windows",
        "observe.window-tree",
        "observe.permissions",
        "observe.displays",
        "observe.identify-point",
        "observe.project-screenshot-point",
        "observe.coordinate-readiness",
        "observe.screen-text",
        "observe.wait-screen-text",
        "observe.screen-rows",
        "observe.wait-screen-rows",
        "observe.image-text",
        "control.activate-app",
        "control.focus-text-input",
        "control.press-button",
        "control.type-text",
        "control.paste-text-preserve-clipboard",
        "control.press-key",
        "control.click-point",
        "control.click-window-point",
        "control.click-screen-text",
        "control.click-screen-row",
        "control.scroll-point",
      ],
      donor_boundary: "Borrow host observation primitives from AIRI, but keep MCP tools, action executors, approval queues, and workflow shells out of AUV core.",
    }
  }

  fn invoke(&self, call: &DriverCall) -> AuvResult<DriverResponse> {
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
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ObservedRect {
  x: i64,
  y: i64,
  width: i64,
  height: i64,
}

#[derive(Clone, Debug, PartialEq)]
struct ObservedDisplay {
  display_id: u32,
  is_main: bool,
  is_built_in: bool,
  bounds: ObservedRect,
  visible_bounds: ObservedRect,
  scale_factor: f64,
  pixel_width: i64,
  pixel_height: i64,
}

#[derive(Clone, Debug, PartialEq)]
struct ObservedDisplaySnapshot {
  displays: Vec<ObservedDisplay>,
  combined_bounds: ObservedRect,
  captured_at: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ObservedWindow {
  app_name: String,
  owner_pid: i64,
  layer: i64,
  title: String,
  bounds: ObservedRect,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ObservedWindowSnapshot {
  frontmost_app_name: String,
  frontmost_window_title: String,
  observed_at: String,
  windows: Vec<ObservedWindow>,
}

#[derive(Clone, Debug, PartialEq)]
struct OcrTextMatch {
  match_index: usize,
  text: String,
  confidence: f64,
  bounds: ObservedRect,
}

#[derive(Clone, Debug, PartialEq)]
struct OcrTextSnapshot {
  recognized_at: String,
  image_path: PathBuf,
  image_width: i64,
  image_height: i64,
  query: String,
  exact: bool,
  case_sensitive: bool,
  matches: Vec<OcrTextMatch>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ObservedOcrRow {
  row_index: usize,
  source: String,
  bounds: ObservedRect,
  text_fragments: Vec<String>,
}

#[derive(Clone, Debug, PartialEq)]
struct DetectedScreenRows {
  strategy: String,
  raw_match_count: usize,
  filtered_match_count: usize,
  rows: Vec<ObservedOcrRow>,
  report: String,
}

#[derive(Clone, Debug, PartialEq)]
struct ObservedPointResolution {
  display: ObservedDisplay,
  local_x: f64,
  local_y: f64,
  backing_pixel_x: i64,
  backing_pixel_y: i64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ScreenshotDimensions {
  width: i64,
  height: i64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ObservedAxNode {
  depth: usize,
  path: String,
  role: String,
  subrole: String,
  title: String,
  description: String,
  help: String,
  identifier: String,
  placeholder: String,
  value: String,
  bounds: ObservedRect,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ObservedAxTreeSnapshot {
  observed_at: String,
  app_name: String,
  bundle_id: String,
  window_title: String,
  nodes: Vec<ObservedAxNode>,
}

#[derive(Clone, Debug, PartialEq)]
struct CoordinateReadinessAssessment {
  ready_for_logical_input: bool,
  matches_main_logical: bool,
  matches_main_physical: bool,
  matches_combined_logical: bool,
  likely_retina_backing_mismatch: bool,
  reason: String,
}
