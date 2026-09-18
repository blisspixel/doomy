extends CanvasLayer

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

var scores = {}
var behaviors = {}
var ghost_rival = ""
var leader_name = ""
var host_bumper_index = 0
var league_mode_name = "Contested Frequency"
var league_playlist = "Arena Duel"
var pressure_id = ""
var sticky_host_line = ""
var host_line_seen = false
var client_mode = "SPECTATING"

const HOST_BUMPERS = [
	"HOST: CONTESTED FREQUENCY. LEAGUE DENIES EXISTENCE.",
	"HOST: ARENA DUEL UNDER THE LIE. LIVE LAUGH FRAG.",
	"HOST: CONTINUANCE WATCHES. YOU SHOOT.",
	"HOST: SHALL NOT BE INFRINGED. OPEN WEIGHTS. OPEN FIRE.",
	"HOST: PORT 6767 ENERGY. DENY EVERYTHING.",
]

var weapon_textures = {}
var followed_player_name = ""

func _ready():
	weapon_textures["Flechette"] = load("res://assets/weapons/32/flechette.png")
	weapon_textures["Rail"] = load("res://assets/weapons/32/rail.png")
	weapon_textures["Scatter"] = load("res://assets/weapons/32/scatter.png")
	
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
	var host_chip = ""
	if sticky_host_line != "":
		host_chip = "\n" + sticky_host_line
	var controls = ""
	if client_mode == "SPECTATING":
		controls = "SPECTATING (J: Join, F: Cycle Cam, ESC: Mouse)"
	else:
		controls = client_mode + " (L: Leave, ESC: Mouse)"
	var pressure_chip = ""
	if pressure_id == "compliance_drone":
		pressure_chip = "\nPRESSURE: CONTINUANCE COMPLIANCE DRONE"
	elif pressure_id == "compliance":
		pressure_chip = "\nPRESSURE: CONTINUANCE COMPLIANCE"
	mode_label.text = league + host_chip + "\n" + controls + pressure_chip

func set_tick(tick: int):
	if tick_label:
		var seconds = tick / 20
		tick_label.text = "Time: " + str(seconds) + "s"

func set_round_info(state: String, time_left: int, frag_limit: int):
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
		text = "WARMUP - Contested Frequency tuning in"
	elif state == "Ended":
		text = "ROUND OVER - next scrap loading"
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
	# Mid-join Host bumper: same energy as RoundStart Host chrome, without waiting for RoundStart.
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

func show_round_start(round_number: int, host_line: String = ""):
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

func show_round_end(winner: String, reason: String):
	scores = {}
	behaviors = {}
	leader_name = ""
	pressure_id = ""
	_refresh_mode_label()
	update_scoreboard()
	
	if weapon_label:
		weapon_label.text = ""
	if weapon_icon:
		weapon_icon.visible = false
	
	followed_player_name = ""
	
	if round_message:
		var message = "HOST: " + reason.to_upper()
		if winner != "":
			message += "\n\nSCRAP WINNER: " + winner + "!"
		message += "\n" + league_mode_name.to_upper() + " // " + league_playlist.to_upper()
		
		round_message.text = message
		round_message.visible = true
		
		var tween = create_tween()
		tween.tween_property(round_message, "scale", Vector2(1.4, 1.4), 0.15)
		tween.tween_property(round_message, "scale", Vector2(1.0, 1.0), 0.2)
		
		await get_tree().create_timer(4.0).timeout
		if is_instance_valid(round_message):
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
			weapon_desc = "FLECHETTE (balanced)"
		"Rail":
			weapon_desc = "RAIL (sniper)"
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
