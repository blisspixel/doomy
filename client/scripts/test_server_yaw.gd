extends SceneTree

# Settles the server-to-Godot facing convention with arithmetic rather than a
# screenshot. Builds a real Node3D and a real Camera3D, applies the conversion,
# and asserts their basis vectors against the server's forward across a sweep.
#
# Run: godot --path client --headless --script res://scripts/test_server_yaw.gd

const STEPS: int = 16
const TOLERANCE: float = 1e-5

func _initialize() -> void:
	var ok: bool = true
	var script: GDScript = load("res://scripts/server_yaw.gd") as GDScript
	if script == null:
		push_error("test_server_yaw: failed to load server_yaw.gd")
		quit(1)
		return

	# Real nodes, but read through the local basis: these have no parent, so it
	# equals the global one, and it does not need a frame inside the tree.
	var pawn: Node3D = Node3D.new()
	var cam: Camera3D = Camera3D.new()

	for i in range(STEPS):
		var yaw: float = TAU * float(i) / float(STEPS)
		var want: Vector3 = script.forward(yaw)

		# A pawn's local +X must point where the server sends it, because that
		# is where the muzzle and the weapon sprite are parented.
		pawn.rotation = Vector3(0.0, script.pawn_rotation_y(yaw), 0.0)
		var pawn_forward: Vector3 = pawn.transform.basis.x
		if pawn_forward.distance_to(want) > TOLERANCE:
			push_error(
				"test_server_yaw: pawn +X at yaw %.3f was %s, wanted %s"
				% [yaw, str(pawn_forward), str(want)]
			)
			ok = false

		# A camera looks down its own -Z, and that has to be the same direction.
		cam.rotation = Vector3(0.0, script.camera_rotation_y(yaw), 0.0)
		var cam_forward: Vector3 = -cam.transform.basis.z
		if cam_forward.distance_to(want) > TOLERANCE:
			push_error(
				"test_server_yaw: camera -Z at yaw %.3f was %s, wanted %s"
				% [yaw, str(cam_forward), str(want)]
			)
			ok = false

	# Yaw zero is the case everyone reasons about, so pin it explicitly.
	if script.forward(0.0).distance_to(Vector3(1.0, 0.0, 0.0)) > TOLERANCE:
		push_error("test_server_yaw: yaw zero must be world +X")
		ok = false

	# And the quarter turn, because a sign error survives the zero case.
	if script.forward(PI / 2.0).distance_to(Vector3(0.0, 0.0, 1.0)) > TOLERANCE:
		push_error("test_server_yaw: a quarter turn must be world +Z")
		ok = false

	pawn.free()
	cam.free()

	if ok:
		print("test_server_yaw: PASS ", STEPS, " angles, pawn +X and camera -Z both agree with the server")
		quit(0)
	else:
		push_error("test_server_yaw: FAIL")
		quit(1)
