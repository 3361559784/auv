#!/usr/bin/env bash
set -euo pipefail

ANCHOR="${1:-Cure For Me}"
CLICK_COUNT="${CLICK_COUNT:-1}"
APP_ID="${APP_ID:-com.tencent.QQMusicMac}"
ACTIVATE_SETTLE_MS="${ACTIVATE_SETTLE_MS:-300}"
MIN_CONFIDENCE="${MIN_CONFIDENCE:-0.90}"
REGION_LEFT_RATIO="${REGION_LEFT_RATIO:-0.14}"
REGION_TOP_RATIO="${REGION_TOP_RATIO:-0.34}"
REGION_RIGHT_RATIO="${REGION_RIGHT_RATIO:-0.90}"
REGION_BOTTOM_RATIO="${REGION_BOTTOM_RATIO:-0.95}"
ANCHOR_OFFSET_X="${ANCHOR_OFFSET_X:-0}"
ANCHOR_OFFSET_Y="${ANCHOR_OFFSET_Y:-0}"
EVIDENCE_LABEL="${EVIDENCE_LABEL:-qqmusic-result-anchor}"
DRY_RUN="${DRY_RUN:-0}"
MAX_DISTURBANCE="${MAX_DISTURBANCE:-}"

RUN_ARGS=()
if [[ "${DRY_RUN}" == "1" ]]; then
  RUN_ARGS+=(--dry-run)
fi
if [[ -n "${MAX_DISTURBANCE}" ]]; then
  RUN_ARGS+=(--max-disturbance "${MAX_DISTURBANCE}")
fi

python3 scripts/recipes/run_recipe.py \
  recipes/macos/qqmusic/select-result-anchor.v0.json \
  "${RUN_ARGS[@]}" \
  --set "app_id=${APP_ID}" \
  --set "anchor_text=${ANCHOR}" \
  --set "activate_settle_ms=${ACTIVATE_SETTLE_MS}" \
  --set "click_count=${CLICK_COUNT}" \
  --set "min_confidence=${MIN_CONFIDENCE}" \
  --set "region_left_ratio=${REGION_LEFT_RATIO}" \
  --set "region_top_ratio=${REGION_TOP_RATIO}" \
  --set "region_right_ratio=${REGION_RIGHT_RATIO}" \
  --set "region_bottom_ratio=${REGION_BOTTOM_RATIO}" \
  --set "anchor_offset_x=${ANCHOR_OFFSET_X}" \
  --set "anchor_offset_y=${ANCHOR_OFFSET_Y}" \
  --set "evidence_label=${EVIDENCE_LABEL}"
