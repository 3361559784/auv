use std::fs;
use std::path::PathBuf;

use auv_game_minecraft::verify::{WorldDiffFailure, WorldDiffVerdict};
use auv_tracing_driver::recorded_operation::RecordedOperationContext;

use crate::contract::{
  ArtifactRef, FailureLayer, VERIFICATION_RESULT_API_VERSION, VerificationMethod,
  VerificationResult,
};
use crate::minecraft::MINECRAFT_SPATIAL_FRAME_ARTIFACT_ROLE;

pub fn map_world_diff_verdict_to_verification_result(
  verdict: &WorldDiffVerdict,
  evidence: Vec<ArtifactRef>,
) -> VerificationResult {
  let failure_layer = match verdict.failure {
    None => None,
    Some(WorldDiffFailure::VerificationUnreliable) => Some(FailureLayer::VerificationUnreliable),
    Some(WorldDiffFailure::StateChangedNoMatch) => Some(FailureLayer::StateChangedNoMatch),
    Some(WorldDiffFailure::SemanticMismatch) => Some(FailureLayer::SemanticMismatch),
  };

  VerificationResult {
    api_version: VERIFICATION_RESULT_API_VERSION.to_string(),
    method: VerificationMethod::SemanticMatch,
    executed: verdict.executed,
    state_changed: verdict.state_changed,
    semantic_matched: verdict.semantic_matched,
    failure_layer,
    evidence,
    consumed_candidate_ref: None,
    consumed_node_ref: None,
    consumed_recognition_artifact_ref: None,
    consumed_recognition_id: None,
    consumed_recognized_item_id: None,
    observed_label: verdict.observed_block_id.clone(),
  }
}

pub fn build_query_wired_witness_absent_verification() -> VerificationResult {
  map_world_diff_verdict_to_verification_result(
    &WorldDiffVerdict {
      executed: true,
      state_changed: false,
      semantic_matched: None,
      failure: Some(WorldDiffFailure::VerificationUnreliable),
      observed_block_id: None,
      observed_item_delta: None,
    },
    Vec::new(),
  )
}

pub fn stage_minecraft_spatial_frame_artifact(
  context: &mut RecordedOperationContext<'_>,
  frame: &auv_game_minecraft::MinecraftSpatialFrame,
) -> Result<(PathBuf, ArtifactRef), String> {
  let artifact_json = serde_json::to_string_pretty(frame)
    .map(|mut json| {
      json.push('\n');
      json
    })
    .map_err(|error| format!("failed to serialize minecraft spatial frame: {error}"))?;
  let artifact_path = std::env::temp_dir().join(format!(
    "auv-minecraft-spatial-frame-{}-{}.json",
    context.run_id(),
    crate::model::now_millis()
  ));
  fs::write(&artifact_path, artifact_json.as_bytes())
    .map_err(|error| format!("failed to write minecraft spatial frame artifact: {error}"))?;
  let staged = context.stage_artifact_file_with_ref(
    MINECRAFT_SPATIAL_FRAME_ARTIFACT_ROLE,
    &artifact_path,
    "minecraft-spatial-frame.json",
    Some("durable minecraft spatial frame with pose, matrices, and raycast truth".to_string()),
  );
  let _ = fs::remove_file(&artifact_path);
  staged.map_err(|error| error.to_string())
}
