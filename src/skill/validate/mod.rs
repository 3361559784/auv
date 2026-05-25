//! Skill manifest + case matrix validation.
//!
//! Performs static checks over skill recipes, inline scroll-scan hook
//! manifests, and case matrices. This module is the validation facade; concrete
//! rule groups live in sibling modules.

mod case_matrix;
mod inline_hook;
mod manifest;

pub(crate) use case_matrix::{
  validate_case_matrix_against_skill, validate_case_matrix_against_skill_with_commands,
  validate_case_matrix_manifest,
};
pub(crate) use inline_hook::build_inline_scan_hook_manifest;
pub(crate) use manifest::{
  parse_step_max, validate_disturbance_policy, validate_skill_manifest,
  validate_skill_manifest_with_commands,
};
