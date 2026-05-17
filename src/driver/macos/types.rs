use std::path::PathBuf;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ObservedRect {
  pub(crate) x: i64,
  pub(crate) y: i64,
  pub(crate) width: i64,
  pub(crate) height: i64,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ObservedDisplay {
  pub(crate) display_id: u32,
  pub(crate) is_main: bool,
  pub(crate) is_built_in: bool,
  pub(crate) bounds: ObservedRect,
  pub(crate) visible_bounds: ObservedRect,
  pub(crate) scale_factor: f64,
  pub(crate) pixel_width: i64,
  pub(crate) pixel_height: i64,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ObservedDisplaySnapshot {
  pub(crate) displays: Vec<ObservedDisplay>,
  pub(crate) combined_bounds: ObservedRect,
  pub(crate) captured_at: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ObservedWindow {
  pub(crate) app_name: String,
  pub(crate) owner_pid: i64,
  pub(crate) layer: i64,
  pub(crate) title: String,
  pub(crate) bounds: ObservedRect,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ObservedWindowSnapshot {
  pub(crate) frontmost_app_name: String,
  pub(crate) frontmost_window_title: String,
  pub(crate) observed_at: String,
  pub(crate) windows: Vec<ObservedWindow>,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct OcrTextMatch {
  pub(crate) match_index: usize,
  pub(crate) text: String,
  pub(crate) confidence: f64,
  pub(crate) bounds: ObservedRect,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct OcrTextSnapshot {
  pub(crate) recognized_at: String,
  pub(crate) image_path: PathBuf,
  pub(crate) image_width: i64,
  pub(crate) image_height: i64,
  pub(crate) query: String,
  pub(crate) exact: bool,
  pub(crate) case_sensitive: bool,
  pub(crate) matches: Vec<OcrTextMatch>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ObservedOcrRow {
  pub(crate) row_index: usize,
  pub(crate) source: String,
  pub(crate) bounds: ObservedRect,
  pub(crate) text_fragments: Vec<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct DetectedScreenRows {
  pub(crate) strategy: String,
  pub(crate) raw_match_count: usize,
  pub(crate) filtered_match_count: usize,
  pub(crate) rows: Vec<ObservedOcrRow>,
  pub(crate) report: String,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ObservedPointResolution {
  pub(crate) display: ObservedDisplay,
  pub(crate) local_x: f64,
  pub(crate) local_y: f64,
  pub(crate) backing_pixel_x: i64,
  pub(crate) backing_pixel_y: i64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ScreenshotDimensions {
  pub(crate) width: i64,
  pub(crate) height: i64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ObservedAxNode {
  pub(crate) depth: usize,
  pub(crate) path: String,
  pub(crate) role: String,
  pub(crate) subrole: String,
  pub(crate) title: String,
  pub(crate) description: String,
  pub(crate) help: String,
  pub(crate) identifier: String,
  pub(crate) placeholder: String,
  pub(crate) value: String,
  pub(crate) bounds: ObservedRect,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ObservedAxTreeSnapshot {
  pub(crate) observed_at: String,
  pub(crate) app_name: String,
  pub(crate) bundle_id: String,
  pub(crate) window_title: String,
  pub(crate) nodes: Vec<ObservedAxNode>,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct CoordinateReadinessAssessment {
  pub(crate) ready_for_logical_input: bool,
  pub(crate) matches_main_logical: bool,
  pub(crate) matches_main_physical: bool,
  pub(crate) matches_combined_logical: bool,
  pub(crate) likely_retina_backing_mismatch: bool,
  pub(crate) reason: String,
}
