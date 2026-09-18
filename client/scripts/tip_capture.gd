extends SceneTree

# Tip screenshot helper. Run under Xvfb + opengl3 (see tools/capture_tip_screenshots.sh).
# Bare --headless has no framebuffer; do not use it for pixel captures.
# Timed stills: Host bumper, killfeed/scoreboard, compliance pressure, mid-join Host flash.

func _initialize() -> void:
	call_deferred("_run_capture")

func _run_capture() -> void:
	var out_dir: String = OS.get_environment("FRAGR_TIP_CAPTURE_DIR")
	if out_dir.is_empty():
		out_dir = ProjectSettings.globalize_path("res://../docs/screenshots")

	# Always load the arena presenter. Boot menu is the editor main_scene for humans.
	change_scene_to_file("res://scenes/main.tscn")
	await create_timer(0.5).timeout

	# Warm shaders / connect / first frames (avoid pink placeholders).
	await create_timer(2.0).timeout
	await RenderingServer.frame_post_draw

	# Pull spectator cam to a scrap-league overview for the first still.
	_pose_overview_camera()

	# Seconds after scene load (warmup ~2s; compliance ~15s into Active).
	# 12s lands mid-scrap for weapons / frags killfeed still.
	var shot_waits: Array[float] = [4.0, 10.0, 12.0, 18.0]
	var shot_names: Array[String] = [
		"01_arena_overview_16x9.png",
		"02_spectator_hud_16x9.png",
		"09_tip_weapons_frags_16x9.png",
		"07_tip_compliance_pressure_16x9.png",
	]

	var elapsed: float = 2.5
	for i in range(shot_waits.size()):
		var target: float = shot_waits[i]
		var delay: float = maxf(0.0, target - elapsed)
		if delay > 0.0:
			await create_timer(delay).timeout
		elapsed = target
		# After overview still, return to follow cam for combat / HUD shots.
		if i == 1:
			_restore_follow_camera()
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

	# Mid-join Host flash proof: disconnect after Active, reconnect, capture bumper once.
	await _capture_midjoin_host_flash(out_dir)
	await _capture_human_join_fp(out_dir)

	print("tip_capture: done")
	quit(0)

func _capture_midjoin_host_flash(out_dir: String) -> void:
	var gm: Node = _find_game_manager()
	if gm == null:
		push_warning("tip_capture: GameManager missing; skip mid-join Host flash shot")
		return
	var net: Node = gm.get_node_or_null("NetClient")
	var hud: Node = gm.get_node_or_null("HUD")
	if net == null or hud == null:
		push_warning("tip_capture: NetClient/HUD missing; skip mid-join Host flash shot")
		return
	if not hud.has_method("reset_host_chrome") or not hud.has_method("set_host_line"):
		push_warning("tip_capture: HUD missing Host flash API; skip mid-join Host flash shot")
		return

	# Force a clean mid-round reconnect so first Active Snapshot flashes Host once.
	if hud.has_method("reset_host_chrome"):
		hud.reset_host_chrome()
	if net.has_method("disconnect_from_server"):
		net.disconnect_from_server()
	await create_timer(0.4).timeout
	if net.has_method("connect_to_server"):
		net.connect_to_server("spectator", "Spectator")

	# First Snapshot after reconnect should raise RoundMessage Host bumper (~2.5s visible).
	await create_timer(0.7).timeout
	await RenderingServer.frame_post_draw
	await RenderingServer.frame_post_draw

	var shot_name: String = "08_tip_host_flash_midjoin_16x9.png"
	var img: Image = get_root().get_viewport().get_texture().get_image()
	if img == null:
		push_error("tip_capture: viewport image was null for " + shot_name)
		quit(1)
		return
	if _looks_like_pink_placeholder(img):
		push_warning("tip_capture: pink-ish frame for " + shot_name + "; saving anyway for inspection")
	var path: String = out_dir.path_join(shot_name)
	var err: Error = img.save_png(path)
	if err != OK:
		push_error("tip_capture: save_png failed (%s) -> %s" % [str(err), path])
		quit(1)
		return
	print("tip_capture: wrote ", path, " size=", img.get_width(), "x", img.get_height())



func _restore_follow_camera() -> void:
	var gm: Node = _find_game_manager()
	if gm == null:
		return
	var cam_root: Node = gm.get_node_or_null("SpectatorCamera")
	if cam_root == null:
		return
	if "follow_mode" in cam_root:
		cam_root.follow_mode = true

func _pose_overview_camera() -> void:
	var gm: Node = _find_game_manager()
	if gm == null:
		return
	var cam_root: Node = gm.get_node_or_null("SpectatorCamera")
	if cam_root == null:
		return
	# High corner overview so floor grit, scrap props, and billboards read together.
	if cam_root is Node3D:
		var n3: Node3D = cam_root
		n3.global_position = Vector3(0, 22, 28)
		n3.look_at(Vector3(0, 0, 0), Vector3.UP)
		if "follow_mode" in n3:
			n3.follow_mode = false

func _find_game_manager() -> Node:
	var root: Window = get_root()
	for child in root.get_children():
		if child.name == "GameManager":
			return child
		var nested: Node = child.get_node_or_null("GameManager")
		if nested != null:
			return nested
	# Main scene root may itself be the GameManager node.
	if root.get_child_count() > 0:
		var first: Node = root.get_child(0)
		if first.get_node_or_null("NetClient") != null and first.get_node_or_null("HUD") != null:
			return first
	return null

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

func _capture_human_join_fp(out_dir: String) -> void:
	# Join as human and grab one FP scrap-juice still (crosshair + viewmodel).
	var gm: Node = _find_game_manager()
	if gm == null:
		push_warning("tip_capture: GameManager missing; skip human join FP shot")
		return
	var net: Node = gm.get_node_or_null("NetClient")
	var hud: Node = gm.get_node_or_null("HUD")
	if net == null or hud == null:
		push_warning("tip_capture: NetClient/HUD missing; skip human join FP shot")
		return

	if net.has_method("disconnect_from_server"):
		net.disconnect_from_server()
	await create_timer(0.4).timeout

	if "is_human_player" in gm:
		gm.is_human_player = true
	if "fp_spawn_flashed" in gm:
		gm.fp_spawn_flashed = false

	if net.has_method("connect_to_server"):
		net.connect_to_server("human", "Human Player")
	if hud.has_method("set_mode"):
		hud.set_mode("PLAYING")

	# Wait for welcome + first snapshots so FP latch can fire.
	await create_timer(2.5).timeout
	if gm.has_method("_refresh_fp_target"):
		gm._refresh_fp_target()
	await create_timer(0.4).timeout
	await RenderingServer.frame_post_draw
	await RenderingServer.frame_post_draw

	var shot_name: String = "10_tip_human_join_fp_16x9.png"
	var img: Image = get_root().get_viewport().get_texture().get_image()
	if img == null:
		push_error("tip_capture: viewport image was null for " + shot_name)
		return
	var path: String = out_dir.path_join(shot_name)
	var err: Error = img.save_png(path)
	if err != OK:
		push_error("tip_capture: save_png failed (%s) -> %s" % [str(err), path])
		return
	print("tip_capture: wrote ", path, " size=", img.get_width(), "x", img.get_height())
