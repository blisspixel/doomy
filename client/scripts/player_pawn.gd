extends Node3D

var player_id: String = ""
var player_name: String = ""
var hp: int = 100

@onready var label: Label3D = $Label3D
@onready var body: MeshInstance3D = $Body
@onready var muzzle: MeshInstance3D = $Body/Muzzle

func _ready():
	if label:
		label.text = player_name
	if muzzle:
		muzzle.visible = false

func set_player_data(id: String, name: String):
	player_id = id
	player_name = name
	if label:
		label.text = name

func update_state(state: Dictionary):
	position = Vector3(state.x, state.y, state.z)
	rotation.y = state.yaw
	hp = state.hp
	
	if label:
		label.text = player_name + " [" + str(hp) + "]"
	
	if muzzle and state.get("just_fired", false):
		show_muzzle_flash()
	
	if body:
		var color_val = float(abs(hash(player_id)) % 100) / 100.0
		var mat = StandardMaterial3D.new()
		mat.albedo_color = Color.from_hsv(color_val, 0.8, 0.9)
		body.material_override = mat

func show_muzzle_flash():
	if muzzle:
		muzzle.visible = true
		await get_tree().create_timer(0.1).timeout
		if is_instance_valid(muzzle):
			muzzle.visible = false
