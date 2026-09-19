#!/usr/bin/env bash
# One-command local Solo Scrap: loopback server with rule bots + Godot human join.
# Offline-capable. No Tailscale. Port 6767.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

BIND="${FRAGR_BIND:-127.0.0.1:6767}"
BOTS="${FRAGR_BOTS:-4}"
MAP="${FRAGR_MAP:-1}"
MAP_ROTATE="${FRAGR_MAP_ROTATE:-0}"
SOLO_BROADCAST="${FRAGR_SOLO_BROADCAST:-1}"
GODOT_BIN="${GODOT_BIN:-}"

resolve_godot() {
  if [ -n "$GODOT_BIN" ] && [ -x "$GODOT_BIN" ]; then
    return 0
  fi
  if [ -x /workspace/godot/Godot_v4.7.2-stable_linux.x86_64 ]; then
    GODOT_BIN=/workspace/godot/Godot_v4.7.2-stable_linux.x86_64
    return 0
  fi
  if command -v godot >/dev/null 2>&1; then
    GODOT_BIN="$(command -v godot)"
    return 0
  fi
  if command -v Godot_v4.7.2-stable_linux.x86_64 >/dev/null 2>&1; then
    GODOT_BIN="$(command -v Godot_v4.7.2-stable_linux.x86_64)"
    return 0
  fi
  echo "Godot 4.7.2-stable not found. Set GODOT_BIN or install the editor binary." >&2
  exit 1
}

resolve_godot

echo "Building fragr-server..."
cargo build -p fragr-server

SERVER_BIN="$ROOT/target/debug/fragr-server"
if [ ! -x "$SERVER_BIN" ]; then
  echo "Missing $SERVER_BIN after build" >&2
  exit 1
fi

# Free stale listener on loopback solo port if we own it.
if command -v fuser >/dev/null 2>&1; then
  fuser -k "${BIND##*:}/tcp" >/dev/null 2>&1 || true
fi

MAP_ARGS=(--map "$MAP")
if [ "$MAP_ROTATE" = "1" ] || [ "$MAP_ROTATE" = "true" ]; then
  MAP_ARGS+=(--map-rotate)
fi
if [ "$SOLO_BROADCAST" = "1" ] || [ "$SOLO_BROADCAST" = "true" ]; then
  MAP_ARGS+=(--solo-broadcast)
fi

# Map face must match geometry: map 1 Arena Duel faces as Larak Lot; map 2 stays Compliance Yard.
if [ "$MAP" = "1" ] || [ "$MAP" = "arena" ] || [ "$MAP" = "arena-duel" ]; then
  MAP_FACE="Larak Lot"
else
  MAP_FACE="Compliance Yard (map $MAP)"
fi
echo "Starting Solo Broadcast server on $BIND with $BOTS NODS (map face=$MAP_FACE, map=$MAP)..."
"$SERVER_BIN" --bind "$BIND" --bots "$BOTS" "${MAP_ARGS[@]}" >/tmp/fragr-solo-server.log 2>&1 &
SERVER_PID=$!

cleanup() {
  if kill -0 "$SERVER_PID" >/dev/null 2>&1; then
    kill "$SERVER_PID" >/dev/null 2>&1 || true
    wait "$SERVER_PID" 2>/dev/null || true
  fi
}
trap cleanup EXIT INT TERM

# Wait until the port accepts connections (or server dies).
for _ in $(seq 1 50); do
  if ! kill -0 "$SERVER_PID" >/dev/null 2>&1; then
    echo "Server exited early. Log:" >&2
    cat /tmp/fragr-solo-server.log >&2 || true
    exit 1
  fi
  if command -v nc >/dev/null 2>&1; then
    if nc -z 127.0.0.1 "${BIND##*:}" >/dev/null 2>&1; then
      break
    fi
  else
    sleep 0.2
    break
  fi
  sleep 0.1
done

export FRAGR_SERVER="$BIND"
export FRAGR_SOLO=1
export FRAGR_MAP="$MAP"

echo "Launching Solo Broadcast Episode 0 (human on loopback). Server pid=$SERVER_PID"
echo "Controls: WASD move, mouse look, LMB fire, L leave to spectate, ESC mouse"
# Jump straight into arena as human; boot menu still available via plain F5.
"$GODOT_BIN" --path "$ROOT/client" res://scenes/main.tscn -- --solo
