# MC-10 semantic validation live closure

Date: 2026-06-27

## Summary

This note records the first fresh local pass where MC-10 semantic validation
consumed an existing MC-9 D11 artifact manifest and produced a `ready` semantic
inspect pair.

This is a **live evidence slice** only. It does **not** claim:

- cloud-provider training quality
- trained splat usefulness
- checkpoint internal semantics
- render preview / viewer quality

## Input lineage

Reused MC-9 D5 live normalized artifacts:

- D11 manifest:
  `.tmp/mc9-d5-live/training-result-artifacts/minecraft-3dgs-training-result-artifact-manifest.json`
- normalized result dir:
  `.tmp/mc9-d5-live/training-result-artifacts/normalized-result/`
- trainer backend: `nerfstudio.splatfacto`

Fresh semantic output directory:

- `.tmp/mc10-smoke-review/semantic/`

## Command

```sh
cargo run --quiet -- minecraft validate-3dgs-training-result \
  --training-result-artifact-manifest .tmp/mc9-d5-live/training-result-artifacts/minecraft-3dgs-training-result-artifact-manifest.json \
  --output-dir .tmp/mc10-smoke-review/semantic
```

## Recorded run

- MC-10 semantic validation run: `run_1782502115784_89371_0`

## Gate results

Observed facts:

- `semantic_status = ready`
- `semantic_reason = null`
- `trainer_backend = nerfstudio.splatfacto`
- `config_trainer = nerfstudio.splatfacto`
- `config_backend_matches = true`
- `models_dir_readable = true`
- `checkpoint_count = 1`
- `status_snapshot_present = true`

Artifacts:

- `.tmp/mc10-smoke-review/semantic/minecraft-3dgs-training-result-semantic.json`
- `.tmp/mc10-smoke-review/semantic/minecraft-3dgs-training-result-semantic-inspect.json`

## Operational note

- `normalized_result_dir` in the D11 manifest is repo-relative (`.tmp/...`); run
  the command from the repository root.
- inspect-server writes to `http://127.0.0.1:8765` were unavailable during this
  pass. Local run-store records and on-disk semantic artifacts still closed the
  gate.

## Verdict

MC-10 is live-closed for **semantic-only normalized training-result validation**
on the MC-9 D5 lineage:

- D11 manifest was the sole input surface
- normalized `config.yml` / `nerfstudio_models/` / `*.ckpt` passed the semantic
  gate
- manifest + inspect pair recorded lineage and known limits honestly
- no render preview was generated

## Known limits (still not proven)

- Trainer execution quality on a real cloud backend
- Checkpoint semantics beyond file presence
- Dedicated read-side inspect summary consumption (MC-11)
