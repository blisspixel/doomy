extends Node3D

var player_id: String = ""
var player_name: String = ""
var hp: int = 100
var player_color: Color = Color.WHITE
var hit_flash_timer: float = 0.0
var idle_anim_timer: float = 0.0
var current_weapon: String = ""
var behavior: String = ""
var is_highlighted: bool = false

var target_position: Vector3 = Vector3.ZERO
var target_yaw: float = 0.0
const INTERP_SPEED: float = 10.0

# Far-cam billboard scale: follow sits ~12m; tip overview ~36m.
# Below REF, scale stays 1 so close follow is unchanged. Beyond REF, scale grows with
# distance / REF (capped) so far spectators still read Cyanex/Kragge silhouettes.
const FAR_CAM_REF_DIST: float = 12.0
const FAR_CAM_MAX_SCALE: float = 3.5
const HIT_SCALE_BOOST: float = 1.12

var _far_cam_scale: float = 1.0

@onready var label: Label3D = $Label3D
@onready var highlight: MeshInstance3D = $Highlight
@onready var body: Sprite3D = $Body
@onready var weapon_sprite: Sprite3D = $Body/WeaponSprite
@onready var muzzle: Sprite3D = $Body/Muzzle
@onready var muzzle_glow: OmniLight3D = $Body/Muzzle/MuzzleGlow
@onready var fire_sound: AudioStreamPlayer3D = $FireSound
@onready var hit_sound: AudioStreamPlayer3D = $HitSound

var muzzle_flash_texture: Texture2D
var rail_beam_texture: Texture2D

var weapon_textures = {}
var cyanex_texture: Texture2D
var kragge_texture: Texture2D

# Art bible muted brand tints (bone/gunmetal/rust/blood/ember + muted cyan/magenta).
# Keep silhouettes readable: labels carry brand color; body stays near-white multiply.
const BOT_COLORS = {
	"Rusher": Color(0.75, 0.28, 0.22),
	"Sniper": Color(0.35, 0.58, 0.62),
	"Flanker": Color(0.82, 0.55, 0.22),
	"Tank": Color(0.55, 0.48, 0.4),
	"Scout": Color(0.62, 0.32, 0.42),
	"Guard": Color(0.45, 0.38, 0.5),
	"Hunter": Color(0.769, 0.4, 0.18),
	"Striker": Color(0.4, 0.62, 0.64),
	"COMPLIANCE-DRONE": Color(0.55, 0.72, 0.35)
}

const KRAGGE_BOTS = ["Rusher", "Tank", "Hunter"]
const CYANEX_BOTS = ["Sniper", "Flanker", "Scout", "Guard", "Striker"]

func _ready():
	if label:
		label.text = player_name
	if muzzle:
		muzzle.visible = false
	if muzzle_glow:
		muzzle_glow.light_energy = 0.0
	if highlight:
		highlight.visible = false
	
	_load_audio_streams()
	
	muzzle_flash_texture = load("res://assets/vfx/32/muzzle_flash.png")
	rail_beam_texture = load("res://assets/vfx/32/rail_beam_tip.png")
	
	weapon_textures["Flechette"] = load("res://assets/weapons/32/flechette.png")
	weapon_textures["Rail"] = load("res://assets/weapons/32/rail.png")
	weapon_textures["Scatter"] = load("res://assets/weapons/32/scatter.png")
	
	cyanex_texture = load("res://assets/characters/64/cyanex_idle_strip.png")
	kragge_texture = load("res://assets/characters/64/kragge_idle_strip.png")

func _load_audio_streams():
	var audio_dir = "res://assets/audio/"
	
	if fire_sound and ResourceLoader.exists(audio_dir + "fire.wav"):
		fire_sound.stream = load(audio_dir + "fire.wav")
	
	if hit_sound and ResourceLoader.exists(audio_dir + "hit.wav"):
		hit_sound.stream = load(audio_dir + "hit.wav")

func _process(delta):
	position = position.lerp(target_position, INTERP_SPEED * delta)
	rotation.y = lerp_angle(rotation.y, target_yaw, INTERP_SPEED * delta)
	
	_update_far_cam_scale()
	
	if hit_flash_timer > 0:
		hit_flash_timer -= delta
		if hit_flash_timer <= 0 and body:
			_update_body_color(false)
	
	idle_anim_timer += delta * 4.0
	if body:
		var frame = int(idle_anim_timer) % 4
		body.frame = frame

func set_player_data(id: String, name: String):
	player_id = id
	player_name = name
	
	if BOT_COLORS.has(name):
		player_color = BOT_COLORS[name]
	else:
		var color_val = float(abs(hash(id)) % 100) / 100.0
		player_color = Color.from_hsv(color_val, 0.8, 0.9)
	
	if KRAGGE_BOTS.has(name):
		body.texture = kragge_texture
	else:
		body.texture = cyanex_texture
	
	# Continuance drone: taller billboard silhouette vs scrap fighters.
	if name == "COMPLIANCE-DRONE" and body:
		body.pixel_size = body.pixel_size * 1.35
	
	if label:
		label.text = name
		label.modulate = player_color
	
	target_position = position
	target_yaw = rotation.y

func update_state(state: Dictionary):
	target_position = Vector3(state.x, state.y, state.z)
	target_yaw = state.yaw
	
	var old_hp = hp
	hp = state.hp
	
	if old_hp > hp and hp > 0:
		show_hit_feedback()
	
	var weapon_name = state.get("weapon", "")
	if weapon_name != current_weapon:
		current_weapon = weapon_name
		_update_weapon_sprite()
	
	if state.has("behavior") and state.behavior != null:
		behavior = str(state.behavior)
	else:
		behavior = ""
	
	if label:
		var behavior_chip = ""
		if behavior != "":
			var short = behavior
			match behavior:
				"Aggressive":
					short = "AGG"
				"Defensive":
					short = "DEF"
				"Flanker":
					short = "FLK"
				"Balanced":
					short = "BAL"
				"Compliance":
					short = "CMP"
			behavior_chip = " [" + short + "]"
		
		var hp_display = str(hp) + " HP"
		if hp < 30:
			hp_display = "!" + hp_display + "!"
		
		var score = int(state.get("score", 0))
		var score_chip = ""
		if score > 0:
			score_chip = " +" + str(score)
		
		label.text = player_name + score_chip + " [" + hp_display + "]" + behavior_chip
	
	if muzzle and state.get("just_fired", false):
		var weapon = state.get("weapon", "")
		show_muzzle_flash(weapon)
	
	if body and hit_flash_timer <= 0:
		_update_body_color(false)

func _update_weapon_sprite():
	if not weapon_sprite:
		return
	
	if current_weapon == "" or not weapon_textures.has(current_weapon):
		weapon_sprite.visible = false
		return
	
	weapon_sprite.texture = weapon_textures[current_weapon]
	weapon_sprite.visible = true
	# Preserve weapon plate readability; light bone lift, not neon wash.
	weapon_sprite.modulate = Color(1.05, 1.02, 0.98)

func _update_body_color(hit: bool):
	if not body:
		return
	
	if hit:
		body.modulate = Color(1.55, 0.35, 0.28)
	else:
		# Near-white multiply so Cyanex/Kragge pixel art reads; brand on label.
		body.modulate = Color(1.0, 1.0, 1.0).lerp(player_color, 0.18)
	_apply_body_scale(hit)

func show_muzzle_flash(weapon: String):
	if fire_sound and fire_sound.stream:
		fire_sound.play()
	
	if not muzzle:
		return
	
	if weapon == "Rail":
		muzzle.texture = rail_beam_texture
		muzzle.modulate = Color(1.8, 1.8, 2.5)
		muzzle.scale = Vector3.ONE * 2.5
		if muzzle_glow:
			muzzle_glow.light_color = Color(0.35, 0.58, 0.65)
			muzzle_glow.light_energy = 4.0
			muzzle_glow.omni_range = 5.0
	else:
		muzzle.texture = muzzle_flash_texture
		muzzle.modulate = Color(1.9, 1.35, 0.7)
		muzzle.scale = Vector3.ONE * 2.0
		if muzzle_glow:
			muzzle_glow.light_color = Color(0.85, 0.45, 0.18)
			muzzle_glow.light_energy = 3.5
			muzzle_glow.omni_range = 4.0
	
	muzzle.visible = true
	
	await get_tree().create_timer(0.06).timeout
	if is_instance_valid(muzzle):
		muzzle.visible = false
		muzzle.scale = Vector3.ONE
		if muzzle_glow:
			muzzle_glow.light_energy = 0.0

func show_hit_feedback():
	if hit_sound and hit_sound.stream:
		hit_sound.play()
	
	hit_flash_timer = 0.25
	_update_body_color(true)
	
	await get_tree().create_timer(0.12).timeout
	if is_instance_valid(body):
		_apply_body_scale(hit_flash_timer > 0)

func get_weapon_name() -> String:
	return current_weapon

func show_winner_glow():
	var tween = create_tween()
	tween.set_parallel(true)
	
	if body:
		tween.tween_property(body, "modulate", Color(2.0, 2.0, 2.0), 0.1)
		tween.tween_property(body, "modulate", Color.WHITE, 0.9).set_delay(0.1)

func set_highlighted(highlighted: bool):
	is_highlighted = highlighted
	if highlight:
		highlight.visible = highlighted

## Pure scale curve for far spectators. Safe to call from headless tests.
static func compute_far_cam_scale(distance: float) -> float:
	if distance <= FAR_CAM_REF_DIST:
		return 1.0
	return minf(distance / FAR_CAM_REF_DIST, FAR_CAM_MAX_SCALE)

func _update_far_cam_scale() -> void:
	var cam: Camera3D = get_viewport().get_camera_3d() if get_viewport() else null
	var dist: float = FAR_CAM_REF_DIST
	if cam != null:
		dist = global_position.distance_to(cam.global_position)
	_far_cam_scale = compute_far_cam_scale(dist)
	_apply_body_scale(hit_flash_timer > 0)
	if label:
		label.scale = Vector3.ONE * _far_cam_scale

func _apply_body_scale(hit: bool) -> void:
	if not body:
		return
	var mult: float = _far_cam_scale
	if hit:
		mult *= HIT_SCALE_BOOST
	body.scale = Vector3.ONE * mult
