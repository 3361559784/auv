use crate::scroll::policies::detection_motion::MotionEvidence;
use auv_view::{ViewAxis, ViewBounds};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct InteractionEvent {
  pub event_index: usize,
  pub phase: InteractionPhase,
  pub kind: InteractionEventKind,
  pub observation_index: Option<usize>,
  pub from_observation: Option<usize>,
  pub to_observation: Option<usize>,
  pub viewport_fingerprint: Option<String>,
  pub scroll: Option<ScrollInteraction>,
  pub motion: Option<MotionEvidence>,
  pub artifacts: Vec<String>,
  pub note: Option<String>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InteractionPhase {
  TopSeek,
  #[default]
  Collect,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InteractionEventKind {
  Probe,
  #[default]
  Observe,
  InputScroll,
  StopDecision,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ScrollInteraction {
  pub axis: ViewAxis,
  pub direction: ScrollDirection,
  pub requested_delta: f64,
  pub policy: String,
  pub delivery_path: Option<String>,
  pub motion: Option<MotionEvidence>,
  pub settle_ms: u64,
  pub anchor: Option<ViewBounds>,
  pub detected_boundary: String,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScrollDirection {
  Up,
  #[default]
  Down,
}
