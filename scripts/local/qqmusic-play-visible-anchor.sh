#!/usr/bin/env bash
set -euo pipefail

QUERY="${1:-aa}"
ANCHOR="${2:-Cure For Me}"
PLAYBACK_TITLE="${3:-Cure For Me - AURORA}"
CLICK_COUNT="${CLICK_COUNT:-2}"
MAX_DISTURBANCE="${MAX_DISTURBANCE:-pointer}"
DRY_RUN="${DRY_RUN:-0}"
VERIFY_MIN_CONFIDENCE="${VERIFY_MIN_CONFIDENCE:-0.90}"
VERIFY_REGION_LEFT_RATIO="${VERIFY_REGION_LEFT_RATIO:-0.22}"
VERIFY_REGION_TOP_RATIO="${VERIFY_REGION_TOP_RATIO:-0.80}"
VERIFY_REGION_RIGHT_RATIO="${VERIFY_REGION_RIGHT_RATIO:-0.45}"
VERIFY_REGION_BOTTOM_RATIO="${VERIFY_REGION_BOTTOM_RATIO:-0.90}"

SELECTION_LOG=""
VERIFY_LOG=""

cleanup() {
  if [[ -n "${SELECTION_LOG}" ]]; then
    rm -f "${SELECTION_LOG}"
  fi
  if [[ -n "${VERIFY_LOG}" ]]; then
    rm -f "${VERIFY_LOG}"
  fi
}

trap cleanup EXIT

if [[ "${DRY_RUN}" == "1" ]]; then
  CLICK_COUNT="${CLICK_COUNT}" \
  MAX_DISTURBANCE="${MAX_DISTURBANCE}" \
  DRY_RUN=1 \
  ./scripts/local/qqmusic-select-result.sh "${QUERY}" "${ANCHOR}"
  exit 0
fi

SELECTION_LOG="$(mktemp -t auv-qqmusic-play-select.XXXXXX.log)"
CLICK_COUNT="${CLICK_COUNT}" \
MAX_DISTURBANCE="${MAX_DISTURBANCE}" \
./scripts/local/qqmusic-select-result.sh "${QUERY}" "${ANCHOR}" | tee "${SELECTION_LOG}"

EVIDENCE_IMAGE="$(sed -n 's/^artifact: //p' "${SELECTION_LOG}" | grep -E '\.(png|jpg|jpeg)$' | tail -n 1)"
if [[ -z "${EVIDENCE_IMAGE}" ]]; then
  echo "error: failed to resolve the post-click evidence artifact from qqmusic-select-result output" >&2
  exit 1
fi
if [[ ! -f "${EVIDENCE_IMAGE}" ]]; then
  echo "error: post-click evidence artifact does not exist: ${EVIDENCE_IMAGE}" >&2
  exit 1
fi

VERIFY_LOG="$(mktemp -t auv-qqmusic-play-verify.XXXXXX.log)"
cargo run --quiet -- invoke debug.findImageText \
  --image_path "${EVIDENCE_IMAGE}" \
  --query "${PLAYBACK_TITLE}" \
  --min_confidence "${VERIFY_MIN_CONFIDENCE}" \
  --region_left_ratio "${VERIFY_REGION_LEFT_RATIO}" \
  --region_top_ratio "${VERIFY_REGION_TOP_RATIO}" \
  --region_right_ratio "${VERIFY_REGION_RIGHT_RATIO}" \
  --region_bottom_ratio "${VERIFY_REGION_BOTTOM_RATIO}" | tee "${VERIFY_LOG}"

if grep -q '^output: Found 0 OCR text matches' "${VERIFY_LOG}"; then
  echo "error: playback verification failed; OCR did not confirm ${PLAYBACK_TITLE} inside the captured player-title region" >&2
  exit 1
fi

SELECTION_RUN_ID="$(sed -n 's/^runId: //p' "${SELECTION_LOG}" | tail -n 1)"
VERIFY_RUN_ID="$(sed -n 's/^runId: //p' "${VERIFY_LOG}" | tail -n 1)"
echo "playbackSelectionRunId: ${SELECTION_RUN_ID}"
echo "playbackVerificationRunId: ${VERIFY_RUN_ID}"
echo "playbackEvidenceArtifact: ${EVIDENCE_IMAGE}"
echo "playbackVerificationQuery: ${PLAYBACK_TITLE}"
