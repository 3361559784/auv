use std::collections::BTreeMap;

use super::super::*;

pub(crate) fn observe_window_region(call: &DriverCall) -> AuvResult<DriverResponse> {
  let label = optional_string(call, "label").unwrap_or_else(|| "window-region-observe".to_string());
  let app_bundle_id = app_identifier(call).filter(|value| looks_like_bundle_identifier(value));
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
  let (display_ref, native_display_id) = match &capture.capture_contract.capture_source {
    crate::driver::macos::capture::types::CaptureSource::Window {
      display_ref,
      native_display_id,
      ..
    } => (Some(display_ref.as_str()), Some(native_display_id.as_str())),
    _ => (None, None),
  };
  let recognition_artifact = row_recognition_artifact(
    "window-region-recognition",
    &format!("{}-recognition", sanitize_file_component(&label)),
    "Structured recognition result for OCR row observation in a window region.",
    RowRecognitionArtifactRequest {
      recognition_id: format!("window_region_{}", sanitize_file_component(&label)),
      source: recognition_source_for_rows(&detection.strategy, &rows),
      surface: crate::contract::RecognitionSurface::Region,
      rows: &rows,
      strategy: &detection.strategy,
      raw_match_count: detection.raw_match_count,
      filtered_match_count: detection.filtered_match_count,
      screenshot_path: capture.screenshot_path.as_path(),
      screenshot_dimensions: &capture.dimensions,
      display_ref,
      native_display_id,
      app_bundle_id: app_bundle_id.as_deref(),
      window_title: None,
      window_number: window_number_from_ref(&capture.capture_source),
      region_hint: Some(observed_rect_to_ratio_region(
        &ocr_region,
        &capture.dimensions,
      )),
      capture_contract: Some(&capture.capture_contract),
      additional_detail: serde_json::json!({
        "scope": &capture.scope,
        "capture_source": &capture.capture_source,
        "region_pixels": {
          "x": ocr_region.x,
          "y": ocr_region.y,
          "width": ocr_region.width,
          "height": ocr_region.height,
        },
        "max_observations": max_observations,
        "min_confidence": min_confidence,
      }),
      known_limits: vec![
        "driver-stage recognition evidence has no runtime artifact refs yet".to_string(),
        "region observation still uses heuristic row filtering for list semantics".to_string(),
      ],
    },
  )?;
  // TODO: Emit a full typed capture contract artifact for window-region
  // observation. This command records the screenshot and OCR-region bounds so
  // scroll scan can crop list item candidates, but it still lacks the same
  // reusable capture contract produced by the dedicated capture commands.
  // TODO: Extend window-region observation into a real region-segmentation
  // pass. Scroll scan needs candidates for list bodies, section separators,
  // sticky headers, empty states, and scrollbars/thumbs so the scan loop can
  // distinguish top/bottom boundaries from ordinary repeated content.
  let screenshot_artifact = ProducedArtifact {
    kind: "screenshot".to_string(),
    source_path: capture.screenshot_path,
    preferred_name: format!("{}.png", sanitize_file_component(&label)),
    note: Some("Window screenshot captured for region observation.".to_string()),
  };
  let mut artifacts = vec![
    screenshot_artifact.clone(),
    json_artifact,
    recognition_artifact,
  ];
  let overlay_rows = rows
    .iter()
    .map(|row| OverlayEvidenceRow {
      row_index: row.row_index,
      source: row.source.clone(),
      bounds: row.bounds.clone(),
      text_fragments: row.text_fragments.clone(),
    })
    .collect::<Vec<_>>();
  artifacts.extend(build_row_observation_overlay_artifacts(
    RowObservationOverlayRequest {
      label: label.clone(),
      screenshot_path: screenshot_artifact.source_path.clone(),
      screenshot_dimensions: capture.dimensions.clone(),
      strategy: detection.strategy.clone(),
      rows: overlay_rows,
    },
  )?);

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
    artifacts,
  })
}

pub(crate) fn scroll_window_region(call: &DriverCall) -> AuvResult<DriverResponse> {
  let app = app_identifier(call).ok_or_else(|| {
    "scroll_window_region requires --target <application-id> or --app".to_string()
  })?;
  let raw_direction = optional_string(call, "direction").unwrap_or_else(|| "down".to_string());
  let direction = normalize_scroll_direction(&raw_direction)?.to_string();
  let amount = optional_f64(call, "amount")?.unwrap_or(6.0).max(1.0);
  let settle_ms = optional_positive_u64(call, "settle_ms")?.unwrap_or(250);
  let region = region_ratios_from_call(call)?;
  activate_target_app(&app)?;
  let snapshot = super::super::observe::observe_windows_snapshot(24, &app)?;
  let xcap_displays = super::super::capture::xcap_backend::list_displays()?;
  let display_snapshot = enumerate_displays()?;
  let selector = parse_app_selector(&app)?;
  let resolved_app = resolve_app_ref(&snapshot, &selector)?;
  let candidate = resolve_window_candidate(
    &snapshot,
    &resolved_app,
    &xcap_displays,
    &parse_window_selection(call)?,
  )?;
  let window = &candidate.window_ref;
  let x =
    window.bounds.x as f64 + window.bounds.width as f64 * ((region.left + region.right) / 2.0);
  let y =
    window.bounds.y as f64 + window.bounds.height as f64 * ((region.top + region.bottom) / 2.0);
  let resolution = resolve_display_point(&display_snapshot, x, y).ok_or_else(|| {
    format!("resolved scroll point ({x:.3}, {y:.3}) is outside all connected displays")
  })?;
  let (delta_x, delta_y) = scan_scroll_delta(&direction, amount)?;
  crate::driver::macos::native::pointer::scroll_point(x, y, delta_x, delta_y)?;
  if settle_ms > 0 {
    std::thread::sleep(std::time::Duration::from_millis(settle_ms));
  }

  let report = [
    "coordinateSpace=global-logical".to_string(),
    format!("applicationId={app}"),
    format!("appSelector={}", resolved_app.selector.raw),
    format!("matchStrategy={}", resolved_app.match_strategy),
    format!(
      "resolvedAppBundleId={}",
      resolved_app
        .resolved_bundle_id
        .clone()
        .unwrap_or_else(|| "n/a".to_string())
    ),
    format!("resolvedAppName={}", resolved_app.resolved_app_name),
    format!("windowId={}", window.window_number),
    format!("windowTitle={}", window.title),
    format!("windowBounds={}", render_rect_compact(&window.bounds)),
    format!(
      "regionRatios={:.3},{:.3},{:.3},{:.3}",
      region.left, region.top, region.right, region.bottom
    ),
    format!("scrollPoint={x:.3},{y:.3}"),
    format!(
      "backingPixelPoint={},{}",
      resolution.backing_pixel_x, resolution.backing_pixel_y
    ),
    format!("displayId={}", resolution.display.display_id),
    format!(
      "displayBounds={}",
      render_rect_compact(&resolution.display.bounds)
    ),
    format!("candidateIndex={}", candidate.candidate_index),
    format!("selectionReason={}", candidate.selection_reason),
    format!(
      "isFullyContainedInDisplay={}",
      candidate.is_fully_contained_in_display
    ),
    format!(
      "displayRef={}",
      candidate
        .display_ref
        .clone()
        .unwrap_or_else(|| "n/a".to_string())
    ),
    format!(
      "nativeDisplayId={}",
      candidate
        .native_display_id
        .clone()
        .unwrap_or_else(|| "n/a".to_string())
    ),
    format!("candidateArea={}", candidate.area),
    format!("direction={direction}"),
    format!("amount={amount:.3}"),
    format!("deltaX={delta_x:.0}"),
    format!("deltaY={delta_y:.0}"),
    format!("settleMs={settle_ms}"),
  ]
  .join("\n");
  let artifact = build_text_artifact(
    "window-region-scroll",
    "txt",
    "window-region-scroll",
    report,
    "Scrolled at the center of a resolved window region.",
  )?;

  Ok(DriverResponse {
    summary: format!("Scrolled window region {direction} by amount {amount:.3}."),
    backend: Some("macos.swift.quartz-scroll-window-region".to_string()),
    signals: BTreeMap::from([
      ("scroll.direction".to_string(), direction),
      ("scroll.amount".to_string(), format!("{amount:.3}")),
    ]),
    notes: vec![
      "coordinateSpace=global-logical".to_string(),
      format!("windowId={}", window.window_number),
      format!("windowBounds={}", render_rect_compact(&window.bounds)),
      format!(
        "regionRatios={:.3},{:.3},{:.3},{:.3}",
        region.left, region.top, region.right, region.bottom
      ),
      format!("scrollPoint={x:.3},{y:.3}"),
      format!(
        "backingPixelPoint={},{}",
        resolution.backing_pixel_x, resolution.backing_pixel_y
      ),
      format!("displayId={}", resolution.display.display_id),
      format!("candidateIndex={}", candidate.candidate_index),
      format!("selectionReason={}", candidate.selection_reason),
      format!("deltaX={delta_x:.0}"),
      format!("deltaY={delta_y:.0}"),
      format!("settleMs={settle_ms}"),
    ],
    artifacts: vec![artifact],
  })
}

fn scan_scroll_delta(direction: &str, amount: f64) -> AuvResult<(f64, f64)> {
  match normalize_scroll_direction(direction)? {
    "down" => Ok((0.0, -amount)),
    "up" => Ok((0.0, amount)),
    "right" => Ok((-amount, 0.0)),
    "left" => Ok((amount, 0.0)),
    _ => unreachable!("normalize_scroll_direction only returns supported directions"),
  }
}

fn normalize_scroll_direction(direction: &str) -> AuvResult<&'static str> {
  match direction.trim().to_ascii_lowercase().as_str() {
    "down" => Ok("down"),
    "up" => Ok("up"),
    "right" => Ok("right"),
    "left" => Ok("left"),
    other => Err(format!(
      "invalid scroll direction {other:?}; expected down, up, left, or right"
    )),
  }
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
  let filter = filter_list_row_candidates(rows);
  let row_candidates = rows
    .iter()
    .map(|row| {
      let accepted = filter.accepted_indices.contains(&row.row_index);
      let reject_reason = filter
        .rejected
        .iter()
        .find(|rejected| rejected.row_index == row.row_index)
        .map(|rejected| rejected.reason.as_str());
      let mut value = serde_json::json!({
        "row_index": row.row_index,
        "source": row.source,
        "text": row.text_fragments.join(" | "),
        "text_fragments": row.text_fragments,
        "accepted_by_row_filter": accepted,
        "bounds": {
          "x": row.bounds.x,
          "y": row.bounds.y,
          "width": row.bounds.width,
          "height": row.bounds.height,
        },
      });
      if let Some(reason) = reject_reason {
        value["reject_reason"] = serde_json::Value::String(reason.to_string());
      }
      value
    })
    .collect::<Vec<_>>();
  let item_candidates = rows
    .iter()
    .filter(|row| filter.accepted_indices.contains(&row.row_index))
    .enumerate()
    .map(|(item_index, row)| {
      serde_json::json!({
        "item_index": item_index,
        "row_candidate_index": row.row_index,
        "source": "row_filter",
        "text": row.text_fragments.join(" | "),
        "text_fragments": row.text_fragments,
        "filter_reason": "accepted_repeating_row_geometry",
        "segmented_region_role": "list_region",
        "bounds": {
          "x": row.bounds.x,
          "y": row.bounds.y,
          "width": row.bounds.width,
          "height": row.bounds.height,
        },
      })
    })
    .collect::<Vec<_>>();
  let rejected_row_candidates = filter
    .rejected
    .iter()
    .filter_map(|rejected| {
      rows
        .iter()
        .find(|row| row.row_index == rejected.row_index)
        .map(|row| (rejected, row))
    })
    .map(|(rejected, row)| {
      serde_json::json!({
        "row_candidate_index": row.row_index,
        "reject_reason": rejected.reason,
        "source": row.source,
        "bounds": {
          "x": row.bounds.x,
          "y": row.bounds.y,
          "width": row.bounds.width,
          "height": row.bounds.height,
        },
      })
    })
    .collect::<Vec<_>>();
  let segmented_regions = filter
    .list_region
    .as_ref()
    .map(|list_region| {
      vec![serde_json::json!({
        "region_index": 0,
        "role": "list_region",
        "confidence": filter.confidence,
        "evidence": filter.evidence,
        "bounds": {
          "x": list_region.x,
          "y": list_region.y,
          "width": list_region.width,
          "height": list_region.height,
        },
      })]
    })
    .unwrap_or_default();
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
    "segmented_regions": segmented_regions,
    "rows": row_candidates,
    "row_candidates": row_candidates,
    "rejected_row_candidates": rejected_row_candidates,
    "item_candidates": item_candidates,
  }))
  .map(|mut rendered| {
    rendered.push('\n');
    rendered
  })
  .map_err(|error| format!("failed to render window region observation json: {error}"))
}

#[derive(Clone, Debug)]
struct ListRowFilterResult {
  accepted_indices: Vec<usize>,
  rejected: Vec<RejectedRowCandidate>,
  list_region: Option<ObservedRect>,
  confidence: &'static str,
  evidence: &'static str,
}

#[derive(Clone, Debug)]
struct RejectedRowCandidate {
  row_index: usize,
  reason: String,
}

fn filter_list_row_candidates(rows: &[ObservedOcrRow]) -> ListRowFilterResult {
  if rows.is_empty() {
    return ListRowFilterResult {
      accepted_indices: Vec::new(),
      rejected: Vec::new(),
      list_region: None,
      confidence: "none",
      evidence: "no_row_candidates",
    };
  }

  // TODO: Add OCR/AX anchor evidence and optional icon/template matching to the
  // Row Filter before making semantic decisions. This is still geometry-only
  // and intentionally rejects only clear height outliers so likely list rows
  // remain available to later hooks. Icon evidence will matter for rows whose
  // identity or state is mostly visual, such as selected/liked/downloaded
  // markers or section-specific affordances.
  let Some((height_band, evidence_count)) = repeating_row_height_band(rows) else {
    let accepted_indices = rows.iter().map(|row| row.row_index).collect::<Vec<_>>();
    return ListRowFilterResult {
      list_region: union_row_bounds(rows),
      accepted_indices,
      rejected: Vec::new(),
      confidence: "low",
      evidence: "insufficient_repeating_row_evidence",
    };
  };

  if rows.len() < 4 || evidence_count < 3 {
    let accepted_indices = rows.iter().map(|row| row.row_index).collect::<Vec<_>>();
    return ListRowFilterResult {
      list_region: union_row_bounds(rows),
      accepted_indices,
      rejected: Vec::new(),
      confidence: "low",
      evidence: "insufficient_repeating_row_evidence",
    };
  }

  let mut accepted = Vec::new();
  let mut rejected = Vec::new();
  for row in rows {
    if height_band.contains(row.bounds.height) {
      accepted.push(row.row_index);
    } else {
      rejected.push(RejectedRowCandidate {
        row_index: row.row_index,
        reason: "height_outside_repeating_row_band".to_string(),
      });
    }
  }

  let accepted_rows = rows
    .iter()
    .filter(|row| accepted.contains(&row.row_index))
    .cloned()
    .collect::<Vec<_>>();

  ListRowFilterResult {
    accepted_indices: accepted,
    rejected,
    list_region: union_row_bounds(&accepted_rows),
    confidence: "heuristic",
    evidence: "repeating_row_height_band",
  }
}

#[derive(Clone, Copy, Debug)]
struct RowHeightBand {
  min: i64,
  max: i64,
}

impl RowHeightBand {
  fn contains(self, height: i64) -> bool {
    self.min <= height && height <= self.max
  }
}

fn repeating_row_height_band(rows: &[ObservedOcrRow]) -> Option<(RowHeightBand, usize)> {
  let mut heights = rows.iter().map(|row| row.bounds.height).collect::<Vec<_>>();
  heights.sort_unstable();
  if heights.is_empty() {
    return None;
  }

  let sample = trimmed_height_sample(&heights);
  let median = sample[sample.len() / 2];
  let min = ((median as f64) * 0.80).floor() as i64;
  let max = ((median as f64) * 1.45).ceil() as i64;
  let band = RowHeightBand {
    min: min.max(1),
    max: max.max(min + 1),
  };
  let evidence_count = heights
    .iter()
    .filter(|height| band.contains(**height))
    .count();
  Some((band, evidence_count))
}

fn trimmed_height_sample(heights: &[i64]) -> &[i64] {
  if heights.len() < 5 {
    return heights;
  }
  let trim = (heights.len() / 5).max(1);
  &heights[trim..heights.len() - trim]
}

fn union_row_bounds(rows: &[ObservedOcrRow]) -> Option<ObservedRect> {
  let mut iter = rows.iter();
  let first = iter.next()?;
  Some(iter.fold(first.bounds.clone(), |bounds, row| {
    union_observed_rects(&bounds, &row.bounds)
  }))
}

fn union_observed_rects(left: &ObservedRect, right: &ObservedRect) -> ObservedRect {
  let min_x = left.x.min(right.x);
  let min_y = left.y.min(right.y);
  let max_x = (left.x + left.width).max(right.x + right.width);
  let max_y = (left.y + left.height).max(right.y + right.height);
  ObservedRect {
    x: min_x,
    y: min_y,
    width: max_x - min_x,
    height: max_y - min_y,
  }
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
  fn scan_scroll_delta_defaults_to_vertical_down() {
    let (delta_x, delta_y) = scan_scroll_delta("down", 6.0).expect("delta");
    assert_eq!(delta_x, 0.0);
    assert!(delta_y < 0.0);
  }

  #[test]
  fn scan_scroll_delta_normalizes_direction_case() {
    let (delta_x, delta_y) = scan_scroll_delta("DOWN", 6.0).expect("delta");
    assert_eq!((delta_x, delta_y), (0.0, -6.0));
  }

  #[test]
  fn scan_scroll_delta_maps_all_cardinal_directions() {
    assert_eq!(scan_scroll_delta("up", 4.0).expect("up"), (0.0, 4.0));
    assert_eq!(scan_scroll_delta("down", 4.0).expect("down"), (0.0, -4.0));
    assert_eq!(scan_scroll_delta("left", 4.0).expect("left"), (4.0, 0.0));
    assert_eq!(scan_scroll_delta("right", 4.0).expect("right"), (-4.0, 0.0));
  }

  #[test]
  fn scan_scroll_delta_rejects_unknown_direction() {
    let error = scan_scroll_delta("diagonal", 4.0).expect_err("direction should fail");
    assert!(error.contains("expected down, up, left, or right"));
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

  #[test]
  fn render_observe_window_region_json_emits_list_item_candidates() {
    let rows = vec![
      observed_row(0, 100, 120, 500, 160),
      observed_row(1, 100, 340, 700, 64),
      observed_row(2, 120, 460, 700, 84),
      observed_row(3, 120, 588, 700, 86),
      observed_row(4, 120, 716, 700, 82),
    ];

    let json = render_observe_window_region_json(
      &rows,
      &ObservedRect {
        x: 80,
        y: 100,
        width: 800,
        height: 720,
      },
      &ScreenshotDimensions {
        width: 1000,
        height: 900,
      },
      std::path::Path::new("/tmp/window.png"),
    )
    .expect("json should render");
    let value: serde_json::Value = serde_json::from_str(&json).expect("json should parse");

    assert_eq!(value["row_candidates"].as_array().unwrap().len(), 5);
    assert_eq!(value["item_candidates"].as_array().unwrap().len(), 3);
    assert_eq!(
      value["item_candidates"][0]["row_candidate_index"],
      serde_json::Value::from(2)
    );
    assert_eq!(
      value["rejected_row_candidates"][0]["reject_reason"],
      serde_json::Value::from("height_outside_repeating_row_band")
    );
    assert_eq!(
      value["segmented_regions"][0]["role"],
      serde_json::Value::from("list_region")
    );
  }

  #[test]
  fn row_filter_keeps_varied_music_rows_and_rejects_clear_outliers() {
    let heights = [213, 65, 113, 85, 100, 113, 92, 113, 97, 113, 63];
    let rows = heights
      .iter()
      .enumerate()
      .map(|(index, height)| observed_row(index, 120, 100 + index as i64 * 120, 700, *height))
      .collect::<Vec<_>>();

    let json = render_observe_window_region_json(
      &rows,
      &ObservedRect {
        x: 80,
        y: 100,
        width: 800,
        height: 1200,
      },
      &ScreenshotDimensions {
        width: 1000,
        height: 1400,
      },
      std::path::Path::new("/tmp/window.png"),
    )
    .expect("json should render");
    let value: serde_json::Value = serde_json::from_str(&json).expect("json should parse");
    let item_row_indices = value["item_candidates"]
      .as_array()
      .unwrap()
      .iter()
      .map(|item| item["row_candidate_index"].as_u64().unwrap())
      .collect::<Vec<_>>();
    let rejected_row_indices = value["rejected_row_candidates"]
      .as_array()
      .unwrap()
      .iter()
      .map(|item| item["row_candidate_index"].as_u64().unwrap())
      .collect::<Vec<_>>();

    assert_eq!(item_row_indices, vec![2, 3, 4, 5, 6, 7, 8, 9]);
    assert_eq!(rejected_row_indices, vec![0, 1, 10]);
  }

  fn observed_row(index: usize, x: i64, y: i64, width: i64, height: i64) -> ObservedOcrRow {
    ObservedOcrRow {
      row_index: index,
      source: "visual-bands".to_string(),
      bounds: ObservedRect {
        x,
        y,
        width,
        height,
      },
      text_fragments: Vec::new(),
    }
  }
}
