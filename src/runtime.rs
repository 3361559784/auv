use std::path::PathBuf;

use crate::catalog::CommandCatalog;
use crate::driver::DriverRegistry;
use crate::model::{
  ArtifactRecord,
  AuvResult,
  DriverCall,
  DriverDescriptor,
  EventRecord,
  InvokeRequest,
  InvokeResult,
  RunRecord,
  RunStatus,
  new_run_id,
  now_millis,
};
use crate::store::LocalStore;

pub struct Runtime {
  project_root: PathBuf,
  commands: CommandCatalog,
  drivers: DriverRegistry,
  store: LocalStore,
}

impl Runtime {
  pub fn new(
    project_root: PathBuf,
    commands: CommandCatalog,
    drivers: DriverRegistry,
    store: LocalStore,
  ) -> Self {
    Self {
      project_root,
      commands,
      drivers,
      store,
    }
  }

  pub fn list_commands(&self) -> &[crate::model::CommandSpec] {
    self.commands.all()
  }

  pub fn list_drivers(&self) -> Vec<DriverDescriptor> {
    self.drivers.descriptors()
  }

  pub fn inspect(&self, run_id: &str) -> AuvResult<String> {
    self.store.render_inspection(run_id)
  }

  pub fn invoke(&self, request: InvokeRequest) -> AuvResult<InvokeResult> {
    let command = self.commands.resolve(&request.command_id).ok_or_else(|| {
      format!(
        "unknown command {}; use `list-commands` to see available entries",
        request.command_id
      )
    })?;
    let driver = self.drivers.get(command.driver_id).ok_or_else(|| {
      format!(
        "command {} resolved to missing driver {}",
        command.id, command.driver_id
      )
    })?;

    let run_id = new_run_id();
    let mut run = RunRecord {
      run_id: run_id.clone(),
      command_id: command.id.to_string(),
      driver_id: command.driver_id.to_string(),
      operation: command.operation.to_string(),
      target_application_id: request.target.application_id.clone(),
      runtime_version: env!("CARGO_PKG_VERSION").to_string(),
      started_at_millis: now_millis(),
      finished_at_millis: None,
      status: RunStatus::Failed,
      inputs: request.inputs.clone(),
      output_summary: String::new(),
      events: Vec::new(),
      artifacts: Vec::new(),
    };

    push_event(
      &mut run.events,
      "run.created",
      format!("implicit run created for command {}", command.id),
    );
    push_event(
      &mut run.events,
      "command.resolved",
      format!(
        "resolved {} -> {}.{}",
        command.id, command.driver_id, command.operation
      ),
    );

    let call = DriverCall {
      operation: command.operation.to_string(),
      target: request.target,
      inputs: request.inputs,
      working_directory: self.project_root.clone(),
    };

    push_event(
      &mut run.events,
      "driver.invoke",
      format!("invoking {}.{}", command.driver_id, command.operation),
    );

    let mut failure_message = None;

    match driver.invoke(&call) {
      Ok(response) => {
        if let Some(backend) = &response.backend {
          push_event(
            &mut run.events,
            "driver.backend",
            format!("backend={backend}"),
          );
        }

        for note in &response.notes {
          push_event(&mut run.events, "driver.note", note.clone());
        }

        let mut persisted_artifacts = Vec::new();
        for (index, artifact) in response.artifacts.into_iter().enumerate() {
          let stored_artifact = self.store.stage_artifact(&run.run_id, index, artifact)?;
          push_event(
            &mut run.events,
            "artifact.captured",
            render_artifact_event(&stored_artifact),
          );
          persisted_artifacts.push(stored_artifact);
        }

        run.status = RunStatus::Completed;
        run.output_summary = response.summary.clone();
        run.artifacts = persisted_artifacts;
        push_event(
          &mut run.events,
          "run.completed",
          response.summary,
        );
      }
      Err(error) => {
        run.status = RunStatus::Failed;
        run.output_summary = format!(
          "Driver invocation failed after run creation. Inspect {} for the recorded trace.",
          run.run_id
        );
        failure_message = Some(error.clone());
        push_event(&mut run.events, "driver.failed", error);
      }
    }

    run.finished_at_millis = Some(now_millis());
    self.store.persist_run(&run)?;

    let artifact_paths = run
      .artifacts
      .iter()
      .map(|artifact| artifact.path.clone())
      .collect::<Vec<_>>();
    let run_id = run.run_id.clone();
    let status = run.status.clone();
    let output_summary = run.output_summary.clone();

    Ok(InvokeResult {
      run_id,
      status,
      output_summary,
      artifact_paths,
      failure_message,
    })
  }
}

fn push_event(events: &mut Vec<EventRecord>, kind: &str, message: String) {
  events.push(EventRecord {
    at_millis: now_millis(),
    kind: kind.to_string(),
    message,
  });
}

fn render_artifact_event(artifact: &ArtifactRecord) -> String {
  let note = artifact.note.clone().unwrap_or_else(|| "n/a".to_string());
  format!(
    "{} kind={} path={} note={}",
    artifact.id,
    artifact.kind,
    artifact.path.display(),
    note
  )
}
