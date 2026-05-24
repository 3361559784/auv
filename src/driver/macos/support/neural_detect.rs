use std::path::Path;

use image::GenericImageView;

use crate::model::AuvResult;

use super::super::ObservedRect;

#[derive(Debug)]
pub(crate) struct NeuralDetectItem {
  pub(crate) x: i64,
  pub(crate) y: i64,
  pub(crate) width: i64,
  pub(crate) height: i64,
  pub(crate) score: f64,
  pub(crate) cls_index: usize,
  pub(crate) label: String,
}

#[derive(Debug)]
pub(crate) struct NeuralDetectOutput {
  pub(crate) items: Vec<NeuralDetectItem>,
  pub(crate) search_x: i64,
  pub(crate) search_y: i64,
  pub(crate) search_width: u32,
  pub(crate) search_height: u32,
  pub(crate) model_input_width: Option<u32>,
  pub(crate) model_input_height: Option<u32>,
}

/// Invokes the ONNX inference bridge (located via `AUV_ONNX_RUNNER` env or PATH).
///
/// Runner stdin protocol (JSON object):
///   screenshot  string        absolute path to the PNG screenshot
///   model       string        absolute path to the ONNX model file
///   classes     string[]      class label array (index == cls_index in results)
///   threshold   number        minimum confidence score [0.0, 1.0]
///   region      object|null   {x,y,width,height} in screenshot pixels
///
/// Runner stdout protocol (JSON object):
///   items             [{label, cls_index, score, x, y, width, height}]
///   model_input_size  [width, height] | null
pub(crate) fn run_neural_detect(
  screenshot_path: &Path,
  model_path: &Path,
  class_names: &[String],
  threshold: f64,
  search_region: Option<&ObservedRect>,
) -> AuvResult<NeuralDetectOutput> {
  let (img_w, img_h) = image::open(screenshot_path)
    .map_err(|e| format!("failed to open screenshot {}: {e}", screenshot_path.display()))?
    .dimensions();

  let (sx, sy, sw, sh) = if let Some(r) = search_region {
    let x = r.x.max(0) as u32;
    let y = r.y.max(0) as u32;
    let max_x = ((r.x + r.width) as u32).min(img_w);
    let max_y = ((r.y + r.height) as u32).min(img_h);
    (x as i64, y as i64, max_x.saturating_sub(x), max_y.saturating_sub(y))
  } else {
    (0i64, 0i64, img_w, img_h)
  };

  let region_val = search_region
    .map(|_| serde_json::json!({"x": sx, "y": sy, "width": sw, "height": sh}));

  let request = serde_json::json!({
    "screenshot": screenshot_path.display().to_string(),
    "model":      model_path.display().to_string(),
    "classes":    class_names,
    "threshold":  threshold,
    "region":     region_val,
  });

  let runner = locate_runner()?;

  let mut child = std::process::Command::new(&runner)
    .stdin(std::process::Stdio::piped())
    .stdout(std::process::Stdio::piped())
    .stderr(std::process::Stdio::piped())
    .spawn()
    .map_err(|e| format!("failed to spawn {}: {e}", runner.display()))?;

  if let Some(mut stdin) = child.stdin.take() {
    serde_json::to_writer(&mut stdin, &request)
      .map_err(|e| format!("failed to write request to runner stdin: {e}"))?;
  }

  let output = child
    .wait_with_output()
    .map_err(|e| format!("failed to wait for runner: {e}"))?;

  if !output.status.success() {
    let stderr = String::from_utf8_lossy(&output.stderr);
    return Err(format!(
      "auv-onnx-runner exited with {}: {}",
      output.status,
      stderr.trim()
    ));
  }

  let stdout = String::from_utf8_lossy(&output.stdout);
  parse_runner_response(&stdout, sx, sy, sw, sh)
}

fn locate_runner() -> AuvResult<std::path::PathBuf> {
  if let Ok(v) = std::env::var("AUV_ONNX_RUNNER") {
    if !v.is_empty() {
      let p = std::path::PathBuf::from(&v);
      if p.exists() {
        return Ok(p);
      }
      return Err(format!("AUV_ONNX_RUNNER={v} does not exist"));
    }
  }

  if let Some(paths) = std::env::var_os("PATH") {
    for dir in std::env::split_paths(&paths) {
      let candidate = dir.join("auv-onnx-runner");
      if candidate.exists() {
        return Ok(candidate);
      }
    }
  }

  Err(
    "auv-onnx-runner not found in PATH; \
     set AUV_ONNX_RUNNER=/path/to/runner to configure the ONNX inference bridge"
      .to_string(),
  )
}

fn parse_runner_response(
  json: &str,
  search_x: i64,
  search_y: i64,
  search_width: u32,
  search_height: u32,
) -> AuvResult<NeuralDetectOutput> {
  let v: serde_json::Value =
    serde_json::from_str(json).map_err(|e| format!("runner output is not valid JSON: {e}"))?;

  let items_arr = v["items"]
    .as_array()
    .ok_or_else(|| "runner response missing 'items' array".to_string())?;

  let mut items = Vec::with_capacity(items_arr.len());
  for (i, item) in items_arr.iter().enumerate() {
    let label = item["label"]
      .as_str()
      .ok_or_else(|| format!("items[{i}].label is missing or not a string"))?
      .to_string();
    let cls_index = item["cls_index"]
      .as_u64()
      .ok_or_else(|| format!("items[{i}].cls_index is missing or not a uint"))?
      as usize;
    let score = item["score"]
      .as_f64()
      .ok_or_else(|| format!("items[{i}].score is missing or not a float"))?;
    let x = item["x"]
      .as_i64()
      .ok_or_else(|| format!("items[{i}].x is missing or not an integer"))?;
    let y = item["y"]
      .as_i64()
      .ok_or_else(|| format!("items[{i}].y is missing or not an integer"))?;
    let width = item["width"]
      .as_i64()
      .ok_or_else(|| format!("items[{i}].width is missing or not an integer"))?;
    let height = item["height"]
      .as_i64()
      .ok_or_else(|| format!("items[{i}].height is missing or not an integer"))?;
    items.push(NeuralDetectItem {
      x,
      y,
      width,
      height,
      score,
      cls_index,
      label,
    });
  }

  let model_size = v["model_input_size"].as_array();
  let model_input_width = model_size
    .and_then(|s| s.first())
    .and_then(|v| v.as_u64())
    .map(|v| v as u32);
  let model_input_height = model_size
    .and_then(|s| s.get(1))
    .and_then(|v| v.as_u64())
    .map(|v| v as u32);

  Ok(NeuralDetectOutput {
    items,
    search_x,
    search_y,
    search_width,
    search_height,
    model_input_width,
    model_input_height,
  })
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn parse_runner_response_valid() {
    let json = r#"{
      "items": [
        {"label":"button","cls_index":3,"score":0.92,"x":10,"y":20,"width":50,"height":30},
        {"label":"icon","cls_index":1,"score":0.75,"x":100,"y":200,"width":24,"height":24}
      ],
      "model_input_size": [640, 640]
    }"#;
    let out = parse_runner_response(json, 0, 0, 1920, 1080).expect("should parse");
    assert_eq!(out.items.len(), 2);
    assert_eq!(out.items[0].label, "button");
    assert_eq!(out.items[0].cls_index, 3);
    assert!((out.items[0].score - 0.92).abs() < 1e-6);
    assert_eq!(out.items[0].x, 10);
    assert_eq!(out.items[0].width, 50);
    assert_eq!(out.model_input_width, Some(640));
    assert_eq!(out.model_input_height, Some(640));
  }

  #[test]
  fn parse_runner_response_empty_items() {
    let json = r#"{"items":[],"model_input_size":null}"#;
    let out = parse_runner_response(json, 0, 0, 100, 100).expect("should parse empty");
    assert!(out.items.is_empty());
    assert!(out.model_input_width.is_none());
    assert!(out.model_input_height.is_none());
  }

  #[test]
  fn parse_runner_response_missing_items_key_errors() {
    let json = r#"{"detections":[]}"#;
    let err = parse_runner_response(json, 0, 0, 100, 100).unwrap_err();
    assert!(err.contains("items"), "error should mention 'items': {err}");
  }

  #[test]
  fn parse_runner_response_missing_field_errors() {
    let json = r#"{"items":[{"label":"x","score":0.5}]}"#;
    let err = parse_runner_response(json, 0, 0, 100, 100).unwrap_err();
    assert!(
      err.contains("cls_index"),
      "error should mention missing field: {err}"
    );
  }

  #[test]
  fn parse_runner_response_search_region_propagated() {
    let json = r#"{"items":[]}"#;
    let out = parse_runner_response(json, 10, 20, 300, 400).expect("should parse");
    assert_eq!(out.search_x, 10);
    assert_eq!(out.search_y, 20);
    assert_eq!(out.search_width, 300);
    assert_eq!(out.search_height, 400);
  }
}
