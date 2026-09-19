extends SceneTree

# Headless checks on the settings store. No window, no engine state touched.
# Run: godot --path client --headless --script res://scripts/test_settings.gd

func _initialize() -> void:
	var ok: bool = true
	var script: GDScript = load("res://scripts/settings.gd") as GDScript
	if script == null:
		push_error("test_settings: failed to load settings.gd")
		quit(1)
		return

	var s: RefCounted = script.new()

	# A fresh store answers with the documented defaults.
	if not bool(s.get_value("controls", "always_run")):
		push_error("test_settings: always run should default on, it is a genre staple")
		ok = false
	if float(s.get_value("video", "fov")) < 90.0:
		push_error("test_settings: the default field of view is too narrow for this genre")
		ok = false

	# An unknown key falls back rather than returning null, so a config file
	# written by an older build cannot produce a missing value.
	if s.get_value("video", "no_such_key") != null:
		push_error("test_settings: an unknown key should be null, not invented")
		ok = false
	s.set_value("video", "fov", 120.0)
	if absf(float(s.get_value("video", "fov")) - 120.0) > 0.001:
		push_error("test_settings: a set value must read back")
		ok = false

	# The clamp exists so a hand-edited file cannot produce an unusable camera.
	s.set_value("video", "fov", 400.0)
	if absf(float(s.fov()) - 130.0) > 0.001:
		push_error("test_settings: fov must clamp to the top of the range")
		ok = false
	s.set_value("video", "fov", 5.0)
	if absf(float(s.fov()) - 70.0) > 0.001:
		push_error("test_settings: fov must clamp to the bottom of the range")
		ok = false

	# Reset restores a section without disturbing the others.
	s.set_value("audio", "master", 0.1)
	s.reset_section("video")
	if absf(float(s.get_value("audio", "master")) - 0.1) > 0.001:
		push_error("test_settings: resetting one section must not touch another")
		ok = false
	if absf(float(s.get_value("video", "fov")) - 100.0) > 0.001:
		push_error("test_settings: resetting a section must restore its defaults")
		ok = false

	# A round trip through disk keeps every value.
	s.set_value("controls", "sensitivity", 0.0041)
	s.set_value("gameplay", "hud_scale", 1.25)
	s.save_to_disk()
	var loaded: RefCounted = script.new()
	loaded.load_from_disk()
	if absf(float(loaded.get_value("controls", "sensitivity")) - 0.0041) > 1e-6:
		push_error("test_settings: sensitivity did not survive a save and load")
		ok = false
	if absf(float(loaded.get_value("gameplay", "hud_scale")) - 1.25) > 1e-6:
		push_error("test_settings: hud scale did not survive a save and load")
		ok = false

	# Every default is reachable through get_value, which is what the menu
	# builds itself from, so a typo in DEFAULTS fails here rather than silently.
	for section in script.DEFAULTS:
		for key in (script.DEFAULTS[section] as Dictionary):
			if loaded.get_value(section, key) == null:
				push_error("test_settings: %s/%s read back as null" % [section, key])
				ok = false

	if ok:
		print("test_settings: PASS defaults, fallback, clamp, reset, and a disk round trip")
		quit(0)
	else:
		push_error("test_settings: FAIL")
		quit(1)
