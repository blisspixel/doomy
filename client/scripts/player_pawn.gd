extends Node3D

var player_id: String = ""
var player_name: String = ""
var hp: int = 100
var player_color: Color = Color.WHITE
var hit_flash_timer: float = 0.0

var target_position: Vector3 = Vector3.ZERO
var target_yaw: float = 0.0
const INTERP_SPEED: float = 10.0

@onready var label: Label3D = $Label3D
@onready var body: MeshInstance3D = $Body
@onready var muzzle: MeshInstance3D = $Body/Muzzle

# Distinct colors for named bots
const BOT_COLORS = {
	"Rusher": Color(1.0, 0.2, 0.2),     # Bright red
	"Sniper": Color(0.2, 0.8, 1.0),     # Cyan
	"Flanker": Color(1.0, 0.8, 0.0),    # Gold
	"Tank": Color(0.2, 1.0, 0.3),       # Lime green
	"Scout": Color(1.0, 0.4, 0.8),      # Pink
	"Guard": Color(0.6, 0.3, 1.0),      # Purple
	"Hunter": Color(1.0, 0.6, 0.0),     # Orange
	"Striker": Color(0.3, 1.0, 1.0)     # Light cyan
}

func _ready():
	if label:
		label.text = player_name
	if muzzle:
		muzzle.visible = false

func _process(delta):
	# Interpolate position and rotation toward target
	position = position.lerp(target_position, INTERP_SPEED * delta)
	rotation.y = lerp_angle(rotation.y, target_yaw, INTERP_SPEED * delta)
	
	if hit_flash_timer > 0:
		hit_flash_timer -= delta
		if hit_flash_timer <= 0 and body:
			_update_body_color(false)

func set_player_data(id: String, name: String):
	player_id = id
	player_name = name
	
	# Assign distinct color based on name
	if BOT_COLORS.has(name):
		player_color = BOT_COLORS[name]
	else:
		# Fallback for custom names
		var color_val = float(abs(hash(id)) % 100) / 100.0
		player_color = Color.from_hsv(color_val, 0.8, 0.9)
	
	if label:
		label.text = name
		label.modulate = player_color
	
	# Initialize interpolation targets to avoid snap on spawn
	target_position = position
	target_yaw = rotation.y

func update_state(state: Dictionary):
	target_position = Vector3(state.x, state.y, state.z)
	target_yaw = state.yaw
	
	var old_hp = hp
	hp = state.hp
	
	if old_hp > hp and hp > 0:
		show_hit_feedback()
	
	if label:
		var weapon_name = state.get("weapon", "")
		var behavior_chip = ""
		if state.has("behavior") and state.behavior != null:
			behavior_chip = " [" + str(state.behavior) + "]"
		
		var weapon_display = ""
		if weapon_name != "":
			weapon_display = "\n" + weapon_name
		
		label.text = player_name + " [" + str(hp) + "]" + behavior_chip + weapon_display
	
	if muzzle and state.get("just_fired", false):
		show_muzzle_flash()
	
	if body and hit_flash_timer <= 0:
		_update_body_color(false)

func _update_body_color(hit: bool):
	if not body:
		return
	
	var mat = StandardMaterial3D.new()
	if hit:
		# Flash bright red on hit
		mat.albedo_color = Color(1.0, 0.3, 0.3)
		mat.emission_enabled = true
		mat.emission = Color(1.0, 0.2, 0.2)
		mat.emission_energy = 2.0
	else:
		mat.albedo_color = player_color
		mat.emission_enabled = true
		mat.emission = player_color * 0.3
		mat.emission_energy = 0.5
	
	body.material_override = mat

func show_muzzle_flash():
	if muzzle:
		muzzle.visible = true
		var mat = StandardMaterial3D.new()
		mat.albedo_color = Color(1.0, 0.95, 0.6)
		mat.emission_enabled = true
		mat.emission = Color(1.0, 0.85, 0.3)
		mat.emission_energy = 6.0
		muzzle.material_override = mat
		
		var original_scale = muzzle.scale
		muzzle.scale = original_scale * 1.5
		
		await get_tree().create_timer(0.08).timeout
		if is_instance_valid(muzzle):
			muzzle.visible = false
			muzzle.scale = original_scale

func show_hit_feedback():
	hit_flash_timer = 0.2
	_update_body_color(true)
	
	var original_scale = body.scale if body else Vector3.ONE
	if body:
		body.scale = original_scale * 1.15
		await get_tree().create_timer(0.1).timeout
		if is_instance_valid(body):
			body.scale = original_scale
