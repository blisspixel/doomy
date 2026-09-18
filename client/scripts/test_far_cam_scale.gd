extends SceneTree

# Headless check for far-cam billboard scale curve (no scene / server required).
# Run: godot --path client --headless --script res://scripts/test_far_cam_scale.gd

func _initialize() -> void:
	var ok: bool = true
	var pawn_script: GDScript = load("res://scripts/player_pawn.gd") as GDScript
	if pawn_script == null:
		push_error("test_far_cam_scale: failed to load player_pawn.gd")
		quit(1)
		return

	var pawn: Node = pawn_script.new() as Node
	if pawn == null or not pawn.has_method("compute_far_cam_scale"):
		push_error("test_far_cam_scale: player_pawn missing compute_far_cam_scale")
		quit(1)
		return

	# Close follow (~12m) and at REF must stay 1.0 so follow cam is unchanged.
	ok = _expect_near(pawn, 0.0, 1.0, "dist 0") and ok
	ok = _expect_near(pawn, 12.0, 1.0, "follow / ref 12m") and ok
	ok = _expect_near(pawn, 11.0, 1.0, "inside follow") and ok

	# Mid / far: proportional boost, never below 1, never above max.
	var mid: float = float(pawn.call("compute_far_cam_scale", 24.0))
	var far: float = float(pawn.call("compute_far_cam_scale", 36.0))
	var capped: float = float(pawn.call("compute_far_cam_scale", 80.0))
	ok = _expect_near(pawn, 24.0, 2.0, "mid 24m") and ok
	ok = _expect_near(pawn, 36.0, 3.0, "far 36m") and ok
	if absf(capped - 3.5) > 0.001:
		push_error("test_far_cam_scale: cap expected 3.5 got %s" % str(capped))
		ok = false

	# Overview tip pose distance ~sqrt(22^2+28^2) ~= 35.6; must be clearly boosted.
	var overview: float = float(pawn.call("compute_far_cam_scale", 35.6))
	if overview < 2.8:
		push_error("test_far_cam_scale: overview scale too small: %s" % str(overview))
		ok = false

	pawn.free()

	if ok:
		print("test_far_cam_scale: PASS mid=", mid, " far=", far, " overview=", overview, " cap=", capped)
		quit(0)
	else:
		push_error("test_far_cam_scale: FAIL")
		quit(1)

func _expect_near(pawn: Object, dist: float, expected: float, label: String) -> bool:
	var got: float = float(pawn.call("compute_far_cam_scale", dist))
	if absf(got - expected) > 0.001:
		push_error("test_far_cam_scale: %s expected %s got %s" % [label, str(expected), str(got)])
		return false
	return true
