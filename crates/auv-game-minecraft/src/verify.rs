use serde::{Deserialize, Serialize};

use crate::types::{BlockPosition, MinecraftBlockTarget, MinecraftSpatialFrame};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorldDiffFailure {
  VerificationUnreliable,
  StateChangedNoMatch,
  SemanticMismatch,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorldDiffRequest {
  pub target: MinecraftBlockTarget,
  pub expected_item_id: Option<String>,
}

impl WorldDiffRequest {
  pub fn new(target: MinecraftBlockTarget) -> Self {
    Self {
      target,
      expected_item_id: None,
    }
  }

  pub fn with_expected_item_id(mut self, expected_item_id: impl Into<String>) -> Self {
    self.expected_item_id = Some(expected_item_id.into());
    self
  }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorldDiffVerdict {
  pub executed: bool,
  pub state_changed: bool,
  pub semantic_matched: Option<bool>,
  pub failure: Option<WorldDiffFailure>,
  pub observed_block_id: Option<String>,
  pub observed_item_delta: Option<i64>,
}

impl WorldDiffVerdict {
  fn unreliable(observed_block_id: Option<String>, observed_item_delta: Option<i64>) -> Self {
    Self {
      executed: true,
      state_changed: false,
      semantic_matched: None,
      failure: Some(WorldDiffFailure::VerificationUnreliable),
      observed_block_id,
      observed_item_delta,
    }
  }
}

pub fn evaluate_world_diff(
  pre: &MinecraftSpatialFrame,
  post: &MinecraftSpatialFrame,
  request: &WorldDiffRequest,
) -> WorldDiffVerdict {
  let observed_item_delta = request
    .expected_item_id
    .as_deref()
    .map(|item_id| inventory_delta(pre, post, item_id));

  if post.world_tick <= pre.world_tick || post.monotonic_timestamp_ms <= pre.monotonic_timestamp_ms
  {
    return WorldDiffVerdict::unreliable(
      target_block_id(post, request.target.block_pos),
      observed_item_delta,
    );
  }

  let Some(pre_witness) = pre_target_witness(pre, request.target.block_pos) else {
    return WorldDiffVerdict::unreliable(
      target_block_id(post, request.target.block_pos),
      observed_item_delta,
    );
  };

  let post_block_id = target_block_id(post, request.target.block_pos);
  let removed = is_removed(&pre_witness, post_block_id.as_deref());
  let semantic_matched = request
    .expected_item_id
    .as_ref()
    .map(|_| removed && observed_item_delta.unwrap_or_default() > 0);

  let failure = if removed {
    match semantic_matched {
      Some(true) | None => None,
      Some(false) => Some(WorldDiffFailure::StateChangedNoMatch),
    }
  } else if observed_item_delta.unwrap_or_default() > 0 {
    Some(WorldDiffFailure::SemanticMismatch)
  } else {
    None
  };

  WorldDiffVerdict {
    executed: true,
    state_changed: removed,
    semantic_matched,
    failure,
    observed_block_id: post_block_id,
    observed_item_delta,
  }
}

fn pre_target_witness(pre: &MinecraftSpatialFrame, block_pos: BlockPosition) -> Option<String> {
  if let Some(hit) = &pre.raycast_hit
    && hit.block_pos == block_pos
    && !is_air_block_id(&hit.block_id)
  {
    return Some(hit.block_id.clone());
  }

  target_block_id(pre, block_pos).filter(|block_id| !is_air_block_id(block_id))
}

fn target_block_id(frame: &MinecraftSpatialFrame, block_pos: BlockPosition) -> Option<String> {
  if let Some(hit) = &frame.raycast_hit
    && hit.block_pos == block_pos
  {
    return Some(hit.block_id.clone());
  }

  frame
    .nearby_blocks
    .iter()
    .find(|block| block.block_pos == block_pos)
    .map(|block| block.block_id.clone())
}

fn inventory_delta(
  pre: &MinecraftSpatialFrame,
  post: &MinecraftSpatialFrame,
  item_id: &str,
) -> i64 {
  inventory_count(post, item_id) - inventory_count(pre, item_id)
}

fn inventory_count(frame: &MinecraftSpatialFrame, item_id: &str) -> i64 {
  frame
    .inventory_summary
    .iter()
    .find(|entry| entry.item_id == item_id)
    .map(|entry| i64::from(entry.count))
    .unwrap_or_default()
}

fn is_removed(pre_block_id: &str, post_block_id: Option<&str>) -> bool {
  if is_air_block_id(pre_block_id) {
    return false;
  }

  match post_block_id {
    // NOTICE(mc3-nearby-block-radius): POST absence counts as removal only because PRE already witnessed a non-air block at the same target.
    None => true,
    Some(block_id) => is_air_block_id(block_id),
  }
}

fn is_air_block_id(block_id: &str) -> bool {
  block_id == "minecraft:air"
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::types::{
    BlockFace, BlockPosition, InventorySummaryEntry, NearbyBlock, NearbyEntity, PlayerPose,
    RaycastHit, Vec3, Viewport,
  };

  fn frame_at(
    world_tick: u64,
    timestamp_ms: u64,
    raycast_hit: Option<RaycastHit>,
    nearby_blocks: Vec<NearbyBlock>,
    inventory_summary: Vec<InventorySummaryEntry>,
  ) -> MinecraftSpatialFrame {
    MinecraftSpatialFrame {
      spatial_frame_id: format!("frame-{world_tick}"),
      world_tick,
      monotonic_timestamp_ms: timestamp_ms,
      viewport: Viewport::new(800, 600),
      view_matrix: [
        1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
      ],
      projection_matrix: [
        1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
      ],
      player_pose: PlayerPose {
        eye_position: Vec3::new(0.0, 0.0, 0.0),
        yaw: 0.0,
        pitch: 0.0,
      },
      raycast_hit,
      nearby_blocks,
      nearby_entities: vec![NearbyEntity {
        entity_id: "pig-1".to_string(),
        entity_kind: "minecraft:pig".to_string(),
      }],
      inventory_summary,
      screenshot_artifact_ref: None,
      mc_capture_skew_ms: None,
    }
  }

  fn target() -> MinecraftBlockTarget {
    MinecraftBlockTarget {
      block_pos: BlockPosition::new(1, 2, 3),
      face: Some(BlockFace::North),
    }
  }

  fn witnessed_stone() -> RaycastHit {
    RaycastHit {
      block_pos: target().block_pos,
      face: BlockFace::North,
      block_id: "minecraft:stone".to_string(),
    }
  }

  #[test]
  fn matches_when_block_disappears_and_inventory_rises() {
    let pre = frame_at(
      10,
      1_000,
      Some(witnessed_stone()),
      vec![NearbyBlock {
        block_pos: target().block_pos,
        block_id: "minecraft:stone".to_string(),
      }],
      vec![InventorySummaryEntry {
        item_id: "minecraft:stone".to_string(),
        count: 1,
      }],
    );
    let post = frame_at(
      11,
      1_050,
      None,
      vec![],
      vec![InventorySummaryEntry {
        item_id: "minecraft:stone".to_string(),
        count: 2,
      }],
    );
    let request = WorldDiffRequest::new(target()).with_expected_item_id("minecraft:stone");

    let verdict = evaluate_world_diff(&pre, &post, &request);

    assert_eq!(
      verdict,
      WorldDiffVerdict {
        executed: true,
        state_changed: true,
        semantic_matched: Some(true),
        failure: None,
        observed_block_id: None,
        observed_item_delta: Some(1),
      }
    );
  }

  #[test]
  fn reports_state_changed_no_match_when_inventory_stays_flat() {
    let pre = frame_at(
      10,
      1_000,
      Some(witnessed_stone()),
      vec![NearbyBlock {
        block_pos: target().block_pos,
        block_id: "minecraft:stone".to_string(),
      }],
      vec![InventorySummaryEntry {
        item_id: "minecraft:stone".to_string(),
        count: 1,
      }],
    );
    let post = frame_at(11, 1_050, None, vec![], vec![]);
    let request = WorldDiffRequest::new(target()).with_expected_item_id("minecraft:stone");

    let verdict = evaluate_world_diff(&pre, &post, &request);

    assert!(verdict.state_changed);
    assert_eq!(verdict.semantic_matched, Some(false));
    assert_eq!(verdict.failure, Some(WorldDiffFailure::StateChangedNoMatch));
    assert_eq!(verdict.observed_item_delta, Some(-1));
  }

  #[test]
  fn reports_semantic_mismatch_when_inventory_rises_but_block_remains() {
    let pre = frame_at(
      10,
      1_000,
      Some(witnessed_stone()),
      vec![NearbyBlock {
        block_pos: target().block_pos,
        block_id: "minecraft:stone".to_string(),
      }],
      vec![InventorySummaryEntry {
        item_id: "minecraft:stone".to_string(),
        count: 1,
      }],
    );
    let post = frame_at(
      11,
      1_050,
      Some(witnessed_stone()),
      vec![NearbyBlock {
        block_pos: target().block_pos,
        block_id: "minecraft:stone".to_string(),
      }],
      vec![InventorySummaryEntry {
        item_id: "minecraft:stone".to_string(),
        count: 2,
      }],
    );
    let request = WorldDiffRequest::new(target()).with_expected_item_id("minecraft:stone");

    let verdict = evaluate_world_diff(&pre, &post, &request);

    assert!(!verdict.state_changed);
    assert_eq!(verdict.semantic_matched, Some(false));
    assert_eq!(verdict.failure, Some(WorldDiffFailure::SemanticMismatch));
    assert_eq!(
      verdict.observed_block_id.as_deref(),
      Some("minecraft:stone")
    );
  }

  #[test]
  fn reports_unreliable_when_pre_witness_is_missing() {
    let pre = frame_at(10, 1_000, None, vec![], vec![]);
    let post = frame_at(11, 1_050, None, vec![], vec![]);
    let request = WorldDiffRequest::new(target()).with_expected_item_id("minecraft:stone");

    let verdict = evaluate_world_diff(&pre, &post, &request);

    assert_eq!(
      verdict,
      WorldDiffVerdict {
        executed: true,
        state_changed: false,
        semantic_matched: None,
        failure: Some(WorldDiffFailure::VerificationUnreliable),
        observed_block_id: None,
        observed_item_delta: Some(0),
      }
    );
  }

  #[test]
  fn reports_unreliable_when_post_tick_is_not_newer() {
    let pre = frame_at(
      10,
      1_000,
      Some(witnessed_stone()),
      vec![NearbyBlock {
        block_pos: target().block_pos,
        block_id: "minecraft:stone".to_string(),
      }],
      vec![],
    );
    let post = frame_at(10, 1_000, None, vec![], vec![]);
    let request = WorldDiffRequest::new(target());

    let verdict = evaluate_world_diff(&pre, &post, &request);

    assert_eq!(
      verdict.failure,
      Some(WorldDiffFailure::VerificationUnreliable)
    );
    assert!(!verdict.state_changed);
    assert_eq!(verdict.semantic_matched, None);
  }

  #[test]
  fn treats_minecraft_air_as_removed() {
    let pre = frame_at(
      10,
      1_000,
      Some(witnessed_stone()),
      vec![NearbyBlock {
        block_pos: target().block_pos,
        block_id: "minecraft:stone".to_string(),
      }],
      vec![],
    );
    let post = frame_at(
      11,
      1_050,
      Some(RaycastHit {
        block_pos: target().block_pos,
        face: BlockFace::North,
        block_id: "minecraft:air".to_string(),
      }),
      vec![NearbyBlock {
        block_pos: target().block_pos,
        block_id: "minecraft:air".to_string(),
      }],
      vec![],
    );
    let request = WorldDiffRequest::new(target());

    let verdict = evaluate_world_diff(&pre, &post, &request);

    assert!(verdict.state_changed);
    assert_eq!(verdict.failure, None);
  }
}
