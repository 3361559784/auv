// data.js — mock data lifted verbatim from the AUV v1alpha1 spec shape.
// Used by every viewer component.

window.AUV_RUNS = [
  {
    run_id: "run_1778947574511_68037_4",
    trace_id: "4b9e2c7f1a3d6e0b8f5a1c2d3e4f5061",
    run_type: "execute",
    state: "running",
    status_code: "unset",
    started_at: "2026-05-20T14:32:54.511Z",
    summary: "macos.qqmusic.play_visible_anchor.v0  ·  query=aa  anchor=\"Cure For Me\"",
    recipe_id: "macos.qqmusic.play_visible_anchor.v0",
    duration: "4.71s (live)",
    spans: 6, artifacts: 3,
  },
  {
    run_id: "run_1778946131088_67910_2",
    trace_id: "f31c08ad2e8a4b117f5b1c2d3e4f5072",
    run_type: "validate",
    state: "ended",
    status_code: "ok",
    started_at: "2026-05-20T14:08:51.088Z",
    summary: "macos.notes.create_and_verify_note.v0  ·  case=notes-marker-baseline",
    recipe_id: "macos.notes.create_and_verify_note.v0",
    duration: "2m 18s",
    spans: 12, artifacts: 6,
  },
  {
    run_id: "run_1778945002311_67885_1",
    trace_id: "b22f9c0c1e3d4a607f5b1c2d3e4f5071",
    run_type: "execute",
    state: "ended",
    status_code: "error",
    started_at: "2026-05-20T13:50:02.311Z",
    summary: "resolve-ocr-anchor returned 0 matches for 晴天",
    recipe_id: "macos.qqmusic.play_visible_anchor.v0",
    duration: "3.50s",
    spans: 5, artifacts: 4,
  },
  {
    run_id: "run_1778944887210_67830_0",
    trace_id: "1e8a7c45f2b9405a8f6b1c2d3e4f5099",
    run_type: "probe",
    state: "ended",
    status_code: "ok",
    started_at: "2026-05-20T13:48:07.210Z",
    summary: "com.tencent.QQMusicMac  ·  permissions, window-state, ax-tree",
    recipe_id: null,
    duration: "8.04s",
    spans: 11, artifacts: 9,
  },
  {
    run_id: "run_1778942100001_67801_0",
    trace_id: "9c3e0a76f2b94055eef6b1c2d3e4f50a8",
    run_type: "validate",
    state: "ended",
    status_code: "ok",
    started_at: "2026-05-20T13:01:40.001Z",
    summary: "macos.textedit.create_and_verify_text.v0  ·  textedit-marker-baseline",
    recipe_id: "macos.textedit.create_and_verify_text.v0",
    duration: "1m 02s",
    spans: 9, artifacts: 4,
  },
];

// Span tree for the active live run (#1). Order in render = depth-first.
window.AUV_SPANS = [
  { id: "s00", parent: null,  name: "auv.execute",         status: "running", t: "4.71s", attrs: { recipe_id: "macos.qqmusic.play_visible_anchor.v0", target: "com.tencent.QQMusicMac" } },
  { id: "s01", parent: "s00", name: "auv.recipe.step",     status: "ok",      t: "0.32s", attrs: { step_id: "open-search" } },
  { id: "s02", parent: "s01", name: "auv.command.invoke",  status: "ok",      t: "0.30s", attrs: { command_id: "debug.pressKey", key: "cmd+f" } },
  { id: "s03", parent: "s00", name: "auv.recipe.step",     status: "ok",      t: "0.94s", attrs: { step_id: "paste-query" } },
  { id: "s04", parent: "s03", name: "auv.command.invoke",  status: "ok",      t: "0.92s", attrs: { command_id: "debug.pasteTextPreserveClipboard", text: "aa" } },
  { id: "s05", parent: "s00", name: "auv.recipe.step",     status: "ok",      t: "0.28s", attrs: { step_id: "dismiss-search-overlay" } },
  { id: "s06", parent: "s00", name: "auv.recipe.step",     status: "ok",      t: "1.10s", attrs: { step_id: "wait-for-ocr-anchor" } },
  { id: "s07", parent: "s00", name: "auv.recipe.step",     status: "ok",      t: "0.42s", attrs: { step_id: "resolve-ocr-anchor", anchor_text: "Cure For Me" } },
  { id: "s08", parent: "s00", name: "auv.recipe.step",     status: "running", t: "1.65s", attrs: { step_id: "double-click-row-anchor" } },
  { id: "s09", parent: "s08", name: "auv.command.invoke",  status: "running", t: "1.62s", attrs: { command_id: "debug.clickScreenText", click_count: 2 } },
  { id: "s10", parent: "s00", name: "auv.recipe.step",     status: "unset",   t: "—",     attrs: { step_id: "capture-evidence" } },
  { id: "s11", parent: "s00", name: "auv.recipe.step",     status: "unset",   t: "—",     attrs: { step_id: "verify-player-title" } },
];

window.AUV_EVENTS = [
  { t: "+0.000s", name: "run.started",        span: "s00", body: "run_type=execute  recipe_id=macos.qqmusic.play_visible_anchor.v0" },
  { t: "+0.020s", name: "command.resolved",   span: "s02", body: "command_id=debug.pressKey  driver=macos" },
  { t: "+0.040s", name: "driver.invoke",      span: "s02", body: "macos.keyboard.pressKey  key=cmd+f" },
  { t: "+0.320s", name: "action.completed",   span: "s02", body: "settle_ms=300" },
  { t: "+0.500s", name: "command.resolved",   span: "s04", body: "command_id=debug.pasteTextPreserveClipboard" },
  { t: "+0.510s", name: "clipboard.locked",   span: "s04", body: "// global clipboard lock acquired" },
  { t: "+0.900s", name: "clipboard.restored", span: "s04", body: "previous clipboard contents restored" },
  { t: "+1.520s", name: "ocr.match_found",    span: "s07", body: "best_match_text=\"Cure For Me\"  confidence=0.94" },
  { t: "+2.020s", name: "artifact.captured",  span: "s07", body: "artifact_0001_screenshot.png" },
  { t: "+3.060s", name: "driver.invoke",      span: "s09", body: "macos.pointer.clickPoint  x=512.3 y=388.6  count=2", live: true },
  { t: "+4.700s", name: "action.started",     span: "s09", body: "settle_ms=900  (running)", live: true },
];

window.AUV_ARTIFACTS = [
  { id: "a01", role: "screenshot.before", mime: "image/png", path: "artifacts/artifact_0001_screenshot.png", sha: "f3c1…0a44", bytes: "2.8 MB", span: "s07" },
  { id: "a02", role: "ax.before",         mime: "application/json", path: "artifacts/artifact_0002_ax.json",  sha: "c4a0…91b2", bytes: "412 KB", span: "s07" },
  { id: "a03", role: "click.overlay",     mime: "image/png", path: "artifacts/artifact_0003_click_overlay.png", sha: "918d…44e0", bytes: "3.1 MB", span: "s09", live: true },
];
