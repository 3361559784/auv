# MC-20 D4: Live evidence closeout (graduation)

Date: 2026-06-30

## Summary

MC-20 D4 **graduates** the canonical CLI `auv minecraft query-wired-live-click`
Layer-3 operator evidence matrix **G0–G8**. This document unifies verdict
documentation, cites prior D2.1 / D3 live runIds, and records the new **G8
`absent`** gate (`attempted=true` + dispatch failed + Layer-3 skipped).

Design reference:
[`2026-06-30-minecraft-mc20-d4-live-evidence-closeout-design.md`](2026-06-30-minecraft-mc20-d4-live-evidence-closeout-design.md)

Prior slices:

- D2.1 G0–G5:
  [`2026-06-30-minecraft-mc20-d2-1-canonical-cli-live-closure.md`](2026-06-30-minecraft-mc20-d2-1-canonical-cli-live-closure.md)
- D3 G6–G7:
  [`2026-06-30-minecraft-mc20-d3-semantic-pass-fail-live-closure.md`](2026-06-30-minecraft-mc20-d3-semantic-pass-fail-live-closure.md)

## Preconditions

- MC-18 semantic manifest: `.tmp/mc18-live/setup/semantic.json`
- MC-18 closed-scene fixtures (committed under `crates/auv-game-minecraft/tests/fixtures/mc18/`)
- Dedicated D4 store: `.tmp/mc20-d4-live/store`
- All inspect excerpts use `auv inspect <runId> --store-root .tmp/mc20-d4-live/store`
  (G2–G7 runIds live in their slice stores; cited by reference)

## Graduation table (G0–G8)

| Gate | ID | Outcome | Run ID / source | Pass |
| --- | --- | --- | --- | --- |
| Parse auto | G0 | — | `cargo test parse_minecraft_query_wired_live_click` (11/11) | yes |
| Parse negative | G1 | CLI exit 1 | D2.1 G1 (no runId) | yes |
| Refusal | G2 | `not_attempted` | `run_1782726278209_6291_0` (D2.1 store) | yes |
| Not consumable | G3 | `not_attempted` | `run_1782726355246_6795_0` (D2.1 store) | yes |
| Click, no witness | G4 | `unreliable` | `run_1782726447885_7262_0` (D2.1 store) | yes |
| Click + tick witness | G5 | `inconclusive` | `run_1782726624570_10870_0` (D2.1 store) | yes |
| Semantic pass | G6 | `passed` | `run_1782730403862_30193_0` (D3 store) | yes |
| Semantic fail | G7 | `failed` | `run_1782730530951_31089_0` (D3 store) | yes |
| Dispatch failed | G8 | `absent` | **`run_1782733909485_89642_0`** (D4 store) | yes |

### G0 — automated parse

```sh
cargo test -p auv-cli parse_minecraft_query_wired_live_click
```

Result (2026-06-30): **11 passed** (includes `--verification-expected-item-id requires --sample`).

### G6 / G7 — Layer-3 producer excerpts (D3, by reference)

G6 `run_1782730403862_30193_0` — `operation-result.verifications[0]`:

```json
{
  "method": {"kind": "semantic_match"},
  "executed": true,
  "state_changed": true,
  "semantic_matched": true,
  "failure_layer": null,
  "observed_label": null
}
```

G7 `run_1782730530951_31089_0` — `operation-result.verifications[0]`:

```json
{
  "method": {"kind": "semantic_match"},
  "executed": true,
  "state_changed": true,
  "semantic_matched": false,
  "failure_layer": "state_changed_no_match",
  "observed_label": null
}
```

Full inspect excerpts: D3 live closure doc.

## G8 — dispatch failed → absent (`run_1782733909485_89642_0`)

Strategy: `click_ready` + deliberate wrong `--target-title` so
`input.clickWindowPoint` invoke fails → `attempted=true`, `click_summary` absent,
Layer-3 skipped.

```sh
cargo run --quiet -- minecraft query-wired-live-click \
  --training-result-semantic-manifest .tmp/mc18-live/setup/semantic.json \
  --target-block 511,73,728 \
  --target-face north \
  --target-semantics hit_face_center \
  --query-provider closed-scene-toy \
  --closed-scene-fixture crates/auv-game-minecraft/tests/fixtures/mc18/visible.json \
  --output-dir .tmp/mc20-d4-live/g8-absent/query-output-v2 \
  --target-app com.todesktop.230313mzl4w92 \
  --target-title "__MC20_D4_NO_SUCH_WINDOW__" \
  --store-root .tmp/mc20-d4-live/store
```

Stdout (no `inspectHint` — dispatch failed, Layer-3 not readable):

```text
runId: run_1782733909485_89642_0
queryStatus: answered
wiringAttempted: true
actionEligibility: click_ready
operationResultArtifact: artifact_0003
```

```sh
cargo run --quiet -- inspect run_1782733909485_89642_0 --store-root .tmp/mc20-d4-live/store
```

Inspect excerpt:

```text
Verifications:
- none

MC-19 Query Wired Live Action:
- operation_result_artifact=artifact_0003 query_artifact=artifact_0001 attempted=true action_eligibility=click_ready window_point=640,360 refusal_reason=command input.clickWindowPoint handler failed: could not resolve a visible app reference for selector "com.todesktop.230313mzl4w92" operation_status=failed operation_message=command input.clickWindowPoint handler failed: could not resolve a visible app reference for selector "com.todesktop.230313mzl4w92" dispatch_command=input.clickWindowPoint dispatch_outcome=failed: ... target_app=com.todesktop.230313mzl4w92 target_title=__MC20_D4_NO_SUCH_WINDOW__ ... verification_outcome=absent verification_source=kind=operation_result artifact_id=artifact_0003 run_id=run_1782733909485_89642_0 ...
```

`operation-result` Layer-3 evidence (empty — field omitted):

```json
{
  "status": "failed",
  "output": {
    "kind": "acknowledged",
    "message": "command input.clickWindowPoint handler failed: could not resolve a visible app reference for selector \"com.todesktop.230313mzl4w92\""
  },
  "known_limits": [
    "...",
    "mc19_v1_d4_query_wired_live_action_non_stub_click_no_gameplay_verification"
  ]
}
```

G8 checklist:

| Check | Result |
| --- | --- |
| `wiringAttempted: true` | yes |
| `operation-result.verifications` empty | yes |
| `verification_outcome=absent` | yes |
| `dispatch_outcome=failed` | yes |
| No `click_summary` + `absent` contradiction | yes |
| No `inspectHint` on stdout | yes |

**NOTICE:** Pre-fix run `run_1782733735523_82706_0` incorrectly showed
`unreliable` because `invoke_click_at_window_point` treated invoke handler
failure as `Ok(summary)`. D4 hardening maps `RunStatus::Failed` → `Err` so
`click_summary` stays absent; superseded, not counted in verdict.

## Verdict

**MC-20 D4 closed** for canonical CLI Layer-3 operator evidence (G0–G8) on
macOS (2026-06-30).

## Honest limits

- G5–G7 use **synthetic** witness shaping; proves semantic plumbing, not
  gameplay break/harvest.
- G8 proves dispatch-failed Layer-3 skip → `absent`; not post-dispatch-success
  absent (that state is an anomaly per D4 design).
- D3.1 post-frame freshness (`read_latest_spatial_frame_newer_than` +
  `MC20_POST_FRAME_WAIT`) is integration-tested; live gameplay harvest not in
  scope.
- Inspect server write 502 warnings observed; local store + text projection
  unaffected.
- MC-20 controller / planner lane remains paused.
- `invoke_click_at_window_point` Failed → `Err` also affects
  `run_minecraft_live_click` and osu `query_live_action`; see D4 design blast
  radius.

## Related

- D4 design:
  [`2026-06-30-minecraft-mc20-d4-live-evidence-closeout-design.md`](2026-06-30-minecraft-mc20-d4-live-evidence-closeout-design.md)
- D2 CLI design:
  [`2026-06-30-minecraft-mc20-d2-query-wired-live-click-cli-design.md`](2026-06-30-minecraft-mc20-d2-query-wired-live-click-cli-design.md)
- D1 producer table:
  [`2026-06-30-minecraft-mc20-d1-query-wired-post-action-verification-design.md`](2026-06-30-minecraft-mc20-d1-query-wired-post-action-verification-design.md)
