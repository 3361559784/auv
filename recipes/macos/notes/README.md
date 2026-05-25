# Notes macOS Recipes

This directory holds the first native-app cross-surface sample for AUV.

Current baseline:

- `create-and-verify-note.v0.json`
- `create-and-verify-note.cases.v0.json`
- `../../docs/ai/references/2026-05-17-auv-native-app-skill-tree.md`

Phase 2 contract-consuming variants:

- `create-and-verify-note.v1.json` / `.cases.v1.json` — swaps only
  `create-note` to `debug.axPressButton`. Recipe-level disturbance is
  `pointer` because `focus-body` still warps the cursor.
- `create-and-verify-note.v2.json` / `.cases.v2.json` — also swaps
  `focus-body` to the new `debug.axFocusTextInput`. **First narrow
  skill whose entire activation chain is cursor-warp-free**;
  recipe-level disturbance drops to `clipboard`.
- `../../docs/ai/references/2026-05-21-phase-3-first-contract-consumer-design.md`

Both v1 and v2 use `expect.signal_equals` to assert the Phase 2 contract
fields. v2 additionally asserts the focus contract
(`cursorDisturbance=none`, `focusMechanism=ax-attribute`,
`setAttribute=AXFocused`).

What it proves:

1. activate Notes
2. create a new note
3. focus the note body through AX
4. write a stable marker through clipboard-backed text entry
5. verify the marker through the AX tree with `verify.axText`

This baseline deliberately avoids screenshot OCR. It is meant to show that
AUV can distill a reusable native-app skill shape from the QQ音乐 work into a
different macOS app with a real `AXTextArea`.

Replay:

```bash
cargo run --quiet -- skill run macos.notes.create_and_verify_note.v0

cargo run --quiet -- skill cases run \
  macos.notes.create_and_verify_note.v1

cargo run --quiet -- skill cases run \
  macos.notes.create_and_verify_note.v2
```

Validated cases:

- v0: `notes-marker-baseline`
- v1: `notes-marker-ax-press`
- v2: `notes-marker-ax-press-and-ax-focus`

Markers:

- v0: `AUV_NOTE_MARKER_2026_05_16`
- v1: `AUV_NOTE_MARKER_2026_05_21_V1`
- v2: `AUV_NOTE_MARKER_2026_05_21_V2`
