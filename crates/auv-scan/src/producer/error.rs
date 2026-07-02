use thiserror::Error;

use crate::artifact::ScanArtifactError;

#[derive(Debug, Error)]
pub enum ScanProducerError {
  #[error(transparent)]
  Artifact(#[from] ScanArtifactError),
  #[error("fixture image missing: {path}")]
  MissingImage { path: String },
  #[error("image has zero width or height")]
  ZeroImageDimension,
  #[error(transparent)]
  Io(#[from] std::io::Error),
  #[error("json parse error: {0}")]
  Json(#[from] serde_json::Error),
}
