# 2026-06-27 Minecraft MC-17 D2 quality baseline live closure

Date: 2026-06-27

Status: live closure on MC-10 semantic lineage + MC-12 / MC-16 / MC-17 derived baseline report.

## Fixed command chain

Reused semantic manifest:

- `.tmp/mc10-smoke-review/semantic/minecraft-3dgs-training-result-semantic.json`

Fresh outputs under `.tmp/mc17-d2-live/` with store root `.tmp/mc17-d2-live/store`.

### MC-12 spatial query

```sh
cargo run --quiet -- minecraft query-3dgs-training-result \
  --training-result-semantic-manifest .tmp/mc10-smoke-review/semantic/minecraft-3dgs-training-result-semantic.json \
  --target-block 511,73,728 \
  --target-face north \
  --target-semantics hit_face_center \
  --output-dir .tmp/mc17-d2-live/query-visible \
  --store-root .tmp/mc17-d2-live/store
```

Run: `run_1782594518255_60230_0`

### MC-16 holdout preview

```sh
cargo run --quiet -- minecraft inspect-3dgs-training-result-holdout \
  --training-result-semantic-manifest .tmp/mc10-smoke-review/semantic/minecraft-3dgs-training-result-semantic.json \
  --output-dir .tmp/mc17-d2-live/holdout-preview \
  --store-root .tmp/mc17-d2-live/store
```

Run: `run_1782594524936_60749_0`

Observed: `status=ready`, `holdoutFrameIndex=6`, checkpoint suffix `step-000001.ckpt`.

### MC-17 holdout render quality (screenshot-copy probe)

```sh
cargo run --quiet -- minecraft measure-3dgs-holdout-render-quality \
  --training-result-semantic-manifest .tmp/mc10-smoke-review/semantic/minecraft-3dgs-training-result-semantic.json \
  --holdout-preview-manifest .tmp/mc17-d2-live/holdout-preview/minecraft-3dgs-training-result-holdout-preview.json \
  --render-command 'python3 -c '"'"'import json,shutil,sys; d=json.loads(sys.stdin.read()); shutil.copy(d["holdout_screenshot_path"], d["requested_rendered_image_path"]); print(json.dumps({"status":"ready","rendered_image_path":d["requested_rendered_image_path"]}))'"'"'' \
  --output-dir .tmp/mc17-d2-live/render-quality \
  --store-root .tmp/mc17-d2-live/store
```

Run: `run_1782594531314_61141_0`

Comparable metric snapshot (copy-probe baseline):

- `verdict=measured_only`
- `image_size_match=true`
- `l1_mean=0`
- `mse=0`
- `psnr` omitted (identical RGB8 pixels)

## Derived baseline acceptance

Example (prints derived JSON):

```sh
cargo run --quiet --example mc17_quality_baseline_report -- \
  --store-root .tmp/mc17-d2-live/store \
  --run-id run_1782594531314_61141_0
```

Observed on MC-17 run:

- `profile_id=mc17-d2-primary-v1`
- `evidence_coverage=complete`
- MC-12 spatial query resolved from store scan (`answered`, `visible`)
- MC-16 holdout witness resolved from MC-17 `holdout_preview_manifest_path`
- MC-17 render metrics on-run (`measured_only`, `l1_mean=0`, `mse=0`)

Inspect text acceptance (use store root `.tmp/mc17-d2-live/store` via example or library call):

- expect `MC-17 Quality Baseline Report:` section
- expect `evidence_coverage=complete`
- expect `spatial_query_status=answered`
- expect `render_quality_status=ready verdict=measured_only`

## Honest boundary

This closure proves **pipeline-comparability** on the fixed profile using the screenshot-copy
probe. It does **not** claim trained-splat usefulness or downstream action eligibility.
