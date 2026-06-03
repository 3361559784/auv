use auv_driver::vision::TextRecognition;

// NOTICE: This is a learned window-local logical point for the song-detail
// back affordance, matching the current live NetEase macOS client observation.
const PLAYING_SONG_DETAIL_RESTORE_POINT: auv_driver::Point = auv_driver::Point::new(82.602, 16.336);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScreenState {
  Default,
  PlayingSongDetail,
  BlockingModal,
  Unknown,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScreenView {
  state: ScreenState,
  restore_point: Option<auv_driver::Point>,
}

impl ScreenView {
  fn new(state: ScreenState, restore_point: Option<auv_driver::Point>) -> Self {
    Self {
      state,
      restore_point,
    }
  }

  /// Build a view when the screen was not classified by this observation.
  pub fn unknown() -> Self {
    Self::new(ScreenState::Unknown, None)
  }

  #[cfg(test)]
  pub(crate) fn for_tests(state: ScreenState, restore_point: Option<auv_driver::Point>) -> Self {
    Self::new(state, restore_point)
  }

  pub fn state(&self) -> ScreenState {
    self.state
  }

  pub fn is_default(&self) -> bool {
    self.state == ScreenState::Default
  }

  pub fn is_playing_song_detail(&self) -> bool {
    self.state == ScreenState::PlayingSongDetail
  }

  pub fn is_blocking_modal(&self) -> bool {
    self.state == ScreenState::BlockingModal
  }

  pub fn restore_point(&self) -> Option<auv_driver::Point> {
    self.restore_point
  }
}

pub fn classify_screen(recognition: &TextRecognition, window_size: auv_driver::Size) -> ScreenView {
  if is_blocking_modal(recognition) {
    return ScreenView::new(ScreenState::BlockingModal, None);
  }

  if has_left_sidebar_marker(recognition, window_size) {
    return ScreenView::new(ScreenState::Default, None);
  }

  if is_playing_song_detail(recognition) {
    return ScreenView::new(
      ScreenState::PlayingSongDetail,
      Some(PLAYING_SONG_DETAIL_RESTORE_POINT),
    );
  }

  ScreenView::new(ScreenState::Unknown, None)
}

fn is_blocking_modal(recognition: &TextRecognition) -> bool {
  contains_text(recognition, "取消")
    && (contains_text(recognition, "打开") || contains_text(recognition, "存储"))
}

fn has_left_sidebar_marker(recognition: &TextRecognition, window_size: auv_driver::Size) -> bool {
  let left_boundary = window_size.width * 0.38;
  recognition.regions.iter().any(|region| {
    region.bounds.origin.x < left_boundary && crate::is_sidebar_marker(region.text.trim())
  })
}

fn is_playing_song_detail(recognition: &TextRecognition) -> bool {
  contains_text(recognition, "评论") && contains_text(recognition, "收藏")
}

fn contains_text(recognition: &TextRecognition, query: &str) -> bool {
  recognition
    .regions
    .iter()
    .any(|region| region.text.contains(query))
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn classify_screen_detects_default_from_left_sidebar_marker() {
    let view = classify_screen(
      &fake_recognition(vec![("发现音乐", 42.0, 96.0, 92.0, 24.0)]),
      auv_driver::Size::new(1200.0, 800.0),
    );

    assert_eq!(view.state(), ScreenState::Default);
    assert!(view.is_default());
    assert_eq!(view.restore_point(), None);
  }

  #[test]
  fn classify_screen_detects_playing_song_detail_and_restore_point() {
    let view = classify_screen(
      &fake_recognition(vec![
        ("评论", 760.0, 182.0, 80.0, 28.0),
        ("收藏", 880.0, 182.0, 80.0, 28.0),
      ]),
      auv_driver::Size::new(1646.0, 1053.0),
    );

    assert_eq!(view.state(), ScreenState::PlayingSongDetail);
    assert!(view.is_playing_song_detail());
    assert_eq!(
      view.restore_point(),
      Some(auv_driver::Point::new(82.602, 16.336))
    );
  }

  #[test]
  fn classify_screen_detects_blocking_modal_before_default() {
    let view = classify_screen(
      &fake_recognition(vec![
        ("推荐", 42.0, 96.0, 52.0, 24.0),
        ("打开", 760.0, 720.0, 80.0, 32.0),
        ("取消", 860.0, 720.0, 80.0, 32.0),
      ]),
      auv_driver::Size::new(1200.0, 800.0),
    );

    assert_eq!(view.state(), ScreenState::BlockingModal);
    assert!(view.is_blocking_modal());
    assert_eq!(view.restore_point(), None);
  }

  #[test]
  fn classify_screen_returns_unknown_without_screen_markers() {
    let view = classify_screen(
      &fake_recognition(vec![("私人雷达", 620.0, 122.0, 120.0, 28.0)]),
      auv_driver::Size::new(1200.0, 800.0),
    );

    assert_eq!(view.state(), ScreenState::Unknown);
    assert_eq!(view.restore_point(), None);
  }

  fn fake_recognition(
    regions: Vec<(&str, f64, f64, f64, f64)>,
  ) -> auv_driver::vision::TextRecognition {
    auv_driver::vision::TextRecognition {
      text: regions
        .iter()
        .map(|(text, _, _, _, _)| *text)
        .collect::<Vec<_>>()
        .join("\n"),
      regions: regions
        .into_iter()
        .map(
          |(text, x, y, width, height)| auv_driver::vision::RecognizedText {
            text: text.to_string(),
            bounds: auv_driver::Rect::new(x, y, width, height),
            confidence: Some(0.9),
          },
        )
        .collect(),
    }
  }
}
