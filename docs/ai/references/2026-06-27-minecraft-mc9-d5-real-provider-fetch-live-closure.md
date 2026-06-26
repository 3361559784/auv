# MC-9 D5 real provider fetch live closure

Date: 2026-06-27

## Summary

This note records the **first fresh live pass** where D6 → D7 → D11 ran with
explicit real provider configuration (`--training-job-endpoint`,
`--training-job-token`) through the full MC-9 D4 fetch stdin contract, producing
non-blocked submit/status/fetch evidence and normalized artifacts on disk.

This is a **live evidence slice** only. It does **not** claim:

- cloud-provider training quality
- trained splat usefulness
- checkpoint semantic validation
- viewer / renderer quality

## Preflight

- **Collabi**: remote writer UI at
  `https://collabi-airi-cu-free-01.koreacentral.cloudapp.azure.com/writer.html`
  was reachable during this pass. No automated writer-token check-in was
  performed from this harness (no callable check-in script/token in session).
- **RustRover MCP**: not available in this harness; Rust navigation used
  existing code paths and CLI/runtime output only.
- **inspect-server** (`http://127.0.0.1:8765`): unavailable; local run-store and
  on-disk artifacts still closed the gate.

## Input lineage

Reused accepted-only MC-7 closure inputs (same lineage as MC-9 D3 live):

- training package: `.tmp/mc7-live/closure/training-package/run.json`
- source scene packet: `.tmp/mc7-live/closure/scene-packet/run.json`
- source run count: `6`
- trainer backend: `nerfstudio.splatfacto`

Fresh working directory:

- `.tmp/mc9-d5-live/`

Provider adapter scripts (local command provider, not in-repo HTTP client):

- `.tmp/mc9-d5-live/bin/submit.py`
- `.tmp/mc9-d5-live/bin/status.py`
- `.tmp/mc9-d5-live/bin/fetch.py`

## Provider configuration (presence only)

- endpoint explicitly provided: **yes** (`https://mc9-live.example.invalid/api`)
- explicit token used: **yes** (CLI `--training-job-token`; value not recorded
  here)
- fetch command explicitly provided: **yes**
  (`python3 .tmp/mc9-d5-live/bin/fetch.py`)

Priority exercised: CLI explicit values for endpoint/token/fetch command on D6,
D7, and D11.

## Recorded runs

- D5 launch-prep reference run: `run_1782497169422_1860_0`
- D6 real-provider submit run: `run_1782497172264_2068_0`
- D7 real-provider status run: `run_1782497174850_2196_0`
- D11 provider-aware fetch run: `run_1782497177374_2359_0`

Inspect text snapshots:

- `.tmp/mc9-d5-live/inspect/d6.txt`
- `.tmp/mc9-d5-live/inspect/d7.txt`
- `.tmp/mc9-d5-live/inspect/d11.txt`

Live command log (no token values):

- `.tmp/mc9-d5-live/live-run.log`

## Gate results

### D6 real-provider submit

Observed facts:

- `status = submitted`
- `accepted_by_provider = true`
- `submission_recorded_at_millis = 1782497172349`
- `job_id = mc9-d5-live-job`
- `job_url = https://mc9-live.example.invalid/api/jobs/mc9-d5-live-job`
- `job_submission_endpoint = https://mc9-live.example.invalid/api`

Artifacts:

- `.tmp/mc9-d5-live/training-job/minecraft-3dgs-training-job.json`
- `.tmp/mc9-d5-live/training-job/minecraft-3dgs-training-job-inspect.json`
- `.tmp/mc9-d5-live/training-job/mc7-training-job-runbook.md`

### D7 provider-status truth

Observed facts:

- `status = succeeded`
- `status_reason = null`
- `status_message = provider-status-saw-job_id=mc9-d5-live-job token=present`
- `result_dir_exists = false`
- `key_result_artifacts_present = false`
- terminal interpretation:
  `provider_status_recorded_local_results_not_yet_observed`

Provider status remained `succeeded` while local result directory was not yet
observed. Local gaps stayed in warnings/observation fields only.

Artifacts:

- `.tmp/mc9-d5-live/training-result/minecraft-3dgs-training-result.json`
- `.tmp/mc9-d5-live/training-result/minecraft-3dgs-training-result-inspect.json`
- `.tmp/mc9-d5-live/training-result/mc7-training-result-runbook.md`

### D11 provider-aware normalized fetch (MC-9 D4 contract)

Observed facts:

- `fetch_status = succeeded`
- `fetch_reason = null`
- `source_result_status = succeeded`
- `source_result_dir_exists = false`
- `required_artifacts_present = false` (source D7 local observation)
- `normalized_artifact_count = 3`
- run-store input event recorded
  `training_job_endpoint_present=true training_job_token_present=true` without
  token plaintext

Warnings prove the D4 provider-aware fetch path was exercised:

- `mc9-d5-provider-fetch-saw-endpoint token_present=True`
- `source local result observation is incomplete; artifact fetch command materialized normalized outputs (fetch/normalize evidence, not provider status verdict)`

Normalized outputs on disk:

- `.tmp/mc9-d5-live/training-result-artifacts/normalized-result/config.yml`
- `.tmp/mc9-d5-live/training-result-artifacts/normalized-result/nerfstudio_models/`
- `.tmp/mc9-d5-live/training-result-artifacts/normalized-result/job_status.json`

Artifacts:

- `.tmp/mc9-d5-live/training-result-artifacts/minecraft-3dgs-training-result-artifact-manifest.json`
- `.tmp/mc9-d5-live/training-result-artifacts/minecraft-3dgs-training-result-artifact-inspect.json`

### Token persistence boundary

Scans of persisted manifests/inspect JSON, normalized outputs, and D6/D7/D11
run-store directories found **no job token plaintext**.

## Verdict

MC-9 D5 is live-closed for **first real provider fetch live success evidence**
under the existing D4 contract:

- D6 submit acceptance recorded with explicit endpoint
- D7 provider status truth recorded independently from local result presence
- D11 fetch succeeded with explicit endpoint/token config and D4 fetch-command
  stdin contract; normalized artifacts materialized on disk
- token remained runtime/stdin-only (not persisted in manifests, inspect, or
  run-store)

## Known limits (still not proven)

- Trainer execution quality on a real cloud backend
- Real remote artifact bytes (this pass uses local command-provider adapters)
- Checkpoint semantics beyond file presence
- inspect-server remote write availability
