# MC-19 D4: Query-to-live-click wiring live closure

Date: 2026-06-28

## Summary

MC-19 D4 closes the wiring evidence chain from MC-12 spatial query + MC-14
derived action readiness to **one honest, recorded, refusable live click attempt**
via non-stub `input.clickWindowPoint`. This slice proves dispatch honesty and
run-store completeness only — not Minecraft gameplay success.

## Preconditions

- macOS with Accessibility permissions for AUV input delivery
- MC-18 semantic fixture: `.tmp/mc18-live/setup/semantic.json`
- MC-18 closed-scene fixtures (committed):
  - `crates/auv-game-minecraft/tests/fixtures/mc18/visible.json`
  - `crates/auv-game-minecraft/tests/fixtures/mc18/outside_window.json`
- **Canonical CLI (MC-20 D2):** `auv minecraft query-wired-live-click` — see [`2026-06-30-minecraft-mc20-d2-query-wired-live-click-cli-design.md`](2026-06-30-minecraft-mc20-d2-query-wired-live-click-cli-design.md)
- Historical live gate harness: `examples/mc19_query_wired_live_action.rs` (thin wrapper when present)
- Dedicated store for closure runs: `.tmp/mc19-live/store`

## Harness command shape

Preferred (MC-20 D2):

```sh
auv minecraft query-wired-live-click \
  --training-result-semantic-manifest .tmp/mc18-live/setup/semantic.json \
  --target-block <x,y,z> \
  [--target-face north] \
  --target-semantics hit_face_center \
  [--query-provider closed-scene-toy --closed-scene-fixture <fixture.json>] \
  --output-dir <dir> \
  --target-app <bundle-id> \
  --target-title <window-title-substring> \
  [--store-root .tmp/mc19-live/store]
```

Historical example wrapper:

```sh
cargo run --quiet --example mc19_query_wired_live_action -- \
  --semantic-manifest .tmp/mc18-live/setup/semantic.json \
  --target-block <x,y,z> \
  [--target-face north] \
  --target-semantics hit_face_center \
  [--query-provider closed-scene-toy --closed-scene-fixture <fixture.json>] \
  --output-dir <dir> \
  --target-app <bundle-id> \
  --target-title <window-title-substring> \
  [--store-root .tmp/mc19-live/store]
```

## Recorded runs (2026-06-28 local pass)

Store root: `.tmp/mc19-live/store`

### 1. click_ready — visible target, wiring dispatches non-stub click

- run: `run_1782590245467_18186_0`
- wiring: `attempted=true`, `action_eligibility=click_ready`
- nested invoke span: `command.resolved` → `input.clickWindowPoint`
- driver outcome: `command.failed` with `main visible window was not found` (title-filtered window resolve; counts as honest non-stub attempt per MC-19 design)
- `operation-result` message: invoke wrapper failure summary (click path reached real handler)
- standalone control (no title filter): `run_1782590236280_17852_0` — direct `input.clickWindowPoint` completed with `clicked window point`

Command:

```sh
cargo run --quiet --example mc19_query_wired_live_action -- \
  --semantic-manifest .tmp/mc18-live/setup/semantic.json \
  --target-block 511,73,728 \
  --target-face north \
  --target-semantics hit_face_center \
  --query-provider closed-scene-toy \
  --closed-scene-fixture crates/auv-game-minecraft/tests/fixtures/mc18/visible.json \
  --output-dir .tmp/mc19-live/click-ready/query-output \
  --target-app com.todesktop.230313mzl4w4u92 \
  --target-title Cursor \
  --store-root .tmp/mc19-live/store
```

Inspect snippets:

```text
minecraft.query_wired_live_action.outcome: attempted=true action_eligibility=click_ready
command.resolved: resolved input.clickWindowPoint
command.failed: main visible window was not found
operation-result.operation_id: auv.minecraft.query_wired_live_action
known_limits includes: mc19_v1_d4_query_wired_live_action_non_stub_click_no_gameplay_verification
```

### 2. answer_non_clickable — outside_window refusal, no dispatch

- run: `run_1782590246310_18190_0`
- wiring: `attempted=false`, `action_eligibility=answer_non_clickable`
- no `input.clickWindowPoint` child span
- `operation-result` message: `visibility=outside_window`

Command:

```sh
cargo run --quiet --example mc19_query_wired_live_action -- \
  --semantic-manifest .tmp/mc18-live/setup/semantic.json \
  --target-block 511,73,728 \
  --target-face north \
  --target-semantics hit_face_center \
  --query-provider closed-scene-toy \
  --closed-scene-fixture crates/auv-game-minecraft/tests/fixtures/mc18/outside_window.json \
  --output-dir .tmp/mc19-live/answer-non-clickable/query-output \
  --target-app com.todesktop.230313mzl4w4u92 \
  --target-title Cursor \
  --store-root .tmp/mc19-live/store
```

### 3. not_consumable — absent target, no dispatch

- run: `run_1782590246843_18194_0`
- wiring: `attempted=false`, `action_eligibility=not_consumable`
- no `input.clickWindowPoint` child span
- `operation-result` message: `status=failed reason=target_block_absent_from_scene_packet`

Command:

```sh
cargo run --quiet --example mc19_query_wired_live_action -- \
  --semantic-manifest .tmp/mc18-live/setup/semantic.json \
  --target-block 9,9,9 \
  --target-semantics hit_face_center \
  --output-dir .tmp/mc19-live/not-consumable/query-output \
  --target-app com.todesktop.230313mzl4w4u92 \
  --target-title Cursor \
  --store-root .tmp/mc19-live/store
```

## Verdict

| Gate | Run ID | Expected wiring | Dispatch evidence | Pass |
| --- | --- | --- | --- | --- |
| click_ready | `run_1782590245467_18186_0` | attempted + click_ready | non-stub `input.clickWindowPoint` span | yes |
| answer_non_clickable | `run_1782590246310_18190_0` | refused + outside_window | no click span | yes |
| not_consumable | `run_1782590246843_18194_0` | refused + MC-12 failed reason | no click span | yes |

**D4 closed** for MC-19 v1 wiring honesty. **D5 closed** for inspect / viewer MC-19 lineage polish.

## Honest limits

- `mc19_v1_d4_query_wired_live_action_non_stub_click_no_gameplay_verification` on all three runs
- Window title + `main_visible` resolve can fail even when bundle-only click succeeds; wired path records the honest driver refusal
- No Minecraft semantic success claim; activation-only input delivery when click resolves
- `--candidate` JSON promotion for `input.clickWindowPoint` remains deferred

## Related

- Design: `docs/ai/references/2026-06-27-minecraft-mc19-query-to-live-click-wiring-design.md`
- MC-18 provider closure: `docs/ai/references/2026-06-27-minecraft-mc18-closed-scene-toy-provider-live-closure.md`
- MC-20 verification live evidence (canonical CLI): [`2026-06-30-minecraft-mc20-d2-1-canonical-cli-live-closure.md`](2026-06-30-minecraft-mc20-d2-1-canonical-cli-live-closure.md)
- MC-14 action readiness closure: `docs/ai/references/2026-06-27-minecraft-mc14-spatial-query-action-facing-live-closure.md`

## D5 inspect polish (2026-06-28)

Terminal `inspect_run` now renders:

```text
MC-19 Query Wired Live Action:
- operation_result_artifact=… query_artifact=… attempted=… action_eligibility=… window_point=… refusal_reason=… operation_status=… operation_message=… dispatch_command=… dispatch_outcome=… target_app=… target_title=… mc14_action_eligibility=… issue=…
```

Recorded snippets from `.tmp/mc19-live/store`:

### click_ready (`run_1782590245467_18186_0`)

```text
MC-19 Query Wired Live Action:
- operation_result_artifact=artifact_0003 query_artifact=artifact_0001 attempted=true action_eligibility=click_ready window_point=640,360 refusal_reason=n/a operation_status=completed operation_message=Command invocation failed after run creation. Inspect run_1782590245467_18186_0 for the recorded trace. dispatch_command=input.clickWindowPoint dispatch_outcome=failed: command input.clickWindowPoint handler failed: main visible window was not found target_app=com.todesktop.230313mzl4w4u92 target_title=Cursor mc14_action_eligibility=click_ready issue=n/a
```

### answer_non_clickable (`run_1782590246310_18190_0`)

```text
MC-19 Query Wired Live Action:
- operation_result_artifact=artifact_0003 query_artifact=artifact_0001 attempted=false action_eligibility=answer_non_clickable window_point=n/a refusal_reason=visibility=outside_window operation_status=completed operation_message=visibility=outside_window dispatch_command=n/a dispatch_outcome=n/a target_app=com.todesktop.230313mzl4w4u92 target_title=Cursor mc14_action_eligibility=answer_non_clickable issue=n/a
```

### not_consumable (`run_1782590246843_18194_0`)

```text
MC-19 Query Wired Live Action:
- operation_result_artifact=artifact_0003 query_artifact=artifact_0001 attempted=false action_eligibility=not_consumable window_point=n/a refusal_reason=status=failed operation_status=completed operation_message=status=failed reason=target_block_absent_from_scene_packet dispatch_command=n/a dispatch_outcome=n/a target_app=com.todesktop.230313mzl4w4u92 target_title=Cursor mc14_action_eligibility=not_consumable issue=n/a
```

Viewer: select `operation-result` for MC-19 summary card; select spatial query manifest for paired `wired_action_*` rows when a same-run MC-19 operation result links via `evidence_artifacts` / `freshness_basis.source_artifact`.
