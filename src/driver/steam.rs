use std::collections::BTreeMap;

use auv_steam::{LibraryQuery, build_library_ls_json_output, query_local_library_apps};

use super::Driver;
use crate::driver::macos::support::artifacts::build_text_artifact;
use crate::model::{AuvResult, DriverCall, DriverDescriptor, DriverResponse};

pub(crate) struct SteamLocalDriver;

impl Driver for SteamLocalDriver {
  fn descriptor(&self) -> DriverDescriptor {
    DriverDescriptor {
      id: "steam.local",
      summary: "Local Steam appmanifest reader that records installed-library evidence through auv-steam.",
      capabilities: &["steam.library.list"],
      donor_boundary: "AUV-native local read driver that reuses auv-steam library queries instead of routing real Steam reads through fixture.observe.",
    }
  }

  fn invoke(&self, call: &DriverCall) -> AuvResult<DriverResponse> {
    if call.operation != "steam_library_list" {
      return Err(format!(
        "driver steam.local does not support operation {}",
        call.operation
      ));
    }

    steam_library_list()
  }
}

fn steam_library_list() -> AuvResult<DriverResponse> {
  // TODO(steam-library-query-inputs-v1): invoke-side filtering is deferred until an
  // owner-approved slice defines how DriverCall.inputs projects into LibraryQuery.
  let query = LibraryQuery::default();
  let result = query_local_library_apps(query).map_err(|diagnostic| {
    format!(
      "steam.library.list.v0 failed: [{}] {}",
      diagnostic.code, diagnostic.message
    )
  })?;

  let output = build_library_ls_json_output(&result);
  let json = serde_json::to_string_pretty(&output)
    .map_err(|error| format!("failed to serialize steam library result: {error}"))?;
  let report = build_text_artifact(
    "steam-library-list",
    "json",
    "steam-library-list",
    format!("{json}\n"),
    "Structured installed Steam library listing from auv-steam.",
  )?;

  let mut signals = BTreeMap::new();
  signals.insert(
    "steam.library.source".to_string(),
    result.resolved_scope.source.clone(),
  );
  signals.insert("steam.library.status".to_string(), "installed".to_string());
  signals.insert(
    "steam.library.app_count".to_string(),
    result.apps.len().to_string(),
  );
  if let Some(first) = result.apps.first() {
    signals.insert(
      "steam.library.first_app.name".to_string(),
      first.name.clone(),
    );
    signals.insert(
      "steam.library.first_app.appid".to_string(),
      first.appid.to_string(),
    );
  }

  Ok(DriverResponse {
    summary: format!(
      "Listed {} installed Steam app(s) through auv-steam local appmanifest grounding.",
      result.apps.len()
    ),
    backend: Some("steam.local_appmanifest.library-list".to_string()),
    signals,
    notes: vec![
      format!("resolvedSource={}", result.resolved_scope.source),
      format!("appCount={}", result.apps.len()),
      "Capability implemented through auv-steam library reuse, not duplicated Steam parsing."
        .to_string(),
    ],
    artifacts: vec![report],
  })
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::model::{DriverRunContext, ExecutionTarget};
  use std::collections::BTreeMap;
  use std::path::PathBuf;

  #[test]
  fn steam_local_driver_rejects_unknown_operation() {
    let driver = SteamLocalDriver;
    let call = DriverCall {
      operation: "observe_fixture_scene".to_string(),
      target: ExecutionTarget::default(),
      inputs: BTreeMap::new(),
      working_directory: PathBuf::from("."),
      run_context: DriverRunContext::default(),
    };

    let error = driver
      .invoke(&call)
      .expect_err("steam.local should reject unrelated operations");
    assert!(error.contains("driver steam.local does not support operation observe_fixture_scene"));
  }
}
