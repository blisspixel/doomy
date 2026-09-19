extends SceneTree

# Headless gate: jammer dish stays unmissable at follow / overview.
# Run: godot --path client --headless --script res://scripts/test_jammer_dish_silhouette.gd

func _initialize() -> void:
	var ok: bool = true
	var builder: GDScript = load("res://scripts/jammer_dish.gd") as GDScript
	if builder == null:
		push_error("test_jammer_dish_silhouette: failed to load jammer_dish.gd")
		quit(1)
		return

	var diameter: float = float(builder.call("diameter"))
	var stack: float = float(builder.call("stack_height"))
	var min_d: float = float(builder.get("MIN_DIAMETER"))
	var min_h: float = float(builder.get("MIN_STACK_HEIGHT"))
	var follow_px: float = float(builder.call("follow_span_px"))
	var overview_px: float = float(builder.call("overview_span_px"))
	var min_follow_px: float = float(builder.get("MIN_FOLLOW_SPAN_PX"))
	var min_overview_px: float = float(builder.get("MIN_OVERVIEW_SPAN_PX"))

	if diameter < min_d:
		push_error("test_jammer_dish_silhouette: diameter %.2f below floor %.2f" % [diameter, min_d])
		ok = false
	if stack < min_h:
		push_error("test_jammer_dish_silhouette: stack %.2f below floor %.2f" % [stack, min_h])
		ok = false

	# World mass must clear stranger-eye floors at tip camera distances.
	if diameter < 10.0:
		push_error("test_jammer_dish_silhouette: dish still too small for stranger eyes: %.2f" % diameter)
		ok = false

	# Projected on-screen footprint (pinhole 75deg / 1280 wide) — closer to stranger eyes than a mesh constant alone.
	if follow_px < min_follow_px:
		push_error(
			"test_jammer_dish_silhouette: follow footprint %.1fpx below floor %.1fpx"
			% [follow_px, min_follow_px]
		)
		ok = false
	if overview_px < min_overview_px:
		push_error(
			"test_jammer_dish_silhouette: overview footprint %.1fpx below floor %.1fpx"
			% [overview_px, min_overview_px]
		)
		ok = false

	var root: Node3D = builder.call("build") as Node3D
	if root == null:
		push_error("test_jammer_dish_silhouette: build() returned null")
		quit(1)
		return

	var required := [
		"GroundRing",
		"GroundPad",
		"Mast",
		"Boom",
		"DishMesh",
		"Rim",
		"Antenna",
		"AntennaTip",
		"LabelBanner",
		"DishLabel",
	]
	for name: String in required:
		if root.get_node_or_null(NodePath(name)) == null:
			push_error("test_jammer_dish_silhouette: missing child %s" % name)
			ok = false

	builder.call("apply_tint", root, true, false)
	var label: Label3D = root.get_node_or_null(NodePath("DishLabel")) as Label3D
	if label == null or label.text != "SEIZE JAMMER":
		push_error("test_jammer_dish_silhouette: live tint label expected SEIZE JAMMER")
		ok = false
	elif label.billboard != BaseMaterial3D.BILLBOARD_ENABLED:
		push_error("test_jammer_dish_silhouette: DishLabel must billboard toward camera")
		ok = false

	builder.call("apply_tint", root, false, true)
	if label == null or label.text != "JAMMER OK":
		push_error("test_jammer_dish_silhouette: seized tint label expected JAMMER OK")
		ok = false

	var dish: MeshInstance3D = root.get_node_or_null(NodePath("DishMesh")) as MeshInstance3D
	if dish == null or dish.mesh == null:
		push_error("test_jammer_dish_silhouette: DishMesh missing mesh")
		ok = false
	elif dish.mesh is SphereMesh:
		var sm: SphereMesh = dish.mesh as SphereMesh
		if sm.radius < 5.0:
			push_error("test_jammer_dish_silhouette: SphereMesh radius too small: %s" % str(sm.radius))
			ok = false

	# Live tint must leave an unshaded override so hangar ambient cannot black out the bowl.
	builder.call("apply_tint", root, true, false)
	if dish != null:
		var mat: Material = dish.get_surface_override_material(0)
		if mat == null or not (mat is StandardMaterial3D):
			push_error("test_jammer_dish_silhouette: DishMesh missing StandardMaterial3D override")
			ok = false
		else:
			var smat: StandardMaterial3D = mat as StandardMaterial3D
			if smat.shading_mode != BaseMaterial3D.SHADING_MODE_UNSHADED:
				push_error("test_jammer_dish_silhouette: DishMesh must be unshaded for hangar punch")
				ok = false

	root.free()

	if ok:
		print(
			"test_jammer_dish_silhouette: PASS diameter=",
			diameter,
			" stack=",
			stack,
			" follow_px=",
			follow_px,
			" overview_px=",
			overview_px,
			" min_d=",
			min_d,
			" min_h=",
			min_h
		)
		quit(0)
	else:
		push_error("test_jammer_dish_silhouette: FAIL")
		quit(1)
