extends Control

# First-class Solo Scrap boot. Loopback 6767 by default; Join Host for MP.

@onready var host_edit: LineEdit = $Center/VBox/HostRow/HostEdit
@onready var status_label: Label = $Center/VBox/StatusLabel
@onready var solo_button: Button = $Center/VBox/SoloButton

const ARENA_SCENE := "res://scenes/main.tscn"
const LOOPBACK := "127.0.0.1:6767"

func _ready() -> void:
	if host_edit:
		host_edit.text = LOOPBACK
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
		status_label.text = "Offline Solo Scrap uses loopback 6767. Start the server first, or run tools/solo_scrap.sh."

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
