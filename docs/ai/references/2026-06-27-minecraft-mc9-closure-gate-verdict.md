# MC-9 closure and gate verdict

Date: 2026-06-27

## Summary

MC-9 closes the **real-provider lane** for the Minecraft offline 3DGS trainer
chain through D1–D5. It does not close cloud trainer quality, checkpoint
semantics, splat usefulness, a multi-provider framework, or renderer / viewer
quality.

The closed MC-9 surface is:

```text
D1 single provider contract binding
-> D2 real provider submit acceptance recording
-> D3 provider status truth
-> D4 provider-aware fetch contract + token non-persistence hardening
-> D5 first real provider fetch live success under D4 contract
```

The right verdict is therefore:

- **closed:** real-provider submit / status / fetch lane through D1–D5
- **not closed:** cloud trainer quality, checkpoint semantics, splat
  usefulness, multi-provider framework, renderer / viewer quality

MC-9 does **not** require a D6 slice to be considered closed. Any D6/D7
run-store token symmetry hardening is follow-up only and does not retroactively
change the D5 closure judgment recorded here.

## Input references

MC-9 closure is a documentation and verdict slice over already-landed code and
live evidence. The primary references are:

- `docs/ai/references/2026-06-18-auv-mc5-onward-execution-plan.md`
- `docs/ai/references/2026-06-27-minecraft-mc9-d3-real-provider-status-closure.md`
- `docs/ai/references/2026-06-27-minecraft-mc9-d3-live-provider-status-and-fetch-closure.md`
- `docs/ai/references/2026-06-27-minecraft-mc9-d4-real-provider-artifact-fetch-closure.md`
- `docs/ai/references/2026-06-27-minecraft-mc9-d5-real-provider-fetch-live-closure.md`
- `docs/ai/references/2026-06-26-minecraft-mc8-closure-gate-verdict.md` (prior
  command-adapter lane boundary)

Code and commit anchors for slices without standalone reference notes:

- D1 contract binding:
  `crates/auv-game-minecraft/src/training_job.rs` (`MC-9 D1 binds this training
  job lane to one provider contract...`); commit `324263c`
- D2 submit acceptance fields:
  `accepted_by_provider`, `submission_recorded_at_millis`; commit `c5d4217`
- D4 D11 run-store token hardening: commit `adca19c`

## What MC-9 closed

### D1 — single provider contract binding

MC-9 D1 binds the D6 training-job lane to **one provider contract**. The
manifest and inspect surfaces record a known limit that multi-provider expansion
is deferred by owner approval.

Evidence is in code, not a standalone D1 note:

- `crates/auv-game-minecraft/src/training_job.rs` inserts the MC-9 D1 known
  limit and keeps the lane on a single provider contract.
- Commit: `324263c` (`fix(auv-game-minecraft): preserve training job provider
  compatibility`)

### D2 — real provider submit acceptance recording

MC-9 D2 closes honest recording of provider submit acceptance on D6:

- `accepted_by_provider` records whether the provider-facing submit path
  accepted the job envelope;
- `submission_recorded_at_millis` is set only when acceptance is true;
- blocked or failed submit paths do not fabricate acceptance.

Evidence:

- fields on `TrainingJobManifest` / `TrainingJobInspectReport` in
  `crates/auv-game-minecraft/src/training_job.rs`
- commit: `c5d4217` (`feat(auv-game-minecraft): record provider acceptance on
  job submit`)
- live D6 observed facts in:
  - `docs/ai/references/2026-06-27-minecraft-mc9-d3-live-provider-status-and-fetch-closure.md`
  - `docs/ai/references/2026-06-27-minecraft-mc9-d5-real-provider-fetch-live-closure.md`

### D3 — provider status truth

MC-9 D3 closes **D7 real provider status evidence**. Provider-reported status
is recorded separately from local result-directory observation. A provider
`succeeded` status does not get rewritten when `result_dir_exists = false`.

Authority:

- contract and code slice:
  `docs/ai/references/2026-06-27-minecraft-mc9-d3-real-provider-status-closure.md`
- first fresh local live pass (D6 submit + D7 status + D11 command fetch):
  `docs/ai/references/2026-06-27-minecraft-mc9-d3-live-provider-status-and-fetch-closure.md`

Responsibility split preserved by MC-9:

```text
D7 (MC-9 D3) -> provider / status-command truth
D11          -> fetch / normalize / required-artifact completeness
```

### D4 — provider-aware fetch contract + token non-persistence hardening

MC-9 D4 closes the **D11 provider-aware fetch command stdin contract** and
explicit remote config input surface (`--training-job-endpoint`,
`--training-job-token`). `job_token` remains runtime/stdin-only and must not
appear in persisted manifest or inspect JSON.

Authority:

- `docs/ai/references/2026-06-27-minecraft-mc9-d4-real-provider-artifact-fetch-closure.md`
- D11 run-store hardening (presence-only input recording, no token plaintext in
  run-store): commit `adca19c` (`fix(auv-cli): harden mc9 d4 artifact fetch
  secrets`)

### D5 — first real provider fetch live success under D4 contract

MC-9 D5 records the **first fresh live pass** where D6 → D7 → D11 ran with
explicit real provider configuration through the full D4 fetch stdin contract,
producing non-blocked submit / status / fetch evidence and normalized artifacts
on disk.

Authority:

- `docs/ai/references/2026-06-27-minecraft-mc9-d5-real-provider-fetch-live-closure.md`

This pass used local command-provider adapters and an explicit
`example.invalid` endpoint. It proves the real-provider lane and D4 contract
closure; it does **not** prove cloud-provider training quality.

## Evidence checklist

The minimum evidence that supports the MC-9 closure verdict is:

| Slice | Evidence source |
| ----- | --------------- |
| D1 | `training_job.rs` MC-9 D1 known limit; commit `324263c` |
| D2 | `accepted_by_provider`, `submission_recorded_at_millis`; commit `c5d4217`; D3/D5 live D6 facts |
| D3 | `mc9-d3-real-provider-status-closure.md`; `mc9-d3-live-provider-status-and-fetch-closure.md` |
| D4 | `mc9-d4-real-provider-artifact-fetch-closure.md`; commit `adca19c` |
| D5 | `mc9-d5-real-provider-fetch-live-closure.md` |

D5 recorded run ids (primary MC-9 live gate):

- D5 launch-prep reference run: `run_1782497169422_1860_0`
- D6 real-provider submit run: `run_1782497172264_2068_0`
- D7 real-provider status run: `run_1782497174850_2196_0`
- D11 provider-aware fetch run: `run_1782497177374_2359_0`

D3 live run ids remain supplementary evidence for D3 status truth and early D2
D6 acceptance facts:

- `run_1782493613903_36423_0` (D6)
- `run_1782493616372_36652_0` (D7)

Local `.tmp/mc9-d3-live/` and `.tmp/mc9-d5-live/` working copies may still
exist as operator-side artifacts, but they are not the committed minimum
evidence for this verdict.

## Explicitly not closed by MC-9

MC-9 does **not** close any of the following:

- cloud-provider training execution quality or runtime correctness on a real
  vendor backend;
- trained splat usefulness or visual quality;
- checkpoint internals or checkpoint semantic validation;
- renderer / viewer / preview quality;
- multi-provider framework or provider switching;
- new Minecraft capture collection;
- MC-8 command-adapter lane redefinition (MC-8 remains the prior adapter closure;
  MC-9 is the real-provider lane closure on top of the MC-7 downstream chain).

Those belong to later slices and must not be backfilled into the MC-9 verdict.

### Follow-up (symmetry hardening, not a blocker)

The following items are recorded for a future hardening slice. They do **not**
block the MC-9 D1–D5 closure verdict and do **not** require an MC-9 D6:

- D6 and D7 recorded operations still do not record
  `training_job_endpoint_present` / `training_job_token_present` presence-only
  input events the way D11 does after commit `adca19c`.
- D6 and D7 runtime paths in `src/minecraft.rs` still pass
  `training_job_token.clone()` into recorded operations.

Current review found **no known persisted token plaintext leak** in manifests,
inspect JSON, or run-store artifacts from the D3/D5 live passes. This follow-up
is **symmetry hardening** only; it does not retroactively change the MC-9 D5
closure judgment.

## Gate verdict

### Closed

MC-9 is closed for the **real-provider lane** through D1–D5.

That means the repository now has a proven, auditable path for:

- binding D6 submit to one provider contract (D1);
- recording provider submit acceptance honestly (D2);
- collecting provider status truth independently from local result observation
  (D3);
- exercising D11 fetch with an explicit provider-aware stdin contract and
  hardened D11 run-store secret handling (D4);
- recording first live non-blocked submit → status → fetch success under that
  D4 contract (D5).

### Not closed

MC-9 is **not** closed for cloud trainer quality, checkpoint semantics, splat
usefulness, multi-provider support, or renderer quality.

Nothing in MC-9 proves that an external cloud provider actually trained a
useful model or stored semantically valid checkpoints. The D5 live success run
proves the real-provider lane, status/fetch separation, and normalized artifact
materialization under the D4 contract only.

## Final wording

Use this wording for follow-up summaries and planning notes:

> MC-9 closes the real-provider submit / status / fetch lane through D1–D5. It
> does not close cloud trainer quality, checkpoint semantics, splat usefulness,
> multi-provider framework work, or renderer / viewer quality. D6/D7 run-store
> token symmetry hardening is follow-up only and is not required for the MC-9
> closure verdict.

Follow-up slice after MC-9:
`docs/ai/references/2026-06-27-minecraft-mc10-result-semantic-validation-design.md`
