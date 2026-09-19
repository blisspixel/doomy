extends CanvasLayer
class_name FragrConsole

## A drop-down console on the tilde key, in the tradition of Quake, Half-Life
## and Counter-Strike before 1.6. It is a player convenience and a development
## tool at the same time: the three control bugs found on 2026-09-19 would all
## have been visible from inside the game with a `yaw` readout and an `fps`
## line, instead of needing a headless harness to prove.
##
## It holds its own state and talks to the game through methods it looks up by
## name, so it never hard-depends on the game manager and can be opened on the
## boot menu where there is no match at all.

signal command_run(line: String)

const MAX_LINES: int = 400
const HISTORY_MAX: int = 64
const OPEN_HEIGHT_RATIO: float = 0.45

var _open: bool = false
var _history: Array[String] = []
var _history_index: int = -1
var _panel: Panel = null
var _output: RichTextLabel = null
var _input: LineEdit = null
var _lines: Array[String] = []

func _ready() -> void:
	layer = 128
	process_mode = Node.PROCESS_MODE_ALWAYS
	_build()
	visible = false
	echo("fragr console. Type help for commands, tilde to close.")

func _build() -> void:
	_panel = Panel.new()
	_panel.anchor_right = 1.0
	_panel.anchor_bottom = OPEN_HEIGHT_RATIO
	_panel.mouse_filter = Control.MOUSE_FILTER_STOP
	add_child(_panel)

	var style: StyleBoxFlat = StyleBoxFlat.new()
	style.bg_color = Color(0.04, 0.04, 0.05, 0.94)
	style.border_color = Color(0.82, 0.55, 0.28, 0.9)
	style.border_width_bottom = 2
	_panel.add_theme_stylebox_override("panel", style)

	_output = RichTextLabel.new()
	_output.anchor_right = 1.0
	_output.anchor_bottom = 1.0
	_output.offset_left = 12.0
	_output.offset_top = 8.0
	_output.offset_right = -12.0
	_output.offset_bottom = -40.0
	_output.scroll_following = true
	_output.bbcode_enabled = false
	_output.add_theme_font_size_override("normal_font_size", 15)
	_panel.add_child(_output)

	_input = LineEdit.new()
	_input.anchor_top = 1.0
	_input.anchor_right = 1.0
	_input.anchor_bottom = 1.0
	_input.offset_left = 12.0
	_input.offset_top = -34.0
	_input.offset_right = -12.0
	_input.offset_bottom = -6.0
	_input.placeholder_text = "command"
	_input.caret_blink = true
	_input.text_submitted.connect(_on_submit)
	_panel.add_child(_input)

## Print a line. Safe before _ready has built the panel.
func echo(line: String) -> void:
	_lines.append(line)
	while _lines.size() > MAX_LINES:
		_lines.pop_front()
	if _output != null:
		_output.text = "\n".join(_lines)

func is_open() -> bool:
	return _open

func toggle() -> void:
	set_open(not _open)

func set_open(open: bool) -> void:
	_open = open
	visible = open
	if _input == null:
		return
	if open:
		_input.grab_focus()
		_input.clear()
	else:
		_input.release_focus()

func _input_event_is_toggle(event: InputEvent) -> bool:
	if not (event is InputEventKey):
		return false
	var key: InputEventKey = event
	if not key.pressed or key.echo:
		return false
	# Quoteleft is the tilde key on every layout that has one.
	return key.physical_keycode == KEY_QUOTELEFT

func _unhandled_input(event: InputEvent) -> void:
	if _input_event_is_toggle(event):
		toggle()
		get_viewport().set_input_as_handled()
		return
	if not _open:
		return
	if event is InputEventKey and event.pressed:
		var key: InputEventKey = event
		if key.physical_keycode == KEY_ESCAPE:
			set_open(false)
			get_viewport().set_input_as_handled()
		elif key.physical_keycode == KEY_UP:
			_recall(-1)
			get_viewport().set_input_as_handled()
		elif key.physical_keycode == KEY_DOWN:
			_recall(1)
			get_viewport().set_input_as_handled()

func _recall(direction: int) -> void:
	if _history.is_empty() or _input == null:
		return
	if _history_index < 0:
		_history_index = _history.size()
	_history_index = clampi(_history_index + direction, 0, _history.size() - 1)
	_input.text = _history[_history_index]
	_input.caret_column = _input.text.length()

func _on_submit(line: String) -> void:
	if _input != null:
		_input.clear()
	var trimmed: String = line.strip_edges()
	if trimmed.is_empty():
		return
	_history.append(trimmed)
	while _history.size() > HISTORY_MAX:
		_history.pop_front()
	_history_index = -1
	echo("> " + trimmed)
	run(trimmed)

## Split a command line into a verb and its arguments. Pure, so it is tested.
static func parse(line: String) -> Array[String]:
	var out: Array[String] = []
	for part in line.strip_edges().split(" ", false):
		var token: String = part.strip_edges()
		if not token.is_empty():
			out.append(token)
	return out

func run(line: String) -> void:
	var argv: Array[String] = parse(line)
	if argv.is_empty():
		return
	var verb: String = argv[0].to_lower()
	var rest: Array[String] = argv.slice(1)
	match verb:
		"help":
			_help()
		"clear":
			_lines.clear()
			if _output != null:
				_output.text = ""
		"quit", "exit":
			get_tree().quit()
		"fullscreen":
			_set_fullscreen(rest.is_empty() or rest[0] != "0")
		"windowed":
			_set_fullscreen(false)
		"vsync":
			var on: bool = rest.is_empty() or rest[0] != "0"
			DisplayServer.window_set_vsync_mode(
				DisplayServer.VSYNC_ENABLED if on else DisplayServer.VSYNC_DISABLED
			)
			echo("vsync " + ("on" if on else "off"))
		"fps":
			echo("%d fps, %.2f ms" % [
				Engine.get_frames_per_second(),
				1000.0 / maxf(1.0, float(Engine.get_frames_per_second())),
			])
		"pos":
			_report_pos()
		"yaw":
			_report_yaw()
		"sens":
			_set_sens(rest)
		"controls", "bind", "keys":
			_report_controls()
		"tip":
			echo(Tips.random_tip())
		"players":
			_report_players()
		"version":
			echo("godot " + str(Engine.get_version_info().get("string", "")))
		_:
			command_run.emit(line)
			echo("unknown command: " + verb + " (try help)")

func _help() -> void:
	echo("help            this")
	echo("clear           wipe the scrollback")
	echo("fps             frames per second and frame time")
	echo("pos             where you are standing")
	echo("yaw             your facing, in the server's convention and Godot's")
	echo("sens [value]    look sensitivity")
	echo("fullscreen 0|1  window mode")
	echo("vsync 0|1       vertical sync")
	echo("players         who is in the match")
	echo("controls        what every key does")
	echo("tip             another piece of questionable advice")
	echo("version         engine version")
	echo("quit            leave")

func _report_controls() -> void:
	echo("move            W A S D, or the arrow keys")
	echo("turn            Left and Right arrows, or Q and E, or the mouse")
	echo("jump            Space")
	echo("fire            Left mouse button, or Ctrl")
	echo("weapon          Mouse wheel, or the bracket keys")
	echo("join / leave    J and L")
	echo("menu            Escape")
	echo("radio           R station, N track, M mute")
	echo("There is no reload key yet. Reloading is specified and not built.")

func _game() -> Node:
	for child in get_tree().get_root().get_children():
		if child.name == "GameManager":
			return child
	return null

func _camera() -> Node:
	var game: Node = _game()
	return game.get_node_or_null("SpectatorCamera") if game != null else null

func _report_pos() -> void:
	var cam: Node = _camera()
	if cam == null or not (cam is Node3D):
		echo("no camera")
		return
	var n3: Node3D = cam
	echo("camera %.2f %.2f %.2f" % [n3.global_position.x, n3.global_position.y, n3.global_position.z])

func _report_yaw() -> void:
	var cam: Node = _camera()
	if cam == null:
		echo("no camera")
		return
	var server_yaw: float = 0.0
	if cam.has_method("consume_yaw"):
		server_yaw = float(cam.call("consume_yaw"))
	var godot_yaw: float = float(cam.get("rotation").y) if cam is Node3D else 0.0
	echo("server yaw %.3f rad (%.1f deg), node rotation.y %.3f rad" % [
		server_yaw, rad_to_deg(server_yaw), godot_yaw,
	])
	echo("server forward " + str(ServerYaw.forward(server_yaw)))

func _set_sens(rest: Array[String]) -> void:
	var cam: Node = _camera()
	if cam == null:
		echo("no camera")
		return
	if rest.is_empty():
		echo("sens " + str(cam.get("mouse_sensitivity")))
		return
	var value: float = rest[0].to_float()
	if value <= 0.0:
		echo("sens must be greater than zero")
		return
	cam.set("mouse_sensitivity", value)
	echo("sens " + str(value))

func _report_players() -> void:
	var game: Node = _game()
	if game == null:
		echo("no match")
		return
	var players: Variant = game.get("players")
	if typeof(players) != TYPE_DICTIONARY:
		echo("no match")
		return
	echo("%d in the match" % (players as Dictionary).size())
	for pid in (players as Dictionary):
		var pawn: Variant = (players as Dictionary)[pid]
		if pawn is Node and is_instance_valid(pawn):
			echo("  " + str((pawn as Node).name))

func _set_fullscreen(on: bool) -> void:
	DisplayServer.window_set_mode(
		DisplayServer.WINDOW_MODE_FULLSCREEN if on else DisplayServer.WINDOW_MODE_WINDOWED
	)
	echo("fullscreen " + ("on" if on else "off"))
