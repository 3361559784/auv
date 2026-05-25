use crate::model::{AuvResult, DisturbanceClass};

use super::super::{SkillCaseMatrix, SkillManifest};
use super::manifest::validate_step_disturbance_against_command;

pub(crate) fn validate_case_matrix_manifest(matrix: &SkillCaseMatrix) -> AuvResult<()> {
  if matrix.skill_id.trim().is_empty() {
    return Err("case matrix skill_id must not be empty".to_string());
  }
  if matrix.version.trim().is_empty() {
    return Err(format!(
      "case matrix {} must declare a non-empty version",
      matrix.skill_id
    ));
  }
  semver::Version::parse(&matrix.version).map_err(|error| {
    format!(
      "case matrix {} has invalid version {}: {error}",
      matrix.skill_id, matrix.version
    )
  })?;
  if matrix.status.trim().is_empty() {
    return Err(format!(
      "case matrix {} must declare a non-empty status",
      matrix.skill_id
    ));
  }
  if matrix.cases.is_empty() {
    return Err(format!(
      "case matrix {} must declare at least one case",
      matrix.skill_id
    ));
  }

  let mut seen_case_ids = std::collections::BTreeSet::new();
  for (index, case) in matrix.cases.iter().enumerate() {
    let case_label = if case.case_id.trim().is_empty() {
      format!("case-{}", index + 1)
    } else {
      case.case_id.clone()
    };
    if case.case_id.trim().is_empty() {
      return Err(format!(
        "case matrix {} has a case with an empty case_id",
        matrix.skill_id
      ));
    }
    if !seen_case_ids.insert(case.case_id.clone()) {
      return Err(format!(
        "case matrix {} contains duplicate case_id {}",
        matrix.skill_id, case.case_id
      ));
    }
    if case.status.trim().is_empty() {
      return Err(format!(
        "case matrix {} case {} must declare a non-empty status",
        matrix.skill_id, case_label
      ));
    }
    if case.disturbance.trim().is_empty() {
      return Err(format!(
        "case matrix {} case {} must declare a non-empty disturbance",
        matrix.skill_id, case_label
      ));
    }
    DisturbanceClass::parse(&case.disturbance).map_err(|error| {
      format!(
        "case matrix {} case {} has invalid disturbance {}: {error}",
        matrix.skill_id, case_label, case.disturbance
      )
    })?;

    for key in case.inputs.keys() {
      if key.trim().is_empty() {
        return Err(format!(
          "case matrix {} case {} has an empty input key",
          matrix.skill_id, case_label
        ));
      }
    }
  }

  Ok(())
}

pub(crate) fn validate_case_matrix_against_skill(
  manifest: &SkillManifest,
  matrix: &SkillCaseMatrix,
) -> AuvResult<()> {
  if matrix.skill_id != manifest.recipe_id {
    return Err(format!(
      "case matrix {} does not match skill {}",
      matrix.skill_id, manifest.recipe_id
    ));
  }

  let recipe_max = if manifest.disturbance_policy.max_disturbance.is_empty() {
    DisturbanceClass::Pointer
  } else {
    DisturbanceClass::parse(&manifest.disturbance_policy.max_disturbance).map_err(|error| {
      format!(
        "skill {} has invalid disturbance_policy.max_disturbance {}: {error}",
        manifest.recipe_id, manifest.disturbance_policy.max_disturbance
      )
    })?
  };

  for case in &matrix.cases {
    let case_disturbance = DisturbanceClass::parse(&case.disturbance).map_err(|error| {
      format!(
        "case matrix {} case {} has invalid disturbance {}: {error}",
        matrix.skill_id, case.case_id, case.disturbance
      )
    })?;
    if case_disturbance > recipe_max {
      return Err(format!(
        "case matrix {} case {} uses disturbance {} above skill max {}",
        matrix.skill_id,
        case.case_id,
        case_disturbance.as_str(),
        recipe_max.as_str()
      ));
    }

    for key in case.inputs.keys() {
      if !manifest.inputs.contains_key(key) {
        return Err(format!(
          "case matrix {} case {} references unknown input {}",
          matrix.skill_id, case.case_id, key
        ));
      }
    }

    for (input_key, spec) in &manifest.inputs {
      if spec.default.is_none() && !case.inputs.contains_key(input_key) {
        return Err(format!(
          "case matrix {} case {} is missing required input {}",
          matrix.skill_id, case.case_id, input_key
        ));
      }
    }
  }

  validate_smart_press_case_status(manifest, matrix)?;

  Ok(())
}

pub(crate) fn validate_case_matrix_against_skill_with_commands(
  manifest: &SkillManifest,
  matrix: &SkillCaseMatrix,
  command_catalog: &[crate::model::CommandSpec],
) -> AuvResult<()> {
  validate_case_matrix_against_skill(manifest, matrix)?;

  for step in &manifest.steps {
    let Some(command) = command_catalog
      .iter()
      .find(|command| command.id == step.command_id)
    else {
      return Err(format!(
        "skill {} step {} references unknown command_id {}",
        manifest.recipe_id,
        if step.id.trim().is_empty() {
          &step.command_id
        } else {
          &step.id
        },
        step.command_id
      ));
    };
    let step_label = if step.id.trim().is_empty() {
      step.command_id.clone()
    } else {
      step.id.clone()
    };
    validate_step_disturbance_against_command(&manifest.recipe_id, &step_label, step, command)?;
  }

  Ok(())
}

/// Phase 3 Rule 2 from
/// `docs/ai/references/2026-05-22-phase-3-mainline-acceptance.md`:
/// any recipe that uses `debug.smartPress` in any step cannot have
/// `status == "validated"` cases. The promotion path for a smart-
/// press recipe is candidate -> evidence -> spawn a non-smart child
/// fixed to whichever strategy actually works.
fn validate_smart_press_case_status(
  manifest: &SkillManifest,
  matrix: &SkillCaseMatrix,
) -> AuvResult<()> {
  let uses_smart_press = manifest
    .steps
    .iter()
    .any(|step| step.command_id == "debug.smartPress");
  if !uses_smart_press {
    return Ok(());
  }
  for case in &matrix.cases {
    if case.status.trim() == "validated" {
      return Err(format!(
        "case matrix {} case {} is status=validated, but recipe {} uses debug.smartPress (rule 2 — smart-press recipes cannot host validated cases; promote to a non-smart child recipe instead). \
         See docs/ai/references/2026-05-22-phase-3-mainline-acceptance.md.",
        matrix.skill_id, case.case_id, manifest.recipe_id,
      ));
    }
  }
  Ok(())
}
