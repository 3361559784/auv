use crate::PlaybackControlState;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlayerState {
  Present,
  Absent,
  Unknown,
}

/// Read-only bottom-player facade backed by reconstructed or verified playback state.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlayerView {
  state: PlayerState,
  control_state: Option<PlaybackControlState>,
  observed_text: Option<String>,
}

impl PlayerView {
  /// Build a view for a caller-proven absent player bar.
  pub fn absent() -> Self {
    Self {
      state: PlayerState::Absent,
      control_state: None,
      observed_text: None,
    }
  }

  /// Build a view when the player was not classified by this observation.
  pub fn unknown() -> Self {
    Self {
      state: PlayerState::Unknown,
      control_state: None,
      observed_text: None,
    }
  }

  /// Build a view from the current bottom playback control state.
  pub fn from_control_state(control_state: PlaybackControlState) -> Self {
    let state = match control_state {
      PlaybackControlState::PlayVisible | PlaybackControlState::PauseVisible => {
        PlayerState::Present
      }
      PlaybackControlState::Unknown => PlayerState::Unknown,
    };

    Self {
      state,
      control_state: Some(control_state),
      observed_text: None,
    }
  }

  /// Attach optional OCR text observed around the player bar.
  pub fn with_observed_text(mut self, observed_text: impl Into<String>) -> Self {
    self.observed_text = Some(observed_text.into());
    self
  }

  pub fn state(&self) -> PlayerState {
    self.state
  }

  pub fn exists(&self) -> bool {
    self.state == PlayerState::Present
  }

  /// NetEase shows a pause affordance while playback is active.
  pub fn is_playing(&self) -> bool {
    self.control_state == Some(PlaybackControlState::PauseVisible)
  }

  pub fn control_state(&self) -> Option<PlaybackControlState> {
    self.control_state
  }

  pub fn observed_text(&self) -> Option<&str> {
    self.observed_text.as_deref()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn pause_control_means_present_and_playing() {
    let view = PlayerView::from_control_state(PlaybackControlState::PauseVisible);

    assert_eq!(view.state(), PlayerState::Present);
    assert!(view.exists());
    assert!(view.is_playing());
  }

  #[test]
  fn play_control_means_present_but_not_playing() {
    let view = PlayerView::from_control_state(PlaybackControlState::PlayVisible);

    assert_eq!(view.state(), PlayerState::Present);
    assert!(view.exists());
    assert!(!view.is_playing());
  }

  #[test]
  fn unknown_control_does_not_claim_player_exists() {
    let view = PlayerView::from_control_state(PlaybackControlState::Unknown);

    assert_eq!(view.state(), PlayerState::Unknown);
    assert!(!view.exists());
    assert!(!view.is_playing());
  }

  #[test]
  fn unknown_player_has_no_control_state_when_not_observed() {
    let view = PlayerView::unknown();

    assert_eq!(view.state(), PlayerState::Unknown);
    assert_eq!(view.control_state(), None);
    assert!(!view.exists());
  }

  #[test]
  fn absent_player_has_no_control_state() {
    let view = PlayerView::absent();

    assert_eq!(view.state(), PlayerState::Absent);
    assert_eq!(view.control_state(), None);
  }
}
