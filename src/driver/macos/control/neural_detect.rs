use super::super::*;
use crate::contract::{
  RatioRegion, RecognitionBox, RecognitionResult, RecognitionScope, RecognitionSource,
  RecognitionSurface, RecognizedItem,
};
use crate::driver::macos::support::neural_detect;

pub(crate) fn find_neural_detect(call: &DriverCall) -> AuvResult<DriverResponse> {
  let label =
    optional_string(call, "label").unwrap_or_else(|| "neural-detect".to_string());

  let model_path_str = optional_string(call, "model")
    .ok_or_else(|| "find_neural_detect requires --model <onnx-path>".to_string())?;
  let model_path = std::path::Path::new(&model_path_str);
  if !model_path.exists() {
    return Err(format!("model file not found: {}", model_path.display()));
  }

  // --classes accepts either a JSON array literal ("[\"a\",\"b\"]") or a path to a JSON file.
  let classes_str = optional_string(call, "classes")
    .ok_or_else(|| "find_neural_detect requires --classes <json-array|json-file>".to_string())?;
  let class_names: Vec<String> = if classes_str.trim_start().starts_with('[') {
    serde_json::from_str(&classes_str)
      .map_err(|e| format!("--classes inline JSON is not a string array: {e}"))?
  } else {
    let text = std::fs::read_to_string(&classes_str)
      .map_err(|e| format!("failed to read classes file {classes_str}: {e}"))?;
    serde_json::from_str(&text)
      .map_err(|e| format!("classes file {classes_str} is not a JSON string array: {e}"))?
  };
  if class_names.is_empty() {
    return Err("--classes must not be empty".to_string());
  }

  let threshold = optional_f64(call, "threshold")?.unwrap_or(0.5);
  if !(0.0..=1.0).contains(&threshold) {
    return Err(format!(
      "invalid --threshold {threshold:.3}: expected 0.0..=1.0"
    ));
  }

  let app_bundle_id = app_identifier(call).filter(|v| looks_like_bundle_identifier(v));
  let capture = super::window_ocr::capture_resolved_window_observation(call, &label)?;
  let search_region =
    parse_ocr_region_constraint(call, capture.dimensions.width, capture.dimensions.height)?;

  let (display_ref, native_display_id) = match &capture.capture_contract.capture_source {
    crate::driver::macos::capture::types::CaptureSource::Window {
      display_ref,
      native_display_id,
      ..
    } => (Some(display_ref.as_str()), Some(native_display_id.as_str())),
    _ => (None, None),
  };

  let detect_output = neural_detect::run_neural_detect(
    capture.screenshot_path.as_path(),
    model_path,
    &class_names,
    threshold,
    search_region.as_ref(),
  )?;

  let items: Vec<RecognizedItem> = detect_output
    .items
    .iter()
    .enumerate()
    .map(|(i, item)| RecognizedItem {
      item_id: format!("neural_detect#{}", i + 1),
      kind: item.label.clone(),
      box_: RecognitionBox {
        x: item.x,
        y: item.y,
        width: item.width,
        height: item.height,
      },
      text: None,
      provider_score: Some(item.score),
      detail: serde_json::json!({
        "cls_index": item.cls_index,
        "label":     item.label,
        "score":     item.score,
      }),
    })
    .collect();

  let best = items.first().cloned();
  let match_count = items.len();

  let region_hint = search_region.as_ref().map(|r| RatioRegion {
    left: r.x as f64 / capture.dimensions.width as f64,
    top: r.y as f64 / capture.dimensions.height as f64,
    right: (r.x + r.width) as f64 / capture.dimensions.width as f64,
    bottom: (r.y + r.height) as f64 / capture.dimensions.height as f64,
  });

  let model_input_size_json =
    detect_output.model_input_width.map(|w| {
      serde_json::json!({"width": w, "height": detect_output.model_input_height})
    });

  let screenshot_path_str = capture.screenshot_path.display().to_string();
  let screenshot_w = capture.dimensions.width;
  let screenshot_h = capture.dimensions.height;
  let capture_source = capture.capture_source.clone();

  let result = RecognitionResult {
    recognition_id: format!("neural_detect_{}", sanitize_file_component(&label)),
    source: RecognitionSource::NeuralNetworkDetect,
    scope: RecognitionScope {
      surface: RecognitionSurface::Window,
      display_ref: display_ref.map(str::to_string),
      native_display_id: native_display_id.map(str::to_string),
      app_bundle_id,
      window_title: None,
      window_number: window_number_from_ref(&capture_source),
      region_hint,
      capture_artifact: None,
      capture_contract_artifact: None,
    },
    best: best.clone(),
    filtered: items.clone(),
    all: items,
    detail: serde_json::json!({
      "provider":         "onnx_detector",
      "model":            model_path_str,
      "classes":          class_names,
      "threshold":        threshold,
      "match_count":      match_count,
      "model_input_size": model_input_size_json,
      "search_region": {
        "x":      detect_output.search_x,
        "y":      detect_output.search_y,
        "width":  detect_output.search_width,
        "height": detect_output.search_height,
      },
      "screenshot": {
        "path":   screenshot_path_str,
        "width":  screenshot_w,
        "height": screenshot_h,
      },
    }),
    evidence: Vec::new(),
    known_limits: vec![
      "requires auv-onnx-runner subprocess; set AUV_ONNX_RUNNER or add it to PATH".to_string(),
      "bounding boxes are in screenshot pixel coordinates".to_string(),
      "classes list must match the model's training class order (index == cls_index)".to_string(),
    ],
  };

  let recognition_json = serde_json::to_string_pretty(&result)
    .map(|mut s| {
      s.push('\n');
      s
    })
    .map_err(|e| format!("failed to encode neural detect result: {e}"))?;

  let recognition_artifact = build_text_artifact(
    "neural-detect-recognition",
    "json",
    &format!("{}-recognition", sanitize_file_component(&label)),
    recognition_json,
    "ONNX neural network detect recognition result.",
  )?;

  let screenshot_artifact = ProducedArtifact {
    kind: "screenshot".to_string(),
    source_path: capture.screenshot_path,
    preferred_name: format!("{}.png", sanitize_file_component(&label)),
    note: Some("Window screenshot used for neural network detection.".to_string()),
  };

  Ok(DriverResponse {
    summary: format!(
      "Neural network detect: found {} item(s) above threshold {:.2}.",
      match_count, threshold
    ),
    backend: Some("macos.onnx.find-neural-detect".to_string()),
    signals: {
      let mut s = std::collections::BTreeMap::new();
      s.insert("match_count".to_string(), match_count.to_string());
      s.insert(
        "best_score".to_string(),
        best
          .as_ref()
          .and_then(|b| b.provider_score)
          .map(|sc| format!("{sc:.3}"))
          .unwrap_or_else(|| "none".to_string()),
      );
      s
    },
    notes: vec![
      format!("model={model_path_str}"),
      format!("threshold={threshold:.3}"),
      format!("matchCount={match_count}"),
      format!("screenshotPixels={screenshot_w}x{screenshot_h}"),
    ],
    artifacts: vec![screenshot_artifact, recognition_artifact],
  })
}
