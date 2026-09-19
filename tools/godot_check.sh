#!/usr/bin/env bash
# Headless Godot checks for the client: import the project, parse every script,
# run the harnesses. Godot can exit 0 with errors in its log, so the log lines
# are the verdict. Set GODOT_BIN to a 4.7.2-stable binary, or have `godot` on PATH.
set -uo pipefail
cd "$(dirname "$0")/.."
GODOT="${GODOT_BIN:-godot}"
"$GODOT" --version || { echo "godot binary not found (set GODOT_BIN)"; exit 1; }
fail=0
import_log=$("$GODOT" --headless --path client --import 2>&1 || true)
if echo "$import_log" | grep -qiE "SCRIPT ERROR|Parse Error"; then
  echo "FAIL import"
  echo "$import_log" | grep -iE "SCRIPT ERROR|Parse Error|at:" | head -10
  fail=1
else
  echo "ok   import"
fi
for script in client/scripts/*.gd; do
  name=$(basename "$script")
  out=$("$GODOT" --headless --path client --check-only --script "res://scripts/$name" 2>&1 || true)
  if echo "$out" | grep -qiE "SCRIPT ERROR|Parse Error"; then
    echo "FAIL $name"
    echo "$out" | grep -iE "SCRIPT ERROR|Parse Error|at:" | head -6
    fail=1
  else
    echo "ok   $name"
  fi
done
for harness in test_radio test_far_cam_scale test_move_golden; do
  out=$("$GODOT" --headless --path client --script "res://scripts/$harness.gd" 2>&1 || true)
  if echo "$out" | grep -q "$harness: PASS"; then
    echo "ok   $harness harness"
  else
    echo "FAIL $harness harness"
    echo "$out" | grep -viE "^Godot Engine|^\s*$" | tail -8
    fail=1
  fi
done
exit $fail
