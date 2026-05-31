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

// --------------------------------------------------------------------------
// Pure framework utilities. These were lifted from `auv-netease-music`'s
// `lib.rs`; they hold no domain knowledge and any view-parser app can call
// them. Tests live next to the functions to lock the behavior so future
// tuning (e.g. confidence thresholds) is intentional.
// --------------------------------------------------------------------------

/// Normalize a label for identity comparisons: lowercase + trim + drop all
/// whitespace. Matches the "normalized label equality" rule from the
/// merge-fixtures spec.
pub fn normalize_identity(value: &str) -> String {
  value
    .trim()
    .to_lowercase()
    .chars()
    .filter(|ch| !ch.is_whitespace())
    .collect()
}

/// Slug form of a label: `normalize_identity` then map every non-
/// alphanumeric ASCII char to `_`. Used to build deterministic candidate /
/// node IDs from raw OCR text.
pub fn slug(value: &str) -> String {
  normalize_identity(value)
    .chars()
    .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '_' })
    .collect()
}

/// Viewport fingerprint = pipe-joined normalized labels of the evidence
/// nodes that were visible in this observation. Used to detect repeated
/// viewports (stuck scroll / loop boundary) per the diagnostic policy.
pub fn viewport_fingerprint(nodes: &[ViewEvidenceNode]) -> String {
  nodes
    .iter()
    .filter_map(|node| node.label.as_deref())
    .map(normalize_identity)
    .collect::<Vec<_>>()
    .join("|")
}

/// REVIEW(confidence-thresholds-v1): the 0.85 / 0.65 split was tuned for
/// Apple Vision OCR scores observed during NetEase capture work. Any view
/// parser using a different OCR provider may need different thresholds;
/// the constants are not load-bearing across providers. When a second
/// provider lands, parameterize via config rather than branching the
/// function.
pub fn confidence_from_ocr(confidence: Option<f32>) -> Confidence {
  match confidence {
    Some(value) if value >= 0.85 => Confidence::High,
    Some(value) if value >= 0.65 => Confidence::Medium,
    _ => Confidence::Low,
  }
}

/// Does the viewport bounding box contain the geometric center of the
/// other box? Used by per-viewport candidate filtering to drop evidence
/// that drifts outside the visible viewport between observations.
pub fn viewport_contains_center(viewport: ViewBounds, bounds: ViewBounds) -> bool {
  let center_x = bounds.x + bounds.width * 0.5;
  let center_y = bounds.y + bounds.height * 0.5;
  center_x >= viewport.x
    && center_x <= viewport.x + viewport.width
    && center_y >= viewport.y
    && center_y <= viewport.y + viewport.height
}

/// Walk a `ViewNodeRecord` tree and accumulate every anchor attached to
/// any node into `anchors`. Order is pre-order (this node, then children).
pub fn collect_anchors(node: &ViewNodeRecord, anchors: &mut Vec<ViewAnchor>) {
  anchors.extend(node.anchors.clone());
  for child in &node.children {
    collect_anchors(child, anchors);
  }
}

/// Walk a `ViewNodeRecord` tree and accumulate every landmark attached to
/// any node into `landmarks`. Order is pre-order (this node, then
/// children).
pub fn collect_landmarks(node: &ViewNodeRecord, landmarks: &mut Vec<ViewLandmark>) {
  landmarks.extend(node.landmarks.clone());
  for child in &node.children {
    collect_landmarks(child, landmarks);
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn normalize_identity_lowercases_and_drops_whitespace() {
    assert_eq!(normalize_identity("  Hello World  "), "helloworld");
    assert_eq!(normalize_identity("我 的 歌单"), "我的歌单");
    assert_eq!(normalize_identity(""), "");
  }

  #[test]
  fn slug_maps_non_alnum_to_underscore() {
    assert_eq!(slug("Hello World"), "helloworld");
    assert_eq!(slug("My-Playlist!"), "my_playlist_");
    assert_eq!(slug("我的歌单"), "____"); // Chinese chars are non-ASCII-alphanumeric
  }

  #[test]
  fn viewport_fingerprint_joins_normalized_labels_with_pipe() {
    let nodes = vec![
      ViewEvidenceNode {
        id: "a".into(),
        source: ViewEvidenceSource::OcrText,
        label: Some("Liked Songs".into()),
        bounds: None,
        confidence: Confidence::High,
      },
      ViewEvidenceNode {
        id: "b".into(),
        source: ViewEvidenceSource::OcrText,
        label: Some("Daily Mix 1".into()),
        bounds: None,
        confidence: Confidence::Medium,
      },
      ViewEvidenceNode {
        // labels: None nodes are skipped
        id: "c".into(),
        source: ViewEvidenceSource::AxNode,
        label: None,
        bounds: None,
        confidence: Confidence::Low,
      },
    ];
    assert_eq!(viewport_fingerprint(&nodes), "likedsongs|dailymix1");
  }

  #[test]
  fn confidence_from_ocr_threshold_mapping() {
    assert_eq!(confidence_from_ocr(Some(0.95)), Confidence::High);
    assert_eq!(confidence_from_ocr(Some(0.85)), Confidence::High); // boundary inclusive
    assert_eq!(confidence_from_ocr(Some(0.80)), Confidence::Medium);
    assert_eq!(confidence_from_ocr(Some(0.65)), Confidence::Medium); // boundary inclusive
    assert_eq!(confidence_from_ocr(Some(0.50)), Confidence::Low);
    assert_eq!(confidence_from_ocr(None), Confidence::Low);
  }

  #[test]
  fn viewport_contains_center_uses_geometric_center() {
    let viewport = ViewBounds::new(0.0, 0.0, 100.0, 100.0);
    // Center (50,50) is inside
    assert!(viewport_contains_center(
      viewport,
      ViewBounds::new(40.0, 40.0, 20.0, 20.0)
    ));
    // Center (150, 50) is outside despite bounds overlapping
    assert!(!viewport_contains_center(
      viewport,
      ViewBounds::new(100.0, 40.0, 100.0, 20.0)
    ));
    // Exact boundary inclusive
    assert!(viewport_contains_center(
      viewport,
      ViewBounds::new(90.0, 90.0, 20.0, 20.0)
    ));
  }

  #[test]
  fn collect_anchors_walks_tree_in_preorder() {
    let anchor = |id: &str| ViewAnchor {
      id: id.into(),
      label: id.into(),
      strength: AnchorStrength::Strong,
      bounds: ViewBounds::default(),
      evidence_ids: Vec::new(),
    };
    let root = ViewNodeRecord {
      anchors: vec![anchor("root")],
      children: vec![
        ViewNodeRecord {
          anchors: vec![anchor("child-a")],
          ..Default::default()
        },
        ViewNodeRecord {
          anchors: vec![anchor("child-b")],
          children: vec![ViewNodeRecord {
            anchors: vec![anchor("grandchild")],
            ..Default::default()
          }],
          ..Default::default()
        },
      ],
      ..Default::default()
    };
    let mut out = Vec::new();
    collect_anchors(&root, &mut out);
    assert_eq!(
      out.iter().map(|a| a.id.as_str()).collect::<Vec<_>>(),
      vec!["root", "child-a", "child-b", "grandchild"]
    );
  }

  #[test]
  fn collect_landmarks_walks_tree_in_preorder() {
    let landmark = |id: &str| ViewLandmark {
      id: id.into(),
      label: id.into(),
      landmark_use: LandmarkUse::SectionAssignment,
      bounds: ViewBounds::default(),
      evidence_ids: Vec::new(),
    };
    let root = ViewNodeRecord {
      landmarks: vec![landmark("root")],
      children: vec![ViewNodeRecord {
        landmarks: vec![landmark("child")],
        ..Default::default()
      }],
      ..Default::default()
    };
    let mut out = Vec::new();
    collect_landmarks(&root, &mut out);
    assert_eq!(
      out.iter().map(|l| l.id.as_str()).collect::<Vec<_>>(),
      vec!["root", "child"]
    );
  }
}
