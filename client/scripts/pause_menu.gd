extends CanvasLayer
class_name PauseMenu

## Escape opens this. In single player it pauses the match; in multiplayer it
## does not, because the other fighters did not agree to stop.
##
## That distinction is honest rather than cosmetic. The simulation lives in the
## server process, so freezing the client would freeze what you see while the
## world carried on without you, and you would come back dead. In a solo match
## the client asks the server to hold, and the server obliges because it is
## nobody else's match. In multiplayer the menu says so out loud.

signal resume_requested
signal leave_requested
signal pause_state_changed(paused: bool)

var solo: bool = false

var _open: bool = false
var _panel: Panel = null
var _column: VBoxContainer = null
var _note: Label = null

func _ready() -> void:
	layer = 100
	process_mode = Node.PROCESS_MODE_ALWAYS
	_build()
	visible = false

func _build() -> void:
	var veil: ColorRect = ColorRect.new()
	veil.color = Color(0.02, 0.02, 0.03, 0.75)
	veil.anchor_right = 1.0
	veil.anchor_bottom = 1.0
	veil.mouse_filter = Control.MOUSE_FILTER_STOP
	add_child(veil)

	var centre: CenterContainer = CenterContainer.new()
	centre.anchor_right = 1.0
	centre.anchor_bottom = 1.0
	add_child(centre)

	_panel = Panel.new()
	_panel.custom_minimum_size = Vector2(420.0, 0.0)
	centre.add_child(_panel)

	_column = VBoxContainer.new()
	_column.add_theme_constant_override("separation", 10)
	_column.anchor_right = 1.0
	_column.offset_left = 24.0
	_column.offset_top = 24.0
	_column.offset_right = -24.0
	_panel.add_child(_column)

	var title: Label = Label.new()
	title.text = "Paused"
	title.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	title.add_theme_font_size_override("font_size", 40)
	_column.add_child(title)

	_note = Label.new()
	_note.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	_note.add_theme_font_size_override("font_size", 14)
	_note.add_theme_color_override("font_color", Color(0.82, 0.55, 0.28))
	_column.add_child(_note)

	_add_button("Resume", func() -> void: close())
	_add_button("Leave match", func() -> void:
		close()
		leave_requested.emit()
	)
	_add_button("Quit to desktop", func() -> void: get_tree().quit())

	var hint: Label = Label.new()
	hint.text = "Escape resumes. Tilde opens the console."
	hint.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	hint.add_theme_font_size_override("font_size", 12)
	hint.add_theme_color_override("font_color", Color(0.55, 0.57, 0.6))
	_column.add_child(hint)

	var spacer: Control = Control.new()
	spacer.custom_minimum_size = Vector2(0.0, 16.0)
	_column.add_child(spacer)

func _add_button(text: String, handler: Callable) -> void:
	var b: Button = Button.new()
	b.text = text
	b.custom_minimum_size = Vector2(0.0, 42.0)
	b.pressed.connect(handler)
	_column.add_child(b)

func is_open() -> bool:
	return _open

func open() -> void:
	if _open:
		return
	_open = true
	visible = true
	_note.text = (
		"The match is held."
		if solo
		else "The match keeps running. Nobody else agreed to stop."
	)
	if solo:
		get_tree().paused = true
	pause_state_changed.emit(solo)
	Input.set_mouse_mode(Input.MOUSE_MODE_VISIBLE)
	await get_tree().process_frame
	for child in _column.get_children():
		if child is Button:
			(child as Button).grab_focus()
			break

func close() -> void:
	if not _open:
		return
	_open = false
	visible = false
	if get_tree().paused:
		get_tree().paused = false
	pause_state_changed.emit(false)
	Input.set_mouse_mode(Input.MOUSE_MODE_CAPTURED)
	resume_requested.emit()

func toggle() -> void:
	if _open:
		close()
	else:
		open()
