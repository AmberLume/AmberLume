#!/usr/bin/env bash
#
# vk-run.sh — run the `desktop` app with all Vulkan validation enabled and scan
# the captured log for validation findings.
#
# There is no in-app debug messenger, so the validation layer prints straight to
# stdout/stderr. This wraps smoke-run.sh (which captures that output) and then
# greps the log for the layer's markers.
#
# Usage:
#   ai_tools/vk-run.sh                                   # all 3 feature tokens, ~8s
#   ai_tools/vk-run.sh --secs 12                         # longer window
#   ai_tools/vk-run.sh --tokens synchronization          # only one feature
#   ai_tools/vk-run.sh --x11 / --wayland                 # force backend
#
# Exit code reflects the RUN (0 = app survived, 1 = app crashed). Validation
# findings are printed for review; treat any 'Validation Error' / 'VUID-' / 'SYNC-'
# as a real issue. 'UNASSIGNED-BestPractices-' lines are advisory.
#
# See ai_tools/docs/vk-run.md
set -eo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

DURATION=8
TOKENS="${AMBERLUME_VK_VALIDATION:-synchronization,best_practices,gpu_assisted}"
LOG="$ROOT/target/ai_tools-vk-run.log"
PASS=()

while [ $# -gt 0 ]; do
  case "$1" in
    --secs)    DURATION="$2"; shift 2 ;;
    --tokens)  TOKENS="$2"; shift 2 ;;
    --log)     LOG="$2"; shift 2 ;;
    --x11)     PASS+=(--x11); shift ;;
    --wayland) PASS+=(--wayland); shift ;;
    -h|--help) sed -n '2,20p' "${BASH_SOURCE[0]}"; exit 0 ;;
    *)         echo "unknown arg: $1" >&2; exit 2 ;;
  esac
done

echo ">> validation tokens: $TOKENS"
set +e
AMBERLUME_VK_VALIDATION="$TOKENS" \
  "$SCRIPT_DIR/smoke-run.sh" --secs "$DURATION" --log "$LOG" "${PASS[@]}"
run_status=$?
set -e

echo "===== validation scan ($LOG) ====="
BENIGN='VALIDATION-SETTINGS|WARNING-Setting-Limit-Adjusted|Both GPU Assisted Validation and Normal Core Check'
errors=$(grep -nE 'Validation Error|VUID-|SYNC-' "$LOG" 2>/dev/null || true)
warns=$(grep -nE 'Validation Warning|Validation Performance Warning|UNASSIGNED-BestPractices-' "$LOG" 2>/dev/null | grep -vE "$BENIGN" || true)
setup=$(grep -cE "$BENIGN" "$LOG" 2>/dev/null || echo 0)

if [ -n "$errors" ]; then
  echo "-- ERRORS / hazards (must review) --"
  echo "$errors"
else
  echo "-- no errors (Validation Error / VUID- / SYNC-) --"
fi
if [ -n "$warns" ]; then
  echo "-- warnings (review) --"
  echo "$warns" | head -n 40
else
  echo "-- no actionable warnings --"
fi
echo "-- benign setup advisories suppressed: $setup line(s) (GPU-AV slowness, unsupported-feature limit adjustments) --"
echo "==================================="
if [ -n "$errors" ]; then
  echo "RESULT: run exit $run_status; VALIDATION ERRORS PRESENT — review above."
else
  echo "RESULT: run exit $run_status (0/124-mapped = survived); no validation errors."
fi
exit "$run_status"
