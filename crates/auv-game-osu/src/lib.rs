pub mod benchmark;
pub mod visual_eval;
pub mod visual_truth;

pub use benchmark::{
  BenchmarkInputs, BenchmarkOutput, CapturePhase, CaptureSample, CaptureTraceSample,
  DispatchSample, LatencyReport, MapSummary, ObjectKind, RunMode, ScheduledAction,
  VerificationSummary, run_benchmark,
};
pub use visual_eval::{
  EvalProjection, FrameEvaluation, FrameLabelOutcome, FrameSpatialOutcome, LabelMap,
  VisualEvalReport, evaluate_visual_truth, iou,
};
pub use visual_truth::{
  CaptureFrame, ExpectedObjectTruth, VisualTruthFrame, VisualTruthManifest,
  build_visual_truth_manifest,
};
