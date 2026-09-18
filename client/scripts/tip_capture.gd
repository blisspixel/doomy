extends SceneTree

# Tip screenshot helper. Run under Xvfb + opengl3 (see tools/capture_tip_screenshots.sh).
# Bare --headless has no framebuffer; do not use it for pixel captures.
# Timed stills: Host bumper, killfeed/scoreboard, compliance pressure.

func _initialize() -> void:
	call_deferred("_run_capture")

func _run_capture() -> void:
	var out_dir: String = OS.get_environment("FRAGR_TIP_CAPTURE_DIR")
	if out_dir.is_empty():
		out_dir = ProjectSettings.globalize_path("res://../docs/screenshots")

	var main_setting: Variant = ProjectSettings.get_setting("application/run/main_scene")
	if typeof(main_setting) == TYPE_STRING and String(main_setting).length() > 0:
		change_scene_to_file(String(main_setting))
		await create_timer(0.5).timeout

	# Warm shaders / connect / first frames (avoid pink placeholders).
	await create_timer(2.0).timeout
	await RenderingServer.frame_post_draw

	# Seconds after scene load (warmup ~2s; compliance ~15s into Active).
	var shot_waits: Array[float] = [4.0, 10.0, 18.0]
	var shot_names: Array[String] = [
		"01_arena_overview_16x9.png",
		"02_spectator_hud_16x9.png",
		"07_tip_compliance_pressure_16x9.png",
	]

	var elapsed: float = 2.5
	for i in range(shot_waits.size()):
		var target: float = shot_waits[i]
		var delay: float = maxf(0.0, target - elapsed)
		if delay > 0.0:
			await create_timer(delay).timeout
		elapsed = target
		await RenderingServer.frame_post_draw
		await RenderingServer.frame_post_draw

		var img: Image = get_root().get_viewport().get_texture().get_image()
		if img == null:
			push_error("tip_capture: viewport image was null for " + shot_names[i])
			quit(1)
			return

		if _looks_like_pink_placeholder(img):
			push_warning("tip_capture: pink-ish frame for " + shot_names[i] + "; saving anyway for inspection")

		var path: String = out_dir.path_join(shot_names[i])
		var err: Error = img.save_png(path)
		if err != OK:
			push_error("tip_capture: save_png failed (%s) -> %s" % [str(err), path])
			quit(1)
			return
		print("tip_capture: wrote ", path, " size=", img.get_width(), "x", img.get_height())

	print("tip_capture: done")
	quit(0)

func _looks_like_pink_placeholder(img: Image) -> bool:
	var w: int = img.get_width()
	var h: int = img.get_height()
	if w < 4 or h < 4:
		return true
	var pinkish: int = 0
	var samples: Array[Vector2i] = [
		Vector2i(w / 2, h / 2),
		Vector2i(w / 4, h / 4),
		Vector2i(3 * w / 4, 3 * h / 4),
		Vector2i(w / 2, h / 4),
		Vector2i(w / 4, h / 2),
	]
	for p in samples:
		var c: Color = img.get_pixel(p.x, p.y)
		if c.r > 0.85 and c.b > 0.85 and c.g < 0.25:
			pinkish += 1
	return pinkish >= 3
