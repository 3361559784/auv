//! Session API service seam (API-P4 boundary).
//!
//! Owns the execute-facing `SessionService` surface separately from the
//! viewer-facing `inspect_server` and the tool-facing `mcp`. This is not a
//! transport/gRPC server: API-P4 explicitly defers the tonic/axum/daemon choice.
//!
//! Current contents:
//! - `summary`: the two-source `GetOperation` read path + join policy (API-P7).

pub mod summary;
