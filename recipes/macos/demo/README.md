# Demo macOS Recipes

This directory holds **presentation-layer** recipes — narrow skills whose
purpose is to demonstrate the AUV design system contract end-to-end on a
live desktop, not to validate any individual app's behaviour.

The product validation for Notes itself lives in
`recipes/macos/notes/`. The recipes here re-use those validated flows
and turn on visual layers (dual cursor, animation, flash) so a human can
see what's happening.

Current candidates:

- `dual-cursor-press-notes.v0.json` — runs the Notes v2 cursor-warp-free
  flow (`create-note` via `debug.axPressButton`, `focus-body` via
  `debug.axFocusTextInput`) with `overlay: true` on both AX steps.
  Asserts `cursorDisturbance=none`, `overlayPresentation=dual-cursor-
  visual-only`, and `dualCursor=true` on every AX step.
- `dual-cursor-press-notes.cases.v0.json` — single candidate case
  `notes-dual-cursor-demo`.
- `smart-press-cross-app.v0.json` + `.cases.v0.json` (Phase 3 #5) —
  discovery matrix for `debug.smartPress` across native macOS apps.
  Each case picks an `(app, query)` tuple; the recipe lets smartPress
  decide per invocation whether the AX path or the pointer-click
  fallback ran, and records the outcome in
  `signals.smartPress.strategy`. Initial cases: Notes 新建备忘录,
  TextEdit 新建文稿. Both stay `candidate` until ≥3 hands-off replays
  pass on a quiescent desktop.

Current truth:

- This demo is not currently a validated presentation baseline.
- A hands-off replay on 2026-05-21 failed at `create-note` because the
  Notes toolbar button `新建备忘录` was present but not AX-pressable in the
  live app state (`run_1779371986165_16879_0`).
- Any replay where a human switches into Notes, types, or pastes during the
  run is contaminated evidence and must not be promoted.

## Replay

```bash
cargo run --quiet -- skill cases run \
  macos.demo.dual_cursor_press_notes.v0 --all-statuses

cargo run --quiet -- skill cases run \
  macos.demo.dual_cursor_press_notes.v0
```

The intended visual is:

1. `you` slate pixel cursor appears at the user's real mouse position.
2. `auv · replay` cyan/lime pixel cursor animates from the user's
   position to the Notes 新建备忘录 toolbar button.
3. At the moment of `AXUIElementPerformAction(AXPress)`, the auv cursor
   flashes the `cursor-auv-click` burst sprite.
4. Both cursors fade.
5. Steps 1–4 repeat for the body text-area focus step.

When the demo is in a good app state, the real hardware cursor never moves
and `cursorDisturbance=none` is recorded in every AX step's artifact.

## What this directory is NOT

- Not a contract for Notes itself — that's `recipes/macos/notes/`.
- Not a validation that the overlay sprite renders pixel-perfectly —
  that's the design-system layer's responsibility (see
  `docs/design/` and `src/driver/.../Overlay.swift`).
- Not safe to run concurrently against the same live app instance.
- Not valid evidence if a human touches the desktop during replay.
