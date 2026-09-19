extends SceneTree

# Xvfb + opengl3 proof stills for the jammer dish (no server required).
# Run via tools/capture_jammer_dish_proof.sh

func _initialize() -> void:
	call_deferred("_run")


func _run() -> void:
	var out_dir: String = OS.get_environment("FRAGR_TIP_CAPTURE_DIR")
	if out_dir.is_empty():
		out_dir = ProjectSettings.globalize_path("res://../docs/screenshots")
	DirAccess.make_dir_recursive_absolute(out_dir)

	var root_3d := Node3D.new()
	root_3d.name = "ProofRoot"
	get_root().add_child(root_3d)

	var env := WorldEnvironment.new()
	var environment := Environment.new()
	environment.background_mode = Environment.BG_COLOR
	environment.background_color = Color(0.039, 0.039, 0.047)
	environment.ambient_light_source = Environment.AMBIENT_SOURCE_COLOR
	environment.ambient_light_color = Color(0.42, 0.38, 0.34)
	environment.ambient_light_energy = 0.45
	env.environment = environment
	root_3d.add_child(env)

	var key := DirectionalLight3D.new()
	key.light_energy = 1.15
	key.rotation_degrees = Vector3(-42.0, 35.0, 0.0)
	root_3d.add_child(key)

	# Dark hangar floor so the dish must punch through on its own.
	var floor := MeshInstance3D.new()
	var floor_mesh := BoxMesh.new()
	floor_mesh.size = Vector3(80.0, 0.2, 80.0)
	floor.mesh = floor_mesh
	floor.position = Vector3(0.0, -0.1, 0.0)
	var floor_mat := StandardMaterial3D.new()
	floor_mat.shading_mode = BaseMaterial3D.SHADING_MODE_UNSHADED
	floor_mat.albedo_color = Color(0.12, 0.11, 0.10)
	floor.set_surface_override_material(0, floor_mat)
	root_3d.add_child(floor)

	var builder: GDScript = load("res://scripts/jammer_dish.gd") as GDScript
	if builder == null:
		push_error("capture_jammer_dish_proof: failed to load jammer_dish.gd")
		quit(1)
		return
	var dish: Node3D = builder.call("build") as Node3D
	dish.position = Vector3(0.0, 0.35, 0.0)
	root_3d.add_child(dish)
	builder.call("apply_tint", dish, true, false)

	var cam := Camera3D.new()
	cam.current = true
	cam.fov = 75.0
	root_3d.add_child(cam)

	# Follow ~12m
	cam.global_position = Vector3(0.0, 3.2, 12.0)
	cam.look_at(Vector3(0.0, 3.5, 0.0), Vector3.UP)
	await create_timer(0.4).timeout
	await RenderingServer.frame_post_draw
	await RenderingServer.frame_post_draw
	_save(out_dir, "20_jammer_dish_follow_16x9.png")

	# Overview ~36m corner
	cam.global_position = Vector3(0.0, 22.0, 28.0)
	cam.look_at(Vector3(0.0, 2.0, 0.0), Vector3.UP)
	await create_timer(0.25).timeout
	await RenderingServer.frame_post_draw
	await RenderingServer.frame_post_draw
	_save(out_dir, "22_jammer_dish_overview_16x9.png")

	# Label read
	cam.global_position = Vector3(4.0, 6.5, 10.0)
	cam.look_at(Vector3(0.0, 9.0, 0.0), Vector3.UP)
	await create_timer(0.25).timeout
	await RenderingServer.frame_post_draw
	await RenderingServer.frame_post_draw
	_save(out_dir, "23_jammer_dish_seize_label_16x9.png")

	print("capture_jammer_dish_proof: done")
	quit(0)


func _save(out_dir: String, name: String) -> void:
	var img: Image = get_root().get_viewport().get_texture().get_image()
	if img == null:
		push_error("capture_jammer_dish_proof: null image for " + name)
		quit(1)
		return
	var path: String = out_dir.path_join(name)
	var err: Error = img.save_png(path)
	if err != OK:
		push_error("capture_jammer_dish_proof: save failed %s -> %s" % [str(err), path])
		quit(1)
		return
	print("capture_jammer_dish_proof: wrote ", path, " size=", img.get_width(), "x", img.get_height())
