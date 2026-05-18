use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use super::{
  OcrTextMatch, ScreenshotDimensions,
  control::common::build_click_point_call,
  support::{
    app_contains_window, assess_coordinate_readiness, filter_ocr_matches, find_now_playing_ax_node,
    group_ocr_matches_into_rows, optional_bool, optional_f64, parse_app_selector,
    parse_display_snapshot, parse_mouse_button, parse_observed_ax_tree,
    parse_ocr_region_constraint, parse_ocr_text_snapshot, parse_shortcut,
    parse_visual_rows_snapshot, process_is_alive, project_main_screenshot_point,
    read_lock_owner_pid, read_png_dimensions, render_rect_compact, resolve_app_ref,
    resolve_display_point, resolve_scroll_deltas, resolve_window_point, resolve_window_ref,
    sanitize_file_component, special_key_code, swift_string_literal, temp_file_path, window_area,
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
  let region = parse_ocr_region_constraint(&call, 1000, 500)
    .expect("region should parse")
    .unwrap();
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
fn group_ocr_matches_into_rows_merges_nearby_vertical_observations() {
  let matches = vec![
    OcrTextMatch {
      match_index: 0,
      text: "Song Title".to_string(),
      confidence: 0.99,
      bounds: super::ObservedRect {
        x: 100,
        y: 100,
        width: 180,
        height: 30,
      },
    },
    OcrTextMatch {
      match_index: 1,
      text: "Artist".to_string(),
      confidence: 0.98,
      bounds: super::ObservedRect {
        x: 110,
        y: 138,
        width: 90,
        height: 24,
      },
    },
    OcrTextMatch {
      match_index: 2,
      text: "Next Row".to_string(),
      confidence: 0.97,
      bounds: super::ObservedRect {
        x: 100,
        y: 260,
        width: 120,
        height: 28,
      },
    },
  ];
  let refs = matches.iter().collect::<Vec<_>>();
  let rows = group_ocr_matches_into_rows(&refs);
  assert_eq!(rows.len(), 2);
  assert_eq!(rows[0].source, "ocr-text");
  assert_eq!(rows[0].text_fragments.len(), 2);
  assert_eq!(rows[1].text_fragments, vec!["Next Row".to_string()]);
}

#[test]
fn find_now_playing_ax_node_matches_title_and_artist() {
  let snapshot = parse_observed_ax_tree(sample_ax_report()).expect("AX report should parse");
  let node = find_now_playing_ax_node(&snapshot, "天空仍灿烂", Some("周杰伦"), Some("0.4.4"))
    .expect("now-playing node should match");
  assert_eq!(node.title, "歌曲名：天空仍灿烂 - 歌手名：周杰伦");
}

#[test]
fn parse_visual_rows_snapshot_parses_visual_band_rows() {
  let snapshot =
    parse_visual_rows_snapshot(sample_visual_row_report()).expect("visual row report should parse");
  assert_eq!(snapshot.strategy, "visual-bands");
  assert_eq!(snapshot.rows.len(), 2);
  assert_eq!(snapshot.rows[0].source, "visual-bands");
  assert_eq!(snapshot.rows[0].bounds.x, 423);
  assert!(snapshot.rows[0].text_fragments.is_empty());
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
fn temp_file_path_is_unique_within_process() {
  let first = temp_file_path("artifact", "txt");
  let second = temp_file_path("artifact", "txt");
  assert_ne!(first, second);
}

#[test]
fn read_lock_owner_pid_parses_pid_field() {
  let path = temp_txt_path("lock-owner");
  fs::write(&path, "pid=4242\nacquiredAt=123\n").expect("lock file should write");
  let pid = read_lock_owner_pid(&path).expect("pid should parse");
  assert_eq!(pid, Some(4242));
  let _ = fs::remove_file(path);
}

#[test]
fn process_is_alive_matches_current_process() {
  assert!(process_is_alive(std::process::id()));
}

#[test]
fn build_click_point_call_populates_required_inputs() {
  let target = ExecutionTarget::default();
  let working_directory = PathBuf::from("/tmp/auv");
  let call = build_click_point_call(
    &target,
    &working_directory,
    12.5,
    48.25,
    "left",
    2,
    Some(300),
    Some("com.apple.TextEdit"),
  );
  assert_eq!(call.operation, "click_point");
  assert_eq!(call.working_directory, working_directory);
  assert_eq!(call.inputs.get("x"), Some(&"12.500".to_string()));
  assert_eq!(call.inputs.get("y"), Some(&"48.250".to_string()));
  assert_eq!(call.inputs.get("button"), Some(&"left".to_string()));
  assert_eq!(call.inputs.get("click_count"), Some(&"2".to_string()));
  assert_eq!(call.inputs.get("settle_ms"), Some(&"300".to_string()));
  assert_eq!(
    call.inputs.get("app"),
    Some(&"com.apple.TextEdit".to_string())
  );
}

#[test]
fn build_click_point_call_omits_optional_inputs_when_absent() {
  let call = build_click_point_call(
    &ExecutionTarget::default(),
    std::path::Path::new("."),
    1.0,
    2.0,
    "right",
    1,
    None,
    None,
  );
  assert!(!call.inputs.contains_key("settle_ms"));
  assert!(!call.inputs.contains_key("app"));
}

#[test]
fn app_contains_window_matches_bundleish_identifiers() {
  assert!(app_contains_window("com.apple.TextEdit", "TextEdit"));
  assert!(app_contains_window("QQ音乐", "QQ音乐"));
  assert!(!app_contains_window("TextEdit", "Notes"));
}

#[test]
fn window_area_uses_window_bounds() {
  let window = super::ObservedWindow {
    window_number: 7,
    app_name: "TextEdit".to_string(),
    owner_pid: 1,
    owner_bundle_id: "com.apple.TextEdit".to_string(),
    layer: 0,
    title: "Untitled".to_string(),
    bounds: super::ObservedRect {
      x: 0,
      y: 0,
      width: 640,
      height: 480,
    },
  };
  assert_eq!(window_area(&window), 307200);
}

#[test]
fn resolve_window_point_supports_offset_mode() {
  let call = build_call([("offset_x", "16"), ("offset_y", "24")]);
  let window = sample_window_ref();
  let (x, y, summary) = resolve_window_point(&call, &window).expect("offset mode should resolve");
  assert_eq!(x, 116.0);
  assert_eq!(y, 224.0);
  assert_eq!(summary, "windowOffset=16.000,24.000");
}

#[test]
fn resolve_window_point_supports_relative_mode() {
  let call = build_call([("relative_x", "0.5"), ("relative_y", "0.25")]);
  let window = sample_window_ref();
  let (x, y, summary) = resolve_window_point(&call, &window).expect("relative mode should resolve");
  assert_eq!(x, 420.0);
  assert_eq!(y, 320.0);
  assert_eq!(summary, "windowRelative=0.500,0.250");
}

#[test]
fn resolve_window_point_rejects_mixed_modes() {
  let call = build_call([
    ("offset_x", "16"),
    ("offset_y", "24"),
    ("relative_x", "0.5"),
    ("relative_y", "0.25"),
  ]);
  let window = sample_window_ref();
  let error = resolve_window_point(&call, &window).expect_err("mixed modes should fail");
  assert!(error.contains("either --offset_x/--offset_y or --relative_x/--relative_y"));
}

#[test]
fn parse_app_selector_recognizes_bundle_id() {
  let selector =
    parse_app_selector("com.netease.163music").expect("bundle id selector should parse");
  assert_eq!(selector.bundle_id.as_deref(), Some("com.netease.163music"));
  assert!(selector.app_name_hint.is_none());
}

#[test]
fn resolve_app_ref_prefers_exact_bundle_id_matches() {
  let selector =
    parse_app_selector("com.netease.163music").expect("bundle id selector should parse");
  let snapshot = super::ObservedWindowSnapshot {
    frontmost_app_name: "NetEaseMusic".to_string(),
    frontmost_app_bundle_id: "com.netease.163music".to_string(),
    frontmost_window_title: "网易云音乐".to_string(),
    observed_at: "2026-05-18T00:00:00Z".to_string(),
    windows: vec![
      super::ObservedWindow {
        window_number: 2,
        app_name: "StatusIndicator".to_string(),
        owner_pid: 20,
        owner_bundle_id: "com.status.helper".to_string(),
        layer: 0,
        title: "StatusIndicator".to_string(),
        bounds: super::ObservedRect {
          x: 10,
          y: 10,
          width: 28,
          height: 28,
        },
      },
      super::ObservedWindow {
        window_number: 9,
        app_name: "NetEaseMusic".to_string(),
        owner_pid: 30,
        owner_bundle_id: "com.netease.163music".to_string(),
        layer: 0,
        title: "网易云音乐".to_string(),
        bounds: super::ObservedRect {
          x: 100,
          y: 100,
          width: 1200,
          height: 900,
        },
      },
    ],
  };

  let resolved = resolve_app_ref(&snapshot, &selector).expect("app ref should resolve");
  assert_eq!(
    resolved.resolved_bundle_id.as_deref(),
    Some("com.netease.163music")
  );
  assert_eq!(resolved.resolved_app_name, "NetEaseMusic");
  assert_eq!(resolved.match_strategy, "bundle-id-exact");
}

#[test]
fn resolve_window_ref_prefers_substantial_main_window() {
  let selector =
    parse_app_selector("com.netease.163music").expect("bundle id selector should parse");
  let snapshot = super::ObservedWindowSnapshot {
    frontmost_app_name: "NetEaseMusic".to_string(),
    frontmost_app_bundle_id: "com.netease.163music".to_string(),
    frontmost_window_title: "网易云音乐".to_string(),
    observed_at: "2026-05-18T00:00:00Z".to_string(),
    windows: vec![
      super::ObservedWindow {
        window_number: 7,
        app_name: "NetEaseMusic".to_string(),
        owner_pid: 30,
        owner_bundle_id: "com.netease.163music".to_string(),
        layer: 0,
        title: "StatusIndicator".to_string(),
        bounds: super::ObservedRect {
          x: 4532,
          y: -929,
          width: 28,
          height: 28,
        },
      },
      super::ObservedWindow {
        window_number: 11,
        app_name: "NetEaseMusic".to_string(),
        owner_pid: 30,
        owner_bundle_id: "com.netease.163music".to_string(),
        layer: 0,
        title: "".to_string(),
        bounds: super::ObservedRect {
          x: 3009,
          y: 265,
          width: 1644,
          height: 1140,
        },
      },
      super::ObservedWindow {
        window_number: 12,
        app_name: "NetEaseMusic".to_string(),
        owner_pid: 30,
        owner_bundle_id: "com.netease.163music".to_string(),
        layer: 0,
        title: "网易云音乐".to_string(),
        bounds: super::ObservedRect {
          x: 3009,
          y: 265,
          width: 1644,
          height: 1140,
        },
      },
    ],
  };

  let resolved = resolve_app_ref(&snapshot, &selector).expect("app ref should resolve");
  let window = resolve_window_ref(&snapshot, &resolved).expect("window ref should resolve");
  assert_eq!(window.window_number, 12);
  assert_eq!(window.title, "网易云音乐");
  assert_eq!(window.owner_bundle_id, "com.netease.163music");
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

fn sample_window_ref() -> super::WindowRef {
  super::WindowRef {
    window_number: 7,
    owner_pid: 1,
    owner_bundle_id: "com.apple.TextEdit".to_string(),
    app_name: "TextEdit".to_string(),
    title: "Untitled".to_string(),
    bounds: super::ObservedRect {
      x: 100,
      y: 200,
      width: 640,
      height: 480,
    },
    layer: 0,
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

fn sample_visual_row_report() -> &'static str {
  "detectedAt=2026-05-15T22:00:00Z\n\
imagePath=/tmp/auv-screen.png\n\
imageWidth=3024\n\
imageHeight=1964\n\
rowStrategy=visual-bands\n\
cropRect=423,668,2298,1198\n\
analysisStrip=46,0,552,1198\n\
row\t0\t423\t712\t2120\t88\t0.423100\n\
row\t1\t423\t826\t2120\t86\t0.401200\n\
rowCount=2\n"
}

fn sample_ax_report() -> &'static str {
  "observedAt=2026-05-16T07:00:00Z\n\
appName=QQ音乐\n\
bundleId=com.tencent.QQMusicMac\n\
pid=1495\n\
windowTitle=\n\
rootRole=AXWindow\n\
node\t0\t0\tAXWindow\tAXStandardWindow\tQQMianWindow\t\t\t\t\t\t66\t33\t1280\t857\n\
node\t1\t0.4\tAXUnknown\t\t播放控制栏\t\t\t\t\t\t298\t800\t1036\t78\n\
node\t2\t0.4.4\tAXUnknown\t\t歌曲名：天空仍灿烂 - 歌手名：周杰伦\t\t\t\t\t\t375\t812\t264\t24\n\
node\t2\t0.4.9\tAXUnknown\t\t播放列表\t\t\t\t\t\t1284\t824\t30\t30\n"
}

fn temp_png_path(label: &str) -> PathBuf {
  std::env::temp_dir().join(format!("auv-{}-{}.png", label, now_millis()))
}

fn temp_txt_path(label: &str) -> PathBuf {
  std::env::temp_dir().join(format!("auv-{}-{}.txt", label, now_millis()))
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
