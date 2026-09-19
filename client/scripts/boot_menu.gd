extends Control

## The front menu, in the shape every shooter has used since Doom: single
## player, multiplayer, settings, quit. It builds itself in code rather than
## living in a scene file, because a menu with submenus is a state machine and
## a state machine is easier to read as code than as a node tree.
##
## It is fully keyboard navigable. Arrow keys move, Enter chooses, Escape goes
## back. A laptop with no mouse is a first-class way to play this.

const ARENA_SCENE: String = "res://scenes/main.tscn"
const LOOPBACK: String = "127.0.0.1:6767"

const MAP_NAMES: Dictionary = {1: "Larak Lot", 2: "Compliance Yard"}

var _page: String = "main"
var _root: VBoxContainer = null
var _status: Label = null
var _host_edit: LineEdit = null
var _map_id: int = 1
var _console: FragrConsole = null

func _ready() -> void:
	_map_id = _preferred_map_id()
	_build_chrome()
	_show("main")
	_console = FragrConsole.new()
	_console.name = "FragrConsole"
	add_child(_console)

func _build_chrome() -> void:
	var back: ColorRect = ColorRect.new()
	back.color = Color(0.05, 0.05, 0.06, 1.0)
	back.anchor_right = 1.0
	back.anchor_bottom = 1.0
	add_child(back)

	var centre: CenterContainer = CenterContainer.new()
	centre.anchor_right = 1.0
	centre.anchor_bottom = 1.0
	add_child(centre)

	var column: VBoxContainer = VBoxContainer.new()
	column.add_theme_constant_override("separation", 10)
	column.custom_minimum_size = Vector2(460.0, 0.0)
	centre.add_child(column)

	var title: Label = Label.new()
	title.text = "fragr"
	title.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	title.add_theme_font_size_override("font_size", 72)
	title.add_theme_color_override("font_color", Color(0.96, 0.90, 0.72))
	column.add_child(title)

	_root = VBoxContainer.new()
	_root.add_theme_constant_override("separation", 8)
	column.add_child(_root)

	_status = Label.new()
	_status.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	_status.add_theme_font_size_override("font_size", 14)
	_status.add_theme_color_override("font_color", Color(0.6, 0.62, 0.64))
	_status.text = "Arrow keys and Enter. Tilde opens the console."
	column.add_child(_status)

func _clear() -> void:
	for child in _root.get_children():
		child.queue_free()

func _button(text: String, handler: Callable) -> Button:
	var b: Button = Button.new()
	b.text = text
	b.custom_minimum_size = Vector2(0.0, 44.0)
	b.pressed.connect(handler)
	_root.add_child(b)
	return b

func _label(text: String) -> void:
	var l: Label = Label.new()
	l.text = text
	l.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	l.add_theme_color_override("font_color", Color(0.55, 0.7, 0.72))
	_root.add_child(l)

func _show(page: String) -> void:
	_page = page
	_clear()
	match page:
		"main":
			_page_main()
		"single":
			_page_single()
		"multi":
			_page_multi()
		"settings":
			_page_settings()
	await get_tree().process_frame
	for child in _root.get_children():
		if child is Button and not (child as Button).disabled:
			(child as Button).grab_focus()
			break

func _page_main() -> void:
	_button("Single Player", func() -> void: _show("single"))
	_button("Multiplayer", func() -> void: _show("multi"))
	_button("Settings", func() -> void: _show("settings"))
	_button("Quit", func() -> void: get_tree().quit())

func _page_single() -> void:
	_label("Campaign")
	_button("Episode 0: Calibration", func() -> void: _launch("solo", LOOPBACK))
	var later: Button = _button("Episode 1: Larak Lot", func() -> void: pass)
	later.disabled = true
	later.tooltip_text = "Not built yet"
	_label("Practice")
	_button("Arena against bots", func() -> void: _launch("join", LOOPBACK))
	_button("Watch the bots", func() -> void: _launch("spectate", LOOPBACK))
	_map_row()
	_button("Back", func() -> void: _show("main"))

func _page_multi() -> void:
	_label("A server is a program you run. Anyone can host one.")
	_button("Host a server on this machine", func() -> void: _launch("join", LOOPBACK))
	_label("Join by address")
	_host_edit = LineEdit.new()
	_host_edit.text = LOOPBACK
	_host_edit.custom_minimum_size = Vector2(0.0, 36.0)
	_root.add_child(_host_edit)
	_button("Connect", func() -> void: _launch("join", _host_address()))
	_button("Connect as spectator", func() -> void: _launch("spectate", _host_address()))
	var browser: Button = _button("Server list", func() -> void: pass)
	browser.disabled = true
	browser.tooltip_text = "Not built yet. Run your own and share the address."
	_map_row()
	_button("Back", func() -> void: _show("main"))

func _page_settings() -> void:
	_label("Display")
	var full: CheckButton = CheckButton.new()
	full.text = "Fullscreen"
	full.button_pressed = DisplayServer.window_get_mode() != DisplayServer.WINDOW_MODE_WINDOWED
	full.toggled.connect(func(on: bool) -> void:
		DisplayServer.window_set_mode(
			DisplayServer.WINDOW_MODE_FULLSCREEN if on else DisplayServer.WINDOW_MODE_WINDOWED
		)
	)
	_root.add_child(full)

	var vsync: CheckButton = CheckButton.new()
	vsync.text = "Vertical sync"
	vsync.button_pressed = DisplayServer.window_get_vsync_mode() != DisplayServer.VSYNC_DISABLED
	vsync.toggled.connect(func(on: bool) -> void:
		DisplayServer.window_set_vsync_mode(
			DisplayServer.VSYNC_ENABLED if on else DisplayServer.VSYNC_DISABLED
		)
	)
	_root.add_child(vsync)

	_label("Audio")
	var volume: HSlider = HSlider.new()
	volume.min_value = 0.0
	volume.max_value = 1.0
	volume.step = 0.05
	volume.value = db_to_linear(AudioServer.get_bus_volume_db(0))
	volume.custom_minimum_size = Vector2(0.0, 28.0)
	volume.value_changed.connect(func(v: float) -> void:
		AudioServer.set_bus_volume_db(0, linear_to_db(maxf(v, 0.0001)))
	)
	_root.add_child(volume)

	_label("Controls: WASD or arrows to move, arrows or Q and E to turn,")
	_label("mouse or Ctrl to fire, wheel to change weapon, J to join, L to leave.")
	_button("Back", func() -> void: _show("main"))

func _map_row() -> void:
	var row: HBoxContainer = HBoxContainer.new()
	var label: Label = Label.new()
	label.text = "Map  "
	row.add_child(label)
	var option: OptionButton = OptionButton.new()
	for id in MAP_NAMES:
		option.add_item(str(MAP_NAMES[id]), int(id))
	option.select(option.get_item_index(_map_id))
	option.item_selected.connect(func(index: int) -> void: _map_id = option.get_item_id(index))
	row.add_child(option)
	_root.add_child(row)

func _host_address() -> String:
	if _host_edit != null and not _host_edit.text.strip_edges().is_empty():
		return _host_edit.text.strip_edges()
	return LOOPBACK

func _preferred_map_id() -> int:
	var raw: String = OS.get_environment("FRAGR_MAP")
	if raw.is_valid_int():
		var id: int = raw.to_int()
		if MAP_NAMES.has(id):
			return id
	return 1

func _unhandled_input(event: InputEvent) -> void:
	if _console != null and _console.is_open():
		return
	if event is InputEventKey and event.pressed and not (event as InputEventKey).echo:
		if (event as InputEventKey).physical_keycode == KEY_ESCAPE and _page != "main":
			_show("main")
			get_viewport().set_input_as_handled()

func _launch(mode: String, host: String) -> void:
	var boot: Dictionary = {
		"mode": mode,
		"host": host,
		"map_id": _map_id,
	}
	get_tree().set_meta("fragr_boot", boot)
	var err: Error = get_tree().change_scene_to_file(ARENA_SCENE)
	if err != OK:
		push_error("boot_menu: failed to load arena scene: " + str(err))
		if _status != null:
			_status.text = "Failed to load arena (" + str(err) + ")"
