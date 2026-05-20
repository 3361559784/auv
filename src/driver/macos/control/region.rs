use super::super::*;

pub(crate) fn observe_window_region(call: &DriverCall) -> AuvResult<DriverResponse> {
  let label = optional_string(call, "label").unwrap_or_else(|| "window-region-observe".to_string());
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
  let region = region_ratios_from_call(call)?;
  let capture = super::window_ocr::capture_resolved_window_observation(call, &label)?;
  let ocr_region = region.to_observed_rect(capture.dimensions.width, capture.dimensions.height)?;
  let detection = detect_screen_rows(
    capture.screenshot_path.as_path(),
    min_confidence,
    max_observations,
    Some(&ocr_region),
  )?;
  let rows = detection.rows;
  let json = render_observe_window_region_json(
    &rows,
    &ocr_region,
    &capture.dimensions,
    &capture.screenshot_path,
  )?;
  let json_artifact = build_text_artifact(
    "window-region-observation",
    "json",
    &format!("{}-rows", sanitize_file_component(&label)),
    json,
    "Machine-readable OCR row observation for a window region.",
  )?;
  // WORKAROUND: Window-region observation records the screenshot and OCR-region
  // bounds, but does not yet emit a full capture contract artifact. Remove this
  // once window capture contract staging is shared with scan artifacts.
  let screenshot_artifact = ProducedArtifact {
    kind: "screenshot".to_string(),
    source_path: capture.screenshot_path,
    preferred_name: format!("{}.png", sanitize_file_component(&label)),
    note: Some("Window screenshot captured for region observation.".to_string()),
  };

  Ok(DriverResponse {
    summary: format!(
      "Observed {} OCR row(s) in resolved window region.",
      rows.len()
    ),
    backend: Some("macos.vision.observe-window-region".to_string()),
    signals: crate::driver::macos::observe::row_detection_signals(rows.len()),
    notes: vec![
      format!("scope={}", capture.scope),
      format!("windowRef={}", capture.capture_source),
      format!("region={}", render_rect_compact(&ocr_region)),
      format!("rows.count={}", rows.len()),
      format!("strategy={}", detection.strategy),
      format!("minConfidence={min_confidence:.3}"),
      format!("maxObservations={max_observations}"),
      format!(
        "screenshotPixels={}x{}",
        capture.dimensions.width, capture.dimensions.height
      ),
    ],
    artifacts: vec![screenshot_artifact, json_artifact],
  })
}

pub(crate) fn scroll_window_region(_call: &DriverCall) -> AuvResult<DriverResponse> {
  Err("scroll_window_region is not implemented yet".to_string())
}

#[derive(Clone, Copy, Debug)]
struct RegionRatios {
  left: f64,
  top: f64,
  right: f64,
  bottom: f64,
}

impl RegionRatios {
  fn to_observed_rect(self, width: i64, height: i64) -> AuvResult<ObservedRect> {
    if width <= 0 || height <= 0 {
      return Err(format!(
        "invalid screenshot dimensions {}x{}: expected positive width and height",
        width, height
      ));
    }
    let (left_px, right_px) = ratio_edges_to_pixels(self.left, self.right, width);
    let (top_px, bottom_px) = ratio_edges_to_pixels(self.top, self.bottom, height);

    Ok(ObservedRect {
      x: left_px,
      y: top_px,
      width: right_px - left_px,
      height: bottom_px - top_px,
    })
  }
}

fn ratio_edges_to_pixels(left: f64, right: f64, size: i64) -> (i64, i64) {
  let size_f = size as f64;
  let left_px = (left * size_f).floor() as i64;
  let left_px = left_px.clamp(0, size - 1);
  let right_px = (right * size_f).ceil() as i64;
  let right_px = right_px.clamp(left_px + 1, size);

  (left_px, right_px)
}

fn region_ratios_from_call(call: &DriverCall) -> AuvResult<RegionRatios> {
  let left = optional_f64(call, "region_left_ratio")?.unwrap_or(0.0);
  let top = optional_f64(call, "region_top_ratio")?.unwrap_or(0.0);
  let right = optional_f64(call, "region_right_ratio")?.unwrap_or(1.0);
  let bottom = optional_f64(call, "region_bottom_ratio")?.unwrap_or(1.0);
  validate_region_ratios(left, top, right, bottom)?;
  Ok(RegionRatios {
    left,
    top,
    right,
    bottom,
  })
}

fn validate_region_ratios(left: f64, top: f64, right: f64, bottom: f64) -> AuvResult<()> {
  if !(0.0 <= left && left < right && right <= 1.0) {
    return Err("invalid region x ratios: expected 0.0 <= left < right <= 1.0".to_string());
  }
  if !(0.0 <= top && top < bottom && bottom <= 1.0) {
    return Err("invalid region y ratios: expected 0.0 <= top < bottom <= 1.0".to_string());
  }
  Ok(())
}

fn render_observe_window_region_json(
  rows: &[ObservedOcrRow],
  region: &ObservedRect,
  dimensions: &ScreenshotDimensions,
  screenshot_path: &std::path::Path,
) -> AuvResult<String> {
  let rows = rows
    .iter()
    .map(|row| {
      serde_json::json!({
        "row_index": row.row_index,
        "source": row.source,
        "text": row.text_fragments.join(" | "),
        "text_fragments": row.text_fragments,
        "bounds": {
          "x": row.bounds.x,
          "y": row.bounds.y,
          "width": row.bounds.width,
          "height": row.bounds.height,
        },
      })
    })
    .collect::<Vec<_>>();
  serde_json::to_string_pretty(&serde_json::json!({
    "extractor": "ocr-row",
    "coordinate_space": "window_screenshot_pixels",
    "screenshot_path": screenshot_path.display().to_string(),
    "screenshot_width": dimensions.width,
    "screenshot_height": dimensions.height,
    "region": {
      "x": region.x,
      "y": region.y,
      "width": region.width,
      "height": region.height,
    },
    "rows": rows,
  }))
  .map(|mut rendered| {
    rendered.push('\n');
    rendered
  })
  .map_err(|error| format!("failed to render window region observation json: {error}"))
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn validate_region_ratios_rejects_inverted_region() {
    let error = validate_region_ratios(0.8, 0.2, 0.4, 0.9).expect_err("region should fail");
    assert!(error.contains("expected 0.0 <= left < right <= 1.0"));
  }

  #[test]
  fn validate_region_ratios_accepts_normal_region() {
    validate_region_ratios(0.1, 0.2, 0.9, 0.8).expect("region should pass");
  }

  #[test]
  fn region_ratios_to_observed_rect_clamps_near_right_edge() {
    let region = RegionRatios {
      left: 0.996,
      top: 0.996,
      right: 1.0,
      bottom: 1.0,
    }
    .to_observed_rect(100, 100)
    .expect("valid dimensions should convert");

    assert_eq!(
      region,
      ObservedRect {
        x: 99,
        y: 99,
        width: 1,
        height: 1,
      }
    );
    assert!(region.x < 100);
    assert!(region.y < 100);
    assert!(region.x + region.width <= 100);
    assert!(region.y + region.height <= 100);
  }

  #[test]
  fn render_observe_window_region_json_includes_coordinate_semantics() {
    let json = render_observe_window_region_json(
      &[],
      &ObservedRect {
        x: 99,
        y: 10,
        width: 1,
        height: 80,
      },
      &ScreenshotDimensions {
        width: 100,
        height: 200,
      },
      std::path::Path::new("/tmp/window.png"),
    )
    .expect("json should render");
    let value: serde_json::Value = serde_json::from_str(&json).expect("json should parse");

    assert_eq!(
      value["coordinate_space"],
      serde_json::Value::String("window_screenshot_pixels".to_string())
    );
    assert_eq!(value["screenshot_width"], serde_json::Value::from(100));
    assert_eq!(value["screenshot_height"], serde_json::Value::from(200));
  }
}
