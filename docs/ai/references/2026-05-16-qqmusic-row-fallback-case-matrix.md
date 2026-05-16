## QQ音乐 Row Fallback Case Matrix

Date: 2026-05-16

Status: validated fallback coverage note

### Purpose

This note records the row-based fallback slice for QQ音乐 playback.

The point is to keep the OCR baseline intact while validating a separate
visible-row activation path for cases where Chinese OCR anchors are not
reliable enough for grounding.

### Executable Coverage Entry

The product-facing coverage entrypoint is:

```bash
auv-cli skill cases run macos.qqmusic.play_visible_row.v0
```

The machine-readable matrix lives at:

- `recipes/macos/qqmusic/play-visible-row.cases.v0.json`

### Validated Cases

The following case is the current canonical row-based fallback baseline:

1. `ascii-aa-row-fallback`
   - query: `aa`
   - row index: `1`
   - playback title: `Cure For Me - AURORA`

What this proves:

- visible-row detection can drive activation without relying on a Chinese OCR anchor
- the row fallback verifies the now-playing title through the AX tree

What it does **not** prove:

- generalized QQ音乐 playback for arbitrary queries
- row fallback stability across layout changes
- pointer-free activation

### Validated Chinese Case

The matrix also includes one validated Chinese case:

1. `chinese-query-row-fallback`
   - query: `周杰伦`
   - row index: `1`
   - target title: `晴天`
   - observed playback title: `天空仍灿烂`

This proves the Chinese fallback path can activate a visible row and verify the
target title through the AX tree without screenshot OCR.
