extends Node3D

const StanceChipScript = preload("res://scripts/stance_chip.gd")

var player_id: String = ""
var player_name: String = ""
var hp: int = 100
var player_color: Color = Color.WHITE
var hit_flash_timer: float = 0.0
var idle_anim_timer: float = 0.0
var current_weapon: String = ""
var behavior: String = ""
var is_highlighted: bool = false
var is_local_fp: bool = false

var target_position: Vector3 = Vector3.ZERO
var target_yaw: float = 0.0
const INTERP_SPEED: float = 10.0

# Far-cam billboard scale: follow sits ~12m; tip overview ~36m.
# Below REF, scale stays 1 so close follow is unchanged. Beyond REF, scale grows with
# distance / REF (capped) so far spectators still read Cyanex/Kragge silhouettes.
const FAR_CAM_REF_DIST: float = 12.0
const FAR_CAM_MAX_SCALE: float = 3.5
## Below this distance a nameplate shrinks with the distance, so its size on
## screen stops growing. A Label3D has a fixed size in the world, which means a
## fighter two metres away wears a name tall enough to hide the room behind it.
const NAMEPLATE_NEAR_DIST: float = 9.0
## How small a close nameplate may get, so it does not vanish at point blank.
const NAMEPLATE_MIN_SCALE: float = 0.22
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
	"Dead Air Dan": Color(0.75, 0.28, 0.22),
	"Nightfall": Color(0.35, 0.58, 0.62),
	"Static Kid": Color(0.82, 0.55, 0.22),
	"Aunt Linda": Color(0.55, 0.48, 0.4),
	"Scout Ant": Color(0.62, 0.32, 0.42),
	"Crackpot": Color(0.45, 0.38, 0.5),
	"Buzzkill": Color(0.769, 0.4, 0.18),
	"Tin Foil Tina": Color(0.4, 0.62, 0.64),
	"COMPLIANCE-DRONE": Color(0.55, 0.72, 0.35),
	"AUDITOR": Color(0.72, 0.78, 0.28),
	"NODS-01": Color(0.45, 0.48, 0.42),
	"NODS-02": Color(0.42, 0.45, 0.4),
	"NODS-03": Color(0.48, 0.5, 0.44),
	"NODS-04": Color(0.4, 0.43, 0.38),
	"NODS-05": Color(0.46, 0.49, 0.41),
	"NODS-06": Color(0.43, 0.46, 0.39),
	"NODS-07": Color(0.47, 0.5, 0.43),
	"NODS-08": Color(0.41, 0.44, 0.37)
}


# Hangar Candy / Kragge grit vs Cyanex / Night Watch signal.
const KRAGGE_BOTS = ["Dead Air Dan", "Aunt Linda", "Buzzkill"]
const CYANEX_BOTS = ["Nightfall", "Static Kid", "Scout Ant", "Crackpot", "Tin Foil Tina"]

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

var fire_streams = {}
var hit_streams = {}
## The impact sound used when the shooter's weapon is not known.
var generic_hit_stream: AudioStream = null

func _load_audio_streams():
	var audio_dir = "res://assets/audio/"
	var fallback_fire = audio_dir + "fire.wav"
	var fallback_hit = audio_dir + "hit.wav"
	
	for w in ["Flechette", "Rail", "Scatter"]:
		var key = w.to_lower()
		var fire_path = audio_dir + "fire_" + key + ".wav"
		var hit_path = audio_dir + "hit_" + key + ".wav"
		if ResourceLoader.exists(fire_path):
			fire_streams[w] = load(fire_path)
		elif ResourceLoader.exists(fallback_fire):
			fire_streams[w] = load(fallback_fire)
		if ResourceLoader.exists(hit_path):
			hit_streams[w] = load(hit_path)
		elif ResourceLoader.exists(fallback_hit):
			hit_streams[w] = load(fallback_hit)
	
	if fire_sound and ResourceLoader.exists(fallback_fire):
		fire_sound.stream = load(fallback_fire)
	if ResourceLoader.exists(fallback_hit):
		generic_hit_stream = load(fallback_hit)
	if hit_sound and ResourceLoader.exists(fallback_hit):
		hit_sound.stream = load(fallback_hit)

## Fraction of the remaining distance to close this frame.
##
## `speed * delta` is the obvious form and it is frame-rate dependent: at 240
## frames a second it closes 4 percent per frame and at 40 it closes 25, which
## are not the same curve, so two machines render a fighter in different places
## from identical snapshots. Above a delta of 1/speed it overshoots outright.
## The exponential form closes the same fraction per unit of *time* whatever
## the frame rate, which is what smoothing was always supposed to mean.
static func smoothing(speed: float, delta: float) -> float:
	return 1.0 - exp(-speed * delta)

func _process(delta):
	var t: float = smoothing(INTERP_SPEED, delta)
	position = position.lerp(target_position, t)
	rotation.y = lerp_angle(rotation.y, target_yaw, t)
	
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
	elif str(name).begins_with("NODS-"):
		player_color = Color(0.44, 0.47, 0.4)
	elif name == "AUDITOR":
		player_color = Color(0.72, 0.78, 0.28)
	else:
		var color_val = float(abs(hash(id)) % 100) / 100.0
		player_color = Color.from_hsv(color_val, 0.8, 0.9)
	
	if KRAGGE_BOTS.has(name):
		body.texture = kragge_texture
	else:
		body.texture = cyanex_texture
	
	# Continuance drone: taller billboard silhouette vs scrap fighters.
	if str(name).begins_with("NODS-") and label:
		label.modulate = Color(0.7, 0.75, 0.65)
	if name == "AUDITOR" and body:
		body.scale = Vector3(1.25, 1.45, 1.25)
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
		var hp_display = str(hp) + " HP"
		if hp < 30:
			hp_display = "!" + hp_display + "!"
		
		var score = int(state.get("score", 0))
		var score_chip = ""
		if score > 0:
			score_chip = " +" + str(score)
		
		# Stance beside callsign so follow / overview reads it without Tab.
		label.text = StanceChipScript.nameplate(player_name, behavior, hp_display, score_chip)
		if behavior != "":
			label.modulate = StanceChipScript.accent_color(true)
		else:
			label.modulate = player_color
	
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
	if fire_sound:
		if fire_streams.has(weapon):
			fire_sound.stream = fire_streams[weapon]
		if fire_sound.stream:
			fire_sound.play()
	
	if not muzzle:
		return
	
	var flash_time = 0.06
	if weapon == "Rail":
		muzzle.texture = rail_beam_texture
		# Gunmetal / cold bone, not neon.
		muzzle.modulate = Color(0.72, 0.78, 0.82)
		muzzle.scale = Vector3.ONE * 2.8
		flash_time = 0.10
		if muzzle_glow:
			muzzle_glow.light_color = Color(0.45, 0.52, 0.55)
			muzzle_glow.light_energy = 4.2
			muzzle_glow.omni_range = 5.5
	elif weapon == "Scatter":
		muzzle.texture = muzzle_flash_texture
		muzzle.modulate = Color(0.95, 0.55, 0.28)
		muzzle.scale = Vector3.ONE * 2.6
		flash_time = 0.08
		if muzzle_glow:
			muzzle_glow.light_color = Color(0.85, 0.42, 0.16)
			muzzle_glow.light_energy = 4.0
			muzzle_glow.omni_range = 4.5
	else:
		# Flechette: tight ember needle.
		muzzle.texture = muzzle_flash_texture
		muzzle.modulate = Color(0.92, 0.78, 0.55)
		muzzle.scale = Vector3.ONE * 1.7
		if muzzle_glow:
			muzzle_glow.light_color = Color(0.78, 0.55, 0.28)
			muzzle_glow.light_energy = 2.8
			muzzle_glow.omni_range = 3.2
	
	muzzle.visible = true
	
	await get_tree().create_timer(flash_time).timeout
	if is_instance_valid(muzzle):
		muzzle.visible = false
		muzzle.scale = Vector3.ONE
		if muzzle_glow:
			muzzle_glow.light_energy = 0.0

## Feedback for being hit. `weapon` is the gun that did it.
##
## When it is unknown this used to fall back to `current_weapon`, which is the
## weapon this fighter is *holding*, not the one that shot them: being shot by
## a rail while carrying a scatter played the scatter's impact. The generic
## impact is the honest sound for an unknown shooter. The weapon reaches here
## properly once shot results carry it; see plans/shot-feedback.md.
func show_hit_feedback(weapon: String = ""):
	if hit_sound:
		if weapon != "" and hit_streams.has(weapon):
			hit_sound.stream = hit_streams[weapon]
		elif generic_hit_stream != null:
			hit_sound.stream = generic_hit_stream
		if hit_sound.stream:
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

## Nameplate scale: capped on screen up close, and sharing the far camera's
## growth beyond the reference distance so a distant fighter is still labelled.
static func compute_nameplate_scale(distance: float) -> float:
	if distance < NAMEPLATE_NEAR_DIST:
		return maxf(distance / NAMEPLATE_NEAR_DIST, NAMEPLATE_MIN_SCALE)
	return compute_far_cam_scale(distance)

func _update_far_cam_scale() -> void:
	var cam: Camera3D = get_viewport().get_camera_3d() if get_viewport() else null
	var dist: float = FAR_CAM_REF_DIST
	if cam != null:
		dist = global_position.distance_to(cam.global_position)
	_far_cam_scale = compute_far_cam_scale(dist)
	_apply_body_scale(hit_flash_timer > 0)
	if label:
		label.scale = Vector3.ONE * compute_nameplate_scale(dist)

func _apply_body_scale(hit: bool) -> void:
	if not body:
		return
	var mult: float = _far_cam_scale
	if hit:
		mult *= HIT_SCALE_BOOST
	body.scale = Vector3.ONE * mult

func set_local_fp(enabled: bool) -> void:
	# Hide local billboard in FP so the HUD viewmodel owns the scrap face.
	is_local_fp = enabled
	if body:
		body.visible = not enabled
	if label:
		label.visible = not enabled
	if highlight:
		highlight.visible = false if enabled else is_highlighted
	if weapon_sprite and enabled:
		weapon_sprite.visible = false
