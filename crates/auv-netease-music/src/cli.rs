// File: crates/auv-netease-music/src/cli.rs
use std::num::{NonZeroU64, NonZeroUsize};
use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Args, Parser, Subcommand};

use crate::output::build_playlist_json_output;
use crate::{Inputs, PlaylistCategory, run_live_scan};

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum OutputMode {
  Human,
  Json,
  JsonFile(PathBuf),
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct PlaylistCommand {
  pub inputs: Inputs,
  pub query: Option<String>,
  pub output: OutputMode,
}

#[derive(Clone, Debug, Parser)]
#[command(
  name = "auv-netease-music",
  disable_help_subcommand = true,
  about = "NetEase Cloud Music CLI"
)]
struct CliArgs {
  #[command(subcommand)]
  command: CliSubcommand,
}

#[derive(Clone, Debug, Subcommand)]
enum CliSubcommand {
  /// List NetEase Cloud Music sidebar playlists.
  Playlist(PlaylistArgs),
}

#[derive(Clone, Debug, Args)]
struct PlaylistArgs {
  #[arg(value_name = "ls|keyword")]
  first: Option<String>,
  #[arg(value_name = "keyword")]
  second: Option<String>,
  #[arg(long = "category")]
  category: Option<PlaylistCategory>,
  #[arg(long = "filter")]
  filter: Option<String>,
  #[arg(long = "json")]
  json: bool,
  #[arg(long = "json-out")]
  json_out: Option<PathBuf>,
  #[arg(long = "app-id")]
  app_id: Option<String>,
  #[arg(long = "artifact-dir")]
  artifact_dir: Option<PathBuf>,
  #[arg(long = "max-scrolls")]
  max_scrolls: Option<NonZeroUsize>,
  #[arg(long = "scroll-amount", value_parser = crate::positive_scroll_amount)]
  scroll_amount: Option<f64>,
  #[arg(long = "scroll-settle-ms")]
  scroll_settle_ms: Option<NonZeroU64>,
  #[arg(long = "sidebar-region")]
  sidebar_region: Option<String>,
  #[arg(long = "hint-ocr-custom-word")]
  custom_words: Vec<String>,
  #[arg(long = "hint-ocr-custom-words")]
  custom_word_csvs: Vec<String>,
  #[arg(long = "hint-ocr-custom-words-file")]
  custom_word_files: Vec<PathBuf>,
  #[arg(long = "hint-ocr-language")]
  ocr_languages: Vec<String>,
  #[arg(long = "hint-ocr-languages")]
  ocr_language_csvs: Vec<String>,
}

fn command_from_args(parsed: CliArgs) -> Result<PlaylistCommand, String> {
  match parsed.command {
    CliSubcommand::Playlist(args) => parse_playlist(args),
  }
}

fn parse_playlist(args: PlaylistArgs) -> Result<PlaylistCommand, String> {
  let mut inputs = Inputs::with_defaults();
  let query = match (args.first.as_deref(), args.second.as_deref()) {
    (None, None) => None,
    (Some("ls"), None) => None,
    (Some("ls"), Some(keyword)) => Some(keyword.to_string()),
    (Some(keyword), None) => Some(keyword.to_string()),
    (Some(_), Some(extra)) => return Err(format!("unexpected extra argument {extra:?}")),
    (None, Some(_)) => unreachable!("clap fills positional arguments in order"),
  };

  if let Some(app_id) = args.app_id {
    inputs.app_id = app_id;
  }
  if let Some(artifact_dir) = args.artifact_dir {
    inputs.artifact_dir = artifact_dir;
  }
  if let Some(max_scrolls) = args.max_scrolls {
    inputs.max_scrolls = max_scrolls.get();
  }
  if let Some(scroll_amount) = args.scroll_amount {
    inputs.scroll_amount = scroll_amount;
  }
  if let Some(scroll_settle_ms) = args.scroll_settle_ms {
    inputs.scroll_settle_ms = scroll_settle_ms.get();
  }
  if let Some(category) = args.category {
    inputs.category = category;
  }
  if let Some(sidebar_region) = args.sidebar_region {
    inputs.sidebar_region = Some(crate::parse_ratio_region(sidebar_region)?);
  }
  for word in args.custom_words {
    crate::push_trimmed(&mut inputs.ocr_options.custom_words, word);
  }
  for csv in args.custom_word_csvs {
    crate::push_csv(&mut inputs.ocr_options.custom_words, &csv);
  }
  for path in args.custom_word_files {
    crate::load_custom_words_file(&mut inputs.ocr_options.custom_words, path)?;
  }
  for language in args.ocr_languages {
    crate::push_ocr_language(&mut inputs.ocr_options, language);
  }
  for csv in args.ocr_language_csvs {
    for language in crate::split_csv(&csv) {
      crate::push_ocr_language(&mut inputs.ocr_options, language);
    }
  }
  let query = args.filter.or(query);
  let output = match args.json_out {
    Some(path) => OutputMode::JsonFile(path),
    None if args.json => OutputMode::Json,
    None => OutputMode::Human,
  };
  Ok(PlaylistCommand {
    inputs,
    query,
    output,
  })
}

/// Entry point for the `auv-netease-music` binary.
pub fn run() -> ExitCode {
  let parsed = match CliArgs::try_parse_from(std::env::args()) {
    Ok(parsed) => parsed,
    Err(error) => {
      let exit_code = error.exit_code();
      let _ = error.print();
      return match u8::try_from(exit_code) {
        Ok(0) => ExitCode::SUCCESS,
        Ok(code) => ExitCode::from(code),
        Err(_) => ExitCode::from(2),
      };
    }
  };

  match command_from_args(parsed) {
    Ok(cmd) => run_playlist(cmd),
    Err(error) => {
      if error.starts_with("error:") {
        eprint!("{error}");
      } else {
        eprintln!("error: {error}");
      }
      ExitCode::from(2)
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn playlist_args() -> PlaylistArgs {
    PlaylistArgs {
      first: None,
      second: None,
      category: None,
      filter: None,
      json: false,
      json_out: None,
      app_id: None,
      artifact_dir: None,
      max_scrolls: None,
      scroll_amount: None,
      scroll_settle_ms: None,
      sidebar_region: None,
      custom_words: Vec::new(),
      custom_word_csvs: Vec::new(),
      custom_word_files: Vec::new(),
      ocr_languages: Vec::new(),
      ocr_language_csvs: Vec::new(),
    }
  }

  #[test]
  fn parse_playlist_without_positional_or_filter_leaves_query_empty() {
    let command = parse_playlist(playlist_args()).expect("playlist args should parse");

    assert_eq!(command.query, None);
    assert_eq!(command.output, OutputMode::Human);
  }

  #[test]
  fn parse_playlist_uses_positional_keyword_as_query() {
    let mut args = playlist_args();
    args.first = Some("daily".to_string());

    let command = parse_playlist(args).expect("playlist args should parse");

    assert_eq!(command.query.as_deref(), Some("daily"));
  }

  #[test]
  fn parse_playlist_prefers_explicit_filter_over_positional_keyword() {
    let mut args = playlist_args();
    args.first = Some("daily".to_string());
    args.filter = Some("liked".to_string());

    let command = parse_playlist(args).expect("playlist args should parse");

    assert_eq!(command.query.as_deref(), Some("liked"));
  }
}

fn run_playlist(cmd: PlaylistCommand) -> ExitCode {
  let scan = match run_live_scan(&cmd.inputs) {
    Ok(scan) => scan,
    Err(error) => {
      eprintln!("scan failed: {error}");
      return ExitCode::from(1);
    }
  };
  let output = build_playlist_json_output(&scan, cmd.query.as_deref());

  match &cmd.output {
    OutputMode::Human => {
      println!("{}", scan.human_summary());
      ExitCode::SUCCESS
    }
    OutputMode::Json => match serde_json::to_string_pretty(&output) {
      Ok(json) => {
        println!("{json}");
        ExitCode::SUCCESS
      }
      Err(error) => {
        eprintln!("encode failed: {error}");
        ExitCode::from(1)
      }
    },
    OutputMode::JsonFile(path) => {
      let json = match serde_json::to_string_pretty(&output) {
        Ok(json) => json,
        Err(error) => {
          eprintln!("encode failed: {error}");
          return ExitCode::from(1);
        }
      };
      if let Err(error) = std::fs::write(path, json) {
        eprintln!("failed to write {}: {error}", path.display());
        return ExitCode::from(1);
      }
      ExitCode::SUCCESS
    }
  }
}
