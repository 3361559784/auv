# AX Copilot Vertical Migration Manifest

Status: planning-only manifest for the future externalization target `moeru-ai/auv-ax-copilot`.

This manifest exists to freeze the quarantine boundary. It does not authorize code movement by itself.

## Dependency Direction

Future `auv-ax-copilot` depends on AUV core contracts/runtime.

It must reuse AUV core types and stores:

- `ArtifactRef`
- `RecognitionResult`
- `VerificationResult`
- run store / trace / inspect read model
- `candidate_promotion` and `candidate_promotion_recording`

It must not copy those into a second contract universe.

## Move Later: Archived Vertical Orchestration

The future external repo should own the archived vertical orchestration layer:

- `src/candidate_action_command.rs`
- `src/candidate_action_decision.rs`
- `candidate-action` CLI branch and tests
- archived TextEdit/AX copilot docs and evidence
- proposer-only action path scoped to `candidate-action`

## Keep In AUV Core

These stay in AUV core:

- `src/candidate_promotion.rs`
- `src/candidate_promotion_recording.rs`
- `src/runtime.rs`
- run store / trace / inspect / read-side lineage
- `ArtifactRef`, `RecognitionResult`, `VerificationResult`
- generic local human approval capability itself:
  - `crates/auv-driver-macos/src/native/auth.rs`
  - `crates/auv-driver-macos/native/swift/Sources/AuvMacosNative/Auth.swift`
  - `crates/auv-driver-macos/src/native/binding.rs`

The local human approval primitive can be reused by future AUV runtime paths beyond the archived copilot demo.

## Human Gesture Boundary By Call Path

Externalize these `candidate-action` orchestration/demo call paths:

- `execute_candidate_action_command`
  - archived top-level orchestration around observe -> promote -> consent -> decide -> execute
- `request_human_gesture_approval`
  - archived command-specific prompting and event recording
- `human_gesture_prompt_reason`
  - archived prompt text tied to `candidate-action` promotion/execution ids
- `human_gesture_granted_by`
  - archived `candidate-action`-specific granted-by labeling
- `human_gesture_scope_note`
  - archived command-specific scope binding notes
- `human_gesture_evidence_note`
  - archived command-specific evidence notes
- `promotion_permission_for_request`
  - only the branch that binds human gesture approval into `candidate-action` promotion consent
- `execution_consent_for_request`
  - only the branch that binds human gesture approval into `candidate-action` execution consent
- `human_gesture_execution_consent`
  - archived execution-consent assembly for `candidate-action`

Keep these core/shared pieces in AUV:

- the native approval request function `request_human_approval`
- shared consent provenance/grade types
- execution-consent validation rules in the shared action seam
- shared artifact storage and lineage readers

## Freeze Rules While It Stays In Main Repo

Until externalization happens:

- do not add a third action class
- do not extend TextEdit proof coverage
- do not continue model proposer integration into `candidate-action`
- do not add product UX polish around archived consent/readiness/demo paths
- do not treat archived copilot runs as AUV core progress
