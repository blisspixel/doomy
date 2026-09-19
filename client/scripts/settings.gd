extends RefCounted
class_name FragrSettings

## Every setting the game has, in one place, saved to user://settings.cfg.
##
## Retro shooter players tune things. The rule here is that anything a player
## would reasonably want to change is a named key with a default, the defaults
## are the ones a first-time player should have, and nothing is hidden in a
## config file with no menu attached to it.

const PATH: String = "user://settings.cfg"

## Section, key, default. The menu is built from this, so adding a setting here
## is the only edit needed to make it appear and persist.
const DEFAULTS: Dictionary = {
	"video": {
		"display_mode": 2,      # 0 windowed, 1 fullscreen, 2 borderless
		"resolution_w": 1920,
		"resolution_h": 1080,
		"fov": 100.0,           # Retro players want this high. Goes to 130.
		"fps_cap": 0,           # 0 is uncapped
		"vsync": false,
		"pixelation": 1.0,      # 1.0 native, 0.25 is a quarter-res retro pass
		"scanlines": false,
		"colour_banding": false,
	},
	"audio": {
		"master": 0.9,
		"music": 0.7,
		"effects": 1.0,
	},
	"controls": {
		"sensitivity": 0.0022,
		"mouse_acceleration": false,
		"invert_y": false,
		"always_run": true,
		"turn_speed": 2.8,      # Radians per second for keyboard turning
		"auto_aim": true,       # Vertical assist, so keyboard-only can play
	},
	"gameplay": {
		"crosshair_style": 1,   # 0 dot, 1 cross, 2 dynamic
		"crosshair_scale": 1.0,
		"head_bob": true,
		"hud_scale": 1.0,
		"damage_numbers": true,
	},
}

var _values: Dictionary = {}

func _init() -> void:
	_values = _copy_defaults()

static func _copy_defaults() -> Dictionary:
	var out: Dictionary = {}
	for section in DEFAULTS:
		out[section] = (DEFAULTS[section] as Dictionary).duplicate(true)
	return out

## Read a setting, falling back to the default rather than to null, so a config
## file written by an older build can never produce a missing value.
func get_value(section: String, key: String) -> Variant:
	if _values.has(section) and (_values[section] as Dictionary).has(key):
		return (_values[section] as Dictionary)[key]
	if DEFAULTS.has(section) and (DEFAULTS[section] as Dictionary).has(key):
		return (DEFAULTS[section] as Dictionary)[key]
	return null

func set_value(section: String, key: String, value: Variant) -> void:
	if not _values.has(section):
		_values[section] = {}
	(_values[section] as Dictionary)[key] = value

func reset_section(section: String) -> void:
	if DEFAULTS.has(section):
		_values[section] = (DEFAULTS[section] as Dictionary).duplicate(true)

func reset_all() -> void:
	_values = _copy_defaults()

func load_from_disk() -> void:
	var cfg: ConfigFile = ConfigFile.new()
	if cfg.load(PATH) != OK:
		return
	for section in DEFAULTS:
		for key in (DEFAULTS[section] as Dictionary):
			var fallback: Variant = (DEFAULTS[section] as Dictionary)[key]
			set_value(section, key, cfg.get_value(section, key, fallback))

func save_to_disk() -> void:
	var cfg: ConfigFile = ConfigFile.new()
	for section in _values:
		for key in (_values[section] as Dictionary):
			cfg.set_value(section, key, (_values[section] as Dictionary)[key])
	cfg.save(PATH)

## Push everything at the engine. Safe to call repeatedly.
func apply() -> void:
	apply_video()
	apply_audio()

func apply_video() -> void:
	var mode: int = int(get_value("video", "display_mode"))
	match mode:
		0:
			DisplayServer.window_set_mode(DisplayServer.WINDOW_MODE_WINDOWED)
			DisplayServer.window_set_flag(DisplayServer.WINDOW_FLAG_BORDERLESS, false)
			DisplayServer.window_set_size(
				Vector2i(int(get_value("video", "resolution_w")), int(get_value("video", "resolution_h")))
			)
		1:
			DisplayServer.window_set_flag(DisplayServer.WINDOW_FLAG_BORDERLESS, false)
			DisplayServer.window_set_mode(DisplayServer.WINDOW_MODE_EXCLUSIVE_FULLSCREEN)
		_:
			DisplayServer.window_set_mode(DisplayServer.WINDOW_MODE_FULLSCREEN)

	DisplayServer.window_set_vsync_mode(
		DisplayServer.VSYNC_ENABLED if bool(get_value("video", "vsync")) else DisplayServer.VSYNC_DISABLED
	)
	Engine.max_fps = maxi(0, int(get_value("video", "fps_cap")))

func apply_audio() -> void:
	_set_bus("Master", float(get_value("audio", "master")))
	_set_bus("Music", float(get_value("audio", "music")))
	_set_bus("Effects", float(get_value("audio", "effects")))

static func _set_bus(name: String, linear: float) -> void:
	var index: int = AudioServer.get_bus_index(name)
	if index < 0:
		# Only Master is guaranteed. A missing bus is not an error until the
		# mix work lands and creates them.
		if name != "Master":
			return
		index = 0
	AudioServer.set_bus_volume_db(index, linear_to_db(clampf(linear, 0.0001, 1.0)))

## Vertical field of view in degrees, clamped to what the menu offers.
func fov() -> float:
	return clampf(float(get_value("video", "fov")), 70.0, 130.0)
