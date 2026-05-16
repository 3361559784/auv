# Notes macOS Recipes

This directory holds the first native-app cross-surface sample for AUV.

Current baseline:

- `create-and-verify-note.v0.json`
- `create-and-verify-note.cases.v0.json`
- `../../docs/ai/references/2026-05-17-auv-native-app-skill-tree.md`

What it proves:

1. activate Notes
2. create a new note
3. focus the note body through AX
4. write a stable marker through clipboard-backed text entry
5. verify the marker through the AX tree with `debug.verifyAxText`

This baseline deliberately avoids screenshot OCR. It is meant to show that
AUV can distill a reusable native-app skill shape from the QQ音乐 work into a
different macOS app with a real `AXTextArea`.

Replay:

```bash
cargo run --quiet -- skill run macos.notes.create_and_verify_note.v0
```

Validated case:

- `notes-marker-baseline`

The current marker is:

- `AUV_NOTE_MARKER_2026_05_16`
