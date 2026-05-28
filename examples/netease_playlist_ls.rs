use std::path::PathBuf;

const DEFAULT_APP_ID: &str = "com.netease.163music";

#[derive(Clone, Debug, PartialEq)]
struct Inputs {
  app_id: String,
  json_out: Option<PathBuf>,
  max_pages: usize,
  max_scrolls: usize,
  scroll_amount: f64,
  sidebar_region: Option<RatioRegion>,
  print_json: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct RatioRegion {
  x: f64,
  y: f64,
  width: f64,
  height: f64,
}

impl RatioRegion {
  const fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
    Self {
      x,
      y,
      width,
      height,
    }
  }
}

fn main() {
  if let Err(error) = run() {
    eprintln!("{error}");
    std::process::exit(1);
  }
}

fn run() -> Result<(), String> {
  let _inputs = parse_inputs(std::env::args().skip(1).collect())?;
  Err("live implementation is added in later tasks".to_string())
}

fn parse_inputs(args: Vec<String>) -> Result<Inputs, String> {
  let mut inputs = Inputs {
    app_id: DEFAULT_APP_ID.to_string(),
    json_out: None,
    max_pages: 24,
    max_scrolls: 48,
    scroll_amount: 6.0,
    sidebar_region: None,
    print_json: false,
  };

  let mut args = args.into_iter();
  while let Some(arg) = args.next() {
    match arg.as_str() {
      "--app-id" => {
        inputs.app_id = next_value(&mut args, "--app-id")?;
      }
      "--json-out" => {
        inputs.json_out = Some(PathBuf::from(next_value(&mut args, "--json-out")?));
      }
      "--max-pages" => {
        inputs.max_pages = parse_usize("--max-pages", next_value(&mut args, "--max-pages")?)?;
        if inputs.max_pages == 0 {
          return Err("--max-pages must be greater than 0".to_string());
        }
      }
      "--max-scrolls" => {
        inputs.max_scrolls = parse_usize("--max-scrolls", next_value(&mut args, "--max-scrolls")?)?;
        if inputs.max_scrolls == 0 {
          return Err("--max-scrolls must be greater than 0".to_string());
        }
      }
      "--scroll-amount" => {
        inputs.scroll_amount =
          parse_f64("--scroll-amount", next_value(&mut args, "--scroll-amount")?)?;
        if !inputs.scroll_amount.is_finite() || inputs.scroll_amount <= 0.0 {
          return Err("--scroll-amount must be greater than 0".to_string());
        }
      }
      "--sidebar-region" => {
        inputs.sidebar_region = Some(parse_ratio_region(next_value(
          &mut args,
          "--sidebar-region",
        )?)?);
      }
      "--print-json" => {
        inputs.print_json = true;
      }
      other => return Err(format!("unknown argument {other}")),
    }
  }

  Ok(inputs)
}

fn next_value(args: &mut impl Iterator<Item = String>, flag: &str) -> Result<String, String> {
  args
    .next()
    .ok_or_else(|| format!("{flag} requires a value"))
}

fn parse_usize(flag: &str, value: String) -> Result<usize, String> {
  value
    .parse()
    .map_err(|_| format!("{flag} expects a positive integer"))
}

fn parse_f64(flag: &str, value: String) -> Result<f64, String> {
  value
    .parse()
    .map_err(|_| format!("{flag} expects a number"))
}

fn parse_ratio_region(value: String) -> Result<RatioRegion, String> {
  let parts = value
    .split(',')
    .map(str::trim)
    .map(|part| {
      part
        .parse::<f64>()
        .map_err(|_| "--sidebar-region expects x,y,width,height".to_string())
    })
    .collect::<Result<Vec<_>, _>>()?;

  if parts.len() != 4 {
    return Err("--sidebar-region expects x,y,width,height".to_string());
  }

  if parts.iter().any(|part| !part.is_finite()) {
    return Err("--sidebar-region expects finite x,y,width,height".to_string());
  }

  if parts[2] <= 0.0 || parts[3] <= 0.0 {
    return Err("--sidebar-region width and height must be greater than 0".to_string());
  }

  Ok(RatioRegion::new(parts[0], parts[1], parts[2], parts[3]))
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn parse_inputs_uses_safe_defaults() {
    let inputs = parse_inputs(Vec::new()).expect("defaults should parse");

    assert_eq!(inputs.app_id, DEFAULT_APP_ID);
    assert_eq!(inputs.json_out, None);
    assert_eq!(inputs.max_pages, 24);
    assert_eq!(inputs.max_scrolls, 48);
    assert_eq!(inputs.scroll_amount, 6.0);
    assert_eq!(inputs.sidebar_region, None);
    assert!(!inputs.print_json);
  }

  #[test]
  fn parse_inputs_accepts_json_and_scan_options() {
    let inputs = parse_inputs(vec![
      "--app-id".to_string(),
      "com.example.music".to_string(),
      "--json-out".to_string(),
      "/tmp/scan.json".to_string(),
      "--max-pages".to_string(),
      "7".to_string(),
      "--max-scrolls".to_string(),
      "9".to_string(),
      "--scroll-amount".to_string(),
      "3.5".to_string(),
      "--sidebar-region".to_string(),
      "0.0,0.1,0.25,0.8".to_string(),
      "--print-json".to_string(),
    ])
    .expect("arguments should parse");

    assert_eq!(inputs.app_id, "com.example.music");
    assert_eq!(inputs.json_out, Some(PathBuf::from("/tmp/scan.json")));
    assert_eq!(inputs.max_pages, 7);
    assert_eq!(inputs.max_scrolls, 9);
    assert_eq!(inputs.scroll_amount, 3.5);
    assert_eq!(
      inputs.sidebar_region,
      Some(RatioRegion::new(0.0, 0.1, 0.25, 0.8))
    );
    assert!(inputs.print_json);
  }

  #[test]
  fn parse_inputs_rejects_unknown_flag() {
    let error = parse_inputs(vec!["--bogus".to_string()]).expect_err("unknown flag should fail");
    assert!(error.contains("unknown argument --bogus"));
  }

  #[test]
  fn parse_inputs_rejects_non_finite_scroll_amount() {
    let error = parse_inputs(vec!["--scroll-amount".to_string(), "NaN".to_string()])
      .expect_err("non-finite scroll amount should fail");

    assert!(error.contains("--scroll-amount must be greater than 0"));
  }

  #[test]
  fn parse_inputs_rejects_non_finite_sidebar_region_component() {
    let error = parse_inputs(vec![
      "--sidebar-region".to_string(),
      "0.0,NaN,0.25,0.8".to_string(),
    ])
    .expect_err("non-finite sidebar region component should fail");

    assert!(error.contains("--sidebar-region expects finite x,y,width,height"));
  }
}
