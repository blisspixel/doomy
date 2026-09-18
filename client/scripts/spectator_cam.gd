extends Node3D

@export var move_speed = 10.0
@export var look_sensitivity = 0.003
@export var auto_cycle_interval = 6.0

var follow_mode = true
var follow_target_index = 0
var available_targets = []
var auto_cycle_timer = 0.0

var mouse_motion = Vector2.ZERO

func _ready():
	Input.set_mouse_mode(Input.MOUSE_MODE_CAPTURED)

func _input(event):
	if event is InputEventMouseMotion:
		mouse_motion = event.relative
	
	if event is InputEventKey and event.pressed and event.keycode == KEY_ESCAPE:
		if Input.get_mouse_mode() == Input.MOUSE_MODE_CAPTURED:
			Input.set_mouse_mode(Input.MOUSE_MODE_VISIBLE)
		else:
			Input.set_mouse_mode(Input.MOUSE_MODE_CAPTURED)

func _process(delta):
	if Input.get_mouse_mode() != Input.MOUSE_MODE_CAPTURED:
		return
	
	if Input.is_action_just_pressed("toggle_follow"):
		toggle_follow_mode()
	
	if follow_mode and len(available_targets) > 0:
		auto_cycle_timer += delta
		if auto_cycle_timer >= auto_cycle_interval:
			cycle_next_target()
			auto_cycle_timer = 0.0
		_follow_target()
	else:
		_free_fly(delta)

func _free_fly(delta):
	if mouse_motion.length() > 0:
		rotation.y -= mouse_motion.x * look_sensitivity
		rotation.x -= mouse_motion.y * look_sensitivity
		rotation.x = clamp(rotation.x, -PI/2, PI/2)
		mouse_motion = Vector2.ZERO
	
	var input_dir = Vector3.ZERO
	if Input.is_action_pressed("move_forward"):
		input_dir.z -= 1
	if Input.is_action_pressed("move_back"):
		input_dir.z += 1
	if Input.is_action_pressed("move_left"):
		input_dir.x -= 1
	if Input.is_action_pressed("move_right"):
		input_dir.x += 1
	
	if input_dir.length() > 0:
		input_dir = input_dir.normalized()
		var move_vec = transform.basis * input_dir
		position += move_vec * move_speed * delta

func _follow_target():
	if len(available_targets) == 0:
		return
	
	follow_target_index = follow_target_index % len(available_targets)
	var target = available_targets[follow_target_index]
	
	if is_instance_valid(target):
		var target_pos = target.global_position
		var offset = Vector3(0, 4, 7)
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
		print("Follow cam ON (auto-cycles every 6s)")
	else:
		print("Free fly ON (WASD + mouse)")

func cycle_next_target():
	if len(available_targets) > 0:
		follow_target_index = (follow_target_index + 1) % len(available_targets)
		auto_cycle_timer = 0.0

func set_available_targets(targets: Array):
	available_targets = targets
	if follow_mode and len(targets) > 0:
		follow_target_index = follow_target_index % len(targets)
