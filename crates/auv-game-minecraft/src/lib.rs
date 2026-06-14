pub mod artifact;
pub mod overlay;
pub mod projection;
pub mod types;

pub use artifact::{MinecraftProjectionArtifact, ProjectionViewportBounds};
pub use overlay::render_projection_overlay;
pub use projection::MinecraftProjector;
pub use types::{
  BlockFace, BlockPosition, InventorySummaryEntry, MinecraftBlockTarget, MinecraftProjectedPoint,
  MinecraftSpatialFrame, NearbyBlock, NearbyEntity, PlayerPose, ProjectionVisibility, RaycastHit,
  Vec3, Viewport,
};

// NOTICE(mc3-live-binding): live screenshot binding, capture_skew_ms calibration,
// and runtime/CLI integration land in MC-3 after a real MC-1 telemetry sample exists.
