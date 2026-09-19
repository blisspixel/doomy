extends Node3D

@export var move_speed = 10.0
## Mouse look in the units every other shooter uses: degrees of turn per mouse
## count at sensitivity 1.0. 0.022 is the Source convention, so a player can
## paste a sensitivity number from another game and get the same hand movement.
## At 800 counts per inch, sensitivity 1.5 is 34.7 cm per 360 degrees, inside
## the band competitive players actually use. The old 0.003 radians per count
## was 6.6 cm per 360, about six times faster than a Counter-Strike default.
const DEGREES_PER_COUNT := 0.022
@export var mouse_sensitivity := 1.5
## Radians per second of turn at full deflection. 2.8 is about a hundred and
## sixty degrees a second, which is roughly Doom's walking turn, and it is the
## rate a keyboard gets because a key is either down or it is not. A stick gets
## everything below it as well.
@export var stick_look_sensitivity = 2.8
@export var stick_turn_scale = 18.0
@export var stick_deadzone = 0.25
@export var auto_cycle_interval = 6.0

var follow_mode = true
var follow_target_index = 0
var available_targets = []
var auto_cycle_timer = 0.0
var frag_follow_timer = 0.0
var frag_follow_target_id = ""
# tip_capture: freeze follow / frag yank while posing at dish origin.
# Held transform is re-applied every frame so set_fp_mode / other yanks cannot stick.
var tip_pose_lock = false
var tip_locked_transform: Transform3D = Transform3D.IDENTITY
var tip_has_locked_transform = false
var camera_shake_intensity = 0.0
var camera_zoom_offset = 0.0

var mouse_motion = Vector2.ZERO

# First-person join: eye follow on local pawn. Pitch is client-only.
var fp_mode = false
var fp_target: Node3D = null
var fp_pitch = 0.0
## First-person facing the client owns. Mouse and stick move it on the frame
## the input arrives; the server is told the absolute value and agrees. Turn
## bits stay for agents and for anything that does not send a yaw.
var fp_yaw = 0.0
var turn_accum = 0.0
## Eye height above the fighter's feet.
const FP_EYE_ABOVE_FEET = 1.6
## The server's y for a standing fighter. Its position is a reference point,
## not the floor, so the eye offset from it is the difference of the two. When
## the fighters were put back on the floor this was missed, and the camera sat
## at three metres looking down on a world built for one and a half.
const FP_SERVER_REFERENCE_Y = 1.5
const FP_EYE_HEIGHT = FP_EYE_ABOVE_FEET - FP_SERVER_REFERENCE_Y
const FP_FORWARD_NUDGE = 0.15
const TURN_ACCUM_THRESHOLD = 2.5

func _ready():
	Input.set_mouse_mode(Input.MOUSE_MODE_CAPTURED)

func _input(event):
	if event is InputEventMouseMotion:
		mouse_motion = event.relative

	# Escape belongs to the pause menu now. The mouse is released and recaptured
	# by whatever opens over the match, so two things no longer fight for it.

func _process(delta):
	camera_shake_intensity = lerp(camera_shake_intensity, 0.0, delta * 10.0)
	camera_zoom_offset = lerp(camera_zoom_offset, 0.0, delta * 5.0)

	if tip_pose_lock:
		mouse_motion = Vector2.ZERO
		if tip_has_locked_transform:
			global_transform = tip_locked_transform
		return

	var mouse_captured = Input.get_mouse_mode() == Input.MOUSE_MODE_CAPTURED
	var pad_active = _gamepad_look_active() or _gamepad_move_active()

	# Mouse-free still allows gamepad scrap / spectator control.
	if not mouse_captured and not pad_active:
		mouse_motion = Vector2.ZERO
		return

	if not mouse_captured:
		mouse_motion = Vector2.ZERO

	if fp_mode:
		_process_fp(delta)
		return

	if Input.is_action_just_pressed("cycle_cam"):
		cycle_next_target()
		if not follow_mode:
			follow_mode = true
			print("Follow cam ON (F / pad cycles targets)")

	if Input.is_action_just_pressed("toggle_follow"):
		toggle_follow_mode()

	if frag_follow_timer > 0:
		frag_follow_timer -= delta
		if frag_follow_timer <= 0:
			frag_follow_target_id = ""
		else:
			_follow_frag_target()
			return

	if follow_mode and len(available_targets) > 0:
		auto_cycle_timer += delta
		if auto_cycle_timer >= auto_cycle_interval:
			cycle_next_target()
			auto_cycle_timer = 0.0
		_follow_target()
	else:
		_free_fly(delta)

func _gamepad_move_active() -> bool:
	return (
		Input.get_action_strength("move_forward") > stick_deadzone
		or Input.get_action_strength("move_back") > stick_deadzone
		or Input.get_action_strength("move_left") > stick_deadzone
		or Input.get_action_strength("move_right") > stick_deadzone
	)

func _gamepad_look_active() -> bool:
	return (
		Input.get_action_strength("turn_left") > stick_deadzone
		or Input.get_action_strength("turn_right") > stick_deadzone
		or Input.get_action_strength("look_up") > stick_deadzone
		or Input.get_action_strength("look_down") > stick_deadzone
	)

func _apply_stick_look(delta: float, apply_yaw_to_node: bool) -> void:
	var turn_l = Input.get_action_strength("turn_left")
	var turn_r = Input.get_action_strength("turn_right")
	var look_u = Input.get_action_strength("look_up")
	var look_d = Input.get_action_strength("look_down")

	var yaw = 0.0
	if turn_l > stick_deadzone:
		yaw -= (turn_l - stick_deadzone) / (1.0 - stick_deadzone)
	if turn_r > stick_deadzone:
		yaw += (turn_r - stick_deadzone) / (1.0 - stick_deadzone)

	var pitch = 0.0
	if look_u > stick_deadzone:
		pitch -= (look_u - stick_deadzone) / (1.0 - stick_deadzone)
	if look_d > stick_deadzone:
		pitch += (look_d - stick_deadzone) / (1.0 - stick_deadzone)

	if abs(yaw) > 0.0:
		if apply_yaw_to_node:
			rotation.y -= yaw * stick_look_sensitivity * delta
		else:
			fp_yaw = wrapf(fp_yaw + yaw * stick_look_sensitivity * delta, 0.0, TAU)

	if abs(pitch) > 0.0:
		if apply_yaw_to_node:
			rotation.x -= pitch * stick_look_sensitivity * delta
			rotation.x = clamp(rotation.x, -PI / 2, PI / 2)
		else:
			fp_pitch -= pitch * stick_look_sensitivity * delta
			fp_pitch = clamp(fp_pitch, -1.15, 1.15)

func _free_fly(delta):
	if mouse_motion.length() > 0:
		var radians_per_count := _radians_per_count()
		rotation.y -= mouse_motion.x * radians_per_count
		rotation.x -= mouse_motion.y * radians_per_count
		rotation.x = clamp(rotation.x, -PI / 2, PI / 2)
		mouse_motion = Vector2.ZERO

	_apply_stick_look(delta, true)

	var input_dir = Vector3.ZERO
	input_dir.z -= Input.get_action_strength("move_forward")
	input_dir.z += Input.get_action_strength("move_back")
	input_dir.x -= Input.get_action_strength("move_left")
	input_dir.x += Input.get_action_strength("move_right")

	var speed_mult = 1.0
	if Input.is_key_pressed(KEY_SHIFT):
		speed_mult = 3.0

	if input_dir.length() > 0.01:
		input_dir = input_dir.normalized()
		var move_vec = transform.basis * input_dir
		position += move_vec * move_speed * speed_mult * delta

func _follow_target():
	if len(available_targets) == 0:
		return

	follow_target_index = follow_target_index % len(available_targets)
	var target = available_targets[follow_target_index]

	if is_instance_valid(target):
		var target_pos = target.global_position
		var offset = Vector3(0, 5.5, 10.5 + camera_zoom_offset)

		if camera_shake_intensity > 0:
			offset += Vector3(
				randf_range(-camera_shake_intensity, camera_shake_intensity),
				randf_range(-camera_shake_intensity, camera_shake_intensity),
				0
			)

		var cam_pos = target_pos + offset.rotated(Vector3.UP, target.rotation.y)
		position = position.lerp(cam_pos, 0.1)

		var look_target = target_pos + Vector3(0, 1.5, 0)
		var desired_transform = global_transform.looking_at(look_target, Vector3.UP)
		global_transform = global_transform.interpolate_with(desired_transform, 0.15)
	else:
		cycle_next_target()

func toggle_follow_mode():
	follow_mode = not follow_mode
	auto_cycle_timer = 0.0
	if follow_mode:
		print("Follow cam ON (auto-cycles every 6s; F / pad cycles)")
	else:
		print("Free fly ON (WASD / left stick + mouse / right stick)")

func cycle_next_target():
	if len(available_targets) > 0:
		follow_target_index = (follow_target_index + 1) % len(available_targets)
		auto_cycle_timer = 0.0
		camera_shake_intensity = 0.12
		camera_zoom_offset = -0.6

func set_available_targets(targets: Array):
	# Drop freed pawns so follow cam / highlight never soft-prison on a dead instance.
	available_targets = []
	for t in targets:
		if is_instance_valid(t):
			available_targets.append(t)
	if follow_mode and len(available_targets) > 0:
		follow_target_index = follow_target_index % len(available_targets)
	else:
		follow_target_index = 0

func camera_punch():
	camera_shake_intensity = 0.3
	camera_zoom_offset = -1.5

func get_followed_target():
	if fp_mode and is_instance_valid(fp_target):
		return fp_target
	if follow_mode and len(available_targets) > 0:
		var idx = follow_target_index % len(available_targets)
		var target = available_targets[idx]
		if is_instance_valid(target):
			return target
	return null

func lock_on_frag(killer_id: String, duration: float = 1.5):
	if tip_pose_lock:
		return
	if fp_mode:
		return
	frag_follow_target_id = killer_id
	frag_follow_timer = duration
	auto_cycle_timer = 0.0

func _follow_frag_target():
	if frag_follow_target_id == "":
		return

	for target in available_targets:
		if is_instance_valid(target) and target.player_id == frag_follow_target_id:
			var target_pos = target.global_position
			var offset = Vector3(0, 5.5, 10.5)
			var cam_pos = target_pos + offset.rotated(Vector3.UP, target.rotation.y)
			position = position.lerp(cam_pos, 0.15)

			var look_target = target_pos + Vector3(0, 1.5, 0)
			var desired_transform = global_transform.looking_at(look_target, Vector3.UP)
			global_transform = global_transform.interpolate_with(desired_transform, 0.2)
			return

func _process_fp(delta):
	# Mouse look: yaw becomes turn bits for Action; pitch stays local.
	if mouse_motion.length() > 0:
		var radians_per_count := _radians_per_count()
		fp_yaw = wrapf(fp_yaw + mouse_motion.x * radians_per_count, 0.0, TAU)
		fp_pitch -= mouse_motion.y * radians_per_count
		fp_pitch = clamp(fp_pitch, -1.15, 1.15)
		mouse_motion = Vector2.ZERO

	# Right stick: same Action turn path + local pitch.
	_apply_stick_look(delta, false)

	if not is_instance_valid(fp_target):
		return

	var eye = fp_target.global_position + Vector3(0, FP_EYE_HEIGHT, 0)
	# The eye looks where the client aims, not where the last snapshot said.
	var yaw = fp_yaw
	eye += ServerYaw.forward(yaw) * FP_FORWARD_NUDGE

	if camera_shake_intensity > 0:
		eye += Vector3(
			randf_range(-camera_shake_intensity, camera_shake_intensity) * 0.35,
			randf_range(-camera_shake_intensity, camera_shake_intensity) * 0.25,
			0
		)

	position = position.lerp(eye, min(1.0, 18.0 * delta))
	# The server's yaw is not a Godot rotation. Assigning it straight to
	# rotation.y pointed the camera ninety degrees away from where the server
	# was moving the fighter, which is why holding forward read as strafing.
	rotation.y = ServerYaw.camera_rotation_y(yaw)
	rotation.x = fp_pitch

## The absolute facing to send with this input, in the server's convention.
## Radians of turn per mouse count at the current sensitivity.
func _radians_per_count() -> float:
	return deg_to_rad(DEGREES_PER_COUNT * mouse_sensitivity)


## Centimetres of mouse travel for a full turn, the number players compare.
## Pure arithmetic, so the harness can assert it without a mouse.
static func cm_per_360(sensitivity: float, counts_per_inch: float) -> float:
	var degrees_per_count := DEGREES_PER_COUNT * sensitivity
	if degrees_per_count <= 0.0 or counts_per_inch <= 0.0:
		return 0.0
	return (360.0 / (degrees_per_count * counts_per_inch)) * 2.54


## The absolute facing to send with this input, in the server's convention.
func consume_yaw() -> float:
	return wrapf(fp_yaw, 0.0, TAU)


func consume_turn_bits() -> Dictionary:
	# Discrete turn for Action. Called each tick while human; clears accum.
	var left = false
	var right = false
	if turn_accum <= -TURN_ACCUM_THRESHOLD:
		left = true
		turn_accum = 0.0
	elif turn_accum >= TURN_ACCUM_THRESHOLD:
		right = true
		turn_accum = 0.0
	return {"turn_left": left, "turn_right": right}

func set_fp_mode(enabled: bool, target: Node3D = null) -> void:
	# tip_capture pose lock: never teleport onto a soldier mid-jammer still.
	if tip_pose_lock:
		return
	fp_mode = enabled
	fp_target = target
	if not enabled:
		fp_pitch = 0.0
		turn_accum = 0.0
		fp_yaw = 0.0
		fp_target = null
	elif is_instance_valid(target):
		# Snap once so join does not tween from spectator orbit, and adopt the
		# fighter's facing so the first mouse move continues from it.
		var yaw = target.rotation.y
		fp_yaw = wrapf(yaw, 0.0, TAU)
		position = target.global_position + Vector3(0, FP_EYE_HEIGHT, 0)
		rotation.y = yaw
		rotation.x = fp_pitch


## tip_capture: latch free-fly pose at dish and re-assert every frame.
func latch_tip_pose(xform: Transform3D) -> void:
	tip_pose_lock = true
	tip_locked_transform = xform
	tip_has_locked_transform = true
	follow_mode = false
	frag_follow_timer = 0.0
	frag_follow_target_id = ""
	fp_mode = false
	fp_target = null
	global_transform = xform


func capture_tip_pose_from_current() -> void:
	latch_tip_pose(global_transform)


func clear_tip_pose_lock() -> void:
	tip_pose_lock = false
	tip_has_locked_transform = false
