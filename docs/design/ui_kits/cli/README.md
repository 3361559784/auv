# `auv-cli` — UI Kit

High-fidelity recreation of `auv-cli` terminal output. The CLI is the primary
product surface today; this kit reproduces its layout, syntax coloring, and
status vocabulary so designers can place real-looking CLI blocks in mocks,
docs, and marketing surfaces.

**Source of truth:** `src/cli.rs` / `src/main.rs` / `recipes/macos/qqmusic/README.md`
in [`moeru-ai/auv`](https://github.com/moeru-ai/auv).

## What's in here

- `index.html` — a click-through demo of three sessions: listing skill cases,
  running a skill dry-run, and inspecting a finished run. Toggle the macOS
  traffic-light tabs.
- `Terminal.jsx` — terminal window chrome + scrollback primitive.
- `Prompt.jsx` — `$ cargo run --quiet -- …` line, plus inline output rows.
- `Output.jsx` — colored output rows (paths, ids, status sigils).
- `tokens.js` — shared color tokens lifted from `colors_and_type.css`.

## What's faithful

- Command invocations come from `help_text()` in `src/cli.rs` and from the
  documented examples in `recipes/macos/qqmusic/README.md`.
- Recipe/bundle/case IDs are the actual phase-1 IDs.
- Status sigils (`● validated`, `◐ candidate`) and disturbance ladder are the
  real vocabulary in `src/model.rs`.

## What's approximated

- Exact column widths in real terminal output may differ; I'm matching the
  *layout* (`key: value` indent two spaces) but not byte-for-byte spacing.
- macOS Terminal.app traffic-light proportions are simplified — this is not a
  Tahoe-grade window mock.
