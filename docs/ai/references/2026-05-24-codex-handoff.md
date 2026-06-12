# Codex Handoff: osu! Benchmark Mainline

Date: 2026-06-12

Status: current handoff for the next coding agent before session compaction

Current HEAD when written: `04ef4de`

## Start Here

Read these files first, in this order:

1. `AGENTS.md`
2. `CLAUDE.md`
3. `docs/TERMS_AND_CONCEPTS.md`
4. `crates/auv-game-osu/src/benchmark.rs`
5. `src/osu.rs`
6. `src/cli.rs`
7. `src/main.rs`
8. `crates/auv-driver/src/input.rs`
9. `crates/auv-driver-macos/src/session.rs`

## Current Goal

Active goal is still:

```text
AUV realtime benchmark lane for osu!, benchmark-first rather than YOLO-first
```

There is no new goal beyond this lane.

Current shape of the lane:

- `P0`: beatmap-driven offline scheduler benchmark — **done as merged skeleton**
- `P1`: typed macOS window dispatch benchmark mode — **done as merged slice**
- `P2`: capture / visual verification — **done as merged slice with local real-app smoke verification**
- `P2.5`: capture timestamp semantics — **done as merged slice with local real-app smoke verification and pushed to main**
- `P3a`: visual dataset / evaluation harness from beatmap truth + corrected timestamped captures — **done as merged slice with local build/test verification**
- `P3`: YOLO/CV as independent validation channel — **not started**

## Current Repo State

Current branch state when written:

```text
main...origin/main
```

Recent commits that matter:

```text
04ef4de feat(osu): add visual truth manifest from beatmap and capture traces
7c63900 fix(osu): correct capture timestamp semantics
33bdabf feat(osu): add capture verification evidence to dispatch benchmark
4d7f06a feat(osu): add typed dispatch benchmark mode
54394b4 feat(osu): add beatmap benchmark skeleton
```

Before coding again, verify live state:

```bash
git status --short --branch
git log --oneline --decorate -5
```

When this handoff was written, `main` matched `origin/main`; the handoff file itself was then refreshed to record that pushed state.

## What Was Completed In This Session

### P0 merged

Commit:

```text
54394b4 feat(osu): add beatmap benchmark skeleton
```

What it did:

- added `crates/auv-game-osu`
- added `rosu-map` based local `.osu` parsing
- generated deterministic action schedules from beatmap truth
- added dry-run timing benchmark output
- emitted artifacts:
  - `parsed_map_summary.json`
  - `action_schedule.json`
  - `dispatch_trace.json`
  - `latency_report.json`
- added CLI entry:

```text
auv-cli osu benchmark <beatmap.osu> [--output-dir <dir>]
```

### P1 merged

Commit:

```text
4d7f06a feat(osu): add typed dispatch benchmark mode
```

What it did:

- extended `RunMode` beyond `DryRun` to include typed dispatch
- extended benchmark inputs to carry:
  - target app
  - dispatch limit
- added typed macOS dispatch path through:

```text
MacosDriver::new()
  -> open_local()
  -> session.window().resolve(...)
  -> session.window().click(...)
```

- extended dispatch trace records with:
  - `delivery_path`
  - `fallback_reason`
- added CLI entry:

```text
auv-cli osu dispatch <beatmap.osu> --target-app <name> [--output-dir <dir>] [--dispatch-limit <n>]
```

### P2 merged

Commit:

```text
33bdabf feat(osu): add capture verification evidence to dispatch benchmark
```

What it did:

- extends typed dispatch benchmark inputs with `capture_verify`
- captures window evidence around each dispatched action
- emits new artifacts:
  - `capture_trace.json`
  - `verification_summary.json`
  - staged `capture-object-*.png` frame evidence
- stages both JSON and PNG evidence into the normal recorded run artifact layout
- extends CLI entry:

```text
auv-cli osu dispatch <beatmap.osu> --target-app <name> [--output-dir <dir>] [--dispatch-limit <n>] [--capture-verify]
```

Smoke verification completed locally against installed `osu!.app` and a real local beatmap file.

- `src/osu.rs` wraps benchmark execution through `Runtime::run_recorded_operation(...)`
- artifacts are staged into the normal `.auv/runs/<run_id>/` layout
- inspect/read surfaces remain reusable without a special osu persistence path

This preserves the active AUV core lane instead of forking a private benchmark recorder.

### P2.5 merged

Commit:

```text
7c63900 fix(osu): correct capture timestamp semantics
```

What it did:

- adds `pre_capture_offset_ms` with default `16`
- moves before capture to `scheduled_time_ms - pre_capture_offset_ms`
- treats `post_capture_offsets_ms` as absolute offsets relative to `actual_dispatch_time_ms`
- splits capture timing into both:
  - `relative_to_scheduled_ms`
  - `relative_to_dispatch_ms`
- keeps `VerificationSummary` aligned with dispatch-time semantics instead of mixing scheduled and dispatch references

Local real-app smoke re-run passed against installed `osu!.app` and a real local beatmap file before the commit was pushed.

- run id: `run_1781278300171_81552_0`
- output dir: `.tmp-osu-dispatch-p25`
- `verificationCapturedActions: 1`
- `verificationMissingFrames: 0`
- produced files:
  - `capture-object-0000-before-16ms.png`
  - `capture-object-0000-after-16ms.png`
  - `capture-object-0000-after-48ms.png`
- `capture_trace.json` now records both scheduled-relative and dispatch-relative offsets

Observed timing on that smoke confirms why P2.5 matters: dispatch itself was late (`scheduled_time_ms = 151`, `actual_dispatch_time_ms = 1283`), so scheduled-relative and dispatch-relative capture semantics must stay separate and explicit.

### P3a implemented locally, not yet pushed

P3a landed as local commit `04ef4de` on top of `7c63900`. Validation passed
but the commit has not been pushed to `origin/main` yet.

What it did:

- adds `crates/auv-game-osu/src/visual_truth.rs` with `VisualTruthManifest`,
  `VisualTruthFrame`, `ExpectedObjectTruth`, and `build_visual_truth_manifest`
- joins schedule + dispatch + capture traces by `object_index`, cross-checks
  kind / scheduled-time / dispatch-time / dispatch-error consistency, and
  fails loudly on any mismatch instead of silently dropping frames
- expands every capture sample into a `VisualTruthFrame` carrying both
  scheduled-relative and dispatch-relative timing plus the beatmap-truth
  expected object (playfield x/y, CS/AR/OD)
- builds the manifest only when `capture_verify` is on, writes
  `visual_truth_manifest.json` next to the other run artifacts, and stages it
  through the existing recorded-operation artifact path in `src/osu.rs`
- exports the new types from `crates/auv-game-osu/src/lib.rs`

Boundaries held:

- beatmap truth stays the primary source; the manifest is offline evidence only
- reuses the existing P2/P2.5 capture artifacts, no parallel evidence path
- no YOLO control, training, LLM in hit loop, or new core-wide contract

## Verification Already Run

The following checks passed for the merged P2 state:

```bash
cargo fmt --check
cargo check
cargo test
cargo build
git diff --check
cargo run -- help | rg "osu benchmark|osu dispatch"
cargo run -- osu benchmark <beatmap.osu> [--output-dir <dir>]
cargo run -- osu dispatch <local beatmap> --target-app "osu!" --dispatch-limit 1 --capture-verify --output-dir .tmp-osu-dispatch-p2
auv-cli inspect run_1781276425182_80682_0
```

Additional verification for the merged P2.5 state:

```bash
cargo fmt --check
cargo check
cargo test
cargo build
git diff --check
cargo run --quiet -- osu benchmark <local beatmap> --output-dir .tmp-osu-benchmark-p25
cargo run --quiet -- osu dispatch <local beatmap> --target-app "osu!" --dispatch-limit 1 --capture-verify --output-dir .tmp-osu-dispatch-p25
```

Notes:

- one intermediate dispatch smoke failed only because the `osu!` window was not visible/resolvable at the time of launch
- the successful rerun produced `run_1781278300171_81552_0`
- in this shell, `auv-cli inspect` was not on `PATH`; inspect validation for P2.5 should therefore be rerun via the built binary path or `cargo run -- inspect ...` if needed

## Collabi State

Collabi was used during this lane.

Active session:

```text
auv-game-osu-p0
```

Claim used for the owned path set:

```text
auv-game-osu-p0-impl
```

The session was updated after both merged slices.

## Current Boundaries

Still true:

- benchmark-first, not YOLO-first
- strongest available signal wins
- `.osu` beatmap truth remains the primary source for scheduling
- no online or ranked automation
- no memory reader dependency in the merged state
- capture verification now exists as a separate evidence channel around typed dispatch
- YOLO/CV control path still does not exist
- osu-specific logic remains in `crates/auv-game-osu`, not in generic core runtime modules

## Next Single Best Step

Do not open a new goal.

`P3a` is implemented locally and validated but not yet pushed. The immediate
next step is to push the `P3a` commit to `origin/main`, then stop and let the
owner choose whether to open `P3`.

After `P3a` is pushed, the candidate next slice is `P3`:

1. introduce a YOLO/CV detector as an independent validation channel that reads
   the staged `visual_truth_manifest.json` frames offline
2. score detector output against beatmap truth instead of replacing it
3. keep the detector out of the hit loop; it stays an evidence/validation lane

If continuing into `P3`, preserve these rules:

- beatmap truth remains the primary source; vision stays a validation lane
- reuse the P3a manifest + P2/P2.5 capture artifacts instead of inventing a parallel evidence path
- do not put LLM or detector inference into the hit loop
- do not introduce a new core-wide contract without owner-approved design

## Useful Mental Model

The lane now proves two different things in sequence:

```text
P0: can AUV derive a deterministic action timeline from structured beatmap truth?
P1: can AUV send real typed macOS window clicks on that timeline and record delivery facts?
```

What is still unproven:

```text
can AUV capture and correlate visual feedback against those actions?
```

That is the natural P2 question.
