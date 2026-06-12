use std::collections::BTreeMap;

use auv_inference_common::{BoundingBox, DetectionSet};
use serde::{Deserialize, Serialize};

use crate::{ObjectKind, VisualTruthManifest};

/// Maps an osu [`ObjectKind`] to the detection label a visual model is expected
/// to emit for that object.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LabelMap {
  entries: Vec<(ObjectKind, String)>,
}

impl LabelMap {
  pub fn new(entries: Vec<(ObjectKind, String)>) -> Self {
    Self { entries }
  }

  pub fn expected_label(&self, kind: &ObjectKind) -> Option<&str> {
    self
      .entries
      .iter()
      .find(|(entry_kind, _)| entry_kind == kind)
      .map(|(_, label)| label.as_str())
  }
}

impl Default for LabelMap {
  fn default() -> Self {
    Self {
      entries: vec![
        (ObjectKind::Circle, "hit_circle".to_string()),
        (ObjectKind::Slider, "slider".to_string()),
        (ObjectKind::Spinner, "spinner".to_string()),
        (ObjectKind::Hold, "hold".to_string()),
      ],
    }
  }
}

/// Whether playfield-space truth can be projected into the capture pixel space
/// the detections live in.
///
/// NOTICE: osu beatmap truth is in playfield coordinates (512x384). Detections
/// are in source-image pixels. Without an explicit calibration the two spaces
/// are not comparable, so spatial scoring must be skipped rather than faked.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum EvalProjection {
  Unavailable {
    reason: String,
  },
  /// Linear playfield-to-pixel mapping: pixel = playfield * scale + offset.
  PlayfieldToPixels {
    scale_x: f32,
    scale_y: f32,
    offset_x: f32,
    offset_y: f32,
    /// Maximum center distance (in pixels) for a detection to count as a
    /// spatial hit for the expected object.
    match_radius_px: f32,
  },
}

/// Per-frame label-presence outcome.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FrameLabelOutcome {
  /// A detection with the expected label exists in the frame.
  Matched,
  /// No detection with the expected label exists in the frame.
  Missing,
  /// No expected label is configured for this object kind, so the frame cannot
  /// be scored for label presence.
  Unmapped,
}

/// Per-frame spatial outcome. Only meaningful when a projection is available.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FrameSpatialOutcome {
  /// Projection available and a labeled detection fell within the match radius.
  Matched,
  /// Projection available but no labeled detection fell within the radius.
  Missing,
  /// Projection unavailable, so spatial scoring was skipped for this frame.
  NotScored,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FrameEvaluation {
  pub object_index: usize,
  pub object_kind: ObjectKind,
  pub capture_file_name: String,
  pub expected_label: Option<String>,
  pub label_outcome: FrameLabelOutcome,
  pub spatial_outcome: FrameSpatialOutcome,
  /// Detections in this frame that did not match the expected label.
  pub spurious_detection_count: usize,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VisualEvalReport {
  pub total_frames: usize,
  pub label_matched_frames: usize,
  pub label_missing_frames: usize,
  pub label_unmapped_frames: usize,
  pub spatial_matched_frames: usize,
  pub spatial_missing_frames: usize,
  pub spatial_unscored_frames: usize,
  pub spurious_detection_count: usize,
  pub projection: EvalProjection,
  pub frames: Vec<FrameEvaluation>,
  pub known_limits: Vec<String>,
}

impl VisualEvalReport {
  /// Label recall over frames that have an expected label.
  pub fn label_recall(&self) -> Option<f32> {
    let scorable = self.label_matched_frames + self.label_missing_frames;
    if scorable == 0 {
      None
    } else {
      Some(self.label_matched_frames as f32 / scorable as f32)
    }
  }
}

/// Evaluate a [`VisualTruthManifest`] against per-frame detections.
///
/// `detections_by_object` pairs an `object_index` (matching a manifest frame)
/// with the [`DetectionSet`] observed for that frame's capture. Frames without
/// a detection entry are treated as having an empty detection set.
pub fn evaluate_visual_truth(
  manifest: &VisualTruthManifest,
  detections_by_object: &[(usize, DetectionSet)],
  projection: &EvalProjection,
  label_map: &LabelMap,
) -> VisualEvalReport {
  let detections_lookup = detections_by_object
    .iter()
    .map(|(object_index, set)| (*object_index, set))
    .collect::<BTreeMap<_, _>>();

  let empty = Vec::new();
  let mut frames = Vec::with_capacity(manifest.frames.len());
  let mut label_matched_frames = 0;
  let mut label_missing_frames = 0;
  let mut label_unmapped_frames = 0;
  let mut spatial_matched_frames = 0;
  let mut spatial_missing_frames = 0;
  let mut spatial_unscored_frames = 0;
  let mut spurious_detection_count = 0;

  for frame in &manifest.frames {
    let detections = detections_lookup
      .get(&frame.object_index)
      .map(|set| &set.detections)
      .unwrap_or(&empty);
    let expected_label = label_map
      .expected_label(&frame.expected_object.object_kind)
      .map(str::to_string);

    let label_outcome = match &expected_label {
      None => FrameLabelOutcome::Unmapped,
      Some(label) => {
        if detections.iter().any(|detection| &detection.label == label) {
          FrameLabelOutcome::Matched
        } else {
          FrameLabelOutcome::Missing
        }
      }
    };

    let frame_spurious = match &expected_label {
      None => detections.len(),
      Some(label) => detections
        .iter()
        .filter(|detection| &detection.label != label)
        .count(),
    };

    let spatial_outcome = match (projection, &expected_label) {
      (EvalProjection::Unavailable { .. }, _) | (_, None) => FrameSpatialOutcome::NotScored,
      (
        EvalProjection::PlayfieldToPixels {
          scale_x,
          scale_y,
          offset_x,
          offset_y,
          match_radius_px,
        },
        Some(label),
      ) => {
        let target_x = frame.expected_object.expected_playfield_x * scale_x + offset_x;
        let target_y = frame.expected_object.expected_playfield_y * scale_y + offset_y;
        let hit = detections.iter().any(|detection| {
          &detection.label == label
            && center_distance(&detection.bbox, target_x, target_y) <= *match_radius_px
        });
        if hit {
          FrameSpatialOutcome::Matched
        } else {
          FrameSpatialOutcome::Missing
        }
      }
    };

    match label_outcome {
      FrameLabelOutcome::Matched => label_matched_frames += 1,
      FrameLabelOutcome::Missing => label_missing_frames += 1,
      FrameLabelOutcome::Unmapped => label_unmapped_frames += 1,
    }
    match spatial_outcome {
      FrameSpatialOutcome::Matched => spatial_matched_frames += 1,
      FrameSpatialOutcome::Missing => spatial_missing_frames += 1,
      FrameSpatialOutcome::NotScored => spatial_unscored_frames += 1,
    }
    spurious_detection_count += frame_spurious;

    frames.push(FrameEvaluation {
      object_index: frame.object_index,
      object_kind: frame.expected_object.object_kind.clone(),
      capture_file_name: frame.capture.file_name.clone(),
      expected_label,
      label_outcome,
      spatial_outcome,
      spurious_detection_count: frame_spurious,
    });
  }

  let known_limits = build_known_limits(projection);

  VisualEvalReport {
    total_frames: manifest.frames.len(),
    label_matched_frames,
    label_missing_frames,
    label_unmapped_frames,
    spatial_matched_frames,
    spatial_missing_frames,
    spatial_unscored_frames,
    spurious_detection_count,
    projection: projection.clone(),
    frames,
    known_limits,
  }
}

fn build_known_limits(projection: &EvalProjection) -> Vec<String> {
  let mut known_limits = vec![
    "label-presence scoring confirms a detection label exists in a frame, not that it is the correct object instance".to_string(),
  ];
  match projection {
    EvalProjection::Unavailable { reason } => {
      known_limits.push(format!(
        "spatial scoring skipped: no playfield-to-pixel calibration ({reason})"
      ));
    }
    EvalProjection::PlayfieldToPixels { .. } => {
      known_limits.push(
        "spatial scoring uses a linear playfield-to-pixel projection; accuracy depends on calibration quality".to_string(),
      );
    }
  }
  known_limits
}

fn center_distance(bbox: &BoundingBox, target_x: f32, target_y: f32) -> f32 {
  let center_x = (bbox.x1 + bbox.x2) / 2.0;
  let center_y = (bbox.y1 + bbox.y2) / 2.0;
  let dx = center_x - target_x;
  let dy = center_y - target_y;
  (dx * dx + dy * dy).sqrt()
}

/// Intersection-over-union of two boxes in the same coordinate space.
pub fn iou(a: &BoundingBox, b: &BoundingBox) -> f32 {
  let inter_x1 = a.x1.max(b.x1);
  let inter_y1 = a.y1.max(b.y1);
  let inter_x2 = a.x2.min(b.x2);
  let inter_y2 = a.y2.min(b.y2);
  let inter_w = (inter_x2 - inter_x1).max(0.0);
  let inter_h = (inter_y2 - inter_y1).max(0.0);
  let intersection = inter_w * inter_h;
  if intersection <= 0.0 {
    return 0.0;
  }
  let union = a.area() + b.area() - intersection;
  if union <= 0.0 {
    0.0
  } else {
    intersection / union
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{CaptureFrame, CapturePhase, ExpectedObjectTruth, MapSummary, VisualTruthFrame};
  use auv_inference_common::{Detection, ImageSize, ModelId};

  fn test_map_summary() -> MapSummary {
    MapSummary {
      beatmap_path: "map.osu".to_string(),
      mode: 0,
      total_objects: 1,
      circle_count: 1,
      slider_count: 0,
      spinner_count: 0,
      hold_count: 0,
      first_object_time_ms: Some(100),
      last_object_time_ms: Some(100),
      approach_rate: 9.0,
      overall_difficulty: 8.0,
      circle_size: 4.0,
      hp_drain_rate: 5.0,
    }
  }

  fn circle_frame(object_index: usize, playfield_x: f32, playfield_y: f32) -> VisualTruthFrame {
    VisualTruthFrame {
      object_index,
      scheduled_time_ms: 100,
      actual_dispatch_time_ms: 104,
      dispatch_error_ms: 4,
      capture: CaptureFrame {
        phase: CapturePhase::AfterDispatch,
        capture_time_ms: 120,
        relative_to_scheduled_ms: 20,
        relative_to_dispatch_ms: 16,
        file_name: format!("frame-{object_index}.png"),
        width: 640,
        height: 480,
        backend: "test".to_string(),
        fallback_reason: None,
      },
      expected_object: ExpectedObjectTruth {
        object_kind: ObjectKind::Circle,
        expected_playfield_x: playfield_x,
        expected_playfield_y: playfield_y,
        circle_size: 4.0,
        approach_rate: 9.0,
        overall_difficulty: 8.0,
      },
    }
  }

  fn manifest_with(frames: Vec<VisualTruthFrame>) -> VisualTruthManifest {
    VisualTruthManifest {
      schema_version: 1,
      beatmap_path: "map.osu".to_string(),
      map_summary: test_map_summary(),
      frames,
    }
  }

  fn detection(label: &str, x1: f32, y1: f32, x2: f32, y2: f32) -> Detection {
    Detection {
      class_id: 0,
      label: label.to_string(),
      confidence: 0.9,
      bbox: BoundingBox { x1, y1, x2, y2 },
    }
  }

  fn detection_set(detections: Vec<Detection>) -> DetectionSet {
    DetectionSet {
      model_id: ModelId("test-osu-detector".to_string()),
      image_size: ImageSize {
        width: 640,
        height: 480,
      },
      detections,
    }
  }

  #[test]
  fn label_presence_counts_hits_and_misses_without_projection() {
    let manifest = manifest_with(vec![
      circle_frame(0, 256.0, 192.0),
      circle_frame(1, 100.0, 100.0),
    ]);
    let detections = vec![
      (
        0,
        detection_set(vec![detection("hit_circle", 10.0, 10.0, 30.0, 30.0)]),
      ),
      (
        1,
        detection_set(vec![detection("slider", 0.0, 0.0, 5.0, 5.0)]),
      ),
    ];

    let report = evaluate_visual_truth(
      &manifest,
      &detections,
      &EvalProjection::Unavailable {
        reason: "no calibration in test".to_string(),
      },
      &LabelMap::default(),
    );

    assert_eq!(report.total_frames, 2);
    assert_eq!(report.label_matched_frames, 1);
    assert_eq!(report.label_missing_frames, 1);
    assert_eq!(report.label_recall(), Some(0.5));
    // frame 0 has no spurious (only hit_circle); frame 1 has one spurious slider.
    assert_eq!(report.spurious_detection_count, 1);
  }

  #[test]
  fn projection_unavailable_marks_all_spatial_frames_not_scored() {
    let manifest = manifest_with(vec![circle_frame(0, 256.0, 192.0)]);
    let detections = vec![(
      0,
      detection_set(vec![detection("hit_circle", 10.0, 10.0, 30.0, 30.0)]),
    )];

    let report = evaluate_visual_truth(
      &manifest,
      &detections,
      &EvalProjection::Unavailable {
        reason: "no playfield mapping".to_string(),
      },
      &LabelMap::default(),
    );

    assert_eq!(report.spatial_unscored_frames, 1);
    assert_eq!(report.spatial_matched_frames, 0);
    assert_eq!(report.spatial_missing_frames, 0);
    assert!(
      report
        .known_limits
        .iter()
        .any(|limit| limit.contains("spatial scoring skipped"))
    );
  }

  #[test]
  fn projection_available_scores_spatial_hit_and_miss() {
    let manifest = manifest_with(vec![
      circle_frame(0, 100.0, 100.0),
      circle_frame(1, 100.0, 100.0),
    ]);
    // Identity projection: pixel == playfield. Target center is (100, 100).
    let projection = EvalProjection::PlayfieldToPixels {
      scale_x: 1.0,
      scale_y: 1.0,
      offset_x: 0.0,
      offset_y: 0.0,
      match_radius_px: 20.0,
    };
    let detections = vec![
      // frame 0: box centered at (100, 100) -> within radius.
      (
        0,
        detection_set(vec![detection("hit_circle", 90.0, 90.0, 110.0, 110.0)]),
      ),
      // frame 1: box centered at (300, 300) -> outside radius.
      (
        1,
        detection_set(vec![detection("hit_circle", 290.0, 290.0, 310.0, 310.0)]),
      ),
    ];

    let report = evaluate_visual_truth(&manifest, &detections, &projection, &LabelMap::default());

    assert_eq!(report.spatial_matched_frames, 1);
    assert_eq!(report.spatial_missing_frames, 1);
    assert_eq!(report.spatial_unscored_frames, 0);
    // Both frames still match on label presence.
    assert_eq!(report.label_matched_frames, 2);
  }

  #[test]
  fn missing_detection_entry_counts_as_label_miss() {
    let manifest = manifest_with(vec![circle_frame(0, 256.0, 192.0)]);

    let report = evaluate_visual_truth(
      &manifest,
      &[],
      &EvalProjection::Unavailable {
        reason: "no calibration".to_string(),
      },
      &LabelMap::default(),
    );

    assert_eq!(report.label_missing_frames, 1);
    assert_eq!(report.label_recall(), Some(0.0));
  }

  #[test]
  fn iou_computes_known_overlap() {
    let a = BoundingBox {
      x1: 0.0,
      y1: 0.0,
      x2: 2.0,
      y2: 2.0,
    };
    let b = BoundingBox {
      x1: 1.0,
      y1: 1.0,
      x2: 3.0,
      y2: 3.0,
    };
    // intersection = 1, union = 4 + 4 - 1 = 7.
    let value = iou(&a, &b);
    assert!((value - 1.0 / 7.0).abs() < 1e-6);

    let disjoint = BoundingBox {
      x1: 10.0,
      y1: 10.0,
      x2: 12.0,
      y2: 12.0,
    };
    assert_eq!(iou(&a, &disjoint), 0.0);
  }
}
