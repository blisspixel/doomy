#!/usr/bin/env bash
# Visual QA tour: boot a server with bots, walk every player-facing state,
# save a still and a measurement for each, and build a contact sheet.
#
# Needs a real framebuffer. On Windows run it from Git Bash and it uses the
# console binary directly. On Linux it wraps the run in Xvfb, because bare
# --headless has no framebuffer and every still comes back empty.
#
# Usage: tools/qa_tour.sh [--publish] [output directory]
#   --publish also copies the approved stills into docs/screenshots/, which is
#   what the README shows. Run it after any change a player would see.
# Default output: .agents/qa/<stamp>/ (gitignored), with a `latest` copy.
set -uo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
STAMP="$(date -u +%Y%m%d-%H%M%S)"
PUBLISH=0
if [ "${1:-}" = "--publish" ]; then
  PUBLISH=1
  shift
fi
OUT_DIR="${1:-$ROOT/.agents/qa/$STAMP}"
SERVER_PORT="${FRAGR_PORT:-6767}"
SERVER_URL="${FRAGR_SERVER:-127.0.0.1:$SERVER_PORT}"
BOTS="${FRAGR_QA_BOTS:-5}"

mkdir -p "$OUT_DIR"

# Godot 4.7.2. FRAGR_GODOT wins; then the console binary on Windows; then PATH.
GODOT_BIN="${FRAGR_GODOT:-}"
if [ -z "$GODOT_BIN" ]; then
  for candidate in \
    "/c/GitHub/_toolchains/godot/Godot_v4.7.2-stable_win64_console.exe" \
    "/c/Program Files/Godot/Godot_v4.7.2-stable_win64_console.exe" \
    "$HOME/godot/Godot_v4.7.2-stable_win64_console.exe"; do
    [ -x "$candidate" ] && GODOT_BIN="$candidate" && break
  done
fi
if [ -z "$GODOT_BIN" ]; then
  if command -v godot >/dev/null 2>&1; then
    GODOT_BIN="$(command -v godot)"
  elif command -v Godot_v4.7.2-stable_linux.x86_64 >/dev/null 2>&1; then
    GODOT_BIN="$(command -v Godot_v4.7.2-stable_linux.x86_64)"
  fi
fi
if [ -z "$GODOT_BIN" ]; then
  echo "qa_tour: no Godot 4.7.2 found. Set FRAGR_GODOT to the binary." >&2
  exit 1
fi

echo "qa_tour: godot   $GODOT_BIN"
echo "qa_tour: output  $OUT_DIR"

# A server with bots, so the tour has a match to photograph.
cargo build -p fragr-server --release >/dev/null 2>&1 || {
  echo "qa_tour: server build failed" >&2; exit 1; }
"$ROOT/target/release/fragr-server" --bind "0.0.0.0:$SERVER_PORT" --bots "$BOTS" >"$OUT_DIR/server.log" 2>&1 &
SERVER_PID=$!
cleanup() {
  kill "$SERVER_PID" >/dev/null 2>&1
  [ -n "${XVFB_PID:-}" ] && kill "$XVFB_PID" >/dev/null 2>&1
  return 0
}
trap cleanup EXIT

# Wait for the port rather than sleeping on faith. A tour against a server
# that never started photographs an empty grey room and calls it a game, which
# is exactly what happened the first time this ran.
UP=0
for _ in $(seq 1 50); do
  if (exec 3<>"/dev/tcp/127.0.0.1/$SERVER_PORT") 2>/dev/null; then exec 3>&- 3<&-; UP=1; break; fi
  if ! kill -0 "$SERVER_PID" 2>/dev/null; then break; fi
  sleep 0.2
done
if [ "$UP" -ne 1 ]; then
  echo "qa_tour: server never came up on port $SERVER_PORT" >&2
  tail -5 "$OUT_DIR/server.log" >&2
  exit 1
fi

export FRAGR_SERVER="$SERVER_URL"
export FRAGR_QA_DIR="$OUT_DIR"

# Import the project first. Running a script against a project Godot has never
# imported gives a client with no textures: every scene that references one
# fails to parse, and the tour photographs an empty grey room with a working
# HUD over it. Incremental, so it costs nothing once warm.
echo "qa_tour: importing assets"
"$GODOT_BIN" --path "$ROOT/client" --headless --import >"$OUT_DIR/import.log" 2>&1 || {
  echo "qa_tour: asset import failed; see $OUT_DIR/import.log" >&2
  exit 1
}

RUN=("$GODOT_BIN" --path "$ROOT/client" --rendering-driver opengl3
     --resolution 1280x720 --script res://scripts/qa_tour.gd)

if [ "$(uname -s)" != "Linux" ]; then
  "${RUN[@]}"
  STATUS=$?
else
  command -v Xvfb >/dev/null 2>&1 || {
    echo "qa_tour: Xvfb needed on Linux for a framebuffer" >&2; exit 1; }
  DISPLAY_NUM="${DISPLAY_NUM:-99}"
  Xvfb ":$DISPLAY_NUM" -screen 0 1280x720x24 >/dev/null 2>&1 &
  XVFB_PID=$!
  sleep 1
  DISPLAY=":$DISPLAY_NUM" LIBGL_ALWAYS_SOFTWARE=1 GALLIUM_DRIVER=llvmpipe "${RUN[@]}"
  STATUS=$?
fi

if [ "$STATUS" -ne 0 ]; then
  echo "qa_tour: the tour failed (exit $STATUS). Server log: $OUT_DIR/server.log" >&2
  exit "$STATUS"
fi

# A `latest` pointer so the critique step never has to know the stamp.
rm -rf "$ROOT/.agents/qa/latest"
mkdir -p "$ROOT/.agents/qa/latest"
cp "$OUT_DIR"/*.png "$OUT_DIR"/manifest.json "$ROOT/.agents/qa/latest/" 2>/dev/null

# Screenshots in the README go stale the moment the HUD changes, and a stale
# screenshot is worse than none because it claims to be the current build.
# `--publish` copies the approved subset into docs/screenshots/ so refreshing
# them is one command rather than a thing someone remembers to do.
if [ "${PUBLISH:-0}" = "1" ]; then
  published=0
  while IFS='|' read -r src dest; do
    [ -z "$src" ] && continue
    if [ -f "$OUT_DIR/$src" ]; then
      cp "$OUT_DIR/$src" "$ROOT/docs/screenshots/$dest"
      published=$((published + 1))
    else
      echo "qa_tour: cannot publish $src; it was not captured" >&2
    fi
  done <<'SHOTS'
05_hud_first_person.png|tour_first_person_16x9.png
04_combat_follow.png|tour_combat_follow_16x9.png
03_arena_overview.png|tour_arena_overview_16x9.png
07_shot_effects_strip.png|tour_shot_strip.png
SHOTS
  echo "qa_tour: published $published stills into docs/screenshots/"
fi

echo "qa_tour: done. Contact sheet: $OUT_DIR/contact.png"
