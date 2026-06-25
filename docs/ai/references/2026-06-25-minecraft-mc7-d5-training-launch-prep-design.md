# 2026-06-25 Minecraft MC-7 D5 training-launch prep design

Date: 2026-06-25

Status: implemented code slice; local validation should prove command, crate logic,
and recorded artifacts. Historical accepted-only lineage remains unrecoverable,
so any future real-source trainer validation still requires a fresh accepted-only
capture lineage.

## Scope

MC-7 D5 is defined as:

- consume one D3 training-package manifest
- produce one trainer launch plan JSON
- produce one trainer launch inspect JSON
- produce one manual runbook Markdown
- do not start training
- do not install dependencies
- do not emit a trained splat

D5 is the first trainer-side consumer, but it remains a prep/readiness slice.

## Input boundary

D5 reads only the D3 training-package surface:

- `run.json`
- `inspect_report.json`
- `compat/nerfstudio/export_report.json`
- `compat/nerfstudio/transforms.json` when D3 exported at least one trainable frame

It does not reopen MC-6 bundles or D2 scene packets directly.

## Output shape

The canonical D5 output directory is:

```text
minecraft-3dgs-training-launch-plan.json
minecraft-3dgs-training-launch-inspect.json
mc7-training-launch-runbook.md
```

Three artifact roles are staged:

- `minecraft-3dgs-training-launch-plan`
- `minecraft-3dgs-training-launch-inspect`
- `minecraft-3dgs-training-launch-runbook`

## Fixed launch policy

D5 intentionally binds one backend and one command shape:

- `trainer_backend = nerfstudio.splatfacto`
- `launch_command = ns-train splatfacto --data <training-package>/compat/nerfstudio --output-dir <d5-output>/trainer-output/nerfstudio-splatfacto`

This is a prep artifact only. D5 does not spawn the command.

## Readiness policy

D5 performs one local probe only:

- `ns-train --help`

Readiness is:

- `ready` when the D3 Nerfstudio view is not blocked, exported frame count is
  greater than zero, `transforms.json` is present/readable, and `ns-train --help`
  succeeds
- `blocked` otherwise

Allowed blockers are intentionally narrow:

- `compatibility_view_blocked`
- `transforms_missing`
- `trainer_command_unavailable`

`partial` compatibility does not block D5 by itself. If D3 exported at least one
trainable frame and still has a readable `transforms.json`, D5 reports `ready`
and carries forward the skip warnings honestly.

## Relationship to D3

D3 remains:

- the canonical training-prep package exporter
- the authority for canonical frames/images and compatibility-view facts

D5 adds:

- the first trainer-side launch contract
- local readiness truth for the `ns-train` entrypoint
- a reproducible runbook without pretending real training has happened

This still does **not** mean real-source trainer validation is closed. Any
future claim about trained output quality or trainer success must come from a
fresh accepted-only capture lineage rerun through D2 -> D3 -> D5 and then an
explicit trainer execution slice.
