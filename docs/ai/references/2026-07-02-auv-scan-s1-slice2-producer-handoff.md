# AUV Scan S1 Slice 2: Producer Wiring — Implementation Handoff

**Date:** 2026-07-02  
**Status:** landed — fixture-first producer + shared artifact bundle writer  
**Prerequisite:** [S1 slice 1 handoff](2026-07-02-auv-scan-s1-slice1-frame-contract-handoff.md), [S1-2/3/4 plan](2026-07-02-auv-scan-s1-s2-s4-producer-read-temporal-plan.md)

## Boundary

**Owning crate:** `crates/auv-scan` only. **No** changes to `scroll_scan`, `run_read`, runtime, CLI, or viewer.

**Option A locked:** fixture-first hermetic producer; `auv_driver::Capture` mapping for in-memory/live path. **Rejected:** scroll_scan coupling, driver-direct artifact write, multi-source trait.

## Owner 审查卡点（五条对照）

| # | 卡点 | 实现 |
| --- | --- | --- |
| 1 | 同一路径产物 | `produce_frame_from_fixture_dir` 与 `produce_frame_from_capture`（feature）均调用 `write_frame_with_image` → `write_frame_artifact` |
| 2 | 失败语义 | Fail-closed：JSON 写失败时删除已写 PNG；无 degraded/partial wire |
| 3 | Non-goals | 见下文 |
| 4 | Merge gate | `cargo test -p auv-scan`（16 tests，无 `live-capture` feature） |
| 5 | Shared 抽取 | 未抽离；ownership 在 `auv-scan`；仅第二个真实 producer 需要时才允许外抽 |

## Stable public API (new in slice 2)

| Symbol | Role |
| --- | --- |
| `ScanProducerError` | Producer errors (`MissingImage`, `ZeroImageDimension`, wraps `ScanArtifactError`) |
| `FrameCaptureMeta` | Capture-site metadata for frame build |
| `ProducedFrame` | `{ json_path, image_path, frame }` |
| `build_scan_frame(meta, w, h)` | Pure frame builder + validate |
| `bounds_to_scan_bounds` / `bounds_to_scan_bounds_f64` | Driver rect → `ScanBounds` |
| `frame_from_capture(capture, meta)` | `Capture` → `ScanFrame` (memory only) |
| `write_frame_with_image(dir, frame, bytes)` | PNG then JSON; rollback PNG on JSON failure |
| `produce_frame_from_fixture_dir(fixture_dir, out_dir)` | Hermetic producer (merge gate) |
| `produce_frame_from_capture` | Behind `live-capture` feature; same write path |

Slice 1 API unchanged.

## Write order

1. Validate `ScanFrame`
2. Write PNG (`image.file_name`)
3. Write `scan-frame-NNNN.json` via `write_frame_artifact`
4. On step 3 failure → remove PNG

## Error variants (`ScanProducerError`)

| Variant | When |
| --- | --- |
| `Artifact` | `ScanArtifactError` from validate/write/read |
| `MissingImage { path }` | Fixture PNG absent |
| `ZeroImageDimension` | width or height 0 |
| `Io` | Filesystem |
| `Json` | Manifest parse |

## Features

| Feature | Default | Purpose |
| --- | --- | --- |
| `live-capture` | off | Enables `producer::live::produce_frame_from_capture` |

Default build: `auv-driver` dep for `frame_from_capture` only (memory-testable).

## Tests (9 new producer + 7 slice 1 = 16 total)

| Test | Assert |
| --- | --- |
| `produce_frame_from_fixture_dir_matches_golden` | == golden JSON |
| `produce_frame_from_fixture_dir_writes_png_sibling` | PNG exists |
| `write_frame_with_image_roundtrip` | read == frame |
| `bounds_to_scan_bounds_rounding_table` | rounding |
| `produce_frame_from_fixture_dir_rejects_missing_png` | variant; no artifacts in out_dir |
| `produce_failure_leaves_no_partial_artifact` | no orphan PNG after JSON blocked |
| `build_scan_frame_rejects_zero_dimension` | `ZeroImageDimension` |
| `frame_from_capture_builds_scan_frame_from_rgba` | 8×8 from `RgbaImage` |
| `frame_from_capture_rejects_zero_dimension` | `ZeroImageDimension` |

Live: `produce_frame_from_capture_writes_artifact` — `#[ignore = "live"]`, requires `--features live-capture`.

## Non-goals (this slice)

- `scroll_scan` / runtime / CLI / viewer / `run_read`
- multi-frame batch, motion, tracks, compare API
- degraded/partial artifacts
- cross-crate shared producer trait

## S1-3 prerequisite

S1-2 produces directories containing `scan-frame-0001.json` + sibling PNG for read-side slice.

## Validation

```sh
cargo fmt --check
cargo check -p auv-scan
cargo test -p auv-scan
git diff --check
```

## Related

- [S1 slice 1 handoff](2026-07-02-auv-scan-s1-slice1-frame-contract-handoff.md)
- [S1-2/3/4 plan](2026-07-02-auv-scan-s1-s2-s4-producer-read-temporal-plan.md)
- [GAN spec](2026-07-02-auv-scan-s1-s2-s4-gan-spec.md)
