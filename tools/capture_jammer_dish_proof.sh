#!/usr/bin/env bash
# Capture jammer dish proof stills via Xvfb + opengl3 (no server required).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
OUT_DIR="${FRAGR_TIP_CAPTURE_DIR:-$ROOT/docs/screenshots}"
DISPLAY_NUM="${DISPLAY_NUM:-98}"

GODOT_BIN=""
if command -v godot >/dev/null 2>&1; then
  GODOT_BIN="$(command -v godot)"
elif command -v Godot_v4.7.2-stable_linux.x86_64 >/dev/null 2>&1; then
  GODOT_BIN="$(command -v Godot_v4.7.2-stable_linux.x86_64)"
else
  echo "godot not on PATH" >&2
  exit 1
fi
if ! command -v Xvfb >/dev/null 2>&1; then
  echo "Xvfb not found" >&2
  exit 1
fi

mkdir -p "$OUT_DIR"
export DISPLAY=":${DISPLAY_NUM}"
export FRAGR_TIP_CAPTURE_DIR="$OUT_DIR"
export LIBGL_ALWAYS_SOFTWARE="${LIBGL_ALWAYS_SOFTWARE:-1}"
export GALLIUM_DRIVER="${GALLIUM_DRIVER:-llvmpipe}"

if [ -e "/tmp/.X${DISPLAY_NUM}-lock" ]; then
  kill "$(cat "/tmp/.X${DISPLAY_NUM}-lock" 2>/dev/null)" >/dev/null 2>&1 || true
  rm -f "/tmp/.X${DISPLAY_NUM}-lock"
fi
Xvfb ":${DISPLAY_NUM}" -screen 0 1280x720x24 -ac >/tmp/fragr-jammer-xvfb.log 2>&1 &
XVFB_PID=$!
cleanup() { kill "$XVFB_PID" >/dev/null 2>&1 || true; }
trap cleanup EXIT
sleep 0.4

echo "Using Godot: $GODOT_BIN"
"$GODOT_BIN" --path "$ROOT/client" --rendering-driver opengl3 --quit-after 6000 \
  --script res://scripts/capture_jammer_dish_proof.gd
echo "Jammer dish proof stills under $OUT_DIR"
