#!/usr/bin/env bash
set -euo pipefail

QUERY="${1:-aa}"
PLAYBACK_TITLE="${2:-Cure For Me - AURORA}"
ROW_INDEX="${ROW_INDEX:-1}"
CLICK_COUNT="${CLICK_COUNT:-2}"
ACTIVATION_SETTLE_MS="${ACTIVATION_SETTLE_MS:-900}"
ROW_WAIT_MIN_ROW_COUNT="${ROW_WAIT_MIN_ROW_COUNT:-3}"
ROW_WAIT_TIMEOUT_MS="${ROW_WAIT_TIMEOUT_MS:-3500}"
ROW_WAIT_POLL_MS="${ROW_WAIT_POLL_MS:-250}"
ROW_MIN_CONFIDENCE="${ROW_MIN_CONFIDENCE:-0.90}"
ROW_MAX_OBSERVATIONS="${ROW_MAX_OBSERVATIONS:-128}"
RESULT_REGION_LEFT_RATIO="${RESULT_REGION_LEFT_RATIO:-0.14}"
RESULT_REGION_TOP_RATIO="${RESULT_REGION_TOP_RATIO:-0.34}"
RESULT_REGION_RIGHT_RATIO="${RESULT_REGION_RIGHT_RATIO:-0.90}"
RESULT_REGION_BOTTOM_RATIO="${RESULT_REGION_BOTTOM_RATIO:-0.95}"
ROW_ANCHOR_MODE="${ROW_ANCHOR_MODE:-title_band}"
ROW_ANCHOR_X_RATIO="${ROW_ANCHOR_X_RATIO:-0.25}"
ROW_ANCHOR_Y_RATIO="${ROW_ANCHOR_Y_RATIO:-0.50}"
VERIFY_MIN_CONFIDENCE="${VERIFY_MIN_CONFIDENCE:-0.90}"
VERIFY_REGION_LEFT_RATIO="${VERIFY_REGION_LEFT_RATIO:-0.22}"
VERIFY_REGION_TOP_RATIO="${VERIFY_REGION_TOP_RATIO:-0.80}"
VERIFY_REGION_RIGHT_RATIO="${VERIFY_REGION_RIGHT_RATIO:-0.45}"
VERIFY_REGION_BOTTOM_RATIO="${VERIFY_REGION_BOTTOM_RATIO:-0.90}"
APP_ID="${APP_ID:-com.tencent.QQMusicMac}"
REVEAL_SHORTCUT="${REVEAL_SHORTCUT:-cmd+f}"
REVEAL_SETTLE_MS="${REVEAL_SETTLE_MS:-300}"
SUBMIT_SETTLE_MS="${SUBMIT_SETTLE_MS:-900}"
DISMISS_OVERLAY_KEY="${DISMISS_OVERLAY_KEY:-escape}"
DISMISS_OVERLAY_SETTLE_MS="${DISMISS_OVERLAY_SETTLE_MS:-300}"
DRY_RUN="${DRY_RUN:-0}"
MAX_DISTURBANCE="${MAX_DISTURBANCE:-pointer}"

RUN_ARGS=()
if [[ "${DRY_RUN}" == "1" ]]; then
  RUN_ARGS+=(--dry-run)
fi
if [[ -n "${MAX_DISTURBANCE}" ]]; then
  RUN_ARGS+=(--max-disturbance "${MAX_DISTURBANCE}")
fi

cargo run --quiet -- skill run \
  macos.qqmusic.play_visible_row.v0 \
  "${RUN_ARGS[@]}" \
  --set "app_id=${APP_ID}" \
  --set "query=${QUERY}" \
  --set "playback_title=${PLAYBACK_TITLE}" \
  --set "row_index=${ROW_INDEX}" \
  --set "click_count=${CLICK_COUNT}" \
  --set "activation_settle_ms=${ACTIVATION_SETTLE_MS}" \
  --set "row_wait_min_row_count=${ROW_WAIT_MIN_ROW_COUNT}" \
  --set "row_wait_timeout_ms=${ROW_WAIT_TIMEOUT_MS}" \
  --set "row_wait_poll_ms=${ROW_WAIT_POLL_MS}" \
  --set "row_min_confidence=${ROW_MIN_CONFIDENCE}" \
  --set "row_max_observations=${ROW_MAX_OBSERVATIONS}" \
  --set "result_region_left_ratio=${RESULT_REGION_LEFT_RATIO}" \
  --set "result_region_top_ratio=${RESULT_REGION_TOP_RATIO}" \
  --set "result_region_right_ratio=${RESULT_REGION_RIGHT_RATIO}" \
  --set "result_region_bottom_ratio=${RESULT_REGION_BOTTOM_RATIO}" \
  --set "row_anchor_mode=${ROW_ANCHOR_MODE}" \
  --set "row_anchor_x_ratio=${ROW_ANCHOR_X_RATIO}" \
  --set "row_anchor_y_ratio=${ROW_ANCHOR_Y_RATIO}" \
  --set "reveal_shortcut=${REVEAL_SHORTCUT}" \
  --set "reveal_settle_ms=${REVEAL_SETTLE_MS}" \
  --set "submit_settle_ms=${SUBMIT_SETTLE_MS}" \
  --set "dismiss_overlay_key=${DISMISS_OVERLAY_KEY}" \
  --set "dismiss_overlay_settle_ms=${DISMISS_OVERLAY_SETTLE_MS}" \
  --set "verification_region_left_ratio=${VERIFY_REGION_LEFT_RATIO}" \
  --set "verification_region_top_ratio=${VERIFY_REGION_TOP_RATIO}" \
  --set "verification_region_right_ratio=${VERIFY_REGION_RIGHT_RATIO}" \
  --set "verification_region_bottom_ratio=${VERIFY_REGION_BOTTOM_RATIO}" \
  --set "verification_min_confidence=${VERIFY_MIN_CONFIDENCE}"
