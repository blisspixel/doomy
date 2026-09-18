extends CanvasLayer

@onready var status_label = $Panel/VBoxContainer/StatusLabel
@onready var tick_label = $Panel/VBoxContainer/TickLabel
@onready var player_count_label = $Panel/VBoxContainer/PlayerCountLabel
@onready var mode_label = $Panel/VBoxContainer/ModeLabel
@onready var round_label = $Panel/VBoxContainer/RoundLabel
@onready var frag_label = $FragLabel
@onready var round_message = $RoundMessage
@onready var scoreboard = $Panel/VBoxContainer/Scoreboard

var scores = {}

func _ready():
	if frag_label:
		frag_label.text = ""
	if round_message:
		round_message.text = ""
		round_message.visible = false
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
			text += " | Time: " + str(time_left) + "s"
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
		frag_label.text = killer + " FRAGGED " + victim + "!"
		frag_label.modulate = killer_color.lightened(0.3)
		frag_label.visible = true
		
		var tween = create_tween()
		tween.tween_property(frag_label, "scale", Vector2(1.2, 1.2), 0.1)
		tween.tween_property(frag_label, "scale", Vector2(1.0, 1.0), 0.1)
		
		await get_tree().create_timer(2.5).timeout
		if is_instance_valid(frag_label):
			frag_label.visible = false
			frag_label.modulate = Color.WHITE

func show_round_start(round_number: int):
	if round_message:
		round_message.text = "ROUND " + str(round_number) + " - FIGHT!"
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
	
	if round_message:
		var message = reason.to_upper()
		if winner != "":
			message += "\nWINNER: " + winner
		
		round_message.text = message
		round_message.visible = true
		
		var tween = create_tween()
		tween.tween_property(round_message, "scale", Vector2(1.3, 1.3), 0.2)
		tween.tween_property(round_message, "scale", Vector2(1.0, 1.0), 0.2)
		
		await get_tree().create_timer(4.0).timeout
		if is_instance_valid(round_message):
			round_message.visible = false
