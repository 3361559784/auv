use super::{ReacquireObservation, ReacquireOutcome};
use crate::ParserDiagnostic;

pub trait ReacquireDriverAdapter {
  fn observe_viewport(&mut self) -> Result<ReacquireObservation, ParserDiagnostic>;
  fn scroll_down(&mut self) -> Result<(), ParserDiagnostic>;
  fn scroll_up(&mut self) -> Result<(), ParserDiagnostic>;
}

pub fn strategy_name(strategy: super::ReacquireStrategy) -> &'static str {
  match strategy {
    super::ReacquireStrategy::DirectId => "direct_id",
    super::ReacquireStrategy::LabelCurrentViewport => "label_current_viewport",
    super::ReacquireStrategy::LabelPlusSection => "label_plus_section",
    super::ReacquireStrategy::ViewportFingerprint => "viewport_fingerprint",
  }
}

pub fn outcome_label(outcome: &ReacquireOutcome) -> &'static str {
  match outcome {
    ReacquireOutcome::Reacquired { .. } => "reacquired",
    ReacquireOutcome::Stale { .. } => "stale",
    ReacquireOutcome::NotFound { .. } => "not_found",
  }
}
