use super::super::*;
use super::window_ocr::{capture_resolved_window_observation, detect_rows_for_capture};
use crate::contract::{
  AnchorRecheckPrecondition, ArtifactRef, Candidate, CandidateEvidence, CandidateLiveness,
  ControlRequirements, FreshnessBasis, LivenessPreconditions, OperationOutput, OperationResult,
  OperationStatus, RatioRegion, TargetGrounding, TargetSpec, WindowRefPrecondition,
};
use crate::trace::{ArtifactId, RunId, SpanId};

const SCREENSHOT_ARTIFACT_ID: &str = "artifact_0001";
const REPORT_ARTIFACT_ID: &str = "artifact_0002";

pub(crate) fn music_search_results(call: &DriverCall) -> AuvResult<DriverResponse> {
  let capture = capture_resolved_window_observation(call, "music-search-results")?;
  let (detection, rows) = detect_rows_for_capture(call, &capture)?;

  let run_id = optional_string(call, "_auv_run_id").unwrap_or_default();
  let span_id = optional_string(call, "_auv_span_id").unwrap_or_default();
  let app_bundle_id = app_identifier(call).unwrap_or_default();

  let evidence_artifact_ref = |artifact_id: &str| ArtifactRef {
    run_id: RunId::new(run_id.as_str()),
    artifact_id: ArtifactId::new(artifact_id),
    span_id: SpanId::new(span_id.as_str()),
    captured_event_id: None,
  };

  let candidates: Vec<Candidate> = rows
    .iter()
    .map(|row| {
      let anchor_text = row.text_fragments.first().cloned();
      let w = capture.dimensions.width.max(1) as f64;
      let h = capture.dimensions.height.max(1) as f64;
      let region = RatioRegion {
        left: row.bounds.x as f64 / w,
        top: row.bounds.y as f64 / h,
        right: (row.bounds.x + row.bounds.width) as f64 / w,
        bottom: (row.bounds.y + row.bounds.height) as f64 / h,
      };
      let joined_label = row.text_fragments.join(" ");
      Candidate {
        candidate_local_id: format!("row#{}", row.row_index + 1),
        kind: "search_result_row".to_string(),
        label: if joined_label.is_empty() {
          None
        } else {
          Some(joined_label)
        },
        target_spec: TargetSpec {
          grounding: TargetGrounding::OcrAnchor,
          anchor_text: anchor_text.clone(),
          region_hint: Some(region),
        },
        evidence: CandidateEvidence {
          artifact_ref: evidence_artifact_ref(SCREENSHOT_ARTIFACT_ID),
          observation: serde_json::json!({
            "provider": "vision_ocr.window_rows",
            "row_index": row.row_index,
            "source": row.source,
            "text_fragments": row.text_fragments,
            "bounds": {
              "x": row.bounds.x,
              "y": row.bounds.y,
              "width": row.bounds.width,
              "height": row.bounds.height,
            }
          }),
        },
        liveness: CandidateLiveness {
          preconditions: LivenessPreconditions {
            window_ref: Some(WindowRefPrecondition {
              app_bundle_id: app_bundle_id.clone(),
              window_title_substring: None,
              window_number: None,
            }),
            anchor_recheck: anchor_text.map(|text| AnchorRecheckPrecondition {
              text,
              region_hint: None,
              expected_min_confidence: 0.5,
              max_pixel_distance: 32.0,
            }),
          },
          ttl_hint_ms: Some(5000),
        },
        control: ControlRequirements {
          requires_app_frontmost: true,
          requires_window_focus: true,
        },
        known_limits: Vec::new(),
      }
    })
    .collect();

  let operation_result = OperationResult {
    run_id: RunId::new(run_id.as_str()),
    status: OperationStatus::Completed,
    operation_id: "music.search.results".to_string(),
    evidence_artifacts: vec![
      evidence_artifact_ref(SCREENSHOT_ARTIFACT_ID),
      evidence_artifact_ref(REPORT_ARTIFACT_ID),
    ],
    output: OperationOutput::Candidates {
      candidates: candidates.clone(),
    },
    freshness_basis: Some(FreshnessBasis {
      source_artifact: Some(evidence_artifact_ref(SCREENSHOT_ARTIFACT_ID)),
      source_operation_id: Some("debug.findWindowRows".to_string()),
      notes: vec!["window-scoped OCR rows".to_string()],
    }),
    known_limits: Vec::new(),
  };

  let operation_result_json = serde_json::to_string_pretty(&operation_result)
    .map(|mut s| {
      s.push('\n');
      s
    })
    .map_err(|error| format!("failed to serialize OperationResult: {error}"))?;

  let screenshot = screenshot_artifact(&capture, "music-search-results", "music search results");
  let report = build_text_artifact(
    "window-rows-report",
    "txt",
    "music-search-results-rows",
    detection.report.clone(),
    "Row-detection report for music.search.results.",
  )?;
  let result_artifact = build_text_artifact(
    "operation-result",
    "json",
    "music-search-results-operation-result",
    operation_result_json,
    "Typed OperationResult candidate set for music.search.results.",
  )?;

  let row_count = rows.len();
  let summary = if row_count > 0 {
    format!(
      "Produced {} search-result candidate(s) from window OCR rows (strategy {}); typed OperationResult at artifact_0003.",
      row_count, detection.strategy
    )
  } else {
    format!(
      "Detected 0 rows inside resolved window after strategy {}; empty candidate set in OperationResult artifact_0003.",
      detection.strategy
    )
  };

  Ok(DriverResponse {
    summary,
    backend: Some(format!(
      "macos.vision.music-search-results.{}",
      detection.strategy
    )),
    signals: crate::driver::macos::observe::row_detection_signals(row_count),
    notes: vec![
      "scope=window".to_string(),
      format!("windowRef={}", capture.capture_source),
      format!("rowStrategy={}", detection.strategy),
      format!("rowCount={row_count}"),
      format!("candidateCount={row_count}"),
      "operationResultArtifact=artifact_0003".to_string(),
    ],
    artifacts: vec![screenshot, report, result_artifact],
  })
}
