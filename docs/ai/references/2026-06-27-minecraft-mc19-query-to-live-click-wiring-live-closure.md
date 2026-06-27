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
- Live gate harness: `examples/mc19_query_wired_live_action.rs`
- Dedicated store for closure runs: `.tmp/mc19-live/store`

## Harness command shape

```sh
cargo run --quiet --example mc19_query_wired_live_action -- \
  --semantic-manifest .tmp/mc18-live/setup/semantic.json \
  --target-block <x,y,z> \
  [--target-face north] \
  --target-semantics hit_face_center \
  [--closed-scene-fixture <fixture.json>] \
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

**D4 closed** for MC-19 v1 wiring honesty. Gameplay verification and inspect MC-19 lineage polish remain deferred (D5).

## Honest limits

- `mc19_v1_d4_query_wired_live_action_non_stub_click_no_gameplay_verification` on all three runs
- Window title + `main_visible` resolve can fail even when bundle-only click succeeds; wired path records the honest driver refusal
- No Minecraft semantic success claim; activation-only input delivery when click resolves
- `--candidate` JSON promotion for `input.clickWindowPoint` remains deferred

## Related

- Design: `docs/ai/references/2026-06-27-minecraft-mc19-query-to-live-click-wiring-design.md`
- MC-18 provider closure: `docs/ai/references/2026-06-27-minecraft-mc18-closed-scene-toy-provider-live-closure.md`
- MC-14 action readiness closure: `docs/ai/references/2026-06-27-minecraft-mc14-spatial-query-action-facing-live-closure.md`
