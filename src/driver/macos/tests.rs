use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use super::{
  ScreenshotDimensions,
  support::{
    assess_coordinate_readiness, filter_ocr_matches, optional_bool, optional_f64,
    parse_display_snapshot, parse_mouse_button, parse_ocr_region_constraint,
    parse_ocr_text_snapshot, parse_shortcut, project_main_screenshot_point,
    read_png_dimensions, render_rect_compact, resolve_display_point, resolve_scroll_deltas,
    sanitize_file_component, special_key_code, swift_string_literal,
  },
};
use crate::{
  driver::{DriverRegistry, fixture::FixtureObserveDriver},
  model::{DriverCall, ExecutionTarget, now_millis},
};

#[test]
fn optional_f64_rejects_non_finite_numbers() {
  let call = build_call([("x", "NaN")]);
  let error = optional_f64(&call, "x").expect_err("NaN should be rejected");
  assert!(error.contains("finite number"));
}

#[test]
fn parse_mouse_button_defaults_to_left() {
  let call = build_call([]);
  assert_eq!(
    parse_mouse_button(&call).expect("button should parse"),
    ("left", 0)
  );
}

#[test]
fn parse_shortcut_accepts_common_modifier_forms() {
  let shortcut = parse_shortcut("cmd+shift+f").expect("shortcut should parse");
  assert_eq!(shortcut.key, "f");
  assert_eq!(shortcut.modifiers, vec!["command down", "shift down"]);
}

#[test]
fn parse_shortcut_rejects_missing_key() {
  let error = parse_shortcut("cmd").expect_err("shortcut should fail");
  assert!(error.contains("expected a form like"));
}

#[test]
fn optional_bool_accepts_true_false_forms() {
  let call = build_call([("replace_existing", "true")]);
  assert_eq!(
    optional_bool(&call, "replace_existing").expect("bool should parse"),
    Some(true)
  );
  let call = build_call([("replace_existing", "0")]);
  assert_eq!(
    optional_bool(&call, "replace_existing").expect("bool should parse"),
    Some(false)
  );
}

#[test]
fn special_key_code_maps_return() {
  assert_eq!(special_key_code("return").expect("return should map"), 36);
}

#[test]
fn resolve_scroll_deltas_accepts_direction_and_pages() {
  let call = build_call([("direction", "down"), ("pages", "0.5")]);
  let (delta_x, delta_y, summary) =
    resolve_scroll_deltas(&call).expect("scroll delta should resolve");
  assert_eq!(delta_x, 0.0);
  assert_eq!(delta_y, -240.0);
  assert!(summary.contains("direction=down"));
}

#[test]
fn resolve_scroll_deltas_accepts_explicit_deltas() {
  let call = build_call([("delta_x", "40"), ("delta_y", "-120")]);
  let (delta_x, delta_y, summary) =
    resolve_scroll_deltas(&call).expect("scroll delta should resolve");
  assert_eq!(delta_x, 40.0);
  assert_eq!(delta_y, -120.0);
  assert!(summary.contains("delta_x=40"));
}

#[test]
fn sanitize_file_component_removes_invalid_characters() {
  assert_eq!(sanitize_file_component("My App!"), "My-App");
  assert_eq!(sanitize_file_component("../../etc/passwd"), "etc-passwd");
  assert_eq!(sanitize_file_component(""), "artifact");
}

#[test]
fn swift_string_literal_escapes_correctly() {
  assert_eq!(swift_string_literal("hello"), "\"hello\"");
  assert_eq!(swift_string_literal("a\"b"), "\"a\\\"b\"");
  assert_eq!(swift_string_literal("a\\b"), "\"a\\\\b\"");
  assert_eq!(swift_string_literal("a\nb"), "\"a\\nb\"");
}

#[test]
fn parse_display_snapshot_computes_combined_bounds() {
  let snapshot = parse_display_snapshot(sample_display_report()).expect("snapshot should parse");
  assert_eq!(snapshot.displays.len(), 2);
  assert_eq!(snapshot.combined_bounds.x, -222);
  assert_eq!(snapshot.combined_bounds.y, -1080);
  assert_eq!(snapshot.combined_bounds.width, 1920);
  assert_eq!(snapshot.combined_bounds.height, 2062);
  assert_eq!(snapshot.displays[0].pixel_width, 3024);
  assert_eq!(snapshot.displays[1].scale_factor, 1.0);
}

#[test]
fn resolve_display_point_maps_to_local_and_backing_pixel_coords() {
  let snapshot = parse_display_snapshot(sample_display_report()).expect("snapshot should parse");
  let resolution = resolve_display_point(&snapshot, 120.0, 80.0).expect("point should resolve");
  assert_eq!(resolution.display.display_id, 1);
  assert_eq!(resolution.local_x, 120.0);
  assert_eq!(resolution.local_y, 80.0);
  assert_eq!(resolution.backing_pixel_x, 240);
  assert_eq!(resolution.backing_pixel_y, 160);
}

#[test]
fn resolve_display_point_returns_none_outside_all_displays() {
  let snapshot = parse_display_snapshot(sample_display_report()).expect("snapshot should parse");
  assert!(resolve_display_point(&snapshot, 4000.0, 4000.0).is_none());
}

#[test]
fn assess_coordinate_readiness_accepts_matching_logical_dimensions() {
  let snapshot = parse_display_snapshot(sample_display_report()).expect("snapshot should parse");
  let assessment = assess_coordinate_readiness(
    &snapshot,
    &ScreenshotDimensions {
      width: 1512,
      height: 982,
    },
  )
  .expect("assessment should succeed");
  assert!(assessment.ready_for_logical_input);
  assert!(assessment.matches_main_logical);
  assert!(!assessment.matches_main_physical);
}

#[test]
fn assess_coordinate_readiness_flags_retina_backing_mismatch() {
  let snapshot = parse_display_snapshot(sample_display_report()).expect("snapshot should parse");
  let assessment = assess_coordinate_readiness(
    &snapshot,
    &ScreenshotDimensions {
      width: 3024,
      height: 1964,
    },
  )
  .expect("assessment should succeed");
  assert!(!assessment.ready_for_logical_input);
  assert!(assessment.matches_main_physical);
  assert!(assessment.likely_retina_backing_mismatch);
}

#[test]
fn parse_ocr_text_snapshot_parses_matches() {
  let snapshot = parse_ocr_text_snapshot(sample_ocr_report()).expect("OCR report should parse");
  assert_eq!(snapshot.query, "I DRINK THE LIGHT");
  assert_eq!(snapshot.image_width, 3024);
  assert_eq!(snapshot.image_height, 1964);
  assert_eq!(snapshot.matches.len(), 2);
  assert_eq!(snapshot.matches[0].match_index, 0);
  assert_eq!(snapshot.matches[0].text, "I DRINK THE LIGHT (Jengi Remix)");
  assert_eq!(snapshot.matches[0].bounds.x, 741);
  assert_eq!(snapshot.matches[1].match_index, 1);
  assert!((snapshot.matches[1].confidence - 0.945678).abs() < f64::EPSILON);
}

#[test]
fn project_main_screenshot_point_maps_retina_pixels_to_logical() {
  let snapshot = parse_display_snapshot(sample_display_report()).expect("snapshot should parse");
  let (logical_x, logical_y) =
    project_main_screenshot_point(&snapshot, 997.5, 1311.5).expect("projection should succeed");
  assert!((logical_x - 498.75).abs() < f64::EPSILON);
  assert!((logical_y - 655.75).abs() < f64::EPSILON);
}

#[test]
fn parse_ocr_region_constraint_accepts_normalized_bounds() {
  let call = build_call([
    ("region_left_ratio", "0.1"),
    ("region_top_ratio", "0.2"),
    ("region_right_ratio", "0.9"),
    ("region_bottom_ratio", "0.8"),
  ]);
  let region =
    parse_ocr_region_constraint(&call, 1000, 500).expect("region should parse").unwrap();
  assert_eq!(render_rect_compact(&region), "100,100,800,300");
}

#[test]
fn filter_ocr_matches_applies_confidence_and_region() {
  let snapshot = parse_ocr_text_snapshot(sample_ocr_report()).expect("OCR report should parse");
  let region = super::ObservedRect {
    x: 700,
    y: 1200,
    width: 700,
    height: 200,
  };
  let filtered = filter_ocr_matches(&snapshot.matches, 0.95, Some(&region));
  assert_eq!(filtered.len(), 1);
  assert_eq!(filtered[0].text, "I DRINK THE LIGHT (Jengi Remix)");
}

#[test]
fn read_png_dimensions_extracts_width_and_height() {
  let path = temp_png_path("png-dimensions");
  write_minimal_png(&path, 3024, 1964);
  let dimensions = read_png_dimensions(&path).expect("PNG dimensions should parse");
  assert_eq!(dimensions.width, 3024);
  assert_eq!(dimensions.height, 1964);
  let _ = fs::remove_file(path);
}

#[test]
fn driver_registry_stores_and_retrieves_drivers() {
  let registry = DriverRegistry::new(vec![Box::new(FixtureObserveDriver)]);
  assert!(registry.get("fixture.observe").is_some());
  assert!(registry.get("missing").is_none());
  assert_eq!(registry.descriptors().len(), 1);
  assert_eq!(registry.descriptors()[0].id, "fixture.observe");
}

fn build_call<const N: usize>(entries: [(&str, &str); N]) -> DriverCall {
  let mut inputs = BTreeMap::new();
  for (key, value) in entries {
    inputs.insert(key.to_string(), value.to_string());
  }

  DriverCall {
    operation: "test".to_string(),
    target: ExecutionTarget::default(),
    inputs,
    working_directory: PathBuf::from("."),
  }
}

fn sample_display_report() -> &'static str {
  "capturedAt=2026-05-13T05:06:06Z\n\
displayCount=2\n\
display\t1\t1\t1\t0\t0\t1512\t982\t0\t65\t1512\t884\t2.000\t3024\t1964\n\
display\t3\t0\t0\t-222\t-1080\t1920\t1080\t-222\t-1080\t1920\t1080\t1.000\t1920\t1080\n"
}

fn sample_ocr_report() -> &'static str {
  "recognizedAt=2026-05-14T10:00:00Z\n\
imagePath=/tmp/auv-screen.png\n\
imageWidth=3024\n\
imageHeight=1964\n\
query=I DRINK THE LIGHT\n\
exact=false\n\
caseSensitive=false\n\
match\t0\tI DRINK THE LIGHT (Jengi Remix)\t0.998901\t741\t1286\t513\t51\n\
match\t1\tTHE GODS WE CAN TOUCH\t0.945678\t1604\t808\t300\t42\n\
matchCount=2\n"
}

fn temp_png_path(label: &str) -> PathBuf {
  std::env::temp_dir().join(format!("auv-{}-{}.png", label, now_millis()))
}

fn write_minimal_png(path: &PathBuf, width: u32, height: u32) {
  let mut bytes = Vec::new();
  bytes.extend_from_slice(&[137, 80, 78, 71, 13, 10, 26, 10]);
  bytes.extend_from_slice(&13u32.to_be_bytes());
  bytes.extend_from_slice(b"IHDR");
  bytes.extend_from_slice(&width.to_be_bytes());
  bytes.extend_from_slice(&height.to_be_bytes());
  bytes.extend_from_slice(&[8, 6, 0, 0, 0]);
  bytes.extend_from_slice(&0u32.to_be_bytes());
  fs::write(path, bytes).expect("minimal png should be writable");
}
