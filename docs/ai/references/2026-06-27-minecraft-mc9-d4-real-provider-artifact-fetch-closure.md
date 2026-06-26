# 2026-06-27 Minecraft MC-9 D4 real provider artifact fetch closure

Date: 2026-06-27

Status: implemented code slice for D11 provider-aware artifact fetch command
contract only. Fresh live provider fetch success evidence belongs to D5.

## Scope

MC-9 D4 closes the **D11 fetch command stdin contract** and explicit remote
config input surface. It does **not**:

- introduce an in-repo HTTP client or cloud SDK
- grade trainer quality or parse checkpoint semantics
- change D7 provider status truth
- run a fresh live provider fetch gate (D5)

## Command surface

Unchanged command name:

```text
auv-cli minecraft fetch-3dgs-training-result-artifacts
```

New optional flags:

- `--training-job-endpoint <url>`
- `--training-job-token <token>`

Existing flag retained:

- `--artifact-fetch-command <command>`

Priority: CLI explicit values → environment variables → D7 manifest defaults
for endpoint only.

Environment variables:

- `AUV_MINECRAFT_TRAINING_JOB_ENDPOINT`
- `AUV_MINECRAFT_TRAINING_JOB_TOKEN`
- `AUV_MINECRAFT_TRAINING_RESULT_ARTIFACT_FETCH_COMMAND`

## Fetch command stdin contract

`TrainingResultArtifactFetchCommandRequest` adds:

- `endpoint`
- `token_present`
- `job_token`

All prior fields remain authoritative context for fetch commands. Default
`endpoint` is CLI/env override when present, otherwise D7
`job_submission_endpoint`.

`job_token` is runtime/stdin only and must not appear in persisted manifest or
inspect JSON.

## Semantic boundaries preserved

D11 still separates:

1. source D7 observation (`source_result_dir_exists`,
   `required_artifacts_present`)
2. normalized fetch outcome (`fetch_status`, `normalized_artifacts`)

Command branch success does not rewrite source observation fields. Local source
gaps may appear in warnings while `fetch_status = succeeded`.

## Historical MC-8 adapter wording

MC-8 D3 closed the command-adapter lane for normalized artifact materialization.
MC-9 D4 updates stale `MC-8 D3` fetch warnings to neutral MC-9 D4 wording that
distinguishes fetch/normalize evidence from provider status verdicts.

Reference:
`docs/ai/references/2026-06-26-minecraft-mc8-closure-gate-verdict.md`
