extends Node3D

var player_id: String = ""
var player_name: String = ""
var hp: int = 100
var player_color: Color = Color.WHITE
var hit_flash_timer: float = 0.0
var idle_anim_timer: float = 0.0
var current_weapon: String = ""

var target_position: Vector3 = Vector3.ZERO
var target_yaw: float = 0.0
const INTERP_SPEED: float = 10.0

@onready var label: Label3D = $Label3D
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

const BOT_COLORS = {
	"Rusher": Color(1.0, 0.2, 0.2),
	"Sniper": Color(0.2, 0.8, 1.0),
	"Flanker": Color(1.0, 0.8, 0.0),
	"Tank": Color(0.2, 1.0, 0.3),
	"Scout": Color(1.0, 0.4, 0.8),
	"Guard": Color(0.6, 0.3, 1.0),
	"Hunter": Color(1.0, 0.6, 0.0),
	"Striker": Color(0.3, 1.0, 1.0)
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
	
	if label:
		var behavior_chip = ""
		if state.has("behavior") and state.behavior != null:
			behavior_chip = " [" + str(state.behavior) + "]"
		
		var hp_display = str(hp) + " HP"
		if hp < 30:
			hp_display = "!" + hp_display + "!"
		
		label.text = player_name + " [" + hp_display + "]" + behavior_chip
	
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
	weapon_sprite.modulate = player_color.lightened(0.3)

func _update_body_color(hit: bool):
	if not body:
		return
	
	if hit:
		body.modulate = Color(1.8, 0.3, 0.3)
		body.scale = Vector3.ONE * 1.2
	else:
		body.modulate = player_color
		body.scale = Vector3.ONE

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
			muzzle_glow.light_color = Color(0.3, 0.8, 1.0)
			muzzle_glow.light_energy = 4.0
			muzzle_glow.omni_range = 5.0
	else:
		muzzle.texture = muzzle_flash_texture
		muzzle.modulate = Color(2.0, 1.8, 1.0)
		muzzle.scale = Vector3.ONE * 2.0
		if muzzle_glow:
			muzzle_glow.light_color = Color(1.0, 0.85, 0.3)
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
		body.scale = Vector3.ONE

func get_weapon_name() -> String:
	return current_weapon
