#!/usr/bin/env bash
# Capture tip-of-tree Godot stills via Xvfb + opengl3 (not bare --headless).
# Requires: Godot 4.7.2-stable on PATH as `godot`, Xvfb, a running fragr-server.
# See docs/plans/tip-screenshots.md.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
OUT_DIR="${OUT_DIR:-$ROOT/docs/screenshots}"
DISPLAY_NUM="${DISPLAY_NUM:-99}"
WAIT_SECS="${WAIT_SECS:-8}"
SERVER_URL="${FRAGR_SERVER:-127.0.0.1:6767}"

if ! command -v godot >/dev/null 2>&1; then
  echo "godot (4.7.2-stable) not on PATH; install editor binary first" >&2
  exit 1
fi
if ! command -v Xvfb >/dev/null 2>&1; then
  echo "Xvfb not found; install xvfb for virtual display capture" >&2
  exit 1
fi

mkdir -p "$OUT_DIR"
export DISPLAY=":${DISPLAY_NUM}"
export FRAGR_SERVER="$SERVER_URL"
export FRAGR_TIP_CAPTURE=1
export FRAGR_TIP_CAPTURE_DIR="$OUT_DIR"
export FRAGR_TIP_CAPTURE_WAIT="$WAIT_SECS"

Xvfb ":${DISPLAY_NUM}" -screen 0 1280x720x24 -ac >/tmp/fragr-xvfb.log 2>&1 &
XVFB_PID=$!
cleanup() {
  kill "$XVFB_PID" >/dev/null 2>&1 || true
}
trap cleanup EXIT
sleep 0.5

# gl_compatibility / opengl3: Vulkan on Xvfb needs lavapipe; keep the simple path.
godot --path "$ROOT/client" --rendering-driver opengl3 --quit-after 12000 \
  --script res://scripts/tip_capture.gd

echo "Tip capture finished. Inspect PNGs under $OUT_DIR"
