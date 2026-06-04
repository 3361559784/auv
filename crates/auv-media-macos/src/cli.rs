//! `auv-now-playing` binary entry point: read the system now-playing state and
//! emit it as a human summary (default) or the `now-playing-v0` JSON contract.

use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;

use crate::now_playing;
use crate::output::{build_now_playing_output, render_human_summary};

#[derive(Parser)]
#[command(
  name = "auv-now-playing",
  about = "Read the macOS system now-playing state"
)]
struct Cli {
  /// Emit the now-playing-v0 JSON object to stdout (default: human summary).
  #[arg(long, conflicts_with = "json_out")]
  json: bool,
  /// Write the now-playing-v0 JSON object to a file.
  #[arg(long, value_name = "path")]
  json_out: Option<PathBuf>,
}

/// Parse argv, perform the read, emit output, and return the process exit code.
///
/// Exit `0` on a successful read, including the nothing-playing case
/// (`present: false` is state, not an error). Non-zero only on a read failure.
pub fn run() -> ExitCode {
  let cli = Cli::parse();

  let state = match now_playing() {
    Ok(state) => state,
    Err(error) => {
      eprintln!("{error}");
      return ExitCode::FAILURE;
    }
  };

  if cli.json || cli.json_out.is_some() {
    let output = build_now_playing_output(&state);
    let json = match serde_json::to_string_pretty(&output) {
      Ok(json) => json,
      Err(error) => {
        eprintln!("failed to encode now-playing JSON: {error}");
        return ExitCode::FAILURE;
      }
    };
    if let Some(path) = cli.json_out {
      if let Err(error) = std::fs::write(&path, format!("{json}\n")) {
        eprintln!("failed to write {}: {error}", path.display());
        return ExitCode::FAILURE;
      }
    } else {
      println!("{json}");
    }
  } else {
    println!("{}", render_human_summary(&state));
  }

  ExitCode::SUCCESS
}
