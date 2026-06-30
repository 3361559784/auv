# SceneBridge A6: NetEase ViewMemory Live Evidence Closure

**Date:** 2026-06-30
**Status:** **owner-approved A6 docs-only** — live evidence protocol + sign-off template;
does **not** flip `AUV_NETEASE_VIEW_MEMORY`, remove NOTICE, or change Rust/proto/MCP.

**Prior work:** [A3 handoff](2026-06-30-auv-scenebridge-a3-implementation-handoff.md) →
[A4 closure](2026-06-30-auv-scenebridge-a4-closure.md) →
[A5 inspect identity charter](2026-06-30-auv-scenebridge-a5-inspect-identity-proof-charter.md)

## One-line summary

**PARTIAL** — hermetic gate green; A6b used AUV computer use (`open-window`, sidebar
scan) but Cases A–E remain **blocked** on `item_count=0` (未登录 / 创建的歌单 0).

Question: does `AUV_NETEASE_VIEW_MEMORY=1` make real `playlist ls → select` use
ViewMemory reacquire and honestly fall back on stale/miss/missing/gate-off?

Answer: **PARTIAL** (A6b probe: computer use OK; matrix blocked on account state).

## Owner freeze block

```text
hermetic：fmt/check + auv-view memory (16) + playlist_select (7) — PASS @ f0b04e0
live A6b：computer use ran; item_count=0; projection items empty — Cases A–E blocked
hit signal：reacquire.outcome=reacquired + skipped_rescan_replay=true + no scroll-sidebar-top-*
fallback：stale/not_found/missing/gate-off → known_limits + rescan replay steps
wire：reacquired / stale / not_found (not hit)
gate：remains default-off; A3e NOTICE removal deferred
```

## Acceptance matrix results

| Case | Expected | Result (2026-06-30 session) |
| --- | --- | --- |
| **A Hit** | `reacquired`, skip top-scroll replay | **blocked** (no playlist items) |
| **B Miss** | `not_found`, rescan replay | **blocked** |
| **C Stale** | `stale` + wire `stale_reason` | **blocked** |
| **D Memory missing** | `reacquire=null`, missing limit | **blocked** |
| **E Gate off** | `reacquire=null`, legacy replay | **blocked** |

A6b blocker: guest account / zero sidebar playlists → `item_count=0`,
`projection.sections[].items` empty. ViewMemory file may still write; `playlist select`
cannot match. See `case-ls-probe.json`.

## Slice classification

| Item | Value |
| --- | --- |
| This note (A6 closure) | **docs-only** |
| Live execution | **owner-labeled** (`proof_class: live`), not CI |
| Hermetic gate | Required pre-condition — **PASS** |
| Not | Rust/proto/MCP, gate default-on, NOTICE removal, run-storage, trace spans, Q5 |

## Evidence attachments

| Path | Status |
| --- | --- |
| [`live/README.md`](evidence/2026-06-30-scenebridge-netease-sidebar/live/README.md) | Protocol + matrix + recipes |
| [`live/SIGNOFF.md`](evidence/2026-06-30-scenebridge-netease-sidebar/live/SIGNOFF.md) | Matrix checkboxes + env |
| [`live/transcript.txt`](evidence/2026-06-30-scenebridge-netease-sidebar/live/transcript.txt) | Redacted hermetic + partial probe |
| `live/case-*.json` | **Not attached** — Cases A–E blocked |
| [`live/case-ls-probe.json`](evidence/2026-06-30-scenebridge-netease-sidebar/live/case-ls-probe.json) | A6b blocker probe |
| [`live/view-memory-playlist_sidebar-probe.json`](evidence/2026-06-30-scenebridge-netease-sidebar/live/view-memory-playlist_sidebar-probe.json) | A6b probe snapshot |
| [`live/examples/`](evidence/2026-06-30-scenebridge-netease-sidebar/live/examples/) | Structure exemplars only (`structure_exemplar`) |

**Git rev (hermetic gate):** `f0b04e0cdb674e7b79be6ed496c13294a59a66ac`

## Anti-misread rules (A6)

1. **Reacquire does not skip the live scan before select** — `resolve_playlist_target_for_query`
   always runs; reacquire only optimizes scroll replay when memory loads
   ([`playlist.rs`](../../../crates/auv-netease-music/src/commands/playlist.rs) L362–418).
2. **`reacquire.outcome` wire values** are `reacquired` / `stale` / `not_found` —
   [`outcome_label`](../../../crates/auv-view/src/memory/reacquire_adapter.rs) — not `hit`.
3. **Hermetic FakeAdapter tests ≠ live proof** (A5 #6); `examples/` JSON is
   `structure_exemplar` only.
4. **Live evidence surface** is CLI JSON + artifact-dir files — no `view.reacquire.*`
   spans, no run-storage `view-memory` role (A5 Tier II–III).
5. **`known_limits` strings** supplement human-readable fallback; pair with structured
   `reacquire` fields for resolution proof (A5 #4).

## Sign-off template

```text
Question: AUV_NETEASE_VIEW_MEMORY=1 时 ls→select 是否真走 reacquire 并诚实回退？
Answer: PARTIAL
Hit: reacquire.outcome=reacquired + skipped_rescan_replay=true + no top-scroll replay
Fallback: stale/miss/missing/gate-off → known_limits + rescan replay steps
Gate: remains default-off; A3e NOTICE removal deferred to future owner slice
```

## Open items (PARTIAL only)

- Owner logs in; ensure at least one named sidebar playlist (not `创建的歌单 0` only).
- Re-run Cases A–E per [`live/README.md`](evidence/2026-06-30-scenebridge-netease-sidebar/live/README.md).
- Attach `case-a-hit-select.json` (and B–E) after successful matrix.
- If live Hit fails after owner run, open a **separate bug-fix slice** — not in A6.

## Done checklist (A6 docs-only)

- [x] Extended live README (Cases A–E, recipes, redaction, bash protocol)
- [x] A6b computer-use probe + blocker artifacts (`case-ls-probe.json`)
- [x] Structure exemplars under `live/examples/` (labeled, not live proof)
- [x] Hermetic pre-gate PASS
- [ ] Cases A–E live PASS on owner Mac
- [x] `git diff --check` before commit

## Explicit non-goals (A6)

- Rust / proto / MCP changes
- Default-on `AUV_NETEASE_VIEW_MEMORY` or NOTICE removal
- Run-storage `view-memory` role + real `source_run_id`
- `view.reacquire.*` trace spans
- Q5 cross-app comparison
- Select skipping live scan (undesigned; out of scope)

## Related

- [A3 implementation handoff](2026-06-30-auv-scenebridge-a3-implementation-handoff.md)
- [A4 closure](2026-06-30-auv-scenebridge-a4-closure.md)
- [A5 inspect identity charter](2026-06-30-auv-scenebridge-a5-inspect-identity-proof-charter.md)
- [Evidence folder](evidence/2026-06-30-scenebridge-netease-sidebar/)
- [anchor-reacquisition-v0](2026-05-29-view-parser-anchor-reacquisition-v0.md)
