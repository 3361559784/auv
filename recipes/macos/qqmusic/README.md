# QQ音乐 macOS Recipes

This directory holds executable recipe manifests for the first QQ音乐 macOS
validation slices.

Current baseline:

- `search-ocr-anchor.v0.json`

This recipe proves only the following chain:

1. focus the QQ音乐 search input
2. type and submit a query
3. resolve a known OCR anchor from the result list
4. click the OCR anchor
5. capture post-click evidence

It does **not** prove playback activation yet.

Current disturbance truth:

- the validated recipe has `max_disturbance=pointer`
- this is not because every step needs the pointer
- it is because the current `focusTextInput` step still performs pointer-level
  focus, and result selection still depends on OCR/pointer fallback

Probe evidence suggests QQ音乐 may admit a keyboard-first search-entry path,
but that is not yet the current recipe contract.

## How to Replay

Dry-run without touching the desktop:

```bash
python3 scripts/recipes/run_recipe.py \
  recipes/macos/qqmusic/search-ocr-anchor.v0.json \
  --dry-run
```

Replay with the convenience wrapper:

```bash
DRY_RUN=1 ./scripts/local/qqmusic-select-result.sh aa "I DRINK THE LIGHT"
./scripts/local/qqmusic-select-result.sh aa "I DRINK THE LIGHT"
```

## Why This Exists

The point is to stop carrying the QQ音乐 baseline as a chat transcript or an
ad-hoc shell sequence. A recipe manifest gives later agents a stable,
inspectable chain they can replay, override, and eventually distill further.
