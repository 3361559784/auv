# SceneBridge A6 live sign-off

`proof_class: live`

**Date:** 2026-06-30
**Git rev (hermetic gate):** `374c0210acc9a20808316dcc725de3e7b7f6a748`
**Environment:** macOS 27.0 (arm64); agent session attempted live probe
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
| **A Hit** | **pending owner execution** | Requires writable view-memory + stable viewport |
| **B Miss** | **pending owner execution** | Manual scroll-away recipe |
| **C Stale** | **pending owner execution** | TTL or baseline drift edit |
| **D Memory missing** | **pending owner execution** | Delete view-memory after `ls` |
| **E Gate off** | **pending owner execution** | `AUV_NETEASE_VIEW_MEMORY` unset |

## Live probe (agent session, 2026-06-30)

Partial attempt: `playlist ls "test"` with gate=1 returned `match_count: 0` and
`view memory write skipped: scan did not produce writable ViewMemory`. Cases A–E
were **not** executable in this session (empty sidebar scan).

## Conclusion

**PARTIAL** — protocol, redaction rules, and sign-off template are ready; live
matrix cases remain **pending owner execution** on a Mac with NetEase visible and
a non-empty sidebar scan.

Gate remains default-off; NOTICE removal deferred to a future owner slice.
