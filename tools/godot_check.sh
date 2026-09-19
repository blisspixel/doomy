#!/usr/bin/env bash
# Headless Godot checks for the client: import the project, parse every script,
# run the harnesses. Godot can exit 0 with errors in its log, so the log lines
# are the verdict. Set GODOT_BIN to a 4.7.2-stable binary, or have `godot` on PATH.
set -uo pipefail
cd "$(dirname "$0")/.."
GODOT="${GODOT_BIN:-godot}"
TMP_PROJECT="${TMPDIR:-/tmp}/fragr-project-godot.$$"
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

for harness in test_radio test_far_cam_scale test_move_golden test_aim_sensitivity test_jammer_dish_silhouette test_spectator_stance_chips; do

  out=$("$GODOT" --headless --path client --script "res://scripts/$harness.gd" 2>&1 || true)
  if echo "$out" | grep -q "$harness: PASS"; then
    echo "ok   $harness harness"
  else
    echo "FAIL $harness harness"
    echo "$out" | grep -viE "^Godot Engine|^\s*$" | tail -8
    fail=1
  fi
done

# GDScript static typing ratchet.
#
# Godot cannot override a project setting from the command line, and warnings
# set to "error" would stop the game loading, so the setting is flipped for the
# duration of the count and restored. The baseline is a number in the repo: it
# may go down, never up. This is how a large typing debt gets paid off without
# one enormous rewrite, and how it cannot quietly grow while that happens.
BASELINE_FILE="client/.gdscript-typing-baseline"
if [ -f "$BASELINE_FILE" ]; then
  baseline=$(tr -dc '0-9' < "$BASELINE_FILE")
  cp client/project.godot "$TMP_PROJECT"
  restore_project() { cp "$TMP_PROJECT" client/project.godot; rm -f "$TMP_PROJECT"; }
  trap restore_project EXIT
  sed -i 's|gdscript/warnings/untyped_declaration=1|gdscript/warnings/untyped_declaration=2|' client/project.godot
  untyped=0
  for script in client/scripts/*.gd; do
    name=$(basename "$script")
    n=$("$GODOT" --headless --path client --check-only --script "res://scripts/$name" 2>&1 | grep -c "Warning treated as error" || true)
    untyped=$((untyped + n))
  done
  restore_project
  trap - EXIT
  if [ "$untyped" -gt "$baseline" ]; then
    echo "FAIL gdscript typing: $untyped untyped declarations, baseline $baseline"
    echo "     Give new variables a static type, or explain why in the pull request."
    fail=1
  elif [ "$untyped" -lt "$baseline" ]; then
    # Never fail for an improvement. The count can also differ between a
    # developer machine and CI, because analysing a script pulls in whatever it
    # instantiates, so only an increase is treated as a regression.
    echo "ok   gdscript typing: $untyped untyped declarations, under the $baseline baseline"
    echo "     Worth lowering the baseline in $BASELINE_FILE to $untyped."
  else
    echo "ok   gdscript typing: $untyped untyped declarations, at baseline"
  fi
fi

exit $fail
