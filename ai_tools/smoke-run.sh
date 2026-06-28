#!/usr/bin/env bash
#
# smoke-run.sh — build the `desktop` app and run it for a few seconds to confirm
# it compiles, starts, and renders. The app has no built-in timed exit, so we
# wrap it with `timeout`.
#
# Usage:
#   ai_tools/smoke-run.sh                    # build + run ~5s (auto backend)
#   ai_tools/smoke-run.sh --secs 8           # run for 8 seconds
#   ai_tools/smoke-run.sh --x11              # force X11 winit backend
#   ai_tools/smoke-run.sh --wayland          # force Wayland winit backend
#   ai_tools/smoke-run.sh --log /path.log    # custom log path
#
# Exit 0 = survived the whole window (rendered fine). Exit 1 = crashed/failed early.
# Requires a live graphical session (Wayland or X11); fails at surface creation if headless.
#
# See ai_tools/docs/smoke-run.md
set -eo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

DURATION=5
BACKEND="auto"
LOG="$ROOT/target/ai_tools-smoke-run.log"

while [ $# -gt 0 ]; do
  case "$1" in
    --secs)    DURATION="$2"; shift 2 ;;
    --log)     LOG="$2"; shift 2 ;;
    --x11)     BACKEND="x11"; shift ;;
    --wayland) BACKEND="wayland"; shift ;;
    -h|--help) sed -n '2,17p' "${BASH_SOURCE[0]}"; exit 0 ;;
    *)         echo "unknown arg: $1" >&2; exit 2 ;;
  esac
done

if [ "$BACKEND" = "auto" ]; then
  if [ -n "${WAYLAND_DISPLAY:-}" ]; then BACKEND="wayland"
  elif [ -n "${DISPLAY:-}" ];      then BACKEND="x11"
  else BACKEND="wayland"; fi
fi

FEATURES=()
[ "$BACKEND" = "x11" ] && FEATURES=(--features x11)

cd "$ROOT"
echo ">> building desktop (backend: $BACKEND)"
cargo build -p desktop "${FEATURES[@]}"

BIN="$ROOT/target/build/debug/desktop"
DIST="$ROOT/target/distribution"
[ -x "$BIN" ] || { echo "binary not found: $BIN" >&2; exit 1; }
[ -d "$DIST/assets" ] || echo "WARN: $DIST/assets missing — run ai_tools/build-assets.sh first" >&2

echo ">> running ${DURATION}s (AMBERLUME_VK_VALIDATION='${AMBERLUME_VK_VALIDATION:-unset}')"
echo ">> log: $LOG"
cd "$DIST"
set +e
timeout --signal=TERM "$DURATION" "$BIN" >"$LOG" 2>&1
status=$?
set -e

echo "----- last log lines -----"
tail -n 20 "$LOG" 2>/dev/null || true
echo "--------------------------"
grep -q "AmberLume created" "$LOG" 2>/dev/null \
  && echo "signal: 'AmberLume created' present (engine init reached)" \
  || echo "signal: 'AmberLume created' NOT found (init did not complete)"

if [ "$status" -eq 124 ] || [ "$status" -eq 0 ]; then
  echo "RESULT: OK — survived ${DURATION}s (timeout exit $status)"
  exit 0
fi
echo "RESULT: FAIL — exited early (exit $status)"
grep -niE 'panic|panicked|error|No suitable device|VK_ERROR' "$LOG" 2>/dev/null | head -n 20 || true
exit 1
