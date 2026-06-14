pub mod artifact;
pub mod input_target;
pub mod overlay;
pub mod projection;
pub mod types;
pub mod verify;

pub use artifact::{MinecraftProjectionArtifact, ProjectionViewportBounds};
pub use input_target::projected_window_point;
pub use overlay::render_projection_overlay;
pub use projection::MinecraftProjector;
pub use types::{
  BlockFace, BlockPosition, InventorySummaryEntry, MinecraftBlockTarget, MinecraftProjectedPoint,
  MinecraftSpatialFrame, NearbyBlock, NearbyEntity, PlayerPose, ProjectionVisibility, RaycastHit,
  Vec3, Viewport,
};
pub use verify::{WorldDiffFailure, WorldDiffRequest, WorldDiffVerdict, evaluate_world_diff};

// NOTICE(mc3-live-binding): offline world-diff verdicts and input-target mapping now live here,
// but real telemetry reads, capture_skew_ms calibration, runtime/CLI integration, and live driver
// dispatch still wait on a real MC-1 telemetry sample.
