# SceneBridge A4: Stale Outcome Closure

**Date:** 2026-06-30  
**Status:** **owner-approved A4-min** — closes the stale reacquire gap left by A3-min;
does **not** open run-storage migration, `ViewNodeId`, AX stages 2/6, or trait
extraction.

**Prior work:** [A3 implementation handoff](2026-06-30-auv-scenebridge-a3-implementation-handoff.md)
(landed) → this note locks the smallest follow-up before A5-scale work.

## One-line summary

A4-min makes `reacquire()` distinguish **stale memory** from **not found**, wires
NetEase `playlist select` to surface that distinction, and proves the happy path
with an injected adapter test — still on artifact-dir bridge + feature gate.

## Owner freeze block

```text
outcomes：ReacquireOutcome::Stale { reason } + RegionGone / ObservationFailed stale reasons
freshness：checked at reacquire() entry via ReacquireConfig::memory_read (not silent load drop)
region gone：successful observes but zero candidates → RegionGone; all observes fail → ObservationFailed
netease wire：load_memory_raw + try_reacquire_playlist_target; select distinguishes Stale vs Miss
proof：playlist_select_uses_reacquire_when_memory_hit (FakeAdapter, no MacosDriver)
deferred：run-storage bridge, stages 2/6, ViewNodeId, RegionParser trait
```

## Slice classification

| Item | Value |
| --- | --- |
| This note (A4 closure) | **docs-only** |
| `auv-view` stale outcome | **owner-approved feature** |
| `auv-netease-music` wire + test | **owner-approved feature** |
| Not | run-storage, promotion, default gate flip |

## `auv-view` changes

| Path | Change |
| --- | --- |
| `memory/read.rs` | `StaleReason::RegionGoneAtReacquisition` |
| `memory/reacquire.rs` | `ReacquireConfig { memory_read, current_baseline_width }`; entry freshness → `Stale`; cascade tracks `saw_any_candidates` |
| `memory/store.rs` | `parse_memory_file()` for hermetic injection |
| `memory/mod.rs` | re-export `StaleReason`, `parse_memory_file` |

**Tests added:**

- `reacquire_stale_on_freshness_rejection`
- `reacquire_stale_when_region_gone`
- `reacquire_not_found_when_candidates_exist_but_no_match`

## `auv-netease-music` changes

| Path | Change |
| --- | --- |
| `view_memory.rs` | `load_memory_raw`, `try_reacquire_playlist_target`, `PlaylistReacquireAttempt` |
| `view_parsers/sidebar/reacquire.rs` | `try_reacquire_for_target` builds live adapter, delegates to `try_reacquire_playlist_target` |
| `commands/playlist.rs` | select uses raw load; stale vs miss in `known_limits` |

**Tests added:**

- `playlist_select_uses_reacquire_when_memory_hit`
- `playlist_select_falls_back_on_stale_memory`

**NOTICE:** injected tests must use `SidebarSectionKind::domain_kind()` strings
(e.g. `netease.favorite_playlists`) — not shorthand section names.

## Done checklist (A4-min)

- [x] `cargo test -p auv-view memory` green (14 tests)
- [x] `playlist_select_uses_reacquire_when_memory_hit` green
- [x] `playlist_select_falls_back_on_stale_memory` green
- [x] `cargo fmt --check` / `cargo check -p auv-view -p auv-netease-music`
- [ ] `git diff --check` on docs
- [ ] Commits land per handoff convention (docs → auv-view → netease)

## Explicit non-goals (A4)

- Run-storage `view-memory` role migration
- AX reacquire stages 2 and 6
- `ViewNodeId` or `CandidateRef` promotion
- `RegionParser` / `ItemParser` trait extraction
- Default-on `AUV_NETEASE_VIEW_MEMORY` or NOTICE removal

## Related

- [A3 boundary review](2026-06-30-auv-scenebridge-a3-prototype-boundary-review.md)
- [A3 implementation handoff](2026-06-30-auv-scenebridge-a3-implementation-handoff.md)
- [anchor-reacquisition-v0](2026-05-29-view-parser-anchor-reacquisition-v0.md)
