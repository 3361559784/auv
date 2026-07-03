# AUV Scan S6b-1 Scene State Run-Read Text Bridge — Handoff

**Date:** 2026-07-03  
**Status:** landed (S6b-1)

## Role

S6b-1 connects S6a `SceneStateInspect` to root-crate `inspect_run` text output. It is a **text-only bridge** — not a viewer, not `inspect_server`, not a runtime producer, and not a durable product wire.

## Staging wire (provisional)

| Constant | Value |
|----------|-------|
| Artifact role | `scan-scene-state-input-v0` |
| `schema_version` | `scan-scene-state-input-v0` |

**NOTICE(s6b1):** This JSON shape is **provisional test-only staging**. It is **not** `scan-scene-state-v0`, must **not** be graduated to `TERMS_AND_CONCEPTS` or treated as a durable contract without an owner-approved slice.

Wire fields:

- `schema_version` (string)
- `frames` (`Vec<auv_scan::ScanFrame>`)
- `observations_by_frame` (`Vec<Vec<{ observation_id, label }>>`)
- `lifecycle_events` (optional lifecycle event array)

`ScanFrameBundle` assembly for read:

- `source_dir` = parent directory of the staged artifact JSON file (`store.artifact_file` path parent)
- `loaded_json_paths` = empty (inline frames only)
- **No** `verify_frame_image_dimensions` in S6b-1

## Multi-artifact policy

| Count of `scan-scene-state-input-v0` JSON artifacts | Outcome |
|------------------------------------------------------|---------|
| 0 | `Missing` |
| 1 | read + build inspect |
| >1 | `Unsupported { reason: "multiple scan-scene-state-input-v0 artifacts" }` |

## API (`src/scene_state_read.rs`)

| Symbol | Role |
|--------|------|
| `SCENE_STATE_INPUT_ARTIFACT_ROLE` | Artifact role constant |
| `SCENE_STATE_INPUT_SCHEMA_VERSION` | Wire schema gate |
| `SceneStateReadOutcome` | `Present` / `Missing` / `Unsupported { reason }` |
| `SceneStateReadError` | Store / IO failures only |
| `build_scene_state_inspect_for_run` | Scan run artifacts → S6a inspect |
| `format_scene_state_read_text` | Text projection for `inspect_run` |

Text semantics:

- Missing → `Scene state: missing scan-scene-state-input-v0 artifact`
- Unsupported → `Scene state: unsupported ({reason})`
- Present → `\nScene state:\n` + `auv_scan::format_scene_state_inspect_text`

## Inspect wiring (`src/inspect_scene_state.rs`)

`append_scene_state_text_from_run` mirrors `inspect_view_parser::append_view_parser_proof_text_from_run`.

`inspect_run` calls it **after** the view-parser append block (`src/inspect.rs`).

**Explicit non-goals for S6b-1:**

- No `run_read.rs` re-export
- No `Serialize` on inspect types
- No `inspect_server` / viewer changes

## Tests

Root crate `scene_state_read` tests:

- `build_scene_state_inspect_for_run_present` — 6 `[scene.*]` section markers
- `build_scene_state_inspect_for_run_missing`
- `build_scene_state_inspect_for_run_unsupported_bad_schema`
- `build_scene_state_inspect_for_run_unsupported_multiple_artifacts`
- `inspect_run_includes_scene_state_block`

Fixture: `crates/auv-scan/tests/fixtures/scan/scene/scene_stable_v0/manifest.json` + `produce_frames_from_fixture_dir` / `load_scan_frames_from_dir`.

## Deferred (S6b+)

| Item | Notes |
|------|-------|
| Runtime producer | Real command writes `scan-scene-state-input-v0` |
| `inspect_server` JSON fields | Viewer consumption |
| Bundle-dir dual artifact | Large frame sets externalized |
| `scan-scene-state-v0` | L2 durable product wire |
| Artifact UI drill-down | Beyond text bridge |

## Related

- [S6a handoff](2026-07-03-auv-scan-s6a-scene-state-inspect-handoff.md)
- [S5a handoff](2026-07-03-auv-scan-s5-scene-state-handoff.md)
- Substrate **S6** (model backends) is **unrelated**
