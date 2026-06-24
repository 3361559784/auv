# 2026-06-24 Minecraft MC-6 canonical clean-rebuild fail record

Date: 2026-06-24

Classification label: `substrate research`.

Purpose: record the result of rerunning the MC-6 sample-build and eval chain from the canonical source lineage in a clean local workspace. This note does not reopen or close MC-6 by itself. It exists to prove that the historical fail state is reproducible without relying on old `/tmp/auv-mc67-live*` drift paths.

Historical boundary:

- this note is preserved as the clean fail record for the older three-run
  canonical lineage
- it is not the current closure verdict after the reopened 2026-06-24 9-run
  live sweep passed MC-6 under the dual-gate contract

## Inputs pinned for this rebuild

The rebuild reused the canonical source lineage recorded in `2026-06-24-minecraft-mc6-canonical-staging-artifact.md`:

- source runs:
  - `run_1781881896971_19131_0` (`rich`)
  - `run_1781881897582_19207_0` (`flat_color`)
  - `run_1781881898175_19213_0` (`repetitive`)
- canonical target evidence:
  - target block position: `511,73,728`
  - target block id: `minecraft:oak_button`
- accepted bundle-export artifacts:
  - `.auv/runs/run_1781881898217_19217_0/artifacts/artifact_0001_minecraft-spatial-bundle-run.json`
  - `.auv/runs/run_1781881898256_19232_0/artifacts/artifact_0001_minecraft-spatial-bundle-run.json`
  - `.auv/runs/run_1781881898290_19247_0/artifacts/artifact_0001_minecraft-spatial-bundle-run.json`

## Clean local rebuild workspace

The clean workspace for this rerun was:

- `.tmp/mc6-rebuild-clean/`

Artifacts written there:

- `.tmp/mc6-rebuild-clean/canonical-inputs.json`
- `.tmp/mc6-rebuild-clean/rehydrated-bundles.json`
- `.tmp/mc6-rebuild-clean/mc6-canonical-samples.json`
- `.tmp/mc6-rebuild-clean/eval/texture_sweep_report.json`

## What was rerun

The first direct attempt to rebuild from the recorded bundle-manifest artifacts failed because the recorded bundle-run JSONs describe bundle contents but do not themselves carry the materialized `screenshots/` and `spatial_frames/` directories. For the clean rerun, those bundle contents were rehydrated into `.tmp/mc6-rebuild-clean/` from the canonical source runs, and then the standard MC-6 chain was rerun:

1. `minecraft build-texture-sweep-samples`
2. `minecraft eval-texture-sweep --require-real-source`

The rerun completed successfully and produced a new sample JSON plus a new report JSON under the clean workspace.

## Reproduced result

The clean rebuild reproduced the same substantive verdict as the current canonical report:

- all three expected profiles are present
- `sample_count = 1` for each profile
- `duration_seconds = 0.0` for each profile
- `pose_error_p95_px = 0.0` for each profile
- `min_occlusion_iou = 1.0` for each profile
- `noise_refusal_exercised = false`
- overall `passed = false`

Per-profile reading from the clean report:

| profile     | samples | duration (s) | pose p95 (px) | min IoU | refusal count | pass |
|-------------|---------|--------------|---------------|---------|---------------|------|
| rich        | 1       | 0.0          | 0.0           | 1.0     | 0             | FAIL |
| flat_color  | 1       | 0.0          | 0.0           | 1.0     | 0             | FAIL |
| repetitive  | 1       | 0.0          | 0.0           | 1.0     | 0             | FAIL |

## What this means

This rerun changes one important thing and leaves one important thing unchanged.

What it changes:

- The fail state is now reproduced from a clean main-repo workspace instead of depending on old `/tmp/auv-mc67-live*` paths.
- The current MC-6 evidence chain is therefore canonical, auditable, and locally reproducible.

What it does not change:

- MC-6 is still not numerically closed.
- The remaining blocker is not provenance or tool-chain drift; it is insufficient measurement coverage.

The current missing evidence is still exactly the same gate-shaped gap:

- more than one measured sample per profile
- real in-game duration approaching or exceeding `30.0 s` per profile
- at least one exercised refusal/noise sample captured into the chain

## Current status

As of this clean rebuild, MC-6 should be read as:

- source lineage: canonical
- local rebuild chain: reproducible
- numerical gate: still failing

That means the next meaningful step was not another rebuild of the same three
single-frame inputs. The next meaningful step was a new live sweep that records
enough multi-frame duration and at least one refusal/noise sample to satisfy
the pre-committed MC-6 thresholds.

That subsequent live sweep has now landed. So read this document narrowly: it
explains why the old canonical three-run lineage fails, not the current MC-6
closure state.
