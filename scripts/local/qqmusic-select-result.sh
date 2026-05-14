#!/usr/bin/env bash
set -euo pipefail

QUERY="${1:-aa}"
ANCHOR="${2:-Cure For Me}"
CLICK_COUNT="${CLICK_COUNT:-1}"
APP_ID="${APP_ID:-com.tencent.QQMusicMac}"
REVEAL_SHORTCUT="${REVEAL_SHORTCUT:-cmd+f}"
REVEAL_SETTLE_MS="${REVEAL_SETTLE_MS:-300}"
SUBMIT_SETTLE_MS="${SUBMIT_SETTLE_MS:-900}"
MAX_DEPTH="${MAX_DEPTH:-5}"
MAX_CHILDREN="${MAX_CHILDREN:-20}"
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
  recipes/macos/qqmusic/search-ocr-anchor.v0.json \
  "${RUN_ARGS[@]}" \
  --set "app_id=${APP_ID}" \
  --set "query=${QUERY}" \
  --set "anchor_text=${ANCHOR}" \
  --set "click_count=${CLICK_COUNT}" \
  --set "reveal_shortcut=${REVEAL_SHORTCUT}" \
  --set "reveal_settle_ms=${REVEAL_SETTLE_MS}" \
  --set "submit_settle_ms=${SUBMIT_SETTLE_MS}" \
  --set "max_depth=${MAX_DEPTH}" \
  --set "max_children=${MAX_CHILDREN}"
