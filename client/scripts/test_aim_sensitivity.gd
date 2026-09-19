extends SceneTree
## Headless check that mouse look uses portable units and lands in the band
## competitive players actually use. Run with:
## godot --headless --path client --script res://scripts/test_aim_sensitivity.gd

const CamScript := preload("res://scripts/spectator_cam.gd")

var failures: Array = []


func _check(condition: bool, message: String) -> void:
	if not condition:
		failures.append(message)


func _initialize() -> void:
	_test_convention()
	_test_default_is_in_the_band()
	_test_edges()
	if failures.is_empty():
		print("test_aim_sensitivity: PASS")
		quit(0)
	else:
		for failure: String in failures:
			printerr("test_aim_sensitivity: FAIL " + failure)
		quit(1)


func _test_convention() -> void:
	# The Source convention: 0.022 degrees per count at sensitivity 1.0.
	_check(
		absf(CamScript.DEGREES_PER_COUNT - 0.022) < 1e-9,
		"degrees per count should be the Source 0.022, got %f" % CamScript.DEGREES_PER_COUNT
	)
	# A known pairing: Counter-Strike's default 1.25 at 800 counts per inch is
	# about 41.6 cm per 360. Same arithmetic, so it must reproduce.
	var cs := CamScript.cm_per_360(1.25, 800.0)
	_check(absf(cs - 41.6) < 0.2, "sensitivity 1.25 at 800 cpi should be about 41.6 cm/360, got %.2f" % cs)
	# Doubling sensitivity halves the travel.
	var a := CamScript.cm_per_360(1.0, 800.0)
	var b := CamScript.cm_per_360(2.0, 800.0)
	_check(absf(a / b - 2.0) < 1e-6, "twice the sensitivity should be half the travel")
	# Doubling the mouse resolution halves the travel too.
	var c := CamScript.cm_per_360(1.0, 1600.0)
	_check(absf(a / c - 2.0) < 1e-6, "twice the counts per inch should be half the travel")


func _test_default_is_in_the_band() -> void:
	var cam = CamScript.new()
	var cm := CamScript.cm_per_360(cam.mouse_sensitivity, 800.0)
	_check(
		cm >= 30.0 and cm <= 50.0,
		"the default should sit in the 30 to 50 cm/360 band at 800 cpi, got %.1f" % cm
	)
	# The old default was 0.003 radians per count, which is far outside it.
	var old_cm := (360.0 / (rad_to_deg(0.003) * 800.0)) * 2.54
	_check(old_cm < 10.0, "sanity: the old default really was under 10 cm/360, got %.1f" % old_cm)
	cam.free()


func _test_edges() -> void:
	_check(CamScript.cm_per_360(0.0, 800.0) == 0.0, "zero sensitivity has no meaningful travel")
	_check(CamScript.cm_per_360(1.0, 0.0) == 0.0, "zero counts per inch has no meaningful travel")
	_check(CamScript.cm_per_360(-1.0, 800.0) == 0.0, "negative sensitivity has no meaningful travel")
