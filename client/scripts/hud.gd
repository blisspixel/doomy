extends CanvasLayer

## Fired whenever the Host takes the air so the radio can duck underneath.
signal host_spoke(seconds: float)

@onready var status_label = $Panel/VBoxContainer/StatusLabel
@onready var tick_label = $Panel/VBoxContainer/TickLabel
@onready var player_count_label = $Panel/VBoxContainer/PlayerCountLabel
@onready var mode_label = $Panel/VBoxContainer/ModeLabel
@onready var round_label = $Panel/VBoxContainer/RoundLabel
@onready var weapon_label = $Panel/VBoxContainer/WeaponLabel
@onready var frag_label = $FragLabel
@onready var round_message = $RoundMessage
@onready var scoreboard = $Panel/VBoxContainer/Scoreboard
@onready var weapon_icon = $WeaponIcon
@onready var crosshair = $Crosshair
@onready var damage_flash = $DamageFlash
@onready var spawn_flash = $SpawnFlash
@onready var streak_flash = $StreakFlash
@onready var fp_weapon = $FpWeapon
@onready var chrome_strip = $ChromeStrip
@onready var on_air_badge = $OnAirBadge
@onready var contested_frequency_badge = $ContestedFrequencyBadge
@onready var hangar_candy_badge = $HangarCandyBadge
var crosshair_hbar = null
var crosshair_vbar = null
var crosshair_dot = null
var crosshair_ring = null
var hit_marker = null
var damage_numbers = null

var scores = {}
var behaviors = {}
var ghost_rival = ""
var leader_name = ""
var host_bumper_index = 0
var league_mode_name = "Contested Frequency"
var league_playlist = "Arena Duel"
var map_label = "Arena Duel"
var pressure_id = ""
var sticky_host_line = ""
var host_line_seen = false
var client_mode = "SPECTATING"
var round_chrome_state = "Warmup"

const HOST_BUMPERS = [
	"HOST: CONTESTED FREQUENCY. LEAGUE DENIES EXISTENCE.",
	"HOST: ARENA DUEL UNDER THE LIE. LIVE LAUGH FRAG.",
	"HOST: CONTINUANCE WATCHES. YOU SHOOT.",
	"HOST: SHALL NOT BE INFRINGED. OPEN WEIGHTS. OPEN FIRE.",
	"HOST: PORT 6767 ENERGY. DENY EVERYTHING.",
]

var weapon_textures = {}
var followed_player_name = ""
var fp_juice_enabled = false
var fp_bob_t = 0.0
var fp_weapon_base_pos = Vector2.ZERO
var fp_weapon_scene_base = Vector2.ZERO
var damage_flash_timer = 0.0
var spawn_flash_timer = 0.0
var streak_flash_timer = 0.0
var hit_marker_timer = 0.0
var fp_kick_timer = 0.0
var fp_kick_amount = Vector2.ZERO
var current_fp_weapon = ""
var floating_damage_nodes = []

func _ready():
	weapon_textures["Flechette"] = load("res://assets/weapons/32/flechette.png")
	weapon_textures["Rail"] = load("res://assets/weapons/32/rail.png")
	weapon_textures["Scatter"] = load("res://assets/weapons/32/scatter.png")
	crosshair_hbar = get_node_or_null("Crosshair/HBar")
	crosshair_vbar = get_node_or_null("Crosshair/VBar")
	crosshair_dot = get_node_or_null("Crosshair/Dot")
	crosshair_ring = get_node_or_null("Crosshair/RingBorder")
	hit_marker = get_node_or_null("HitMarker")
	damage_numbers = get_node_or_null("DamageNumbers")

	if frag_label:
		frag_label.text = ""
	if round_message:
		round_message.text = ""
		round_message.visible = false
	if weapon_label:
		weapon_label.text = ""
	if weapon_icon:
		weapon_icon.visible = false
	set_mode("SPECTATING")
	update_scoreboard()
	if crosshair:
		crosshair.visible = false
	if damage_flash:
		damage_flash.visible = false
		damage_flash.modulate.a = 0.0
	if spawn_flash:
		spawn_flash.visible = false
		spawn_flash.modulate.a = 0.0
	if streak_flash:
		streak_flash.visible = false
		streak_flash.modulate.a = 0.0
	if fp_weapon:
		fp_weapon.visible = false
		fp_weapon_base_pos = fp_weapon.position
		fp_weapon_scene_base = fp_weapon.position
	_update_broadcast_chrome("Warmup")

func set_status(text: String):
	if status_label:
		status_label.text = "Status: " + text

func set_league_identity(mode_name: String, playlist: String):
	if mode_name != "":
		league_mode_name = mode_name
	if playlist != "":
		league_playlist = playlist
	_refresh_mode_label()
	update_scoreboard()

func set_map_name(name: String):
	if name != "":
		map_label = name
	_refresh_mode_label()

func set_pressure(pressure: String):
	pressure_id = pressure
	_refresh_mode_label()

func set_mode(mode: String):
	client_mode = mode
	_refresh_mode_label()

func _refresh_mode_label():
	if not mode_label:
		return
	var league = league_mode_name.to_upper() + " // " + league_playlist.to_upper()
	var map_chip = "\nMAP: " + map_label.to_upper()
	var host_chip = ""
	if sticky_host_line != "":
		host_chip = "\n" + sticky_host_line
	var controls = ""
	if client_mode == "SPECTATING":
		controls = "SPECTATING (J/A: Join, F/D-pad: Cycle, V/Back: Free-fly, R/N/M: Radio, ESC: Mouse) | Pad OK"
	else:
		controls = client_mode + " (L/Start: Leave, sticks move/look, RT/A: Fire, LB/RB: Weapon, Y/T: Speak, R/N/M: Radio) | Pad OK"
	var pressure_chip = ""
	if pressure_id == "compliance_drone":
		pressure_chip = "\nPRESSURE: CONTINUANCE COMPLIANCE DRONE"
	elif pressure_id == "compliance":
		pressure_chip = "\nPRESSURE: CONTINUANCE COMPLIANCE"
	mode_label.text = league + map_chip + host_chip + "\n" + controls + pressure_chip

func set_tick(tick: int):
	if tick_label:
		var seconds = tick / 20
		tick_label.text = "Time: " + str(seconds) + "s"

func set_round_info(state: String, time_left: int, frag_limit: int):
	round_chrome_state = state
	_update_broadcast_chrome(state)
	if not round_label:
		return
	var text = "Round: " + state
	if state == "Active":
		if frag_limit > 0:
			text = "ARENA DUEL // FIRST TO " + str(frag_limit)
			if time_left > 0:
				text += " | " + str(time_left) + "s"
		elif time_left > 0:
			text += " | Time: " + str(time_left) + "s"
		if leader_name != "":
			text += "\nLEADER: " + leader_name
		if ghost_rival != "":
			text += " | RIVAL: " + ghost_rival
		if pressure_id == "compliance_drone":
			text += "\nARTICLE 7 ENFORCEMENT"
		elif pressure_id == "compliance":
			text += "\nAPPROVED LANES ONLY"
	elif state == "Warmup":
		text = "WARMUP // " + map_label.to_upper()
		if time_left > 0:
			text += " // GOES LIVE IN " + str(time_left)
		else:
			text += " // Contested Frequency tuning in"
	elif state == "Ended":
		text = "ROUND OVER - podium holds"
		if leader_name != "":
			text += "\nMVP: " + leader_name
	round_label.text = text

func set_player_count(count: int):
	if player_count_label:
		player_count_label.text = "Fighters: " + str(count)

func update_scoreboard():
	if not scoreboard:
		return
	var sorted_scores = []
	for player in scores.keys():
		sorted_scores.append({"name": player, "kills": scores[player]})
	sorted_scores.sort_custom(func(a, b): return a.kills > b.kills)
	var text = "SCRAP LEAGUE\n" + league_mode_name.to_upper() + "\n"
	for i in range(min(8, len(sorted_scores))):
		var entry = sorted_scores[i]
		var chip = ""
		if behaviors.has(entry.name):
			chip = " [" + _short_behavior(behaviors[entry.name]) + "]"
		var marker = "*" if i == 0 and entry.kills > 0 else " "
		text += str(i + 1) + "." + marker + entry.name + chip + ": " + str(entry.kills) + "\n"
	scoreboard.text = text if len(sorted_scores) > 0 else "SCRAP LEAGUE\n" + league_mode_name.to_upper() + "\n(waiting for scrap)"

func _short_behavior(behavior: String) -> String:
	match behavior:
		"Aggressive":
			return "AGG"
		"Defensive":
			return "DEF"
		"Flanker":
			return "FLK"
		"Balanced":
			return "BAL"
		"Compliance":
			return "CMP"
		_:
			return behavior.substr(0, 3).to_upper()

func sync_scores_from_players(player_list: Array):
	var next_scores = {}
	var next_behaviors = {}
	for player_data in player_list:
		var pname = str(player_data.get("name", "?"))
		next_scores[pname] = int(player_data.get("score", 0))
		var beh = player_data.get("behavior", null)
		if beh != null:
			next_behaviors[pname] = str(beh)
	scores = next_scores
	behaviors = next_behaviors
	leader_name = ""
	var best = -1
	for pname in scores.keys():
		if scores[pname] > best:
			best = scores[pname]
			leader_name = pname + " (" + str(best) + ")"
	if best <= 0:
		leader_name = ""
	update_scoreboard()

func set_ghost_rival(rival: String):
	ghost_rival = rival

func show_frag(killer: String, victim: String, killer_color: Color = Color.WHITE, victim_color: Color = Color.WHITE):
	if not scores.has(killer):
		scores[killer] = 0
	scores[killer] += 1

	update_scoreboard()

	if frag_label:
		var message = killer + " SCRAPPED " + victim

		if randf() < 0.067:
			var quips = [
				killer + " took " + victim + " off the air",
				killer + " > " + victim + " (skill issue)",
				"so back (" + killer + " -> " + victim + ")"
			]
			message = quips[randi() % quips.size()]

		frag_label.text = message
		frag_label.modulate = killer_color.lightened(0.4)
		frag_label.visible = true

		var tween = create_tween()
		tween.tween_property(frag_label, "scale", Vector2(1.3, 1.3), 0.08)
		tween.tween_property(frag_label, "scale", Vector2(1.0, 1.0), 0.12)

		await get_tree().create_timer(2.8).timeout
		if is_instance_valid(frag_label):
			frag_label.visible = false
			frag_label.modulate = Color.WHITE

func reset_host_chrome():
	# Clear sticky Host + flash latch so a reconnect mid-round can flash once again.
	sticky_host_line = ""
	host_line_seen = false
	_refresh_mode_label()


func _update_broadcast_chrome(state: String) -> void:
	# Contested Frequency / ON AIR / Hangar Candy grit. Dull, not neon.
	var warm = state == "Warmup"
	var live = state == "Active"
	var ended = state == "Ended"
	if chrome_strip:
		chrome_strip.visible = true
		var a = 0.92 if live else (0.88 if warm else 0.7)
		chrome_strip.modulate = Color(1, 1, 1, a)
	if on_air_badge:
		on_air_badge.visible = live
		if live:
			on_air_badge.modulate = Color(1, 1, 1, 0.95)
	if contested_frequency_badge:
		# Warm on Warmup / Host face; quieter while live so ON AIR owns the scrap.
		contested_frequency_badge.visible = true
		var ca = 0.95 if warm else (0.72 if live else 0.8)
		contested_frequency_badge.modulate = Color(0.95, 0.95, 0.98, ca)
	if hangar_candy_badge:
		hangar_candy_badge.visible = true
		var ha = 0.85 if (warm or ended) else 0.75
		hangar_candy_badge.modulate = Color(1, 1, 1, ha)

func flash_broadcast_chrome(kind: String = "host") -> void:
	# Brief badge lift on Host / Warmup bumper without neon wash.
	var badge = contested_frequency_badge
	if kind == "on_air":
		badge = on_air_badge
		if on_air_badge:
			on_air_badge.visible = true
	elif kind == "hangar":
		badge = hangar_candy_badge
	if badge == null:
		return
	var base_a = badge.modulate.a
	badge.modulate.a = minf(base_a + 0.15, 1.0)
	var tween = create_tween()
	tween.tween_property(badge, "modulate:a", base_a, 0.45)

func set_host_line(line: String, flash_on_first: bool = false) -> bool:
	# Returns true when this call triggered the one-shot mid-join Host flash.
	if line == "":
		return false
	sticky_host_line = line
	_refresh_mode_label()
	if flash_on_first and not host_line_seen:
		host_line_seen = true
		show_host_join(line)
		return true
	return false

func show_host_join(host_line: String):
	host_spoke.emit(3.0)
	# Mid-join Host bumper: same energy as RoundStart Host chrome, without waiting for RoundStart.
	flash_broadcast_chrome("host")
	if round_message:
		var line = host_line
		if line == "":
			line = "HOST: CONTESTED FREQUENCY. LEAGUE DENIES EXISTENCE. ARENA DUEL IS LIVE."
		round_message.text = line + "\n" + league_mode_name.to_upper() + " // " + league_playlist.to_upper()
		round_message.visible = true
		var tween = create_tween()
		tween.tween_property(round_message, "scale", Vector2(1.2, 1.2), 0.15)
		tween.tween_property(round_message, "scale", Vector2(1.0, 1.0), 0.2)
		await get_tree().create_timer(2.5).timeout
		if is_instance_valid(round_message):
			round_message.visible = false

func show_warmup_bumper(host_line: String, secs_left: int = 0):
	host_spoke.emit(3.0)
	# Warmup / pre-round Host drama: roster + map bumper readable before RoundStart.
	flash_broadcast_chrome("host")
	if round_message:
		var line = host_line
		if line == "":
			line = "HOST: CONTESTED FREQUENCY. " + map_label.to_upper() + " TUNES IN."
		var sub = "WARMUP // " + map_label.to_upper()
		if secs_left > 0:
			sub += " // " + str(secs_left)
		round_message.text = line + "\n" + sub
		round_message.visible = true
		var tween = create_tween()
		tween.tween_property(round_message, "scale", Vector2(1.2, 1.2), 0.12)
		tween.tween_property(round_message, "scale", Vector2(1.0, 1.0), 0.18)
		await get_tree().create_timer(2.2).timeout
		if is_instance_valid(round_message):
			round_message.visible = false

func show_round_start(round_number: int, host_line: String = ""):
	host_spoke.emit(3.0)
	flash_broadcast_chrome("on_air")
	if round_message:
		var line = host_line
		if line == "":
			line = HOST_BUMPERS[host_bumper_index % HOST_BUMPERS.size()]
			host_bumper_index += 1
		sticky_host_line = line
		host_line_seen = true
		_refresh_mode_label()
		round_message.text = line + "\n" + league_playlist.to_upper() + " ROUND " + str(round_number) + " - FIGHT!"
		if ghost_rival != "":
			round_message.text += "\nGHOST RIVAL: " + ghost_rival
		round_message.visible = true
		var tween = create_tween()
		tween.tween_property(round_message, "scale", Vector2(1.3, 1.3), 0.2)
		tween.tween_property(round_message, "scale", Vector2(1.0, 1.0), 0.2)
		await get_tree().create_timer(3.0).timeout
		if is_instance_valid(round_message):
			round_message.visible = false

func show_compliance_ping(message: String, duration_sec: float = 6.0):
	if round_message:
		var line = message
		if line == "":
			line = "HOST: CONTINUANCE COMPLIANCE PING. APPROVED LANES ONLY."
		sticky_host_line = line
		host_line_seen = true
		_refresh_mode_label()
		round_message.text = line + "\n" + league_mode_name.to_upper() + " PRESSURE"
		round_message.visible = true
		var tween = create_tween()
		tween.tween_property(round_message, "scale", Vector2(1.25, 1.25), 0.15)
		tween.tween_property(round_message, "scale", Vector2(1.0, 1.0), 0.2)
		await get_tree().create_timer(max(duration_sec, 2.0)).timeout
		if is_instance_valid(round_message):
			round_message.visible = false


func show_boss_spawn(message: String, name: String = "COMPLIANCE-DRONE"):
	host_spoke.emit(3.0)
	if round_message:
		var line = message
		if line == "":
			line = "HOST: CONTINUANCE COMPLIANCE DRONE ON DECK. ARTICLE 7 ENFORCEMENT."
		sticky_host_line = line
		host_line_seen = true
		pressure_id = "compliance_drone"
		_refresh_mode_label()
		round_message.text = line + "\nBOSS: " + name
		round_message.visible = true
		var tween = create_tween()
		tween.tween_property(round_message, "scale", Vector2(1.3, 1.3), 0.15)
		tween.tween_property(round_message, "scale", Vector2(1.0, 1.0), 0.2)
		await get_tree().create_timer(4.0).timeout
		if is_instance_valid(round_message):
			round_message.visible = false

func show_boss_down(message: String, killer: String = ""):
	host_spoke.emit(3.0)
	if round_message:
		var line = message
		if line == "":
			line = "HOST: DRONE DOWN. CONTINUANCE DENIES THE INCIDENT. SCRAP ON."
		sticky_host_line = line
		host_line_seen = true
		pressure_id = ""
		_refresh_mode_label()
		var killer_chip = ""
		if killer != "":
			killer_chip = "\nFRAG BY " + killer
		round_message.text = line + killer_chip
		round_message.visible = true
		var tween = create_tween()
		tween.tween_property(round_message, "scale", Vector2(1.25, 1.25), 0.12)
		tween.tween_property(round_message, "scale", Vector2(1.0, 1.0), 0.18)
		await get_tree().create_timer(3.5).timeout
		if is_instance_valid(round_message):
			round_message.visible = false

func show_killstreak(player_name: String, streak: int, tier: String, message: String):
	host_spoke.emit(2.5)
	# Arena multi-kill Host bumper + brief ember flash (spectator and join).
	streak_flash_timer = 0.4
	if streak_flash:
		streak_flash.visible = true
		streak_flash.modulate = Color(1.0, 0.72, 0.22, 0.5)
	var line = message
	if line == "":
		match tier:
			"double":
				line = "HOST: DOUBLE FREQUENCY. " + player_name + " DENIES THE DENIAL."
			"triple":
				line = "HOST: TRIPLE SCRAP. CONTINUANCE LOSES COUNT."
			"rampage":
				line = "HOST: FREQUENCY RAMPAGE. " + player_name + " BREAKS EVERY APPROVED LANE."
			_:
				line = "HOST: MULTI SCRAP. " + player_name + " IS LIVE."
	if round_message:
		round_message.text = line + "\nSTREAK " + str(streak) + " // " + tier.to_upper()
		round_message.visible = true
		var tween = create_tween()
		tween.tween_property(round_message, "scale", Vector2(1.35, 1.35), 0.1)
		tween.tween_property(round_message, "scale", Vector2(1.0, 1.0), 0.18)
		await get_tree().create_timer(2.8).timeout
		if is_instance_valid(round_message):
			round_message.visible = false
	if frag_label:
		frag_label.text = tier.to_upper() + " // " + player_name
		frag_label.modulate = Color(1.0, 0.85, 0.35)
		frag_label.visible = true
		var ft = create_tween()
		ft.tween_property(frag_label, "scale", Vector2(1.4, 1.4), 0.08)
		ft.tween_property(frag_label, "scale", Vector2(1.0, 1.0), 0.14)
		await get_tree().create_timer(2.2).timeout
		if is_instance_valid(frag_label):
			frag_label.visible = false
			frag_label.modulate = Color.WHITE

func show_speak(player: String, line: String):
	# Killfeed-adjacent callout; keep string literals simple for Godot.
	if frag_label:
		var message = player + ": " + line
		frag_label.text = message
		frag_label.modulate = Color(0.85, 0.95, 1.0)
		frag_label.visible = true
		var tween = create_tween()
		tween.tween_property(frag_label, "scale", Vector2(1.15, 1.15), 0.06)
		tween.tween_property(frag_label, "scale", Vector2(1.0, 1.0), 0.1)
		await get_tree().create_timer(2.5).timeout
		if is_instance_valid(frag_label):
			frag_label.visible = false
			frag_label.modulate = Color.WHITE
	if round_message and line != "":
		round_message.text = "CALL: " + player + "\n" + line
		round_message.visible = true
		await get_tree().create_timer(2.0).timeout
		if is_instance_valid(round_message):
			round_message.visible = false

func show_round_end(mvp_name: String, reason: String, mvp_frags: int = 0, host_line: String = "", podium = []):
	host_spoke.emit(4.0)
	# Round-end MVP / podium Host drama (Contested Frequency voice).
	scores = {}
	behaviors = {}
	leader_name = mvp_name
	pressure_id = ""
	if host_line != "":
		sticky_host_line = host_line
		host_line_seen = true
	_refresh_mode_label()
	update_scoreboard()

	if weapon_label:
		weapon_label.text = ""
	if weapon_icon:
		weapon_icon.visible = false

	followed_player_name = ""

	# Brief ember podium flash (same grit as killstreak).
	streak_flash_timer = 0.55
	if streak_flash:
		streak_flash.visible = true
		streak_flash.modulate = Color(1.0, 0.78, 0.28, 0.55)

	if round_message:
		var message = host_line
		if message == "":
			if mvp_name != "":
				message = "HOST: ROUND MVP. " + mvp_name + " WITH " + str(mvp_frags) + " FRAGS. CONTINUANCE DENIES THE PODIUM."
			else:
				message = "HOST: ROUND CLOSED. NO MVP. LEAGUE DENIES THE SCRAP."
		if reason != "":
			message += "\n" + reason.to_upper()
		# Podium: top three scrap scores.
		var lines = []
		if typeof(podium) == TYPE_ARRAY:
			var n = mini(3, podium.size())
			for i in range(n):
				var row = podium[i]
				var nm = str(row.get("name", "?")) if typeof(row) == TYPE_DICTIONARY else str(row)
				var sc = str(row.get("score", "?")) if typeof(row) == TYPE_DICTIONARY else ""
				var rank = str(i + 1)
				if sc != "":
					lines.append("#" + rank + " " + nm + " " + sc)
				else:
					lines.append("#" + rank + " " + nm)
		if lines.size() > 0:
			message += "\nPODIUM: " + " | ".join(PackedStringArray(lines))
		message += "\n" + league_mode_name.to_upper() + " // " + league_playlist.to_upper()

		round_message.text = message
		round_message.visible = true

		var tween = create_tween()
		tween.tween_property(round_message, "scale", Vector2(1.45, 1.45), 0.12)
		tween.tween_property(round_message, "scale", Vector2(1.0, 1.0), 0.22)

		await get_tree().create_timer(5.5).timeout
		if is_instance_valid(round_message):
			round_message.visible = false

	if frag_label and mvp_name != "":
		frag_label.text = "MVP // " + mvp_name + " // " + str(mvp_frags)
		frag_label.modulate = Color(1.0, 0.88, 0.4)
		frag_label.visible = true
		var ft = create_tween()
		ft.tween_property(frag_label, "scale", Vector2(1.45, 1.45), 0.1)
		ft.tween_property(frag_label, "scale", Vector2(1.0, 1.0), 0.16)
		await get_tree().create_timer(4.0).timeout
		if is_instance_valid(frag_label):
			frag_label.visible = false
			frag_label.modulate = Color.WHITE

func show_pickup_toast(player_name: String, weapon_name: String, kind: String = "weapon", amount: int = 0):
	if not round_message:
		return
	var what: String
	if kind == "health":
		what = ("+%d HP" % amount) if amount > 0 else "MEDKIT"
	elif kind == "armor":
		what = ("+%d ARMOR" % amount) if amount > 0 else "ARMOR"
	else:
		what = weapon_name.to_upper() if weapon_name != "" else "PAD"
	var line = "SCRAP PAD: %s grabbed %s" % [player_name, what]
	round_message.text = line
	round_message.visible = true
	if kind == "health":
		round_message.modulate = Color(0.72, 0.32, 0.28)
	elif kind == "armor":
		round_message.modulate = Color(0.55, 0.52, 0.46)
	else:
		round_message.modulate = Color(0.86, 0.82, 0.74)
	var tree = get_tree()
	if tree:
		await tree.create_timer(1.6).timeout
		if round_message:
			round_message.visible = false

func set_followed_weapon(weapon_name: String, player_name: String = "", behavior: String = ""):
	if not weapon_label or not weapon_icon:
		return

	followed_player_name = player_name

	if weapon_name == "" or not weapon_textures.has(weapon_name):
		weapon_label.text = ""
		weapon_icon.visible = false
		return

	var weapon_desc = ""
	match weapon_name:
		"Flechette":
			weapon_desc = "FLECHETTE (mid)"
		"Rail":
			weapon_desc = "RAIL (long)"
		"Scatter":
			weapon_desc = "SCATTER (close)"

	var display_text = weapon_desc
	if player_name != "":
		var role_chip = ""
		if behavior != "":
			role_chip = " [" + _short_behavior(behavior) + "]"
		display_text = "FOLLOWING: " + player_name + role_chip + "\n" + weapon_desc

	weapon_label.text = display_text
	weapon_icon.texture = weapon_textures[weapon_name]
	weapon_icon.modulate = Color(1.15, 1.1, 1.05, 1)
	weapon_icon.visible = true

func _process(delta):
	if damage_flash_timer > 0:
		damage_flash_timer -= delta
		if damage_flash:
			damage_flash.visible = true
			damage_flash.modulate.a = clampf(damage_flash_timer / 0.22, 0.0, 0.55)
		if damage_flash_timer <= 0 and damage_flash:
			damage_flash.visible = false
			damage_flash.modulate.a = 0.0
	if spawn_flash_timer > 0:
		spawn_flash_timer -= delta
		if spawn_flash:
			spawn_flash.visible = true
			spawn_flash.modulate.a = clampf(spawn_flash_timer / 0.35, 0.0, 0.45)
		if spawn_flash_timer <= 0 and spawn_flash:
			spawn_flash.visible = false
			spawn_flash.modulate.a = 0.0
	if streak_flash_timer > 0:
		streak_flash_timer -= delta
		if streak_flash:
			streak_flash.visible = true
			streak_flash.modulate.a = clampf(streak_flash_timer / 0.4, 0.0, 0.5)
		if streak_flash_timer <= 0 and streak_flash:
			streak_flash.visible = false
			streak_flash.modulate.a = 0.0
	if hit_marker_timer > 0:
		hit_marker_timer -= delta
		if hit_marker:
			hit_marker.visible = true
			hit_marker.modulate.a = clampf(hit_marker_timer / 0.18, 0.0, 1.0)
		if hit_marker_timer <= 0 and hit_marker:
			hit_marker.visible = false
			hit_marker.modulate.a = 0.0
	if fp_kick_timer > 0:
		fp_kick_timer -= delta
	_update_floating_damage(delta)
	if fp_juice_enabled and fp_weapon and fp_weapon.visible:
		fp_bob_t += delta * 9.0
		var bob_scale = 1.0
		match current_fp_weapon:
			"Rail":
				bob_scale = 0.55
			"Scatter":
				bob_scale = 1.35
			_:
				bob_scale = 1.0
		var bob_y = sin(fp_bob_t) * 4.0 * bob_scale
		var bob_x = cos(fp_bob_t * 0.5) * 2.0 * bob_scale
		var kick = Vector2.ZERO
		if fp_kick_timer > 0:
			var k = clampf(fp_kick_timer / 0.12, 0.0, 1.0)
			kick = fp_kick_amount * k
		fp_weapon.position = fp_weapon_base_pos + Vector2(bob_x, bob_y) + kick

func set_fp_juice(enabled: bool) -> void:
	fp_juice_enabled = enabled
	if crosshair:
		crosshair.visible = enabled
	if not enabled:
		if fp_weapon:
			fp_weapon.visible = false
		if damage_flash:
			damage_flash.visible = false
			damage_flash.modulate.a = 0.0
		if spawn_flash:
			spawn_flash.visible = false
			spawn_flash.modulate.a = 0.0
		if hit_marker:
			hit_marker.visible = false
		damage_flash_timer = 0.0
		spawn_flash_timer = 0.0
		hit_marker_timer = 0.0
		fp_kick_timer = 0.0
		fp_bob_t = 0.0
		current_fp_weapon = ""
		_clear_floating_damage()

func set_fp_weapon(weapon_name: String) -> void:
	if not fp_weapon:
		return
	if not fp_juice_enabled or weapon_name == "" or not weapon_textures.has(weapon_name):
		fp_weapon.visible = false
		return
	var changed = weapon_name != current_fp_weapon
	current_fp_weapon = weapon_name
	fp_weapon.texture = weapon_textures[weapon_name]
	# Distinct viewmodel pose per role (bone/gunmetal, not neon).
	# Only re-base on swap so walk bob / fire kick survive snapshot ticks.
	if changed:
		match weapon_name:
			"Rail":
				fp_weapon.modulate = Color(0.82, 0.86, 0.88, 1)
				fp_weapon.scale = Vector2(1.15, 1.15)
				fp_weapon_base_pos = fp_weapon_scene_base + Vector2(-20, -20)
			"Scatter":
				fp_weapon.modulate = Color(1.05, 0.88, 0.7, 1)
				fp_weapon.scale = Vector2(1.25, 1.1)
				fp_weapon_base_pos = fp_weapon_scene_base + Vector2(20, 10)
			_:
				fp_weapon.modulate = Color(1.08, 1.04, 0.98, 1)
				fp_weapon.scale = Vector2(1.0, 1.0)
				fp_weapon_base_pos = fp_weapon_scene_base
		fp_weapon.position = fp_weapon_base_pos
		_apply_crosshair_for_weapon(weapon_name)
	fp_weapon.visible = true

func _apply_crosshair_for_weapon(weapon_name: String) -> void:
	if not crosshair or not fp_juice_enabled:
		return
	# Bone grit crosshair shapes per role.
	var bone = Color(0.91, 0.886, 0.839, 0.85)
	var ember = Color(0.85, 0.62, 0.38, 0.8)
	var gun = Color(0.7, 0.74, 0.76, 0.95)
	if crosshair_hbar:
		crosshair_hbar.visible = true
		crosshair_hbar.color = bone
	if crosshair_vbar:
		crosshair_vbar.visible = true
		crosshair_vbar.color = bone
	if crosshair_dot:
		crosshair_dot.visible = false
	if crosshair_ring:
		crosshair_ring.visible = false
	match weapon_name:
		"Rail":
			if crosshair_hbar:
				crosshair_hbar.visible = false
			if crosshair_vbar:
				crosshair_vbar.visible = false
			if crosshair_dot:
				crosshair_dot.visible = true
				crosshair_dot.color = gun
				crosshair_dot.offset_left = -2.0
				crosshair_dot.offset_top = -2.0
				crosshair_dot.offset_right = 2.0
				crosshair_dot.offset_bottom = 2.0
		"Scatter":
			if crosshair_hbar:
				crosshair_hbar.offset_left = -18.0
				crosshair_hbar.offset_right = 18.0
				crosshair_hbar.offset_top = -1.0
				crosshair_hbar.offset_bottom = 1.0
				crosshair_hbar.color = ember
			if crosshair_vbar:
				crosshair_vbar.offset_top = -18.0
				crosshair_vbar.offset_bottom = 18.0
				crosshair_vbar.offset_left = -1.0
				crosshair_vbar.offset_right = 1.0
				crosshair_vbar.color = ember
			if crosshair_ring:
				crosshair_ring.visible = true
				crosshair_ring.color = Color(0.78, 0.55, 0.32, 0.22)
		_:
			if crosshair_hbar:
				crosshair_hbar.offset_left = -10.0
				crosshair_hbar.offset_right = 10.0
				crosshair_hbar.offset_top = -1.0
				crosshair_hbar.offset_bottom = 1.0
			if crosshair_vbar:
				crosshair_vbar.offset_top = -10.0
				crosshair_vbar.offset_bottom = 10.0
				crosshair_vbar.offset_left = -1.0
				crosshair_vbar.offset_right = 1.0

func show_hit_marker(damage: int = 0, weapon_name: String = "") -> void:
	# Light grit confirm when local / followed player scores a hit.
	if not fp_juice_enabled and client_mode == "SPECTATING":
		# Spectator follow path still gets a brief marker.
		pass
	hit_marker_timer = 0.18
	if hit_marker:
		hit_marker.visible = true
		var col = Color(0.91, 0.82, 0.7, 0.95)
		match weapon_name:
			"Rail":
				col = Color(0.72, 0.78, 0.82, 0.95)
				hit_marker_timer = 0.28
			"Scatter":
				col = Color(0.9, 0.55, 0.32, 0.95)
				hit_marker_timer = 0.14
			_:
				col = Color(0.91, 0.82, 0.7, 0.95)
		hit_marker.modulate = col
	if damage > 0:
		_spawn_floating_damage(damage, weapon_name)
	# Fire kick on confirm sells the shot.
	_fp_fire_kick(weapon_name)

func show_fire_juice(weapon_name: String = "") -> void:
	_fp_fire_kick(weapon_name if weapon_name != "" else current_fp_weapon)

func _fp_fire_kick(weapon_name: String) -> void:
	if not fp_juice_enabled:
		return
	fp_kick_timer = 0.12
	match weapon_name:
		"Rail":
			fp_kick_amount = Vector2(8, 22)
			fp_kick_timer = 0.18
		"Scatter":
			fp_kick_amount = Vector2(14, 10)
			fp_kick_timer = 0.10
		_:
			fp_kick_amount = Vector2(6, 8)

func _spawn_floating_damage(damage: int, weapon_name: String) -> void:
	if not damage_numbers:
		return
	var label = Label.new()
	label.text = str(damage)
	label.mouse_filter = Control.MOUSE_FILTER_IGNORE
	var col = Color(0.91, 0.82, 0.7, 1)
	match weapon_name:
		"Rail":
			col = Color(0.75, 0.82, 0.86, 1)
		"Scatter":
			col = Color(0.92, 0.55, 0.3, 1)
		_:
			col = Color(0.95, 0.78, 0.45, 1)
	label.add_theme_color_override("font_color", col)
	label.add_theme_font_size_override("font_size", 22 if weapon_name != "Rail" else 28)
	var ox = randf_range(-28.0, 28.0)
	label.position = Vector2(ox, -20.0)
	damage_numbers.add_child(label)
	floating_damage_nodes.append({"node": label, "t": 0.0, "life": 0.55, "ox": ox})

func _update_floating_damage(delta: float) -> void:
	var keep = []
	for entry in floating_damage_nodes:
		var node = entry.get("node")
		if node == null or not is_instance_valid(node):
			continue
		entry["t"] += delta
		var t = float(entry["t"])
		var life = float(entry["life"])
		var progress = clampf(t / life, 0.0, 1.0)
		node.position = Vector2(float(entry["ox"]), -20.0 - progress * 48.0)
		node.modulate.a = 1.0 - progress
		if t < life:
			keep.append(entry)
		else:
			node.queue_free()
	floating_damage_nodes = keep

func _clear_floating_damage() -> void:
	for entry in floating_damage_nodes:
		var node = entry.get("node")
		if node != null and is_instance_valid(node):
			node.queue_free()
	floating_damage_nodes = []

func show_damage_flash() -> void:
	if not fp_juice_enabled:
		return
	damage_flash_timer = 0.22
	if damage_flash:
		damage_flash.visible = true
		damage_flash.modulate = Color(0.55, 0.08, 0.06, 0.55)

func show_spawn_flash() -> void:
	if not fp_juice_enabled:
		return
	spawn_flash_timer = 0.35
	if spawn_flash:
		spawn_flash.visible = true
		# Ember grit flash on spawn / join.
		spawn_flash.modulate = Color(0.85, 0.45, 0.18, 0.45)


var radio_label: Label = null
var radio_tween: Tween = null

## Bottom-right radio toast: station on switch, title on each new track, then fade.
func show_radio(station: String, title: String) -> void:
	if radio_label == null:
		radio_label = Label.new()
		radio_label.name = "RadioLabel"
		radio_label.set_anchors_preset(Control.PRESET_BOTTOM_RIGHT)
		radio_label.anchor_left = 1.0
		radio_label.anchor_top = 1.0
		radio_label.anchor_right = 1.0
		radio_label.anchor_bottom = 1.0
		radio_label.offset_left = -520.0
		radio_label.offset_top = -44.0
		radio_label.offset_right = -16.0
		radio_label.offset_bottom = -16.0
		radio_label.horizontal_alignment = HORIZONTAL_ALIGNMENT_RIGHT
		radio_label.add_theme_color_override("font_color", Color(0.91, 0.89, 0.84))
		radio_label.add_theme_color_override("font_outline_color", Color(0.04, 0.04, 0.05))
		radio_label.add_theme_constant_override("outline_size", 4)
		add_child(radio_label)
	var text := "ON AIR: " + station
	if title != "":
		text += "  //  " + title
	radio_label.text = text
	radio_label.modulate.a = 1.0
	radio_label.visible = true
	if radio_tween != null and radio_tween.is_valid():
		radio_tween.kill()
	radio_tween = create_tween()
	radio_tween.tween_interval(3.5)
	radio_tween.tween_property(radio_label, "modulate:a", 0.0, 1.0)
