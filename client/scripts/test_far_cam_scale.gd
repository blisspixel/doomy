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

	# A nameplate must not grow without limit as the camera closes on it. A
	# Label3D has a fixed world size, so at two metres it was tall enough to
	# hide the room behind the fighter wearing it.
	if not pawn.has_method("compute_nameplate_scale"):
		push_error("test_far_cam_scale: player_pawn missing compute_nameplate_scale")
		ok = false
	else:
		var near_screen: float = float(pawn.call("compute_nameplate_scale", 2.0)) / 2.0
		var mid_screen: float = float(pawn.call("compute_nameplate_scale", 6.0)) / 6.0
		if absf(near_screen - mid_screen) > 0.01:
			push_error("test_far_cam_scale: nameplate screen size not capped up close")
			ok = false
		if float(pawn.call("compute_nameplate_scale", 0.05)) < 0.2:
			push_error("test_far_cam_scale: nameplate vanished at point blank")
			ok = false
		var far_here: float = float(pawn.call("compute_nameplate_scale", 30.0))
		var far_curve: float = float(pawn.call("compute_far_cam_scale", 30.0))
		if absf(far_here - far_curve) > 0.001:
			push_error("test_far_cam_scale: distant nameplate should follow the far curve")
			ok = false
		if ok:
			print("ok   nameplate scale capped near, follows far curve")

	# Smoothing must close the same fraction of the gap per unit of time
	# whatever the frame rate, or two machines render a fighter in different
	# places from identical snapshots.
	if not pawn.has_method("smoothing"):
		push_error("test_far_cam_scale: player_pawn missing smoothing")
		ok = false
	else:
		# One step at 1/30 s must equal two steps at 1/60 s, to rounding.
		var one_big: float = float(pawn.call("smoothing", 10.0, 1.0 / 30.0))
		var small: float = float(pawn.call("smoothing", 10.0, 1.0 / 60.0))
		var two_small: float = 1.0 - (1.0 - small) * (1.0 - small)
		if absf(one_big - two_small) > 1e-6:
			push_error("test_far_cam_scale: smoothing is frame-rate dependent")
			ok = false
		# It must never overshoot, which the naive speed-times-delta form does
		# for any delta above a tenth of a second at this speed.
		if float(pawn.call("smoothing", 10.0, 1.0)) > 1.0:
			push_error("test_far_cam_scale: smoothing overshot on a long frame")
			ok = false
		if float(pawn.call("smoothing", 10.0, 0.0)) != 0.0:
			push_error("test_far_cam_scale: a zero frame must move nothing")
			ok = false
		if ok:
			print("ok   smoothing is frame-rate independent")

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
