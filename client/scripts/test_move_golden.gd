extends SceneTree
## Headless check that the GDScript movement step matches the Rust step
## against the golden vectors in client/golden/move_vectors.json. Run with:
## godot --headless --path client --script res://scripts/test_move_golden.gd
##
## Tolerances: 1e-4 on every recorded state for the short cases, and 1e-2 at
## the checkpoints of the thousand-step wander, since the two languages do
## their arithmetic in different precisions.

const GOLDEN_PATH := "res://golden/move_vectors.json"
const STEP_TOLERANCE := 1e-4
const LONG_TOLERANCE := 1e-2
const FIELDS := ["x", "z", "vx", "vz", "yaw"]

var failures: Array = []


func _check(condition: bool, message: String) -> void:
	if not condition:
		failures.append(message)


func _initialize() -> void:
	var file := FileAccess.open(GOLDEN_PATH, FileAccess.READ)
	if file == null:
		printerr("test_move_golden: FAIL cannot open " + GOLDEN_PATH)
		quit(1)
		return
	var parsed: Variant = JSON.parse_string(file.get_as_text())
	if typeof(parsed) != TYPE_DICTIONARY:
		printerr("test_move_golden: FAIL golden file is not a JSON object")
		quit(1)
		return
	var golden: Dictionary = parsed
	_check(int(golden.get("version", 0)) == 1, "golden version is 1")
	_check(absf(float(golden.get("radius", 0.0)) - MoveStep.RADIUS) < 1e-9, "radius matches")
	_check(absf(float(golden.get("top_speed", 0.0)) - MoveStep.TOP_SPEED) < 1e-9, "top speed matches")
	_check(absf(float(golden.get("tau_accel", 0.0)) - MoveStep.TAU_ACCEL) < 1e-9, "tau_accel matches")
	_check(absf(float(golden.get("tau_decel", 0.0)) - MoveStep.TAU_DECEL) < 1e-9, "tau_decel matches")
	var dt: float = float(golden.get("dt", 0.0))
	_check(absf(dt - MoveStep.DT_60HZ) < 1e-7, "dt is the 60 Hz step")

	var cases: Array = golden.get("cases", [])
	_check(cases.size() >= 10, "at least ten cases, got %d" % cases.size())
	var checked := 0
	for case: Dictionary in cases:
		checked += _run_case(case, dt)
	_test_unit_behaviour()

	if failures.is_empty():
		print("test_move_golden: PASS (%d cases, %d states)" % [cases.size(), checked])
		quit(0)
	else:
		for failure: String in failures:
			printerr("test_move_golden: FAIL " + failure)
		quit(1)


func _run_case(case: Dictionary, dt: float) -> int:
	var name: String = str(case.get("name", "?"))
	var arena: Dictionary = case["arena"]
	var state: Dictionary = (case["start"] as Dictionary).duplicate()
	var inputs: Array = case["inputs"]
	var expected: Array = case["expected"]
	var stride: int = int(case.get("stride", 1))
	var tolerance: float = LONG_TOLERANCE if stride > 1 else STEP_TOLERANCE
	var checked := 0
	var next_expected := 0
	for i in range(inputs.size()):
		state = MoveStep.step(state, inputs[i], dt, arena)
		if (i + 1) % stride == 0:
			if next_expected >= expected.size():
				_check(false, "%s: more checkpoints than expected states" % name)
				break
			var want: Dictionary = expected[next_expected]
			for field: String in FIELDS:
				var a: float = float(state[field])
				var b: float = float(want[field])
				if absf(a - b) > tolerance:
					_check(false, "%s step %d %s: got %.6f want %.6f" % [name, i + 1, field, a, b])
					break
			next_expected += 1
			checked += 1
	_check(next_expected == expected.size(), "%s: consumed every expected state" % name)
	return checked


func _test_unit_behaviour() -> void:
	_check(absf(MoveStep.normalize_yaw(-0.5) - (2.0 * PI - 0.5)) < 1e-6, "negative yaw wraps up")
	_check(MoveStep.normalize_yaw(2.0 * PI) < 1e-6, "full turn wraps to zero")
	_check(MoveStep.normalize_yaw(NAN) == 0.0, "nan yaw becomes zero")
	var none := MoveStep.make_input(false, false, false, false, 0.0)
	_check(MoveStep.wish_dir(none, 0.0) == Vector2.ZERO, "no keys, no wish")
	var diag := MoveStep.make_input(true, false, false, true, 0.0)
	_check(absf(MoveStep.wish_dir(diag, 0.0).length() - 1.0) < 1e-6, "diagonal wish is unit")
	var arena := {"half": 25.0, "solids": [MoveStep.solid_from_center(0.0, 0.0, 1.0, 1.0)]}
	_check(MoveStep.arena_blocked(arena, 1.4, 0.0), "radius inflates solids")
	_check(not MoveStep.arena_blocked(arena, 1.6, 0.0), "outside the inflated solid is free")
	var s := MoveStep.make_state(10.0, 0.0, 0.0)
	var go := MoveStep.make_input(true, false, false, false, 0.0)
	for i in range(30):
		s = MoveStep.step(s, go, MoveStep.DT_60HZ, arena)
	_check(absf(float(s["vx"]) - MoveStep.TOP_SPEED) < 1e-3, "reaches top speed in half a second")
