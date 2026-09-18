extends Node3D

## Scrap crate / pad billboard for mid-map pickups (weapon, health, armor).
## Palette: bone-white / gunmetal / ember / blood-ember (not neon).

var pickup_id: String = ""
var weapon_name: String = ""
var pickup_kind: String = "weapon"
var amount: int = 0
var available: bool = true

@onready var label: Label3D = $Label3D
@onready var body: MeshInstance3D = $Body
@onready var icon: Sprite3D = $Icon

const COLORS = {
	"Flechette": Color(0.86, 0.82, 0.74),  # bone-white
	"Rail": Color(0.42, 0.46, 0.50),       # gunmetal
	"Scatter": Color(0.72, 0.38, 0.22),    # ember
	"health": Color(0.62, 0.22, 0.20),     # dried blood
	"armor": Color(0.48, 0.44, 0.38),      # scrap gunmetal
}

var weapon_textures = {}

func _ready():
	weapon_textures["Flechette"] = load("res://assets/weapons/32/flechette.png")
	weapon_textures["Rail"] = load("res://assets/weapons/32/rail.png")
	weapon_textures["Scatter"] = load("res://assets/weapons/32/scatter.png")
	_apply_look()

func setup(id: String, weapon: String, pos: Vector3, kind: String = "weapon", pad_amount: int = 0) -> void:
	pickup_id = id
	weapon_name = weapon
	pickup_kind = kind if kind != "" else "weapon"
	amount = pad_amount
	position = pos
	_apply_look()

func set_available(is_available: bool) -> void:
	available = is_available
	visible = available
	_apply_look()

func _label_text() -> String:
	if pickup_kind == "health":
		return "MEDKIT" if amount <= 0 else ("+%d HP" % amount)
	if pickup_kind == "armor":
		return "ARMOR" if amount <= 0 else ("+%d ARM" % amount)
	if weapon_name != "":
		return weapon_name.to_upper()
	return "PAD"

func _tint() -> Color:
	if pickup_kind == "health":
		return COLORS["health"]
	if pickup_kind == "armor":
		return COLORS["armor"]
	return COLORS.get(weapon_name, Color(0.7, 0.68, 0.64))

func _apply_look() -> void:
	var tint = _tint()
	if label:
		label.text = _label_text()
		label.modulate = tint
		label.outline_modulate = Color(0.12, 0.11, 0.10)
	if body and body.get_active_material(0):
		var mat = body.get_active_material(0).duplicate()
		mat.albedo_color = tint.darkened(0.25)
		mat.emission_enabled = true
		# Health pads get a soft ember glow (blood/ember, not neon).
		var glow = 0.22 if pickup_kind == "health" else 0.18
		mat.emission = tint * glow
		mat.emission_energy_multiplier = 0.7 if pickup_kind == "health" else 0.6
		body.set_surface_override_material(0, mat)
	elif body:
		var mat = StandardMaterial3D.new()
		mat.albedo_color = tint.darkened(0.25)
		mat.emission_enabled = true
		mat.emission = tint * 0.18
		mat.emission_energy_multiplier = 0.6
		body.set_surface_override_material(0, mat)
	if icon:
		if pickup_kind == "weapon" and weapon_textures.has(weapon_name):
			icon.texture = weapon_textures[weapon_name]
			icon.visible = true
			icon.modulate = Color(1.02, 1.0, 0.96)
		else:
			# Medkit / armor: hide weapon icon; label carries the scrap read.
			icon.visible = false
