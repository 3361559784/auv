# 2026-06-24 Minecraft MC-6 canonical staging artifact

Date: 2026-06-24

Classification label: `substrate research`.

Purpose: record the historical MC-6 staging lineage, the fresh reopened Gate 1
geometry lineage, and the final 2026-06-24 Gate 2 closure lineage. This note
does not rerun MC-6 by itself. It exists to stop future slices from drifting
back to old `/tmp/auv-mc67-live*` paths, from confusing the historical
3024x1964 screenshots with the reopened geometry evidence, or from losing the
accepted live run ids and closure artifacts.

## Historical staging lineage

- `run_1781881896971_19131_0` — `rich`
  - screenshot: `.auv/runs/run_1781881896971_19131_0/artifacts/artifact_0001_rich-frame.png`
  - spatial frame: `.auv/runs/run_1781881896971_19131_0/artifacts/artifact_0002_minecraft-spatial-frame.json`
  - resource pack evidence: `file/auv-mc6-rich`
- `run_1781881897582_19207_0` — `flat_color`
  - screenshot: `.auv/runs/run_1781881897582_19207_0/artifacts/artifact_0001_flat-frame.png`
  - spatial frame: `.auv/runs/run_1781881897582_19207_0/artifacts/artifact_0002_minecraft-spatial-frame.json`
  - resource pack evidence: `file/auv-mc6-flat-color`
- `run_1781881898175_19213_0` — `repetitive`
  - screenshot: `.auv/runs/run_1781881898175_19213_0/artifacts/artifact_0001_repetitive-frame.png`
  - spatial frame: `.auv/runs/run_1781881898175_19213_0/artifacts/artifact_0002_minecraft-spatial-frame.json`
  - resource pack evidence: `file/auv-mc6-repetitive`

Historical read boundary:

- These three screenshots are all `3024x1964`.
- They remain valid for the historical fail report and the clean rebuild record.
- They are no longer the accepted reopened Gate 1 geometry evidence.

## Fresh Gate 1 geometry lineage

- `rich`
  - capture run: `run_1782249745960_13265_0`
  - screenshot:
    `.auv/runs/run_1782249745960_13265_0/artifacts/artifact_0001_window-capture-window-capture.png`
  - bridge run: `run_1782249819524_13379_0`
  - spatial frame:
    `.auv/runs/run_1782249819524_13379_0/artifacts/artifact_0002_minecraft-spatial-frame.json`
  - `telemetry_session_id`: `0bf5f5b3-6a34-4a15-b373-f641447c75ff`
- `flat_color`
  - capture run: `run_1782250118864_14927_0`
  - screenshot:
    `.auv/runs/run_1782250118864_14927_0/artifacts/artifact_0001_window-capture-window-capture.png`
  - bridge run: `run_1782250218139_15240_0`
  - spatial frame:
    `.auv/runs/run_1782250218139_15240_0/artifacts/artifact_0002_minecraft-spatial-frame.json`
  - `telemetry_session_id`: `feb47bfc-8e27-44e0-af1a-1f1d270c59fa`
- `repetitive`
  - capture run: `run_1782250420931_16392_0`
  - screenshot:
    `.auv/runs/run_1782250420931_16392_0/artifacts/artifact_0001_window-capture-window-capture.png`
  - bridge run: `run_1782250477803_16666_0`
  - spatial frame:
    `.auv/runs/run_1782250477803_16666_0/artifacts/artifact_0002_minecraft-spatial-frame.json`
  - `telemetry_session_id`: `9b9af73c-1c2e-459b-a55f-b840f1c31723`

## Fresh Gate 2 closure lineage

- `repetitive`
  - `accepted-early`
    - capture run: `run_1782281165555_53632_0`
    - bridge run: `run_1782281195384_53866_0`
    - `screen_state`: `in_game`
  - `accepted-late`
    - capture run: `run_1782281366295_54733_0`
    - bridge run: `run_1782281378426_54930_0`
    - `screen_state`: `in_game`
  - `refusal-menu`
    - capture run: `run_1782282071489_57893_0`
    - bridge run: `run_1782282101783_58126_0`
    - `screen_state`: `menu`
    - refusal reason: `MenuLoadingScreen`
  - `telemetry_session_id = 9b9af73c-1c2e-459b-a55f-b840f1c31723`
- `rich`
  - `accepted-early`
    - capture run: `run_1782283188235_69082_0`
    - bridge run: `run_1782283214391_69376_0`
    - `screen_state`: `in_game`
  - `accepted-late`
    - capture run: `run_1782283386128_70571_0`
    - bridge run: `run_1782283416142_70808_0`
    - `screen_state`: `in_game`
  - `refusal-menu`
    - capture run: `run_1782283457672_71083_0`
    - bridge run: `run_1782283473776_71280_0`
    - `screen_state`: `menu`
    - refusal reason: `MenuLoadingScreen`
  - `telemetry_session_id = 246a3105-1417-470c-943f-5e48abd09224`
- `flat_color`
  - `accepted-early`
    - capture run: `run_1782283814387_75282_0`
    - bridge run: `run_1782283850920_75557_0`
    - `screen_state`: `in_game`
  - `accepted-late`
    - capture run: `run_1782283980548_76079_0`
    - bridge run: `run_1782284010123_76280_0`
    - `screen_state`: `in_game`
  - `refusal-menu`
    - capture run: `run_1782284054773_76561_0`
    - bridge run: `run_1782284093349_76766_0`
    - `screen_state`: `menu`
    - refusal reason: `MenuLoadingScreen`
  - `telemetry_session_id = 8d5bf3fc-36e0-4d61-b5cd-163d9e775990`

## Canonical target

The current canonical target comes from all three source spatial frames:

- target block position: `511,73,728`
- target block id: `minecraft:oak_button`

Audit note:

- The position and block id above are taken from `raycast_hit.block_pos` and `raycast_hit.block_id` inside the three canonical `minecraft-spatial-frame` artifacts.
- This is strong evidence for the current chain, but it is still derived from recorded frame truth rather than from a separately preserved CLI `--target-block` argument.

## Canonical bundle manifests

These are the accepted bundle-export artifacts tied to the three canonical source runs:

- `.auv/runs/run_1781881898217_19217_0/artifacts/artifact_0001_minecraft-spatial-bundle-run.json`
- `.auv/runs/run_1781881898256_19232_0/artifacts/artifact_0001_minecraft-spatial-bundle-run.json`
- `.auv/runs/run_1781881898290_19247_0/artifacts/artifact_0001_minecraft-spatial-bundle-run.json`

## Historical canonical built artifacts

These are the current accepted downstream artifacts built from the canonical MC-6 source lineage:

- samples: `.auv/runs/run_1782228448232_17661_0/artifacts/artifact_0001_texture_sweep_samples.json`
- report: `.auv/runs/run_1782228448788_17887_0/artifacts/artifact_0002_texture_sweep_report.json`

Current status read from the report:

- all three expected profiles are present
- `sample_count = 1` for each profile
- `duration_seconds = 0.0` for each profile
- `noise_refusal_exercised = false`
- `passed = false`

Therefore the chain is canonical and auditable, but it is not yet numerically sufficient to close MC-6.

Current reopened live status before the fresh sweep closure:

- Gate 1 geometry is now pinned to the fresh 2026-06-24 `window.capture`
  lineage above.
- Gate 2 completeness was still missing multi-profile live source runs.
- Therefore the old report remains the current historical fail report, not the
  final reopened closure report.

## Reopened closure artifacts

The accepted 2026-06-24 closure artifacts from the fresh 9-run live sweep are:

- local bundle/sample/eval workspace:
  `.tmp/mc6-live-a-20260624/`
- sample-build artifact:
  `.auv/runs/run_1782284483150_79654_0/artifacts/artifact_0001_texture_sweep_samples.json`
- final eval artifact:
  `.auv/runs/run_1782284485217_79709_0/artifacts/artifact_0002_texture_sweep_report.json`
- local eval copy:
  `.tmp/mc6-live-a-20260624/eval/texture_sweep_report.json`

Final report reading:

- all three expected profiles are present
- `sample_count = 2` for each profile
- `noise_refusal_exercised = true`
- overall `passed = true`
- per-profile duration:
  - `flat_color = 159.244 s`
  - `repetitive = 183.035 s`
  - `rich = 201.762 s`

Therefore the historical three-run lineage remains the accepted fail/staging
artifact, but the reopened 9-run lineage is now the accepted MC-6 closure
artifact.

## Clean working directory

Future regenerated MC-6 artifacts for this exact three-run lineage should stage under:

- `.tmp/mc6-rebuild-clean/`

Rules for that directory:

- do not reuse `/tmp/auv-mc67-live*`
- do not treat old rehydrated temp paths as canonical inputs
- if the export/build/eval chain is rerun, keep the inputs pinned to the canonical source runs and bundle manifests listed above
