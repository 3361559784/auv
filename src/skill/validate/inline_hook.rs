use std::collections::BTreeMap;

use crate::model::AuvResult;

use super::super::{SkillInlineHook, SkillInvocation, SkillManifest, SkillVerification};
use super::manifest::validate_skill_manifest_with_commands;

const SCROLL_SCAN_INLINE_HOOK_STAGES: &[&str] = &[
  "per_page_after_observe",
  "per_list_item_candidate",
  "on_stop_candidate",
];

const SCROLL_SCAN_HOOK_RETURN_SCHEMA: &str = "auv.scan.hook_decision.v0";

pub(crate) fn build_inline_scan_hook_manifest(
  parent: &SkillManifest,
  hook_name: &str,
) -> AuvResult<Option<SkillManifest>> {
  let Some(hook) = parent.hooks.get(hook_name) else {
    return Ok(None);
  };
  Ok(Some(synthesize_inline_scan_hook_manifest(
    parent, hook_name, hook,
  )?))
}

fn synthesize_inline_scan_hook_manifest(
  parent: &SkillManifest,
  hook_name: &str,
  hook: &SkillInlineHook,
) -> AuvResult<SkillManifest> {
  validate_inline_scan_hook_contract(&parent.recipe_id, hook_name, hook)?;
  Ok(SkillManifest {
    recipe_id: format!("{}.hook.{}", parent.recipe_id, hook_name),
    version: parent.version.clone(),
    status: "inline-sub-recipe".to_string(),
    platform: parent.platform.clone(),
    target_app: parent.target_app.clone(),
    strategy: parent.strategy.clone(),
    invocation: SkillInvocation {
      kind: "sub_recipe".to_string(),
      host: "scroll_scan".to_string(),
      stage: hook_name.to_string(),
      context_schema: hook.input_schema.clone(),
      return_schema: hook.return_schema.clone(),
    },
    objective: format!(
      "Inline scroll-scan hook {hook_name} synthesized from {}",
      parent.recipe_id
    ),
    inputs: BTreeMap::new(),
    preconditions: vec![format!(
      "Synthetic inline hook derived from parent recipe {}.",
      parent.recipe_id
    )],
    disturbance_policy: parent.disturbance_policy.clone(),
    steps: hook.steps.clone(),
    verification: SkillVerification {
      expected_signals: vec!["last.scan.hook.decision".to_string()],
      success_criteria: vec![format!(
        "Inline hook {hook_name} may emit last.scan.hook.* signals for scroll-scan orchestration."
      )],
      non_goals: vec![
        "This synthetic inline hook manifest exists only to reuse the shared recipe step runner."
          .to_string(),
      ],
    },
    hooks: BTreeMap::new(),
    known_limits: BTreeMap::from([(
      "context".to_string(),
      "input_schema is explicit, but current scroll-scan hook execution still injects scalar scan.* overrides rather than one typed context object.".to_string(),
    )]),
  })
}

pub(super) fn validate_skill_inline_hooks(
  manifest: &SkillManifest,
  command_catalog: &[crate::model::CommandSpec],
) -> AuvResult<()> {
  for hook_name in manifest.hooks.keys() {
    let inline_manifest = build_inline_scan_hook_manifest(manifest, hook_name)?.expect(
      "hook key came from manifest.hooks; inline scan hook synthesis should always return Some",
    );
    validate_skill_manifest_with_commands(&inline_manifest, command_catalog).map_err(|error| {
      format!(
        "skill {} inline hook {} is invalid: {error}",
        manifest.recipe_id, hook_name
      )
    })?;
  }
  Ok(())
}

fn validate_inline_scan_hook_contract(
  recipe_id: &str,
  hook_name: &str,
  hook: &SkillInlineHook,
) -> AuvResult<()> {
  if !SCROLL_SCAN_INLINE_HOOK_STAGES.contains(&hook_name) {
    return Err(format!(
      "skill {} declares unsupported inline hook {}; allowed stages: {}",
      recipe_id,
      hook_name,
      SCROLL_SCAN_INLINE_HOOK_STAGES.join(", ")
    ));
  }
  if hook.input_schema.trim().is_empty() {
    return Err(format!(
      "skill {} inline hook {} must declare a non-empty input_schema",
      recipe_id, hook_name
    ));
  }
  if hook.return_schema.trim().is_empty() {
    return Err(format!(
      "skill {} inline hook {} must declare a non-empty return_schema",
      recipe_id, hook_name
    ));
  }
  if hook.return_schema != SCROLL_SCAN_HOOK_RETURN_SCHEMA {
    return Err(format!(
      "skill {} inline hook {} return_schema {} does not match required {}",
      recipe_id, hook_name, hook.return_schema, SCROLL_SCAN_HOOK_RETURN_SCHEMA
    ));
  }
  if hook.steps.is_empty() {
    return Err(format!(
      "skill {} inline hook {} must declare at least one step",
      recipe_id, hook_name
    ));
  }
  Ok(())
}
