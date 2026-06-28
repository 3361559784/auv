use std::env;
use std::path::PathBuf;

use auv_cli::run_read::{
  collect_quality_baseline_evidence_for_run,
  derive_minecraft_training_result_quality_baseline_report, quality_baseline_profile_v1,
};
use auv_tracing_driver::store::LocalStore;

struct Args {
  store_root: PathBuf,
  run_id: String,
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
  Ok(())
}

fn parse_args(args: Vec<String>) -> Result<Args, String> {
  let mut store_root = None;
  let mut run_id = None;
  let mut iter = args.into_iter();
  while let Some(flag) = iter.next() {
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
  })
}
