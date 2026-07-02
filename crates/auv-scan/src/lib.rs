//! Temporal scan contracts — `scan-frame-v0` wire, artifact IO, and slice-2 producers.

#[cfg(test)]
mod fixture;

pub mod artifact;
pub mod frame;
pub mod producer;

pub use artifact::{
  ScanArtifactError, frame_artifact_file_name, read_frame_artifact, write_frame_artifact,
};
pub use frame::{SCAN_FRAME_SCHEMA_VERSION, ScanBounds, ScanFrame, ScanImageRef};
#[cfg(feature = "live-capture")]
pub use producer::live::produce_frame_from_capture;
pub use producer::{
  FrameCaptureMeta, ProducedFrame, ScanProducerError, bounds_to_scan_bounds,
  bounds_to_scan_bounds_f64, build_scan_frame, frame_from_capture, produce_frame_from_fixture_dir,
  write_frame_with_image,
};
