extends Node3D

## Scrap crate / pad billboard for mid-map weapon pickups.
## Palette: bone-white / gunmetal / ember (not neon).

var pickup_id: String = ""
var weapon_name: String = ""
var available: bool = true

@onready var label: Label3D = $Label3D
@onready var body: MeshInstance3D = $Body
@onready var icon: Sprite3D = $Icon

const COLORS = {
	"Flechette": Color(0.86, 0.82, 0.74),  # bone-white
	"Rail": Color(0.42, 0.46, 0.50),       # gunmetal
	"Scatter": Color(0.72, 0.38, 0.22),    # ember
}

var weapon_textures = {}

func _ready():
	weapon_textures["Flechette"] = load("res://assets/weapons/32/flechette.png")
	weapon_textures["Rail"] = load("res://assets/weapons/32/rail.png")
	weapon_textures["Scatter"] = load("res://assets/weapons/32/scatter.png")
	_apply_look()

func setup(id: String, weapon: String, pos: Vector3) -> void:
	pickup_id = id
	weapon_name = weapon
	position = pos
	_apply_look()

func set_available(is_available: bool) -> void:
	available = is_available
	visible = available
	_apply_look()

func _apply_look() -> void:
	var tint = COLORS.get(weapon_name, Color(0.7, 0.68, 0.64))
	if label:
		label.text = weapon_name.to_upper() if weapon_name != "" else "PAD"
		label.modulate = tint
		label.outline_modulate = Color(0.12, 0.11, 0.10)
	if body and body.get_active_material(0):
		var mat = body.get_active_material(0).duplicate()
		mat.albedo_color = tint.darkened(0.25)
		mat.emission_enabled = true
		mat.emission = tint * 0.18
		mat.emission_energy_multiplier = 0.6
		body.set_surface_override_material(0, mat)
	elif body:
		var mat = StandardMaterial3D.new()
		mat.albedo_color = tint.darkened(0.25)
		mat.emission_enabled = true
		mat.emission = tint * 0.18
		mat.emission_energy_multiplier = 0.6
		body.set_surface_override_material(0, mat)
	if icon:
		if weapon_textures.has(weapon_name):
			icon.texture = weapon_textures[weapon_name]
			icon.visible = true
			icon.modulate = Color(1.02, 1.0, 0.96)
		else:
			icon.visible = false
