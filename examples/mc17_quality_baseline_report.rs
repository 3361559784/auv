use std::env;
use std::path::PathBuf;

use auv_cli::run_read::{
  collect_quality_baseline_evidence_for_run,
  derive_minecraft_training_result_quality_baseline_report,
  derive_minecraft_training_result_quality_verdict, quality_baseline_profile_v1,
  quality_baseline_verdict_thresholds_probe_v1,
  quality_baseline_verdict_thresholds_trained_render_v1,
};
use auv_tracing_driver::store::LocalStore;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum VerdictMode {
  Probe,
  TrainedRender,
  Both,
}

struct Args {
  store_root: PathBuf,
  run_id: String,
  verdict_mode: VerdictMode,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
  let args = parse_args(env::args().skip(1).collect())?;
  let store = LocalStore::new(args.store_root)?;
  let profile = quality_baseline_profile_v1().map_err(|error| std::io::Error::other(error))?;
  let bundle = collect_quality_baseline_evidence_for_run(&store, &args.run_id, &profile)
    .map_err(|error| std::io::Error::other(error.to_string()))?;
  let report = derive_minecraft_training_result_quality_baseline_report(
    &profile,
    bundle.spatial_query.as_ref(),
    bundle.holdout_preview.as_ref(),
    bundle.render_quality.as_ref(),
    &bundle.collection_issues,
  );
  println!("{}", serde_json::to_string_pretty(&report)?);

  match args.verdict_mode {
    VerdictMode::Probe => {
      let thresholds = quality_baseline_verdict_thresholds_probe_v1()
        .map_err(|error| std::io::Error::other(error))?;
      let verdict = derive_minecraft_training_result_quality_verdict(&report, &thresholds);
      println!("{}", serde_json::to_string_pretty(&verdict)?);
    }
    VerdictMode::TrainedRender => {
      let thresholds = quality_baseline_verdict_thresholds_trained_render_v1()
        .map_err(|error| std::io::Error::other(error))?;
      let verdict = derive_minecraft_training_result_quality_verdict(&report, &thresholds);
      println!("{}", serde_json::to_string_pretty(&verdict)?);
    }
    VerdictMode::Both => {
      let probe = quality_baseline_verdict_thresholds_probe_v1()
        .map_err(|error| std::io::Error::other(error))?;
      let trained_render = quality_baseline_verdict_thresholds_trained_render_v1()
        .map_err(|error| std::io::Error::other(error))?;
      let payload = serde_json::json!({
        "probe": derive_minecraft_training_result_quality_verdict(&report, &probe),
        "trained_render": derive_minecraft_training_result_quality_verdict(&report, &trained_render),
      });
      println!("{}", serde_json::to_string_pretty(&payload)?);
    }
  }

  Ok(())
}

fn parse_args(args: Vec<String>) -> Result<Args, String> {
  let mut store_root = None;
  let mut run_id = None;
  let mut verdict_mode = VerdictMode::Both;
  let mut iter = args.into_iter();
  while let Some(flag) = iter.next() {
    if flag == "--verdict-mode" {
      let value = iter
        .next()
        .ok_or_else(|| "--verdict-mode requires a value".to_string())?;
      verdict_mode = match value.as_str() {
        "probe" => VerdictMode::Probe,
        "trained_render" => VerdictMode::TrainedRender,
        "both" => VerdictMode::Both,
        other => return Err(format!("unknown --verdict-mode value: {other}")),
      };
      continue;
    }
    let value = iter
      .next()
      .ok_or_else(|| format!("{flag} requires a value"))?;
    match flag.as_str() {
      "--store-root" => store_root = Some(PathBuf::from(value)),
      "--run-id" => run_id = Some(value),
      other => return Err(format!("unknown argument: {other}")),
    }
  }
  Ok(Args {
    store_root: store_root.ok_or("--store-root is required")?,
    run_id: run_id.ok_or("--run-id is required")?,
    verdict_mode,
  })
}
