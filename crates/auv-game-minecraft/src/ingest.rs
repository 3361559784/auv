use std::io::{BufRead, BufReader, Read};
use std::path::Path;

use crate::types::MinecraftSpatialFrame;

/// Outcome of scanning an append-only telemetry stream for its most recent frame.
#[derive(Clone, Debug, PartialEq)]
pub struct LatestFrameScan {
  /// The most recent successfully parsed frame, if any non-empty line parsed.
  pub frame: Option<MinecraftSpatialFrame>,
  /// Total non-empty lines observed.
  pub line_count: u64,
  /// Non-empty lines that failed to parse as a `MinecraftSpatialFrame`.
  pub malformed_line_count: u64,
}

impl LatestFrameScan {
  fn empty() -> Self {
    Self {
      frame: None,
      line_count: 0,
      malformed_line_count: 0,
    }
  }
}

/// Read the most recent `MinecraftSpatialFrame` from an append-only telemetry
/// JSONL file without loading the whole file into memory.
///
/// The sidecar writes one frame per line, oldest first. Readers consume only
/// flushed durable records, so the freshest binding candidate is the last
/// well-formed line. This streams line by line and retains only the latest
/// parsed frame, so a multi-hundred-megabyte sample costs one line of peak
/// memory rather than the whole file.
pub fn read_latest_spatial_frame(path: &Path) -> Result<LatestFrameScan, String> {
  let file = std::fs::File::open(path).map_err(|error| {
    format!(
      "failed to open telemetry sample {}: {error}",
      path.display()
    )
  })?;
  scan_latest_spatial_frame(file)
}

/// Core scan over any byte reader. Separated from file opening so the binding
/// logic is unit-testable without touching the filesystem.
pub fn scan_latest_spatial_frame<R: Read>(reader: R) -> Result<LatestFrameScan, String> {
  let mut buffered = BufReader::new(reader);
  let mut scan = LatestFrameScan::empty();
  let mut line = String::new();

  loop {
    line.clear();
    let read = buffered
      .read_line(&mut line)
      .map_err(|error| format!("failed to read telemetry sample line: {error}"))?;
    if read == 0 {
      break;
    }
    let trimmed = line.trim();
    if trimmed.is_empty() {
      continue;
    }
    scan.line_count += 1;
    match serde_json::from_str::<MinecraftSpatialFrame>(trimmed) {
      Ok(frame) => scan.frame = Some(frame),
      Err(_) => scan.malformed_line_count += 1,
    }
  }

  Ok(scan)
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::types::{PlayerPose, Vec3, Viewport};

  fn frame_line(id: &str, tick: u64, ts: u64) -> String {
    let frame = MinecraftSpatialFrame {
      spatial_frame_id: id.to_string(),
      world_tick: tick,
      monotonic_timestamp_ms: ts,
      viewport: Viewport::new(1708, 960),
      view_matrix: [0.0; 16],
      projection_matrix: [0.0; 16],
      player_pose: PlayerPose {
        eye_position: Vec3::new(-3.5, 70.62, -9.5),
        yaw: 0.0,
        pitch: 0.0,
      },
      raycast_hit: None,
      nearby_blocks: Vec::new(),
      nearby_entities: Vec::new(),
      inventory_summary: Vec::new(),
      screenshot_artifact_ref: None,
      mc_capture_skew_ms: None,
    };
    serde_json::to_string(&frame).expect("frame serializes")
  }

  #[test]
  fn returns_last_frame_from_multiple_lines() {
    let body = format!(
      "{}\n{}\n{}\n",
      frame_line("frame-1", 1, 1000),
      frame_line("frame-2", 2, 2000),
      frame_line("frame-3", 3, 3000),
    );
    let scan = scan_latest_spatial_frame(body.as_bytes()).expect("scan succeeds");
    assert_eq!(scan.line_count, 3);
    assert_eq!(scan.malformed_line_count, 0);
    let frame = scan.frame.expect("a frame is present");
    assert_eq!(frame.spatial_frame_id, "frame-3");
    assert_eq!(frame.world_tick, 3);
    assert_eq!(frame.monotonic_timestamp_ms, 3000);
  }

  #[test]
  fn skips_blank_lines_without_counting_them() {
    let body = format!(
      "\n{}\n   \n{}\n\n",
      frame_line("a", 1, 10),
      frame_line("b", 2, 20)
    );
    let scan = scan_latest_spatial_frame(body.as_bytes()).expect("scan succeeds");
    assert_eq!(scan.line_count, 2);
    assert_eq!(scan.malformed_line_count, 0);
    assert_eq!(scan.frame.expect("frame").spatial_frame_id, "b");
  }

  #[test]
  fn counts_malformed_lines_and_keeps_last_valid_frame() {
    let body = format!(
      "{}\nnot json\n{}\n{{\"partial\":true}}\n",
      frame_line("valid-1", 1, 10),
      frame_line("valid-2", 2, 20),
    );
    let scan = scan_latest_spatial_frame(body.as_bytes()).expect("scan succeeds");
    assert_eq!(scan.line_count, 4);
    assert_eq!(scan.malformed_line_count, 2);
    assert_eq!(scan.frame.expect("frame").spatial_frame_id, "valid-2");
  }

  #[test]
  fn empty_stream_yields_no_frame() {
    let scan = scan_latest_spatial_frame("".as_bytes()).expect("scan succeeds");
    assert_eq!(scan.line_count, 0);
    assert_eq!(scan.malformed_line_count, 0);
    assert!(scan.frame.is_none());
  }

  #[test]
  fn all_malformed_yields_no_frame_but_counts_lines() {
    let scan = scan_latest_spatial_frame("nope\nstill nope\n".as_bytes()).expect("scan succeeds");
    assert_eq!(scan.line_count, 2);
    assert_eq!(scan.malformed_line_count, 2);
    assert!(scan.frame.is_none());
  }
}
