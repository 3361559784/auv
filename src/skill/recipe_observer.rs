use crate::model::{DisturbanceClass, InvokeRequest, InvokeResult};
use crate::trace::RunId;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecipeStartedReport {
  pub recipe_id: String,
  pub version: String,
  pub objective: String,
  pub target: String,
  pub max_disturbance: DisturbanceClass,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecipeStepReport {
  pub index: usize,
  pub total: usize,
  pub max_disturbance: DisturbanceClass,
  pub disturbance_classes: String,
}

pub trait RecipeRunReporter {
  fn recipe_started(&self, _event: RecipeStartedReport) {}

  fn step_started(&self, _step_id: &str, _request: &InvokeRequest, _step: RecipeStepReport) {}

  fn step_finished(&self, _step_id: &str, _result: &InvokeResult) {}

  fn recipe_finished(&self, _recipe_id: &str, _run_id: &RunId) {}
}

pub struct NoopRecipeRunReporter;

impl RecipeRunReporter for NoopRecipeRunReporter {}

pub struct ConsoleRecipeRunReporter;

impl RecipeRunReporter for ConsoleRecipeRunReporter {
  fn recipe_started(&self, event: RecipeStartedReport) {
    println!("skill: {}", event.recipe_id);
    println!("version: {}", event.version);
    println!("objective: {}", event.objective);
    println!("target: {}", event.target);
    println!("max disturbance: {}", event.max_disturbance.as_str());
  }

  fn step_started(&self, step_id: &str, request: &InvokeRequest, step: RecipeStepReport) {
    print_step_preview(
      step.index + 1,
      step.total,
      step_id,
      request,
      step.max_disturbance,
      &step.disturbance_classes,
    );
  }

  fn step_finished(&self, _step_id: &str, result: &InvokeResult) {
    print_invoke_result(result);
  }
}

fn print_step_preview(
  index: usize,
  total: usize,
  step_id: &str,
  request: &InvokeRequest,
  step_max: DisturbanceClass,
  step_classes: &str,
) {
  let mut command = vec![
    "auv-cli".to_string(),
    "invoke".to_string(),
    request.command_id.clone(),
  ];
  if let Some(target) = &request.target.application_id {
    command.push("--target".to_string());
    command.push(target.clone());
  }
  for (key, value) in &request.inputs {
    command.push(format!("--{key}"));
    command.push(value.clone());
  }
  println!(
    "[{index}/{total}] {step_id} (disturbance max={}; classes={step_classes}) -> {}",
    step_max.as_str(),
    command.join(" ")
  );
}

fn print_invoke_result(result: &InvokeResult) {
  println!("runId: {}", result.run_id);
  println!("status: {}", result.status.as_str());
  println!("output: {}", result.output_summary);
  for artifact in &result.artifact_paths {
    println!("artifact: {}", artifact.display());
  }
}
