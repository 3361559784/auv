#!/usr/bin/env bash
set -euo pipefail

# This local verification script only preserves the textual clipboard view.
# It is good enough to validate the new paste primitive for developer testing,
# but it is not a general clipboard fidelity proof for arbitrary non-text items.

APP_ID="${APP_ID:-com.tencent.QQMusicMac}"
QUERY="${QUERY:-aa}"
ANCHOR_TEXT="${ANCHOR_TEXT:-I DRINK THE LIGHT}"
REVEAL_SHORTCUT="${REVEAL_SHORTCUT:-cmd+f}"
REVEAL_SETTLE_MS="${REVEAL_SETTLE_MS:-300}"
SUBMIT_SETTLE_MS="${SUBMIT_SETTLE_MS:-900}"
SENTINEL_TEXT="${SENTINEL_TEXT:-__AUV_TEXTEDIT_SENTINEL__}"
CLIPBOARD_SENTINEL="${CLIPBOARD_SENTINEL:-__AUV_CLIPBOARD_SENTINEL__}"

ORIGINAL_CLIPBOARD_TEXT="$(pbpaste 2>/dev/null || true)"
restore_clipboard_text() {
  printf '%s' "${ORIGINAL_CLIPBOARD_TEXT}" | pbcopy
}
trap restore_clipboard_text EXIT

printf '%s' "${CLIPBOARD_SENTINEL}" | pbcopy

osascript - "${SENTINEL_TEXT}" <<'APPLESCRIPT'
on run argv
  set sentinelText to item 1 of argv
  tell application "TextEdit"
    activate
    make new document
    set text of front document to sentinelText
  end tell
end run
APPLESCRIPT

sleep 0.3

FRONT_BEFORE="$(osascript -e 'tell application "System Events" to return name of first application process whose frontmost is true')"
TEXTEDIT_BEFORE="$(osascript -e 'tell application "TextEdit" to return text of front document')"

if [[ "${TEXTEDIT_BEFORE}" != "${SENTINEL_TEXT}" ]]; then
  echo "sentinel setup failed: TextEdit front document does not match the expected sentinel" >&2
  exit 1
fi

cargo run --quiet -- invoke debug.pressKey \
  --target "${APP_ID}" \
  --key "${REVEAL_SHORTCUT}" \
  --settle_ms "${REVEAL_SETTLE_MS}"

cargo run --quiet -- invoke debug.pasteTextPreserveClipboard \
  --target "${APP_ID}" \
  --text "${QUERY}" \
  --replace_existing true \
  --submit_key return \
  --submit_settle_ms "${SUBMIT_SETTLE_MS}"

OCR_OUTPUT="$(cargo run --quiet -- invoke debug.findScreenText --query "${ANCHOR_TEXT}")"
FRONT_AFTER="$(osascript -e 'tell application "System Events" to return name of first application process whose frontmost is true')"
TEXTEDIT_AFTER="$(osascript -e 'tell application "TextEdit" to return text of front document')"
CLIPBOARD_AFTER="$(pbpaste 2>/dev/null || true)"

if [[ "${TEXTEDIT_AFTER}" != "${SENTINEL_TEXT}" ]]; then
  echo "sentinel failure: TextEdit front document was contaminated by QQ音乐 search-entry automation" >&2
  echo "front_before=${FRONT_BEFORE}" >&2
  echo "front_after=${FRONT_AFTER}" >&2
  echo "textedit_after=${TEXTEDIT_AFTER}" >&2
  exit 1
fi

if [[ "${CLIPBOARD_AFTER}" != "${CLIPBOARD_SENTINEL}" ]]; then
  echo "sentinel failure: clipboard text was not restored after QQ音乐 search-entry automation" >&2
  echo "expected=${CLIPBOARD_SENTINEL}" >&2
  echo "actual=${CLIPBOARD_AFTER}" >&2
  exit 1
fi

echo "front_before=${FRONT_BEFORE}"
echo "front_after=${FRONT_AFTER}"
echo "textedit_preserved=true"
echo "clipboard_restored=true"
echo "${OCR_OUTPUT}"
