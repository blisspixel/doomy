extends SceneTree

# Headless gate: jammer dish silhouette stays unmissable at follow / overview.
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

	if diameter < min_d:
		push_error("test_jammer_dish_silhouette: diameter %.2f below floor %.2f" % [diameter, min_d])
		ok = false
	if stack < min_h:
		push_error("test_jammer_dish_silhouette: stack %.2f below floor %.2f" % [stack, min_h])
		ok = false

	# Footprint should track server seize radius (~3m), not the old ~1.1m speck.
	if diameter < 5.5:
		push_error("test_jammer_dish_silhouette: dish still too small for 12m follow: %.2f" % diameter)
		ok = false

	var root: Node3D = builder.call("build") as Node3D
	if root == null:
		push_error("test_jammer_dish_silhouette: build() returned null")
		quit(1)
		return

	var required := [
		"GroundRing", "GroundPad", "Mast", "Boom", "DishMesh", "Rim", "Antenna", "AntennaTip", "DishLabel"
	]
	for name in required:
		if root.get_node_or_null(NodePath(name)) == null:
			push_error("test_jammer_dish_silhouette: missing child %s" % name)
			ok = false

	builder.call("apply_tint", root, true, false)
	var label: Label3D = root.get_node_or_null(NodePath("DishLabel")) as Label3D
	if label == null or label.text != "SEIZE JAMMER":
		push_error("test_jammer_dish_silhouette: live tint label expected SEIZE JAMMER")
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
		if sm.radius < 2.5:
			push_error("test_jammer_dish_silhouette: SphereMesh radius too small: %s" % str(sm.radius))
			ok = false

	root.free()

	if ok:
		print(
			"test_jammer_dish_silhouette: PASS diameter=",
			diameter,
			" stack=",
			stack,
			" min_d=",
			min_d,
			" min_h=",
			min_h
		)
		quit(0)
	else:
		push_error("test_jammer_dish_silhouette: FAIL")
		quit(1)
