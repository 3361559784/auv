# SceneBridge A6 live sign-off

`proof_class: live`

**Date:** 2026-06-30 (A6b computer-use session)
**Git rev (hermetic gate):** `f0b04e0cdb674e7b79be6ed496c13294a59a66ac`
**Environment:** macOS 27.0 (arm64); NetEase foreground; **未登录**; 创建的歌单 0
**Closure:** [A6 live evidence closure](../../2026-06-30-auv-scenebridge-a6-live-evidence-closure.md)

## Hermetic pre-gate

| Check | Result |
| --- | --- |
| `cargo fmt --check` | PASS |
| `cargo check -p auv-view -p auv-netease-music` | PASS |
| `cargo test -p auv-view memory` | PASS (16 tests) |
| `cargo test -p auv-netease-music playlist_select` | PASS (7 tests) |
| `git diff --check` | PASS |

## Live acceptance matrix

| Case | Status | Notes |
| --- | --- | --- |
| **A Hit** | **blocked** | `item_count=0`; no playlist label for select |
| **B Miss** | **blocked** | Depends on Case A baseline |
| **C Stale** | **blocked** | Depends on Case A baseline |
| **D Memory missing** | **blocked** | Depends on Case A baseline |
| **E Gate off** | **blocked** | Depends on Case A baseline |

## A6b computer-use probe

Commands: `open-window`, `playlist ls --category all|favorite` (MacosDriver scroll + OCR).

| Signal | Observed |
| --- | --- |
| `item_count` / `match_count` | 0 / 0 |
| `view-memory-playlist_sidebar.json` | Written (probe attached) |
| `projection.sections[].items` | Empty (`创建的歌单0` header only) |
| `playlist select` | `no playlist matched` |

Probe attachments (blocker evidence, not Case PASS):

- [`case-ls-probe.json`](case-ls-probe.json)
- [`view-memory-playlist_sidebar-probe.json`](view-memory-playlist_sidebar-probe.json)

## Conclusion

**PARTIAL** — AUV computer use ran; hermetic gate green. Cases A–E **not executed**
because sidebar has no playlist items (guest / zero-playlist account state).

**Owner unblock:** log in + at least one sidebar playlist name, then re-run matrix.

Gate remains default-off; NOTICE removal deferred.
