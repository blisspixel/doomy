extends CanvasLayer
class_name LoadingCard

## The card between deciding to play and playing. It shows the controls and one
## piece of advice from inside the world, of varying quality.
##
## It exists for two reasons. Joining a match used to drop you straight behind a
## gun with no statement of what any key did, which is a hard way to learn a
## control scheme. And a shooter that has a loading screen gets to put something
## in it, which every game worth copying has known since Doom.

signal dismissed

const HOLD_SECONDS: float = 4.0

var _elapsed: float = 0.0
var _armed: bool = false
var _bar: ProgressBar = null

func _ready() -> void:
	layer = 90
	process_mode = Node.PROCESS_MODE_ALWAYS
	_build()

func _build() -> void:
	var back: ColorRect = ColorRect.new()
	back.color = Color(0.04, 0.04, 0.05, 1.0)
	back.anchor_right = 1.0
	back.anchor_bottom = 1.0
	add_child(back)

	var centre: CenterContainer = CenterContainer.new()
	centre.anchor_right = 1.0
	centre.anchor_bottom = 1.0
	add_child(centre)

	var column: VBoxContainer = VBoxContainer.new()
	column.add_theme_constant_override("separation", 18)
	column.custom_minimum_size = Vector2(760.0, 0.0)
	centre.add_child(column)

	var title: Label = Label.new()
	title.text = "fragr"
	title.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	title.add_theme_font_size_override("font_size", 56)
	title.add_theme_color_override("font_color", Color(0.96, 0.90, 0.72))
	column.add_child(title)

	var controls: Label = Label.new()
	controls.text = _controls_text()
	controls.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	controls.add_theme_font_size_override("font_size", 17)
	controls.add_theme_color_override("font_color", Color(0.86, 0.87, 0.88))
	column.add_child(controls)

	var rule: ColorRect = ColorRect.new()
	rule.color = Color(0.82, 0.55, 0.28, 0.5)
	rule.custom_minimum_size = Vector2(0.0, 2.0)
	column.add_child(rule)

	var tip: Label = Label.new()
	tip.text = Tips.random_tip()
	tip.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	tip.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	tip.custom_minimum_size = Vector2(760.0, 0.0)
	tip.add_theme_font_size_override("font_size", 20)
	tip.add_theme_color_override("font_color", Color(0.82, 0.72, 0.5))
	column.add_child(tip)

	_bar = ProgressBar.new()
	_bar.max_value = HOLD_SECONDS
	_bar.value = 0.0
	_bar.show_percentage = false
	_bar.custom_minimum_size = Vector2(0.0, 6.0)
	column.add_child(_bar)

	var hint: Label = Label.new()
	hint.text = "Any key to begin"
	hint.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	hint.add_theme_font_size_override("font_size", 13)
	hint.add_theme_color_override("font_color", Color(0.55, 0.57, 0.6))
	column.add_child(hint)

## The control scheme, in the order a new player needs it.
static func _controls_text() -> String:
	return (
		"W A S D or arrows to move        Arrows or Q and E to turn\n"
		+ "Mouse or Ctrl to fire        Wheel to change weapon\n"
		+ "J to join        L to leave        Escape for the menu        Tilde for the console"
	)

func _process(delta: float) -> void:
	_elapsed += delta
	if _bar != null:
		_bar.value = minf(_elapsed, HOLD_SECONDS)
	# A short arming delay, so the keypress that started the match does not
	# also dismiss the card before anyone has read a word of it.
	if _elapsed > 0.35:
		_armed = true
	if _elapsed >= HOLD_SECONDS:
		dismiss()

func _unhandled_input(event: InputEvent) -> void:
	if not _armed:
		return
	var pressed: bool = (
		(event is InputEventKey and (event as InputEventKey).pressed)
		or (event is InputEventMouseButton and (event as InputEventMouseButton).pressed)
		or (event is InputEventJoypadButton and (event as InputEventJoypadButton).pressed)
	)
	if pressed:
		dismiss()
		get_viewport().set_input_as_handled()

func dismiss() -> void:
	if not visible:
		return
	visible = false
	dismissed.emit()
	queue_free()
