use std::thread;
use std::time::Duration;

use crate::ax_recognition::{
  AxBestSelectionStrategy, AxRecognitionArtifactRequest, AxRecognitionPolicy,
  record_ax_tree_recognition_artifact,
};
use crate::candidate_action_decision::{
  CandidateActionDecisionRequest, CandidateActionExecutionConsent,
  CandidateActionExecutionConsentAction, CandidateActionExecutionRequest,
  CandidateActionPostActionProbe, MacosCandidateActionExecutor,
  execute_and_record_single_candidate_action, record_candidate_action_decision_artifact,
};
use crate::candidate_promotion_recording::{
  CandidatePromotionArtifactRequest, CandidatePromotionConsentInput,
  explicit_consent_for_candidate_promotion, freshness_from_capture_backed_recognition,
  record_candidate_promotion_artifact_with_recognition_projection,
};
use crate::model::{AuvResult, now_millis};
use crate::recorded_operation::RecordedOperationContext;
use crate::stability::StabilityPolicy;

#[derive(Clone, Debug, PartialEq)]
pub struct CandidateActionCommandRequest {
  pub app_bundle_id: String,
  pub query: String,
  pub role: String,
  pub reveal_shortcut: Option<String>,
  pub reveal_settle_ms: u64,
  pub stable_frames: u32,
  pub stable_frame_delay_ms: u64,
  pub max_centroid_drift_px: f64,
  pub require_stable_text: bool,
  pub promotion_id: String,
  pub decision_id: String,
  pub execution_id: String,
  pub granted_by: String,
  pub promotion_scope_note: String,
  pub promotion_evidence_note: String,
  pub execution_scope_note: String,
  pub execution_evidence_note: String,
}

impl CandidateActionCommandRequest {
  pub fn validate(&self) -> AuvResult<()> {
    if self.app_bundle_id.trim().is_empty() {
      return Err("--target-app is required".to_string());
    }
    if self.query.trim().is_empty() {
      return Err("--query is required".to_string());
    }
    if self.role.trim().is_empty() {
      return Err("--role is required".to_string());
    }
    if self.stable_frames == 0 {
      return Err("--stable-frames must be greater than 0".to_string());
    }
    if self.granted_by.trim().is_empty() {
      return Err("--granted-by is required".to_string());
    }
    if self.promotion_scope_note.trim().is_empty() {
      return Err("--promotion-scope-note must not be empty".to_string());
    }
    if self.promotion_evidence_note.trim().is_empty() {
      return Err("--promotion-evidence-note must not be empty".to_string());
    }
    if self.execution_scope_note.trim().is_empty() {
      return Err("--execution-scope-note must not be empty".to_string());
    }
    if self.execution_evidence_note.trim().is_empty() {
      return Err("--execution-evidence-note must not be empty".to_string());
    }
    Ok(())
  }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CandidateActionCommandOutput {
  pub promotion_artifact_id: String,
  pub decision_artifact_id: String,
  pub execution_artifact_id: String,
}

#[cfg(target_os = "macos")]
pub fn execute_candidate_action_command(
  context: &mut RecordedOperationContext<'_>,
  request: &CandidateActionCommandRequest,
) -> AuvResult<CandidateActionCommandOutput> {
  request.validate()?;

  context.record_event(
    "candidate.action.command.observe.begin",
    Some(format!(
      "capturing {} AX frame(s) for app {} query {:?} role {:?}",
      request.stable_frames, request.app_bundle_id, request.query, request.role
    )),
  );

  let mut observations = Vec::new();
  let mut recognition_artifact_ref = None;

  for frame_index in 0..request.stable_frames {
    activate_app(&request.app_bundle_id)?;
    if let Some(shortcut) = request.reveal_shortcut.as_deref() {
      press_shortcut(shortcut)?;
      if request.reveal_settle_ms > 0 {
        thread::sleep(Duration::from_millis(request.reveal_settle_ms));
      }
    }

    let capture =
      auv_driver_macos::native::ax_tree::capture_ax_tree_snapshot(&request.app_bundle_id, 8, 80)?;
    let report = auv_driver_macos::native::ax_tree::render_ax_tree_report(&capture);
    let ax_report_path = std::env::temp_dir().join(format!(
      "auv-candidate-action-command-ax-{}-{}-{}.txt",
      frame_index,
      now_millis(),
      std::process::id()
    ));
    std::fs::write(&ax_report_path, report).map_err(|error| {
      format!(
        "failed to write temporary AX tree report {}: {error}",
        ax_report_path.display()
      )
    })?;

    let recognition_id = format!("{}-frame-{}", request.promotion_id, frame_index);
    let (_, recorded_recognition_artifact_ref, recognition) = record_ax_tree_recognition_artifact(
      context,
      &capture.snapshot,
      &ax_report_path,
      "ax-tree",
      &format!("{recognition_id}.txt"),
      Some(format!(
        "Source AX tree artifact for consent-gated command frame {frame_index}."
      )),
      &AxRecognitionArtifactRequest {
        recognition_id: recognition_id.clone(),
        policy: AxRecognitionPolicy {
          query: Some(request.query.clone()),
          role: Some(request.role.clone()),
          require_bounds: true,
          best_selection: AxBestSelectionStrategy::SingleFilteredItem,
        },
        artifact_role: "ax-recognition".to_string(),
        artifact_label: format!("{recognition_id}-recognition"),
        artifact_note:
          "AX tree-backed RecognitionResult runtime artifact for consent-gated command".to_string(),
      },
    )?;
    let _ = std::fs::remove_file(&ax_report_path);
    recognition_artifact_ref = Some(recorded_recognition_artifact_ref);
    observations.push(recognition);

    if frame_index + 1 < request.stable_frames && request.stable_frame_delay_ms > 0 {
      thread::sleep(Duration::from_millis(request.stable_frame_delay_ms));
    }
  }

  let latest = observations
    .last()
    .ok_or_else(|| "candidate action command captured no observations".to_string())?;

  let mut promotion_request = CandidatePromotionArtifactRequest::new(
    request.promotion_id.clone(),
    format!("{}-promotion", request.promotion_id),
  );
  promotion_request.source_recognition_artifact = recognition_artifact_ref;
  promotion_request.stability_policy = StabilityPolicy {
    min_frames: request.stable_frames,
    max_centroid_drift_px: request.max_centroid_drift_px,
    require_stable_text: request.require_stable_text,
  };
  promotion_request.freshness = Some(
    freshness_from_capture_backed_recognition(
      latest,
      "candidate.action.command.capture_ax_tree",
      "freshness derived from same-run AX capture",
    )
    .map_err(|error| error.to_string())?,
  );
  promotion_request.permission = Some(
    explicit_consent_for_candidate_promotion(
      &promotion_request.promotion_id,
      latest,
      CandidatePromotionConsentInput {
        granted_by: request.granted_by.clone(),
        scope_note: request.promotion_scope_note.clone(),
        evidence_note: request.promotion_evidence_note.clone(),
        approved_at_millis: now_millis(),
      },
    )
    .map_err(|error| error.to_string())?,
  );

  let (promotion_artifact_ref, promotion) =
    record_candidate_promotion_artifact_with_recognition_projection(
      context,
      &observations,
      &promotion_request,
    )?;

  context.record_event(
    "candidate.action.command.promotion.ready",
    Some(format!(
      "promotion {} recorded; building action decision",
      promotion_artifact_ref.artifact_id
    )),
  );

  let decision_request = CandidateActionDecisionRequest::new(
    request.decision_id.clone(),
    format!("{}-decision", request.decision_id),
  )
  .with_source_candidate_promotion_artifact(promotion_artifact_ref.clone());
  let (decision_artifact_ref, decision) =
    record_candidate_action_decision_artifact(context, &promotion, &decision_request)?;

  context.record_event(
    "candidate.action.command.execution.begin",
    Some(format!(
      "decision {} recorded; executing one approved candidate action",
      decision_artifact_ref.artifact_id
    )),
  );

  let execution_request = CandidateActionExecutionRequest::new(
    request.execution_id.clone(),
    format!("{}-execution", request.execution_id),
  )
  .with_source_candidate_action_decision_artifact(decision_artifact_ref.clone())
  .with_post_action_probe(CandidateActionPostActionProbe::focused_ax_node_reobserved())
  .with_consent(CandidateActionExecutionConsent {
    consent_id: format!("consent-{}", request.execution_id),
    granted_by: request.granted_by.clone(),
    scope_note: request.execution_scope_note.clone(),
    run_id: decision_artifact_ref.run_id.as_str().to_string(),
    source_promotion_id: promotion.promotion_id.clone(),
    source_decision_id: decision.decision_id.clone(),
    candidate_local_id: decision.candidate_local_id.clone(),
    approved_action: CandidateActionExecutionConsentAction::ExecuteSingleCandidateAction,
    approved_at_millis: now_millis(),
    evidence_note: request.execution_evidence_note.clone(),
  });

  let mut executor = MacosCandidateActionExecutor;
  let (execution_artifact_ref, _execution) = execute_and_record_single_candidate_action(
    context,
    &mut executor,
    &promotion,
    &decision,
    &execution_request,
  )?;

  Ok(CandidateActionCommandOutput {
    promotion_artifact_id: promotion_artifact_ref.artifact_id.as_str().to_string(),
    decision_artifact_id: decision_artifact_ref.artifact_id.as_str().to_string(),
    execution_artifact_id: execution_artifact_ref.artifact_id.as_str().to_string(),
  })
}

#[cfg(not(target_os = "macos"))]
pub fn execute_candidate_action_command(
  _context: &mut RecordedOperationContext<'_>,
  _request: &CandidateActionCommandRequest,
) -> AuvResult<CandidateActionCommandOutput> {
  Err("candidate action command is currently implemented only for macOS".to_string())
}

#[cfg(target_os = "macos")]
fn activate_app(app_bundle_id: &str) -> AuvResult<()> {
  use auv_driver::Driver;

  let driver = auv_driver_macos::MacosDriver::new();
  let session = driver
    .open_local()
    .map_err(|error| format!("failed to open macOS driver session: {error}"))?;
  let windows = session
    .window()
    .list()
    .map_err(|error| format!("failed to list windows before activation: {error}"))?;
  let target = windows
    .into_iter()
    .find(|window| window.app_bundle_id.as_deref() == Some(app_bundle_id))
    .ok_or_else(|| format!("failed to find a visible window for app {app_bundle_id}"))?;
  let _lease = session
    .window()
    .prepare_for_input(
      &target,
      auv_driver::input::PrepareForInputOptions {
        activation: auv_driver::input::ActivationPolicy::Foreground {
          settle: Duration::from_millis(250),
        },
        preserve_frontmost: false,
        install_focus_guard: false,
        settle: Duration::ZERO,
      },
    )
    .map_err(|error| format!("failed to activate target app {app_bundle_id}: {error}"))?;
  Ok(())
}

#[cfg(target_os = "macos")]
fn press_shortcut(shortcut: &str) -> AuvResult<()> {
  use auv_driver::Driver;

  let driver = auv_driver_macos::MacosDriver::new();
  let session = driver
    .open_local()
    .map_err(|error| format!("failed to open macOS driver session: {error}"))?;
  let _ = session
    .input()
    .press_key(auv_driver::input::KeyPressOptions {
      key: shortcut.to_string(),
      settle: Duration::ZERO,
    })
    .map_err(|error| format!("failed to press reveal shortcut {shortcut}: {error}"))?;
  Ok(())
}
