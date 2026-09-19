#!/usr/bin/env bash
# Capture tip-of-tree Godot stills via Xvfb + opengl3 (not bare --headless).
# Requires: Godot 4.7.2-stable on PATH as `godot` (or Godot_v4.7.2-stable_linux.x86_64),
# Xvfb, and a running fragr-server. See docs/plans/tip-screenshots.md.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
# Prefer FRAGR_TIP_CAPTURE_DIR; else this repo. Ignore ambient OUT_DIR from other clones.
OUT_DIR="${FRAGR_TIP_CAPTURE_DIR:-$ROOT/docs/screenshots}"
DISPLAY_NUM="${DISPLAY_NUM:-99}"
SERVER_URL="${FRAGR_SERVER:-127.0.0.1:6767}"

GODOT_BIN=""
if command -v godot >/dev/null 2>&1; then
  GODOT_BIN="$(command -v godot)"
elif command -v Godot_v4.7.2-stable_linux.x86_64 >/dev/null 2>&1; then
  GODOT_BIN="$(command -v Godot_v4.7.2-stable_linux.x86_64)"
else
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
# Software GL helps on headless boxes without a GPU.
export LIBGL_ALWAYS_SOFTWARE="${LIBGL_ALWAYS_SOFTWARE:-1}"
export GALLIUM_DRIVER="${GALLIUM_DRIVER:-llvmpipe}"

# Kill stale Xvfb on this display if any.
if [ -e "/tmp/.X${DISPLAY_NUM}-lock" ]; then
  kill "$(cat "/tmp/.X${DISPLAY_NUM}-lock" 2>/dev/null)" >/dev/null 2>&1 || true
  rm -f "/tmp/.X${DISPLAY_NUM}-lock"
fi

Xvfb ":${DISPLAY_NUM}" -screen 0 1280x720x24 -ac >/tmp/fragr-xvfb.log 2>&1 &
XVFB_PID=$!
cleanup() {
  kill "$XVFB_PID" >/dev/null 2>&1 || true
}
trap cleanup EXIT
sleep 0.5

echo "Using Godot: $GODOT_BIN"

# First-run import so textures/shaders exist (skip if .godot already warm).
if [ ! -d "$ROOT/client/.godot/imported" ]; then
  echo "Importing Godot client assets (first run)..."
  "$GODOT_BIN" --path "$ROOT/client" --rendering-driver opengl3 --import --headless --quit-after 120 || true
fi

# Timed stills: Calibration/Larak Lot, Host bumper / Contested Frequency, scoreboard+killfeed, mid-join Host flash.
# Expect fragr-server with --solo-broadcast so episode chrome matches tip face.
# --quit-after is frames; give headroom for ~20s of wall clock.
# gl_compatibility / opengl3: Vulkan on Xvfb needs lavapipe; keep the simple path.
"$GODOT_BIN" --path "$ROOT/client" --rendering-driver opengl3 --quit-after 30000 \
  --script res://scripts/tip_capture.gd

echo "Tip capture finished. Inspect PNGs under $OUT_DIR"
