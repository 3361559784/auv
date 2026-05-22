use std::collections::BTreeMap;
use std::io::{BufRead, BufReader};

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

pub(crate) fn music_validate_candidate_liveness(call: &DriverCall) -> AuvResult<DriverResponse> {
  let source_run_id = required_non_empty_string(call, "source_run_id")?;
  let source_artifact_id = optional_non_empty_string(call, "source_artifact_id")
    .unwrap_or_else(|| "artifact_0003".to_string());
  let candidate_local_id = required_non_empty_string(call, "candidate_local_id")?;

  let store_root = call.working_directory.join(".auv");
  let run_dir = store_root.join("runs").join(&source_run_id);
  let artifacts_jsonl_path = run_dir.join("artifacts.jsonl");

  let artifact_relative_path =
    find_artifact_path_in_jsonl(&artifacts_jsonl_path, &source_artifact_id)?;
  let artifact_abs_path = run_dir.join(&artifact_relative_path);

  let json_content = std::fs::read_to_string(&artifact_abs_path).map_err(|error| {
    format!("failed to read artifact {source_artifact_id} from run {source_run_id}: {error}")
  })?;

  let operation_result: OperationResult =
    serde_json::from_str(&json_content).map_err(|error| {
      format!("failed to parse OperationResult from {source_artifact_id}: {error}")
    })?;

  let candidates = match &operation_result.output {
    OperationOutput::Candidates { candidates } => candidates,
    OperationOutput::Verification { .. } => {
      return Err(format!(
        "artifact {source_artifact_id} contains a verification result, not a candidate set"
      ))
    }
    OperationOutput::Acknowledged { .. } => {
      return Err(format!(
        "artifact {source_artifact_id} contains an acknowledged result, not a candidate set"
      ))
    }
  };

  let candidate = candidates
    .iter()
    .find(|c| c.candidate_local_id == candidate_local_id)
    .ok_or_else(|| {
      let available = candidates
        .iter()
        .map(|c| c.candidate_local_id.as_str())
        .collect::<Vec<_>>()
        .join(", ");
      format!(
        "candidate {candidate_local_id} not found in {source_artifact_id}; available: [{available}]"
      )
    })?;

  if let Some(window_ref) = &candidate.liveness.preconditions.window_ref {
    let snapshot =
      crate::driver::macos::observe::observe_windows_snapshot(128, &window_ref.app_bundle_id)?;
    let selector = parse_app_selector(&window_ref.app_bundle_id)?;
    resolve_app_ref(&snapshot, &selector).map_err(|_| {
      format!(
        "candidate {candidate_local_id} liveness failed: app {} has no visible windows",
        window_ref.app_bundle_id
      )
    })?;
  }

  let anchor_recheck_ran = if let Some(anchor_recheck) = &candidate.liveness.preconditions.anchor_recheck {
    let app_bundle_id = candidate
      .liveness
      .preconditions
      .window_ref
      .as_ref()
      .map(|w| w.app_bundle_id.clone())
      .unwrap_or_default();
    if app_bundle_id.is_empty() {
      return Err(format!(
        "candidate {candidate_local_id} has anchor_recheck but no window_ref.app_bundle_id; cannot capture window"
      ));
    }
    let mut recheck_call = call.clone();
    recheck_call.inputs.insert("app".to_string(), app_bundle_id);
    let capture =
      capture_resolved_window_observation(&recheck_call, "liveness-anchor-recheck").map_err(
        |error| {
          format!(
            "candidate {candidate_local_id} liveness failed: window capture failed: {error}"
          )
        },
      )?;
    let ocr_result = crate::driver::macos::native::ocr::find_text(
      &capture.screenshot_path,
      &anchor_recheck.text,
      false,
      false,
      64,
      None,
    )?;
    let found = ocr_result
      .snapshot
      .matches
      .iter()
      .any(|m| m.confidence >= anchor_recheck.expected_min_confidence);
    if !found {
      return Err(format!(
        "candidate {candidate_local_id} liveness failed: anchor '{}' not found with confidence >= {:.2}",
        anchor_recheck.text, anchor_recheck.expected_min_confidence
      ));
    }
    true
  } else {
    false
  };

  let anchor_text = candidate
    .target_spec
    .anchor_text
    .clone()
    .unwrap_or_default();
  let label = candidate.label.clone().unwrap_or_default();

  Ok(DriverResponse {
    summary: format!(
      "Candidate {candidate_local_id} liveness OK; anchor_text={anchor_text}"
    ),
    backend: Some("macos.contract.music-validate-candidate-liveness".to_string()),
    signals: BTreeMap::from([
      ("candidate.resolved".to_string(), "true".to_string()),
      ("candidate.local_id".to_string(), candidate_local_id.clone()),
      ("candidate.anchor_text".to_string(), anchor_text),
      ("candidate.label".to_string(), label),
      ("candidate.liveness_ok".to_string(), "true".to_string()),
      (
        "candidate.anchor_recheck_ran".to_string(),
        anchor_recheck_ran.to_string(),
      ),
    ]),
    notes: vec![
      format!("sourceRunId={source_run_id}"),
      format!("sourceArtifactId={source_artifact_id}"),
      format!("candidateLocalId={candidate_local_id}"),
      format!("operationId={}", operation_result.operation_id),
    ],
    artifacts: Vec::new(),
  })
}

fn find_artifact_path_in_jsonl(
  jsonl_path: &std::path::Path,
  artifact_id: &str,
) -> AuvResult<String> {
  let file = std::fs::File::open(jsonl_path).map_err(|error| {
    format!(
      "failed to open artifacts.jsonl at {}: {error}",
      jsonl_path.display()
    )
  })?;
  let reader = BufReader::new(file);
  for line in reader.lines() {
    let line = line.map_err(|error| format!("failed to read artifacts.jsonl: {error}"))?;
    if line.trim().is_empty() {
      continue;
    }
    let record: serde_json::Value = serde_json::from_str(&line)
      .map_err(|error| format!("failed to parse artifacts.jsonl entry: {error}"))?;
    if record.get("artifact_id").and_then(|v| v.as_str()) == Some(artifact_id) {
      return record
        .get("path")
        .and_then(|v| v.as_str())
        .map(|p| p.to_string())
        .ok_or_else(|| {
          format!("artifact {artifact_id} record has no 'path' field in artifacts.jsonl")
        });
    }
  }
  Err(format!(
    "artifact {artifact_id} not found in artifacts.jsonl at {}",
    jsonl_path.display()
  ))
}
