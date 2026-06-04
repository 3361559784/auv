use crate::view_parsers::sidebar::region::{
  DefaultScreenRestoreReason, broad_sidebar_probe_bounds, detect_blocking_modal,
  detect_default_screen_restore, detect_sidebar_region, fallback_playlist_sidebar_region,
};
use crate::view_parsers::sidebar::test_support::fake_recognition;
use crate::{RatioRect, ViewBounds};

#[test]
fn detect_sidebar_region_uses_manual_region_when_provided() {
  let region = detect_sidebar_region(
    Some(RatioRect::new(0.0, 0.1, 0.25, 0.8)),
    auv_driver::Size::new(1000.0, 800.0),
    &fake_recognition(Vec::new()),
  )
  .expect("manual sidebar region should be accepted");

  assert_eq!(region.name, Some("playlist_sidebar".to_string()));
  assert_eq!(
    region.bounds,
    Some(ViewBounds::new(0.0, 80.0, 250.0, 640.0))
  );
  assert_eq!(region.coordinate_space, Some("window".to_string()));
}

#[test]
fn detect_sidebar_region_starts_at_playlist_marker() {
  let region = detect_sidebar_region(
    None,
    auv_driver::Size::new(1646.0, 1053.0),
    &fake_recognition(vec![
      ("推荐", 8.0, 20.0, 40.0, 20.0),
      ("创建的歌单", 8.0, 443.0, 110.0, 20.0),
      ("Coding BGM", 32.0, 485.0, 120.0, 20.0),
      ("Reverberation", 98.0, 994.0, 160.0, 20.0),
    ]),
  )
  .expect("playlist marker should define the scroll body");

  assert_eq!(
    region.bounds,
    Some(ViewBounds::new(0.0, 443.0, 344.28, 528.0))
  );
}

#[test]
fn detect_sidebar_region_falls_back_to_full_sidebar_without_playlist_marker() {
  let region = detect_sidebar_region(
    None,
    auv_driver::Size::new(1000.0, 800.0),
    &fake_recognition(vec![("推荐", 8.0, 20.0, 40.0, 20.0)]),
  )
  .expect("navigation marker should preserve full sidebar fallback");

  assert_eq!(region.bounds, Some(ViewBounds::new(0.0, 0.0, 228.0, 718.0)));
}

#[test]
fn detect_sidebar_region_handles_negative_window_height_without_panic() {
  let region = detect_sidebar_region(
    None,
    auv_driver::Size::new(1646.0, -1.0),
    &fake_recognition(vec![
      ("推荐", 8.0, 20.0, 40.0, 20.0),
      ("创建的歌单", 8.0, 443.0, 110.0, 20.0),
      ("Coding BGM", 32.0, 485.0, 120.0, 20.0),
    ]),
  )
  .expect("negative window height should not crash sidebar detection");

  let bounds = region.bounds.expect("bounds should still be produced");
  assert!(bounds.y >= 0.0, "y must be floored to 0, got {}", bounds.y);
}

#[test]
fn detect_sidebar_region_rejects_unanchored_playlist_like_rows() {
  let error = detect_sidebar_region(
    None,
    auv_driver::Size::new(1000.0, 800.0),
    &fake_recognition(vec![
      ("Future Garage", 72.0, 320.0, 140.0, 20.0),
      ("Progressive House", 72.0, 366.0, 170.0, 20.0),
      ("Trance", 72.0, 412.0, 80.0, 20.0),
    ]),
  )
  .expect_err("playlist-like rows without a sidebar marker should not anchor the sidebar");

  assert_eq!(error.code, "sidebar_region_not_found");
}

#[test]
fn detect_sidebar_region_ignores_main_content_without_sidebar_marker() {
  let error = detect_sidebar_region(
    None,
    auv_driver::Size::new(1000.0, 800.0),
    &fake_recognition(vec![
      ("网易云音乐", 52.0, 40.0, 100.0, 20.0),
      ("Future Garage", 72.0, 320.0, 140.0, 20.0),
      ("Progressive House", 72.0, 366.0, 170.0, 20.0),
      ("Trance", 72.0, 412.0, 80.0, 20.0),
      ("每日推荐", 430.0, 300.0, 120.0, 30.0),
      ("推荐歌单", 520.0, 520.0, 150.0, 30.0),
    ]),
  )
  .expect_err("main content rows should not anchor the sidebar");

  assert_eq!(error.code, "sidebar_region_not_found");
}

#[test]
fn fallback_playlist_sidebar_region_starts_below_library_rows() {
  let region = fallback_playlist_sidebar_region(auv_driver::Size::new(1418.0, 1002.0));
  let bounds = region.bounds.expect("fallback should carry bounds");

  assert_eq!(region.name, Some("playlist_sidebar".to_string()));
  assert!(bounds.y >= 220.0);
  assert!(bounds.y > 0.0);
  assert!(bounds.height > 0.0);
  assert!(bounds.width >= 280.0);
}

#[test]
fn fallback_playlist_sidebar_region_handles_negative_window_without_silent_negative() {
  let region = fallback_playlist_sidebar_region(auv_driver::Size::new(-10.0, -10.0));
  let bounds = region.bounds.expect("bounds should still be produced");

  assert!(bounds.x >= 0.0, "x must be ≥ 0, got {}", bounds.x);
  assert!(bounds.y >= 0.0, "y must be ≥ 0, got {}", bounds.y);
  assert!(
    bounds.width >= 0.0,
    "width must be ≥ 0, got {}",
    bounds.width
  );
  assert!(
    bounds.height >= 0.0,
    "height must be ≥ 0, got {}",
    bounds.height
  );
}

#[test]
fn broad_sidebar_probe_bounds_handles_negative_window_width_without_silent_negative() {
  let bounds = broad_sidebar_probe_bounds(auv_driver::Size::new(-50.0, 800.0));

  assert!(
    bounds.width >= 0.0,
    "probe width must be ≥ 0, got {}",
    bounds.width
  );
  assert!(
    bounds.height >= 0.0,
    "probe height must be ≥ 0, got {}",
    bounds.height
  );
}

#[test]
fn detect_default_screen_restore_targets_song_detail_back_affordance() {
  let restore = detect_default_screen_restore(
    &fake_recognition(vec![
      ("私藏推荐", 90.0, 86.0, 120.0, 28.0),
      ("评论", 760.0, 182.0, 80.0, 28.0),
      ("收藏", 880.0, 182.0, 80.0, 28.0),
    ]),
    auv_driver::Size::new(1646.0, 1053.0),
  )
  .expect("song detail screen should expose a restore click");

  assert_eq!(restore.reason, DefaultScreenRestoreReason::SongDetailScreen);
  assert_eq!(restore.point, auv_driver::Point::new(82.602, 16.336));
}

#[test]
fn detect_default_screen_restore_ignores_normal_sidebar_screen() {
  let restore = detect_default_screen_restore(
    &fake_recognition(vec![
      ("推荐", 8.0, 20.0, 40.0, 20.0),
      ("评论", 760.0, 182.0, 80.0, 28.0),
      ("收藏", 880.0, 182.0, 80.0, 28.0),
    ]),
    auv_driver::Size::new(1646.0, 1053.0),
  );

  assert_eq!(restore, None);
}

#[test]
fn detect_default_screen_restore_ignores_blocking_modal() {
  let restore = detect_default_screen_restore(
    &fake_recognition(vec![
      ("评论", 760.0, 182.0, 80.0, 28.0),
      ("收藏", 880.0, 182.0, 80.0, 28.0),
      ("打开", 760.0, 720.0, 80.0, 32.0),
      ("取消", 860.0, 720.0, 80.0, 32.0),
    ]),
    auv_driver::Size::new(1646.0, 1053.0),
  );

  assert_eq!(restore, None);
}

#[test]
fn detect_blocking_modal_reports_cancel_or_open_dialog_markers() {
  let diagnostic = detect_blocking_modal(&fake_recognition(vec![
    ("打开", 760.0, 720.0, 80.0, 32.0),
    ("取消", 860.0, 720.0, 80.0, 32.0),
  ]))
  .expect("open dialog markers should be reported as blocking modal");

  assert_eq!(diagnostic.code, "blocking_modal_dialog");
}
