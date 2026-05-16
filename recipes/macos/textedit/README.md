# TextEdit macOS Recipes

This directory holds the third native-app sample for the AUV skill tree.

Current baseline:

- `create-and-verify-text.v0.json`
- `create-and-verify-text.cases.v0.json`

What it proves:

1. activate TextEdit
2. focus the main text area through AX
3. paste a stable marker through the clipboard
4. verify that marker through the AX tree with `debug.verifyAxText`

This sample reuses the same generic AX text verification contract introduced
for Notes, but it exercises a different native editor surface.

Replay:

```bash
cargo run --quiet -- skill run macos.textedit.create_and_verify_text.v0
```

Validated case:

- `textedit-marker-baseline`
