//! Generic view-parser IR shared by AUV app crates.
//!
//! v0 extraction: these types previously lived inside
//! `auv-netease-music/src/lib.rs`. They are framework-level and are not
//! NetEase-specific. App crates (NetEase, future QQ Music, etc.) build
//! their domain projections on top of these types instead of redefining
//! them per app.
//!
//! NOTICE(pub-fields-v0):
//!
//! Every type below exposes `pub` fields. v0 keeps the framework crate's
//! API surface intentionally wide so app crates can construct records
//! via struct literals without going through constructors. Tighten the
//! surface (constructors, builders, `non_exhaustive`) only when a real
//! consumer pressure shows up.
//!
//! Cross-references:
//!
//! - `docs/ai/references/2026-05-29-view-parser-ir-shapes-v0.md` is the
//!   spec these types target. The spec's `ViewNodeId` / `ViewCandidateId`
//!   newtype IDs are NOT yet adopted here; v0 stays at plain `String`
//!   ids to match the existing `auv-netease-music` shape and avoid a
//!   second migration. A future revision can promote the ids to
//!   newtypes once `playlist get <anchor>` lands and requires stable
//!   cross-run identity.

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScanAppContext {
  pub app_id: Option<String>,
  pub name: Option<String>,
  pub version: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ScanWindowContext {
  pub id: Option<String>,
  pub title: Option<String>,
  pub bounds: Option<ViewBounds>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ViewRegionRecord {
  pub id: Option<String>,
  pub name: Option<String>,
  pub bounds: Option<ViewBounds>,
  pub coordinate_space: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ViewViewportRecord {
  pub page_index: usize,
  pub bounds: ViewBounds,
  pub axis: ViewAxis,
  pub scroll_offset: Option<f64>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ViewEvidenceNode {
  pub id: String,
  pub source: ViewEvidenceSource,
  pub label: Option<String>,
  pub bounds: Option<ViewBounds>,
  pub confidence: Confidence,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ViewEvidenceSource {
  OcrText,
  AxNode,
  IconMatch,
  #[default]
  Visual,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ViewReconstructionRecord {
  pub root: ViewNodeRecord,
  pub anchor_index: Vec<ViewAnchor>,
  pub landmark_index: Vec<ViewLandmark>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ViewNodeRecord {
  pub id: String,
  pub kind: ViewNodeKind,
  pub domain_kind: Option<String>,
  pub layout: Option<ViewLayout>,
  pub label: Option<String>,
  pub bounds: ViewBounds,
  pub scrollable: Option<ViewScrollable>,
  pub anchors: Vec<ViewAnchor>,
  pub landmarks: Vec<ViewLandmark>,
  pub actions: Vec<ViewAction>,
  pub evidence: Vec<ViewEvidenceNode>,
  pub children: Vec<ViewNodeRecord>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ViewNodeKind {
  Container,
  Collection,
  Section,
  Item,
  Text,
  Icon,
  #[default]
  Unknown,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ViewLayout {
  VStack,
  HStack,
  Group,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ViewAxis {
  #[default]
  Vertical,
  Horizontal,
  Both,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ViewScrollable {
  pub axis: ViewAxis,
  pub boundary: ScrollBoundarySummary,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ViewAnchor {
  pub id: String,
  pub label: String,
  pub strength: AnchorStrength,
  pub bounds: ViewBounds,
  pub evidence_ids: Vec<String>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnchorStrength {
  #[default]
  Strong,
  Medium,
  Weak,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ViewLandmark {
  pub id: String,
  pub label: String,
  #[serde(rename = "use")]
  pub landmark_use: LandmarkUse,
  pub bounds: ViewBounds,
  pub evidence_ids: Vec<String>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LandmarkUse {
  ViewportPose,
  BoundaryDetection,
  AnchorReacquire,
  #[default]
  SectionAssignment,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ViewAction {
  Open,
  Select,
  Scroll,
  ObserveOnly,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScrollBoundarySummary {
  pub top: BoundaryConfidence,
  pub bottom: BoundaryConfidence,
  pub left: BoundaryConfidence,
  pub right: BoundaryConfidence,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BoundaryConfidence {
  Confirmed,
  Likely,
  #[default]
  Unknown,
  Contradicted,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Confidence {
  High,
  Medium,
  #[default]
  Low,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParserDiagnostic {
  pub code: String,
  pub message: String,
  pub node_id: Option<String>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ViewBounds {
  pub x: f64,
  pub y: f64,
  pub width: f64,
  pub height: f64,
}

impl ViewBounds {
  pub const fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
    Self {
      x,
      y,
      width,
      height,
    }
  }
}
