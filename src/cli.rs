use std::collections::BTreeMap;

use auv_cli::model::{AuvResult, DisturbanceClass, ExecutionTarget, InvokeRequest};

pub enum CliCommand {
  Help,
  ListCommands,
  ListDrivers,
  Invoke(InvokeRequest),
  Inspect {
    run_id: String,
  },
  SkillList,
  SkillShow {
    query: String,
  },
  SkillRun {
    query: String,
    dry_run: bool,
    max_disturbance: Option<DisturbanceClass>,
    overrides: BTreeMap<String, String>,
  },
}

pub fn parse_cli(arguments: &[String]) -> AuvResult<CliCommand> {
  if arguments.is_empty() {
    return Ok(CliCommand::Help);
  }

  match arguments[0].as_str() {
    "help" | "--help" | "-h" => Ok(CliCommand::Help),
    "list-commands" => Ok(CliCommand::ListCommands),
    "list-drivers" => Ok(CliCommand::ListDrivers),
    "inspect" => parse_inspect(arguments),
    "invoke" => parse_invoke(arguments),
    "skill" => parse_skill(arguments),
    other => Err(format!(
      "unknown subcommand {other}; use `help` to see supported commands"
    )),
  }
}

pub fn help_text() -> String {
  String::from(
    "\
auv-cli prototype

USAGE
  auv-cli list-commands
  auv-cli list-drivers
  auv-cli invoke <command-id> [--target <application-id>] [--label <text>]
  auv-cli inspect <run-id>
  auv-cli skill list
  auv-cli skill show <skill-id-or-path>
  auv-cli skill run <skill-id-or-path> [--dry-run] [--max-disturbance <class>] [--set key=value]

NOTES
  - Names are provisional and reflect the current phase-0/1 runtime skeleton.
  - The CLI is a thin frontend over the library runtime in src/lib.rs.
  - `debug.captureScreen`, `debug.probeDisplays`, `debug.projectScreenshotPoint`, `debug.identifyPoint`, `debug.probeCoordinateReadiness`, `debug.observeWindows`, `debug.observeWindowTree`, `debug.probePermissions`, `debug.focusTextInput`, `debug.pressButton`, `debug.clickPoint`, and `debug.scrollPoint` are the current desktop donor entrypoints.
  - `debug.observeWindowTree`, `debug.focusTextInput`, and `debug.pressButton` accept `--reveal_shortcut cmd+f`-style hints when an app hides the target UI until a keyboard shortcut reveals it.
  - `--reveal_settle_ms <millis>` can be used to make the reveal step explicit instead of depending on hard-coded timing assumptions.
  - `debug.typeText` supports `--replace_existing true`, `--submit_key return`, and `--submit_settle_ms 800` for repeatable text-entry flows.
  - `debug.pressKey` supports both special keys like `Return` and shortcuts like `cmd+f`, with optional `--settle_ms`.
  - `debug.clickWindowPoint` accepts either `--offset_x/--offset_y` or `--relative_x/--relative_y` against the target window bounds.
  - `debug.findScreenText` and `debug.clickScreenText` use macOS Vision OCR over a captured screenshot and operate in screenshot-pixel anchors projected back to logical points.
  - `debug.findImageText` runs the same OCR matching over an existing image artifact, which is useful for verifying captured evidence without recapturing the live desktop.
  - `debug.clickScreenText` supports `--match_index` and `--click_count` when the query resolves to multiple OCR anchors.
  - `skill run` is the product-facing recipe entrypoint: it resolves a recipe manifest from `recipes/`, validates disturbance policy, replays steps through the shared runtime, and carries step artifact paths into later verification steps.
",
  )
}

fn parse_inspect(arguments: &[String]) -> AuvResult<CliCommand> {
  if arguments.len() != 2 {
    return Err("usage: auv-cli inspect <run-id>".to_string());
  }

  Ok(CliCommand::Inspect {
    run_id: arguments[1].clone(),
  })
}

fn parse_invoke(arguments: &[String]) -> AuvResult<CliCommand> {
  if arguments.len() < 2 {
    return Err(
      "usage: auv-cli invoke <command-id> [--target <application-id>] [--label <text>]".to_string(),
    );
  }

  let command_id = arguments[1].clone();
  let mut target = ExecutionTarget::default();
  let mut inputs = BTreeMap::new();
  let mut index = 2;

  while index < arguments.len() {
    let argument = &arguments[index];
    if !argument.starts_with("--") {
      return Err(format!("unexpected positional argument {argument}"));
    }
    if index + 1 >= arguments.len() {
      return Err(format!("flag {argument} requires a value"));
    }

    let value = arguments[index + 1].clone();
    match argument.as_str() {
      "--target" => {
        target.application_id = Some(value);
      }
      "--label" => {
        inputs.insert("label".to_string(), value);
      }
      other => {
        let key = other.trim_start_matches("--");
        inputs.insert(key.to_string(), value);
      }
    }

    index += 2;
  }

  Ok(CliCommand::Invoke(InvokeRequest {
    command_id,
    target,
    inputs,
  }))
}

fn parse_skill(arguments: &[String]) -> AuvResult<CliCommand> {
  if arguments.len() < 2 {
    return Err("usage: auv-cli skill <list|show|run> ...".to_string());
  }

  match arguments[1].as_str() {
    "list" => {
      if arguments.len() != 2 {
        return Err("usage: auv-cli skill list".to_string());
      }
      Ok(CliCommand::SkillList)
    }
    "show" => {
      if arguments.len() != 3 {
        return Err("usage: auv-cli skill show <skill-id-or-path>".to_string());
      }
      Ok(CliCommand::SkillShow {
        query: arguments[2].clone(),
      })
    }
    "run" => parse_skill_run(arguments),
    other => Err(format!(
      "unknown skill subcommand {other}; use `auv-cli skill list` to inspect the current catalog"
    )),
  }
}

fn parse_skill_run(arguments: &[String]) -> AuvResult<CliCommand> {
  if arguments.len() < 3 {
    return Err(
      "usage: auv-cli skill run <skill-id-or-path> [--dry-run] [--max-disturbance <class>] [--set key=value]".to_string(),
    );
  }

  let query = arguments[2].clone();
  let mut dry_run = false;
  let mut max_disturbance = None;
  let mut overrides = BTreeMap::new();
  let mut index = 3;

  while index < arguments.len() {
    match arguments[index].as_str() {
      "--dry-run" => {
        dry_run = true;
        index += 1;
      }
      "--max-disturbance" => {
        if index + 1 >= arguments.len() {
          return Err("--max-disturbance requires a value".to_string());
        }
        max_disturbance = Some(DisturbanceClass::parse(&arguments[index + 1])?);
        index += 2;
      }
      "--set" => {
        if index + 1 >= arguments.len() {
          return Err("--set requires key=value".to_string());
        }
        let raw = &arguments[index + 1];
        let Some((key, value)) = raw.split_once('=') else {
          return Err(format!("invalid --set value {raw:?}; expected key=value"));
        };
        if key.trim().is_empty() {
          return Err(format!("invalid --set value {raw:?}; missing key"));
        }
        overrides.insert(key.trim().to_string(), value.to_string());
        index += 2;
      }
      other => {
        return Err(format!("unexpected skill-run argument {other}"));
      }
    }
  }

  Ok(CliCommand::SkillRun {
    query,
    dry_run,
    max_disturbance,
    overrides,
  })
}
