class_name MoveStep
extends RefCounted
## The shared movement step, mirrored line for line from server/src/movement.rs.
## Pure functions over dictionaries: no nodes, no physics server, no randomness.
## The headless harness test_move_golden.gd proves this mirror matches the
## Rust step against client/golden/move_vectors.json. Change the model in Rust,
## regenerate the vectors, then bring this file into line.

const RADIUS: float = 0.5
const TOP_SPEED: float = 5.0
const TAU_ACCEL: float = 0.06
const TAU_DECEL: float = 0.04
const DT_60HZ: float = 1.0 / 60.0


static func make_state(x: float, z: float, yaw: float) -> Dictionary:
	return {"x": x, "z": z, "vx": 0.0, "vz": 0.0, "yaw": yaw}


static func make_input(forward: bool, back: bool, left: bool, right: bool, yaw: float, speed_scale: float = 1.0) -> Dictionary:
	return {"forward": forward, "back": back, "left": left, "right": right, "yaw": yaw, "speed_scale": speed_scale}


static func solid_from_center(cx: float, cz: float, half_x: float, half_z: float) -> Dictionary:
	return {"min_x": cx - half_x, "max_x": cx + half_x, "min_z": cz - half_z, "max_z": cz + half_z}


static func solid_blocks(solid: Dictionary, x: float, z: float, radius: float) -> bool:
	return (
		x >= solid["min_x"] - radius
		and x <= solid["max_x"] + radius
		and z >= solid["min_z"] - radius
		and z <= solid["max_z"] + radius
	)


static func arena_blocked(arena: Dictionary, x: float, z: float) -> bool:
	for solid in arena["solids"]:
		if solid_blocks(solid, x, z, RADIUS):
			return true
	return false


static func arena_clamp(arena: Dictionary, x: float, z: float) -> Vector2:
	var limit: float = float(arena["half"]) - RADIUS
	return Vector2(clampf(x, -limit, limit), clampf(z, -limit, limit))


static func normalize_yaw(yaw: float) -> float:
	if not is_finite(yaw):
		return 0.0
	var two_pi: float = 2.0 * PI
	var y: float = fmod(yaw, two_pi)
	if y < 0.0:
		y += two_pi
	if y >= two_pi:
		y -= two_pi
	return y


static func wish_dir(input: Dictionary, yaw: float) -> Vector2:
	var dx: float = 0.0
	var dz: float = 0.0
	if input.get("forward", false):
		dx += cos(yaw)
		dz += sin(yaw)
	if input.get("back", false):
		dx -= cos(yaw)
		dz -= sin(yaw)
	if input.get("left", false):
		dx += cos(yaw - PI / 2.0)
		dz += sin(yaw - PI / 2.0)
	if input.get("right", false):
		dx += cos(yaw + PI / 2.0)
		dz += sin(yaw + PI / 2.0)
	var len: float = sqrt(dx * dx + dz * dz)
	if len > 0.0:
		return Vector2(dx / len, dz / len)
	return Vector2.ZERO


## Advance one fighter by dt seconds. Same order as the Rust step: yaw, wish,
## velocity approach, integrate, clamp, axis-separated slide with the blocked
## axis velocity zeroed.
static func step(state: Dictionary, input: Dictionary, dt: float, arena: Dictionary) -> Dictionary:
	var yaw: float = normalize_yaw(float(input.get("yaw", 0.0)))
	var wish: Vector2 = wish_dir(input, yaw)
	var scale: float = float(input.get("speed_scale", 1.0))
	if not is_finite(scale):
		scale = 1.0
	scale = clampf(scale, 0.0, 1.0)
	var target_x: float = wish.x * TOP_SPEED * scale
	var target_z: float = wish.y * TOP_SPEED * scale

	var vx: float = float(state["vx"])
	var vz: float = float(state["vz"])
	var current_speed: float = sqrt(vx * vx + vz * vz)
	var target_speed: float = sqrt(target_x * target_x + target_z * target_z)
	var tau: float = TAU_ACCEL if target_speed > current_speed else TAU_DECEL
	var blend: float = minf(dt / tau, 1.0)
	vx = vx + (target_x - vx) * blend
	vz = vz + (target_z - vz) * blend

	var old_x: float = float(state["x"])
	var old_z: float = float(state["z"])
	var clamped: Vector2 = arena_clamp(arena, old_x + vx * dt, old_z + vz * dt)
	var nx: float = clamped.x
	var nz: float = clamped.y

	var x: float
	var z: float
	if not arena_blocked(arena, nx, nz):
		x = nx
		z = nz
	elif not arena_blocked(arena, nx, old_z):
		vz = 0.0
		x = nx
		z = old_z
	elif not arena_blocked(arena, old_x, nz):
		vx = 0.0
		x = old_x
		z = nz
	else:
		vx = 0.0
		vz = 0.0
		var stay: Vector2 = arena_clamp(arena, old_x, old_z)
		x = stay.x
		z = stay.y

	return {"x": x, "z": z, "vx": vx, "vz": vz, "yaw": yaw}
