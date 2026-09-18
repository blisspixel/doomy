extends Control

# First-class Solo Scrap boot. Loopback 6767 by default; Join Host for MP.

@onready var host_edit: LineEdit = $Center/VBox/HostRow/HostEdit
@onready var status_label: Label = $Center/VBox/StatusLabel
@onready var solo_button: Button = $Center/VBox/SoloButton
@onready var map_option: OptionButton = $Center/VBox/MapRow/MapOption

const ARENA_SCENE := "res://scenes/main.tscn"
const LOOPBACK := "127.0.0.1:6767"

func _ready() -> void:
	if host_edit:
		host_edit.text = LOOPBACK
	_setup_map_option()
	if solo_button:
		solo_button.grab_focus()
	# One-command / scripted path skips the menu.
	if _wants_solo_boot():
		_launch("solo", LOOPBACK)
		return
	if "--human" in OS.get_cmdline_user_args() or "--human" in OS.get_cmdline_args():
		_launch("solo", LOOPBACK)
		return
	if status_label:
		status_label.text = "Solo Broadcast Episode 0 (Calibration / Larak Lot) on loopback 6767. Run tools/solo_scrap.sh. Keyboard and gamepad both work (stick deadzone 0.25)."

func _setup_map_option() -> void:
	if not map_option:
		return
	map_option.clear()
	map_option.add_item("1 Arena Duel", 1)
	map_option.add_item("2 Compliance Yard", 2)
	var prefer = _preferred_map_id()
	for i in range(map_option.item_count):
		if map_option.get_item_id(i) == prefer:
			map_option.select(i)
			break

func _preferred_map_id() -> int:
	var env_map = OS.get_environment("FRAGR_MAP")
	if env_map == "2" or env_map.to_lower().begins_with("compliance"):
		return 2
	var args = OS.get_cmdline_user_args()
	args.append_array(OS.get_cmdline_args())
	for i in range(args.size()):
		if args[i] == "--map" and i + 1 < args.size():
			var v = str(args[i + 1]).to_lower()
			if v == "2" or v.begins_with("compliance"):
				return 2
			return 1
		if str(args[i]).begins_with("--map="):
			var v2 = str(args[i]).substr(6).to_lower()
			if v2 == "2" or v2.begins_with("compliance"):
				return 2
			return 1
	return 1

func _selected_map_id() -> int:
	if map_option:
		return int(map_option.get_selected_id())
	return _preferred_map_id()

func _wants_solo_boot() -> bool:
	if OS.get_environment("FRAGR_SOLO") == "1":
		return true
	var args = OS.get_cmdline_user_args()
	args.append_array(OS.get_cmdline_args())
	return "--solo" in args

func _launch(mode: String, host: String) -> void:
	var boot := {
		"mode": mode,
		"host": host,
		"map_id": _selected_map_id(),
	}
	get_tree().set_meta("fragr_boot", boot)
	var err := get_tree().change_scene_to_file(ARENA_SCENE)
	if err != OK:
		push_error("boot_menu: failed to load arena scene: " + str(err))
		if status_label:
			status_label.text = "Failed to load arena (" + str(err) + ")"

func _on_solo_pressed() -> void:
	_launch("solo", LOOPBACK)

func _on_spectate_pressed() -> void:
	_launch("spectate", LOOPBACK)

func _on_join_pressed() -> void:
	var host := LOOPBACK
	if host_edit and host_edit.text.strip_edges() != "":
		host = host_edit.text.strip_edges()
	_launch("join", host)
