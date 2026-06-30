# SceneBridge A6: NetEase ViewMemory Live Evidence Closure

**Date:** 2026-06-30
**Status:** **owner-approved A6 docs-only** — live evidence protocol + sign-off template;
does **not** flip `AUV_NETEASE_VIEW_MEMORY`, remove NOTICE, or change Rust/proto/MCP.

**Prior work:** [A3 handoff](2026-06-30-auv-scenebridge-a3-implementation-handoff.md) →
[A4 closure](2026-06-30-auv-scenebridge-a4-closure.md) →
[A5 inspect identity charter](2026-06-30-auv-scenebridge-a5-inspect-identity-proof-charter.md)

## One-line summary

**PARTIAL** — hermetic gate green and live protocol/sign-off are ready, but Cases A–E
were **not** executed on a successful desktop scan in this session; owner must run the
matrix on a visible NetEase sidebar before PASS.

Question: does `AUV_NETEASE_VIEW_MEMORY=1` make real `playlist ls → select` use
ViewMemory reacquire and honestly fall back on stale/miss/missing/gate-off?

Answer: **PARTIAL** (protocol ready; live matrix pending owner execution).

## Owner freeze block

```text
hermetic：fmt/check + auv-view memory (16) + playlist_select (7) — PASS @ 374c0210
live probe：playlist ls returned match_count=0; ViewMemory write skipped — Cases A–E blocked
hit signal：reacquire.outcome=reacquired + skipped_rescan_replay=true + no scroll-sidebar-top-*
fallback：stale/not_found/missing/gate-off → known_limits + rescan replay steps
wire：reacquired / stale / not_found (not hit)
gate：remains default-off; A3e NOTICE removal deferred
```

## Acceptance matrix results

| Case | Expected | Result (2026-06-30 session) |
| --- | --- | --- |
| **A Hit** | `reacquired`, skip top-scroll replay | **pending owner execution** |
| **B Miss** | `not_found`, rescan replay | **pending owner execution** |
| **C Stale** | `stale` + wire `stale_reason` | **pending owner execution** |
| **D Memory missing** | `reacquire=null`, missing limit | **pending owner execution** |
| **E Gate off** | `reacquire=null`, legacy replay | **pending owner execution** |

Live probe blocker: empty sidebar scan (`view memory write skipped: scan did not produce
writable ViewMemory`). Agent session could not fabricate PASS live JSON.

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
| `live/case-*.json` | **Not attached** — pending owner live run |
| [`live/examples/`](evidence/2026-06-30-scenebridge-netease-sidebar/live/examples/) | Structure exemplars only (`structure_exemplar`) |

**Git rev (hermetic gate):** `374c0210acc9a20808316dcc725de3e7b7f6a748`

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

- Owner executes Cases A–E per [`live/README.md`](evidence/2026-06-30-scenebridge-netease-sidebar/live/README.md)
  with NetEase foreground and a query yielding non-empty matches + writable ViewMemory.
- Attach redacted `case-a-hit-select.json` (and B/C as recommended) after owner run.
- If live Hit fails after owner run, open a **separate bug-fix slice** — not in A6.

## Done checklist (A6 docs-only)

- [x] Extended live README (Cases A–E, recipes, redaction, bash protocol)
- [x] `SIGNOFF.md` + `transcript.txt` (PARTIAL / pending)
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
