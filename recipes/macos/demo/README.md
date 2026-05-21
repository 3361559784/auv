# Demo macOS Recipes

This directory holds **presentation-layer** recipes — narrow skills whose
purpose is to demonstrate the AUV design system contract end-to-end on a
live desktop, not to validate any individual app's behaviour.

The product validation for Notes itself lives in
`recipes/macos/notes/`. The recipes here re-use those validated flows
and turn on visual layers (dual cursor, animation, flash) so a human can
see what's happening.

Current baseline:

- `dual-cursor-press-notes.v0.json` — runs the Notes v2 cursor-warp-free
  flow (`create-note` via `debug.axPressButton`, `focus-body` via
  `debug.axFocusTextInput`) with `overlay: true` on both AX steps.
  Asserts `cursorDisturbance=none`, `overlayPresentation=dual-cursor-
  visual-only`, and `dualCursor=true` on every AX step.
- `dual-cursor-press-notes.cases.v0.json` — single candidate case
  `notes-dual-cursor-demo`.

## Replay

```bash
cargo run --quiet -- skill cases run \
  macos.demo.dual_cursor_press_notes.v0 --all-statuses

cargo run --quiet -- skill cases run \
  macos.demo.dual_cursor_press_notes.v0
```

The visual is:

1. `you` slate pixel cursor appears at the user's real mouse position.
2. `auv · replay` cyan/lime pixel cursor animates from the user's
   position to the Notes 新建备忘录 toolbar button.
3. At the moment of `AXUIElementPerformAction(AXPress)`, the auv cursor
   flashes the `cursor-auv-click` burst sprite.
4. Both cursors fade.
5. Steps 1–4 repeat for the body text-area focus step.

The real hardware cursor never moves. `cursorDisturbance=none` is
recorded in every AX step's artifact.

## What this directory is NOT

- Not a contract for Notes itself — that's `recipes/macos/notes/`.
- Not a validation that the overlay sprite renders pixel-perfectly —
  that's the design-system layer's responsibility (see
  `docs/design/` and `src/driver/.../Overlay.swift`).
- Not safe to run concurrently against the same live app instance.
