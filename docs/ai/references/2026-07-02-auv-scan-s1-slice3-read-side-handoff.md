# AUV Scan S1 Slice 3: Read-side Consume — Implementation Handoff

**Date:** 2026-07-03  
**Status:** implemented — crate-local reader for `scan-frame-v0` artifact directories  
**Prerequisite:** [S1 slice 1 handoff](2026-07-02-auv-scan-s1-slice1-frame-contract-handoff.md), [S1 slice 2 handoff](2026-07-02-auv-scan-s1-slice2-producer-handoff.md), [S1-2/3/4 plan](2026-07-02-auv-scan-s1-s2-s4-producer-read-temporal-plan.md)

## Boundary

**Owning crate:** `crates/auv-scan` only. **No** changes to `scroll_scan`, `run_read`, runtime, CLI, or viewer.

**Substrate alignment:** Completes the S1 batch acceptance **read + verify** half of frame binding + artifact contract ([S-line substrate](2026-07-03-s-line-streaming-observation-substrate.md)). Substrate **S3** (coverage ledger) remains future work.

## Owner 审查卡点（三条对照）

| # | 卡点 | 实现 |
| --- | --- | --- |
| 1 | 测试辅助不污染公开 API | `FrameFieldExpectation` / `assert_frame_matches_expectation` in `reader::test_support`, `#[cfg(test)]` only — **not** re-exported from `lib.rs` |
| 2 | `summarize` 职责单一 | `summarize_scan_frame_text(frame)` reads in-memory `ScanFrame` only — **no** PNG IO |
| 3 | 顺序语义不掩盖错误 | Duplicate `sequence_index` → `DuplicateSequenceIndex` before sort; post-sort `found <= previous` → `NonMonotonicSequenceIndex`; filename lexicographic tie-break is determinism only |

## Stable public API (new in slice 3)

| Symbol | Role |
| --- | --- |
| `ScanFrameBundle` | `{ frames, source_dir, loaded_json_paths }` from a directory load |
| `ScanInspectError` | Read-side errors (see below) |
| `load_scan_frames_from_dir(dir)` | Glob top-level `scan-frame-*.json`, validate, order, fail on duplicates |
| `verify_frame_image_dimensions(source_dir, frame)` | **Only** reader entry that reads PNG bytes for dimension check |
| `summarize_scan_frame_text(frame)` | Metadata-only one-line summary (no disk IO) |

Slice 1–2 API unchanged.

**Not stable public API:** `FrameFieldExpectation`, `assert_frame_matches_expectation` — test-only in `reader.rs`.

## Error variants (`ScanInspectError`)

| Variant | When |
| --- | --- |
| `Artifact` | `ScanArtifactError` from `read_frame_artifact` |
| `NoFramesFound` | Directory has no matching `scan-frame-*.json` |
| `ImageFileMissing { path }` | PNG sibling absent during verify |
| `ImageDimensionMismatch { expected_w, expected_h, actual_w, actual_h }` | Wire vs PNG size mismatch |
| `DuplicateSequenceIndex { index, first_file, second_file }` | Two artifacts share `sequence_index` |
| `NonMonotonicSequenceIndex { previous, found }` | Post-sort order violation (belt-and-suspenders after duplicate check) |
| `Io` | Filesystem / image decode |

## Load semantics

1. `read_dir` top level only; match `scan-frame-<digits>.json`
2. `read_frame_artifact` each match
3. Empty set → `NoFramesFound`
4. HashMap duplicate `sequence_index` check → `DuplicateSequenceIndex`
5. Sort by `sequence_index`, then filename lexicographic tie-break
6. Adjacent pairs: `found <= previous` → `NonMonotonicSequenceIndex`

## Tests (9 new reader + 16 prior = 25 total)

| Test | Input source | Assert |
| --- | --- | --- |
| `load_scan_frames_from_dir_reads_golden_directory` | Temp copy of `single_frame_v0/golden` JSON + PNG | `len == 1`; matches golden |
| `load_scan_frames_from_dir_sorts_by_sequence_index` | Temp: 2 JSON (index 0/1, write order reversed) + PNGs | 0 → 1 |
| `load_scan_frames_from_dir_rejects_duplicate_sequence_index` | Temp: 2 JSON same index | `DuplicateSequenceIndex` |
| `verify_frame_image_dimensions_matches_png` | Golden copy dir | `Ok` |
| `verify_frame_image_dimensions_rejects_mismatch` | Golden copy + in-memory height change | `ImageDimensionMismatch` |
| `load_scan_frames_from_dir_empty_dir_errors` | Empty temp dir | `NoFramesFound` |
| `load_scan_frames_from_dir_rejects_bad_schema` | Temp bad schema artifact | `SchemaMismatch` via `Artifact` |
| `summarize_scan_frame_text_includes_key_fields` | In-memory `ScanFrame` from fixture builder | Key fields present; no verify |
| `producer_then_reader_roundtrip` | **Only** `produce_frame_from_fixture_dir` output | load + verify |

Each test uses a **single fixed input source** (no golden-or-producer ambiguity).

## Non-goals (this slice)

- `run_read` / `inspect_server` / viewer
- Wire / `scan-frame-v0` schema changes
- Tracks, coverage ledger, motion, compare API, traits
- Cross-frame temporal questions (substrate S0)
- PNG reads inside `summarize_scan_frame_text`

## S1-4 prerequisite

S1-3 loads and verifies single- or multi-frame artifact directories produced by S1-2 (or golden copies). Multi-frame temporal outline is the next plan slice.

## Validation

```sh
cargo fmt --check
cargo check -p auv-scan
cargo test -p auv-scan
git diff --check
```

Merge gate: **25** tests pass (default features, no `live-capture`).

## Related

- [S1 slice 1 handoff](2026-07-02-auv-scan-s1-slice1-frame-contract-handoff.md)
- [S1 slice 2 handoff](2026-07-02-auv-scan-s1-slice2-producer-handoff.md)
- [S1-2/3/4 plan](2026-07-02-auv-scan-s1-s2-s4-producer-read-temporal-plan.md)
- [GAN spec](2026-07-02-auv-scan-s1-s2-s4-gan-spec.md)
- [S-line substrate](2026-07-03-s-line-streaming-observation-substrate.md)
