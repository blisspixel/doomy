extends SceneTree

# Tip screenshot helper. Run under Xvfb + opengl3 (see tools/capture_tip_screenshots.sh).
# Bare --headless has no framebuffer; do not use it for pixel captures.
# Timed stills: Calibration/Larak Lot chrome, Host bumper, killfeed/scoreboard, mid-join Host flash.

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

	# Unmissable Warmup TV bumper still (forced chrome so capture always lands).
	await _capture_warmup_tv_bumper(out_dir)

	# Solo Broadcast Episode 0 title + objective chip (Calibration / Larak Lot).
	await _capture_calibration_larak(out_dir)

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

	# Jammer dish proof: force live dish + aimed follow / overview (Warmup-TV pattern).
	# Casino #116 stills 20-22 were empty because FP never looked at origin.
	await _capture_jammer_dish_proof(out_dir)

	# Mid-join Host flash proof: disconnect after Active, reconnect, capture bumper once.
	await _capture_midjoin_host_flash(out_dir)
	await _capture_human_join_fp(out_dir)

	print("tip_capture: done")
	quit(0)

func _capture_warmup_tv_bumper(out_dir: String) -> void:
	# Full-frame Contested Frequency Warmup TV: Larak Lot face, roster chips, GOES LIVE IN N.
	var gm: Node = _find_game_manager()
	if gm == null:
		push_warning("tip_capture: GameManager missing; skip Warmup TV bumper shot")
		return
	var hud: Node = gm.get_node_or_null("HUD")
	if hud == null:
		push_warning("tip_capture: HUD missing; skip Warmup TV bumper shot")
		return
	if not hud.has_method("show_warmup_bumper"):
		push_warning("tip_capture: HUD missing show_warmup_bumper; skip Warmup TV bumper shot")
		return

	if hud.has_method("set_map_name"):
		hud.set_map_name("Larak Lot")
	if hud.has_method("set_league_identity"):
		hud.set_league_identity("Contested Frequency", "Larak Lot")
	if hud.has_method("set_episode_chrome"):
		hud.set_episode_chrome(
			"ep0",
			"Solo Broadcast: Calibration",
			"Clear NODS. Seize jammer dish. Drop the Auditor.",
			"",
			"nods"
		)

	var roster: Array = [
		"NODS-1",
		"NODS-2",
		"NODS-3",
		"NODS-4",
	]
	var host_line: String = "HOST: CONTESTED FREQUENCY. LARAK LOT TUNES IN. NODS-1, NODS-2, +2 ON THE SCRAP. GOES LIVE IN 2."
	hud.show_warmup_bumper(host_line, 2, roster)
	await create_timer(0.25).timeout
	await RenderingServer.frame_post_draw
	await RenderingServer.frame_post_draw

	var shot_name: String = "11_tip_warmup_tv_bumper_16x9.png"
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

	if hud.has_method("hide_warmup_tv"):
		hud.hide_warmup_tv()
	elif "round_message" in hud and hud.round_message:
		hud.round_message.visible = false



func _capture_calibration_larak(out_dir: String) -> void:
	# Solo Broadcast Episode 0 face: title card + objective + Larak Lot line.
	var gm: Node = _find_game_manager()
	if gm == null:
		push_warning("tip_capture: GameManager missing; skip Calibration still")
		return
	var hud: Node = gm.get_node_or_null("HUD")
	if hud == null:
		push_warning("tip_capture: HUD missing; skip Calibration still")
		return

	if hud.has_method("set_map_name"):
		hud.set_map_name("Larak Lot")
	if hud.has_method("set_league_identity"):
		hud.set_league_identity("Contested Frequency", "Larak Lot")
	if "episode_title_shown" in hud:
		hud.episode_title_shown = false
	if hud.has_method("set_episode_chrome"):
		hud.set_episode_chrome(
			"ep0",
			"Solo Broadcast: Calibration",
			"Clear NODS. Seize jammer dish. Drop the Auditor.",
			"NODS 0/4",
			"nods"
		)
	elif hud.has_method("show_episode_title_card"):
		hud.show_episode_title_card(
			"Solo Broadcast: Calibration",
			"Clear NODS. Seize jammer dish. Drop the Auditor."
		)

	await create_timer(0.35).timeout
	await RenderingServer.frame_post_draw
	await RenderingServer.frame_post_draw

	var shot_name: String = "12_tip_calibration_larak_16x9.png"
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

	if hud.has_method("hide_warmup_tv"):
		hud.hide_warmup_tv()
	elif "round_message" in hud and hud.round_message:
		hud.round_message.visible = false


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




func _force_live_jammer_dish(gm: Node) -> void:
	# Latch so nods-phase Snapshot nulls cannot hide the forced live dish mid-capture.
	if "tip_force_jammer_dish" in gm:
		gm.tip_force_jammer_dish = true
	var hud: Node = gm.get_node_or_null("HUD")
	if hud != null and hud.has_method("set_map_name"):
		hud.set_map_name("Larak Lot")
	if not gm.has_method("_sync_jammer_dish"):
		return
	var dish := {
		"live": true,
		"seized": false,
		"x": 0.0,
		"y": 0.35,
		"z": 0.0,
	}
	gm.call("_sync_jammer_dish", dish)


func _capture_jammer_dish_proof(out_dir: String) -> void:
	# Live hangar tip face: force dish + aim so stills include the bowl under
	# racks / killfeed / HUD (studio black-void proof alone is not tip face).
	var gm: Node = _find_game_manager()
	if gm == null:
		push_warning("tip_capture: GameManager missing; skip jammer dish proof")
		return
	if not gm.has_method("_sync_jammer_dish"):
		push_warning("tip_capture: GameManager missing _sync_jammer_dish; skip jammer dish proof")
		return

	_force_live_jammer_dish(gm)
	# Let one Snapshot cycle land; latch must keep the dish visible.
	await create_timer(0.45).timeout
	_force_live_jammer_dish(gm)

	# Follow-distance still (~12m), looking straight at the dish in the hangar.
	_pose_jammer_follow_camera()
	await create_timer(0.35).timeout
	_force_live_jammer_dish(gm)
	await RenderingServer.frame_post_draw
	await RenderingServer.frame_post_draw
	_save_viewport_png(out_dir, "20_jammer_dish_follow_16x9.png")

	# Overview still (~36m corner), dish centered in frame.
	_pose_overview_camera()
	await create_timer(0.25).timeout
	_force_live_jammer_dish(gm)
	await RenderingServer.frame_post_draw
	await RenderingServer.frame_post_draw
	_save_viewport_png(out_dir, "22_jammer_dish_overview_16x9.png")

	# Closer label-read still so SEIZE JAMMER is unmistakable in hangar light.
	_pose_jammer_label_camera()
	await create_timer(0.25).timeout
	_force_live_jammer_dish(gm)
	await RenderingServer.frame_post_draw
	await RenderingServer.frame_post_draw
	_save_viewport_png(out_dir, "23_jammer_dish_seize_label_16x9.png")

	_restore_follow_camera()


func _save_viewport_png(out_dir: String, shot_name: String) -> void:
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


func _pose_jammer_follow_camera() -> void:
	var gm: Node = _find_game_manager()
	if gm == null:
		return
	var cam_root: Node = gm.get_node_or_null("SpectatorCamera")
	if cam_root == null:
		return
	if cam_root is Node3D:
		var n3: Node3D = cam_root
		# ~12m follow: eye-height, looking at dish origin.
		n3.global_position = Vector3(0.0, 3.2, 12.0)
		n3.look_at(Vector3(0.0, 3.5, 0.0), Vector3.UP)
		if "follow_mode" in n3:
			n3.follow_mode = false


func _pose_jammer_label_camera() -> void:
	var gm: Node = _find_game_manager()
	if gm == null:
		return
	var cam_root: Node = gm.get_node_or_null("SpectatorCamera")
	if cam_root == null:
		return
	if cam_root is Node3D:
		var n3: Node3D = cam_root
		# Mid distance, slightly above, aimed at the SEIZE JAMMER banner.
		n3.global_position = Vector3(4.0, 6.5, 10.0)
		n3.look_at(Vector3(0.0, 8.5, 0.0), Vector3.UP)
		if "follow_mode" in n3:
			n3.follow_mode = false



func _aim_local_fp_at_jammer(gm: Node) -> void:
	# Point the local FP pawn at dish origin so Join FP stills include it.
	if gm == null:
		return
	var dish := Vector3(0.0, 0.35, 0.0)
	if "players" in gm and typeof(gm.players) == TYPE_DICTIONARY:
		for pid: String in gm.players.keys():
			var pawn: Node = gm.players[pid]
			if pawn == null or not is_instance_valid(pawn):
				continue
			var is_fp: bool = ("is_local_fp" in pawn and pawn.is_local_fp)
			if not is_fp:
				continue
			if pawn is Node3D:
				var n3: Node3D = pawn
				var delta: Vector3 = dish - n3.global_position
				delta.y = 0.0
				if delta.length_squared() > 0.01:
					var yaw: float = atan2(-delta.x, -delta.z)
					if "target_yaw" in pawn:
						pawn.target_yaw = yaw
					n3.rotation.y = yaw
			return
	# Fallback: spectator follow pose aimed at dish.
	_pose_jammer_follow_camera()


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
	# Force jammer dish into live hangar FP so Join still is not empty racks.
	_force_live_jammer_dish(gm)
	_aim_local_fp_at_jammer(gm)
	await create_timer(0.45).timeout
	_force_live_jammer_dish(gm)
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
