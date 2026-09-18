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

func set_mode(mode: String):
	if mode_label:
		if mode == "SPECTATING":
			mode_label.text = "SPECTATING (J: Join, F: Cycle Cam, ESC: Mouse)"
		else:
			mode_label.text = mode + " (L: Leave, ESC: Mouse)"

func set_tick(tick: int):
	if tick_label:
		var seconds = tick / 20
		tick_label.text = "Time: " + str(seconds) + "s"

func set_round_info(state: String, time_left: int, frag_limit: int):
	if not round_label:
		return
	
	var text = "Round: " + state
	if state == "Active":
		if time_left > 0:
			var time_display = str(time_left) + "s"
			if time_left == 67:
				time_display = "67s (!)"
			text += " | Time: " + time_display
		if frag_limit > 0:
			text += " | Frag limit: " + str(frag_limit)
	
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
	
	var text = "SCOREBOARD\n"
	for i in range(min(8, len(sorted_scores))):
		var entry = sorted_scores[i]
		text += entry.name + ": " + str(entry.kills) + "\n"
	
	scoreboard.text = text if len(sorted_scores) > 0 else "SCOREBOARD\n(no kills yet)"

func show_frag(killer: String, victim: String, killer_color: Color = Color.WHITE, victim_color: Color = Color.WHITE):
	if not scores.has(killer):
		scores[killer] = 0
	scores[killer] += 1
	
	update_scoreboard()
	
	if frag_label:
		var message = killer + " FRAGGED " + victim + "!"
		
		if randf() < 0.067:
			var quips = [
				killer + " [67] " + victim,
				killer + " > " + victim + " (skill issue)",
				"so back (" + killer + " \u2192 " + victim + ")"
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

func show_round_start(round_number: int):
	if round_message:
		var host_lines = [
			"HOST: ROUND " + str(round_number) + ". LIVE LAUGH FRAG.",
			"HOST: FIGHTERS UP. PORT 6767 ENERGY.",
			"HOST: CONTESTED FREQUENCY. LEAGUE DENIES EXISTENCE.",
			"ROUND " + str(round_number) + " - FIGHT!"
		]
		round_message.text = host_lines[randi() % host_lines.size()]
		round_message.visible = true
		
		var tween = create_tween()
		tween.tween_property(round_message, "scale", Vector2(1.3, 1.3), 0.2)
		tween.tween_property(round_message, "scale", Vector2(1.0, 1.0), 0.2)
		
		await get_tree().create_timer(3.0).timeout
		if is_instance_valid(round_message):
			round_message.visible = false

func show_round_end(winner: String, reason: String):
	scores = {}
	update_scoreboard()
	
	if weapon_label:
		weapon_label.text = ""
	if weapon_icon:
		weapon_icon.visible = false
	
	followed_player_name = ""
	
	if round_message:
		var message = "HOST: " + reason.to_upper()
		if winner != "":
			message += "\n\nWINNER: " + winner + "!"
		
		round_message.text = message
		round_message.visible = true
		
		var tween = create_tween()
		tween.tween_property(round_message, "scale", Vector2(1.4, 1.4), 0.15)
		tween.tween_property(round_message, "scale", Vector2(1.0, 1.0), 0.2)
		
		await get_tree().create_timer(4.0).timeout
		if is_instance_valid(round_message):
			round_message.visible = false

func set_followed_weapon(weapon_name: String, player_name: String = ""):
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
		display_text = player_name + "\n" + weapon_desc
	
	weapon_label.text = display_text
	weapon_icon.texture = weapon_textures[weapon_name]
	weapon_icon.visible = true
