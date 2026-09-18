extends CanvasLayer

@onready var status_label = $Panel/VBoxContainer/StatusLabel
@onready var tick_label = $Panel/VBoxContainer/TickLabel
@onready var player_count_label = $Panel/VBoxContainer/PlayerCountLabel
@onready var frag_label = $FragLabel

func _ready():
	if frag_label:
		frag_label.text = ""

func set_status(text: String):
	if status_label:
		status_label.text = "Status: " + text

func set_tick(tick: int):
	if tick_label:
		tick_label.text = "Tick: " + str(tick)

func set_player_count(count: int):
	if player_count_label:
		player_count_label.text = "Players: " + str(count)

func show_frag(killer: String, victim: String):
	if frag_label:
		frag_label.text = killer + " fragged " + victim
		frag_label.visible = true
		await get_tree().create_timer(3.0).timeout
		if is_instance_valid(frag_label):
			frag_label.visible = false
