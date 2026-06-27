# MC-15 checkpoint-native query provider live closure

Date: 2026-06-27

## Summary

First fresh local pass where MC-12 spatial query used the in-repo `checkpoint_native`
provider to read MC-10 semantic lineage normalized result inputs and produce dual-backend
compare evidence against `projection_reference`.

This is a **provider seam closure** only. It does **not** claim:

- Gaussian render inference inside the provider
- trained splat usefulness or quality judgment
- action dispatch or render preview (MC-16)

## Input lineage

Reused MC-10 semantic manifest from MC-9 D5 / MC-7 closure:

- `.tmp/mc10-smoke-review/semantic/minecraft-3dgs-training-result-semantic.json`
- scene packet: `.tmp/mc7-live/closure/scene-packet/run.json`
- normalized result: `.tmp/mc9-d5-live/training-result-artifacts/normalized-result/`
- checkpoint witness: `nerfstudio_models/step-000001.ckpt`

## Commands

Primary gate (visible target, checkpoint-native provider):

```sh
cargo run --quiet -- minecraft query-3dgs-training-result \
  --training-result-semantic-manifest .tmp/mc10-smoke-review/semantic/minecraft-3dgs-training-result-semantic.json \
  --target-block 511,73,728 \
  --target-face north \
  --target-semantics hit_face_center \
  --query-provider checkpoint-native \
  --output-dir .tmp/mc15-live/query-checkpoint-native-visible
```

Regression negative control (absent block):

```sh
cargo run --quiet -- minecraft query-3dgs-training-result \
  --training-result-semantic-manifest .tmp/mc10-smoke-review/semantic/minecraft-3dgs-training-result-semantic.json \
  --target-block 9,9,9 \
  --query-provider checkpoint-native \
  --output-dir .tmp/mc15-live/query-checkpoint-native-absent
```

## Recorded runs

### Visible (`query-checkpoint-native-visible`)

- run: `run_1782549547280_65028_0`
- `status = answered`
- `selectedBackend = checkpoint_native`
- `visibility = Visible`
- `screenPoint = 853.9998593711567,480.0002071831308`
- `basisFrameId = checkpoint:step-000001.ckpt`
- `comparisonVerdict = match`
- provider + reference both answered; inspect `provider_message` records deferred Gaussian inference

Artifacts:

- `.tmp/mc15-live/query-checkpoint-native-visible/minecraft-3dgs-training-result-query.json`
- `.tmp/mc15-live/query-checkpoint-native-visible/minecraft-3dgs-training-result-query-inspect.json`

### Absent block (`query-checkpoint-native-absent`)

- run: `run_1782549551072_65027_0`
- `status = failed`
- `selectedBackend = none`
- `comparisonVerdict = not_comparable`
- provider still records checkpoint basis witness while projection fails for absent target

Artifacts:

- `.tmp/mc15-live/query-checkpoint-native-absent/minecraft-3dgs-training-result-query.json`
- `.tmp/mc15-live/query-checkpoint-native-absent/minecraft-3dgs-training-result-query-inspect.json`

## Honest limits

- v1 projection math remains `scene_packet + MinecraftProjector`; checkpoint files are input witnesses only
- MC-13/14 read-side consumers accept `selected_backend = checkpoint_native` without schema changes
