extends SceneTree

# Tip screenshot helper. Run under Xvfb + opengl3 (see tools/capture_tip_screenshots.sh).
# Bare --headless has no framebuffer; do not use it for pixel captures.

func _initialize() -> void:
	call_deferred("_run_capture")

func _run_capture() -> void:
	var wait_secs := float(OS.get_environment("FRAGR_TIP_CAPTURE_WAIT"))
	if wait_secs <= 0.0:
		wait_secs = 8.0
	var out_dir := OS.get_environment("FRAGR_TIP_CAPTURE_DIR")
	if out_dir.is_empty():
		out_dir = "res://../docs/screenshots"

	# Prefer opening the main scene so the thin client can connect and present.
	var main := ProjectSettings.get_setting("application/run/main_scene")
	if typeof(main) == TYPE_STRING and String(main).length() > 0:
		change_scene_to_file(String(main))
		await create_timer(0.5).timeout

	await create_timer(wait_secs).timeout
	await RenderingServer.frame_post_draw

	var img: Image = get_root().get_viewport().get_texture().get_image()
	if img == null:
		push_error("tip_capture: viewport image was null")
		quit(1)
		return

	var path := out_dir.path_join("01_arena_overview_16x9.png")
	var err := img.save_png(path)
	if err != OK:
		push_error("tip_capture: save_png failed (%s) -> %s" % [str(err), path])
		quit(1)
		return

	print("tip_capture: wrote ", path)
	quit(0)
