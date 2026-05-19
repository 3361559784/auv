# NetEaseMusic macOS Recipes

This directory holds narrow NetEaseMusic automation recipes discovered through
`cargo run --quiet -- invoke ...` exploration.

Current baseline:

- `play-visible-anchor.v0.json`
- `play-visible-anchor.cases.v0.json`

The current recipe proves only this fixed-layout chain:

1. activate and capture the NetEaseMusic window
2. click the observed search box point
3. paste and submit `AURORA Cure For Me`
4. capture the search result page
5. verify `Cure For Me` and `AURORA` are visible in the result page image
6. double-click the observed first left-list result point
7. capture the post-play window
8. verify `Cure For Me` and `AURORA` in the bottom-player image region

It intentionally does not claim generalized NetEaseMusic playback. The current
search and result activation steps use fixed global logical coordinates:

```text
search_click_x=3509
search_click_y=398
result_click_x=3457
result_click_y=727
click_interval_ms=80
```

`click_interval_ms=80` is part of the validated contract. Earlier immediate
`click_count=2` events were too fast for stable NetEaseMusic result activation.

If the NetEaseMusic window moves, changes size, or lands on another display,
rediscover these points before treating the recipe as valid.

Dry-run:

```bash
cargo run --quiet -- skill run macos.netease_cloud_music.play_visible_anchor.v0 --dry-run
```

Live run:

```bash
cargo run --quiet -- skill run macos.netease_cloud_music.play_visible_anchor.v0
```

Case run:

```bash
cargo run --quiet -- skill cases run macos.netease_cloud_music.play_visible_anchor.v0 \
  --case aurora-cure-for-me-fixed-layout
```
