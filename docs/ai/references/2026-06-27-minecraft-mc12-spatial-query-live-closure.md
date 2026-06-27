# MC-12 spatial query live closure

Date: 2026-06-27

## Summary

This note records the first fresh local pass where MC-12 spatial query consumed an
existing MC-10 semantic manifest and produced an `answered` projection-reference
inspect pair for a real MC-7 scene-packet lineage block target.

This is a **live evidence slice** only. It does **not** claim:

- Gaussian-native or checkpoint-native inference
- cloud-provider training quality
- trained splat usefulness
- render preview / viewer quality

## Input lineage

Reused MC-10 semantic output from the MC-9 D5 / MC-7 closure chain:

- MC-10 semantic manifest (sole MC-12 input surface):
  `.tmp/mc10-smoke-review/semantic/minecraft-3dgs-training-result-semantic.json`
- MC-10 semantic validation run: `run_1782502115784_89371_0`
- upstream scene packet (via semantic lineage):
  `.tmp/mc7-live/closure/scene-packet/run.json`
- upstream normalized training result (via semantic lineage):
  `.tmp/mc9-d5-live/training-result-artifacts/normalized-result/`
- trainer backend: `nerfstudio.splatfacto`

Fresh MC-12 output directories:

- `.tmp/mc12-live/query-visible/` — primary gate (raycast hit block)
- `.tmp/mc12-live/query-absent/` — secondary negative control (block absent)
- `.tmp/mc12-live/query-outside-window/` — provider-contract evidence (`outside_window` via `--query-command` stub)

## Target selection

From MC-7 scene packet frame payloads (`spatial_frame.raycast_hit` on all six
`in_game` frames):

- block: `511,73,728`
- face: `north` (oak button hit on frame index 1; same hit repeated across frames)
- semantics: `hit_face_center`

Negative control:

- block: `9,9,9` (not present in raycast hit or nearby blocks)

## Commands

Primary gate (reference-only, visible answer):

```sh
cargo run --quiet -- minecraft query-3dgs-training-result \
  --training-result-semantic-manifest .tmp/mc10-smoke-review/semantic/minecraft-3dgs-training-result-semantic.json \
  --target-block 511,73,728 \
  --target-face north \
  --target-semantics hit_face_center \
  --output-dir .tmp/mc12-live/query-visible
```

Secondary negative control:

```sh
cargo run --quiet -- minecraft query-3dgs-training-result \
  --training-result-semantic-manifest .tmp/mc10-smoke-review/semantic/minecraft-3dgs-training-result-semantic.json \
  --target-block 9,9,9 \
  --output-dir .tmp/mc12-live/query-absent
```

Provider-contract evidence (`outside_window`; not emit-able from `projection_reference` on this lineage):

```sh
cargo run --quiet -- minecraft query-3dgs-training-result \
  --training-result-semantic-manifest .tmp/mc10-smoke-review/semantic/minecraft-3dgs-training-result-semantic.json \
  --target-block 511,73,728 \
  --query-command "python3 .tmp/mc12-live/outside-window-query-stub.py" \
  --output-dir .tmp/mc12-live/query-outside-window
```

Stub (`.tmp/mc12-live/outside-window-query-stub.py`) reads the MC-12 request JSON from stdin and prints a
provider answer matching the unit-test contract in
`visibility_outside_window_is_answered_not_failed` (no invalid `reason` enum; optional fields only as
supported by the parser).

## Recorded runs

### Primary (`query-visible`)

- MC-12 spatial query run: `run_1782543398186_14786_0`
- Terminal summary:
  - `status = answered`
  - `selectedBackend = projection_reference`
  - `visibility = Visible`
  - `screenPoint = 853.9998593711567,480.0002071831308`
  - `basisFrameId = frame-355416-47699343801916`
  - `comparisonVerdict = reference_only`

Observed manifest / inspect facts:

- `status = answered`
- `selected_backend = projection_reference`
- `visibility = visible`
- `basis_frame_id = frame-355416-47699343801916`
- `comparison_verdict = reference_only`
- `reference_status = answered`
- `provider_status = blocked` (no `--query-command`; intentional v1 convention)
- `reference_source_frame_json_path =
  .tmp/mc7-live/closure/scene-packet/frames/frame_000006.json` (latest in_game frame
  with matching raycast block)

Artifacts:

- `.tmp/mc12-live/query-visible/minecraft-3dgs-training-result-query.json`
- `.tmp/mc12-live/query-visible/minecraft-3dgs-training-result-query-inspect.json`

### Secondary (`query-absent`)

- MC-12 spatial query run: `run_1782543409758_15819_0`
- Terminal summary:
  - `status = failed`
  - `reason = target_block_absent_from_scene_packet`
  - `selectedBackend = none`
  - `comparisonVerdict = not_comparable`

Artifacts:

- `.tmp/mc12-live/query-absent/minecraft-3dgs-training-result-query.json`
- `.tmp/mc12-live/query-absent/minecraft-3dgs-training-result-query-inspect.json`



### Provider contract (`query-outside-window`)

This pass is **provider-contract live evidence**, not reference-native or Gaussian-native
`outside_window` inference. The same block target (`511,73,728`) still projects as **visible** on
`projection_reference`; the stub proves MC-12 honors an external provider `answered` +
`outside_window` outcome end-to-end.

- MC-12 spatial query run: `run_1782543551237_21825_0`
- Stub: `.tmp/mc12-live/outside-window-query-stub.py`
- Terminal summary:
  - `status = answered`
  - `selectedBackend = command_provider`
  - `visibility = OutsideWindow`
  - `screenPoint = none`
  - `basisFrameId = provider-frame-mc12-live-outside-window`
  - `comparisonVerdict = divergent` (provider `outside_window` vs reference `visible`)

Observed manifest / inspect facts:

- `status = answered`
- `selected_backend = command_provider`
- `visibility = outside_window`
- `basis_frame_id = provider-frame-mc12-live-outside-window`
- `comparison_verdict = divergent`
- `provider_status = answered`
- `reference_status = answered` (reference still visible on scene-packet lineage)
- `reference_basis_frame_id = frame-355416-47699343801916`

Artifacts:

- `.tmp/mc12-live/query-outside-window/minecraft-3dgs-training-result-query.json`
- `.tmp/mc12-live/query-outside-window/minecraft-3dgs-training-result-query-inspect.json`

## Operational notes

- Run commands from the repository root; semantic and scene-packet paths in MC-10
  lineage are repo-relative.
- MC-7 scene packet frame JSON uses the D2 `ScenePacketFramePayload` envelope
  (`spatial_frame` nested). MC-12 reference loading now accepts that envelope (with
  flat-frame fallback for unit fixtures) so real MC-7 lineage queries can read
  raycast/nearby block witnesses.
- inspect-server writes to `http://127.0.0.1:8765` were unavailable during this pass.
  Local run-store records and on-disk query artifacts still closed the gate.
- Primary visible gate used reference-only (`--query-command` omitted). A separate
  `query-outside-window` pass used `--query-command` with
  `.tmp/mc12-live/outside-window-query-stub.py` because `projection_reference` cannot emit
  `outside_window` on this projector/lineage.

## Verdict

MC-12 is **live-closed** for **block-only spatial query over MC-10 semantic manifests**
on the MC-9 D5 / MC-7 lineage:

- MC-10 semantic manifest was the sole CLI input surface
- projection_reference returned an auditable visible answer for a real raycast block
- manifest + inspect pair recorded lineage, backend selection, and known limits honestly
- negative control recorded honest `failed` / `target_block_absent_from_scene_packet`
- command-provider stub recorded `outside_window` as `answered` (not `failed`) with honest
  `divergent` comparison vs reference
- no Gaussian-native inference or trainer quality claims

## Known limits (still not proven)

- Checkpoint-native / Gaussian-native query cores
- Command-provider backends beyond contract stubbing (when configured)
- Entity / anchor / label query
- Trainer execution quality on a real cloud backend
- Splat usefulness or render preview fidelity
