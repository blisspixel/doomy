extends RefCounted
class_name JammerDishBuilder
## Solo Broadcast jammer dish world silhouette (client-only).
## Pixel-3D / Rock & Roll Racing saturated scrap palette:
## bone / gunmetal / rust / ember. No neon flood.
## Sized to read at spectator follow (~12m) and tip overview (~36m).
## Server soft-touch seize radius is 3.0m; footprint matches that scale.

# Horizontal dish bowl (diameter ~5.6m). Gate tests use these floors.
const DISH_RADIUS: float = 2.8
const DISH_FLAT_HEIGHT: float = 1.35
const MAST_HEIGHT: float = 2.55
const MAST_BOTTOM_RADIUS: float = 0.95
const MAST_TOP_RADIUS: float = 0.55
const RING_RADIUS: float = 3.15
const RING_HEIGHT: float = 0.12
const RIM_INNER: float = 2.55
const RIM_THICKNESS: float = 0.18
const ANTENNA_HEIGHT: float = 1.75
const LABEL_Y: float = 5.35
const LABEL_FONT: int = 96

# Headless size gates (unmissable bar).
const MIN_DIAMETER: float = 5.0
const MIN_STACK_HEIGHT: float = 4.8

const BONE := Color(0.91, 0.86, 0.74)
const GUNMETAL := Color(0.42, 0.46, 0.50)
const RUST := Color(0.62, 0.32, 0.20)
const EMBER := Color(0.92, 0.58, 0.26)
const SEIZED_OK := Color(0.48, 0.66, 0.50)


static func diameter() -> float:
	return DISH_RADIUS * 2.0


static func stack_height() -> float:
	# Mast top + dish half-height + antenna tip (label sits above; mesh stack).
	return MAST_HEIGHT + (DISH_FLAT_HEIGHT * 0.5) + ANTENNA_HEIGHT


static func build() -> Node3D:
	var root := Node3D.new()
	root.name = "JammerDish"

	# Hazard ground ring: seize-radius footprint, rust grit.
	var ring := MeshInstance3D.new()
	ring.name = "GroundRing"
	var ring_mesh := CylinderMesh.new()
	ring_mesh.top_radius = RING_RADIUS
	ring_mesh.bottom_radius = RING_RADIUS + 0.12
	ring_mesh.height = RING_HEIGHT
	ring_mesh.radial_segments = 24
	ring.mesh = ring_mesh
	# Root sits at Snapshot y~0.35; drop ring to floor.
	ring.position = Vector3(0.0, -0.28, 0.0)
	root.add_child(ring)

	# Inner pad fill (slightly raised) so the ring reads as a pad, not a hoop only.
	var pad := MeshInstance3D.new()
	pad.name = "GroundPad"
	var pad_mesh := CylinderMesh.new()
	pad_mesh.top_radius = RING_RADIUS * 0.72
	pad_mesh.bottom_radius = RING_RADIUS * 0.78
	pad_mesh.height = RING_HEIGHT * 0.55
	pad_mesh.radial_segments = 20
	pad.mesh = pad_mesh
	pad.position = Vector3(0.0, -0.24, 0.0)
	root.add_child(pad)

	# Gunmetal mast / pedestal.
	var mast := MeshInstance3D.new()
	mast.name = "Mast"
	var mast_mesh := CylinderMesh.new()
	mast_mesh.top_radius = MAST_TOP_RADIUS
	mast_mesh.bottom_radius = MAST_BOTTOM_RADIUS
	mast_mesh.height = MAST_HEIGHT
	mast_mesh.radial_segments = 16
	mast.mesh = mast_mesh
	mast.position = Vector3(0.0, MAST_HEIGHT * 0.5 - 0.2, 0.0)
	root.add_child(mast)

	# Cross-boom yoke under the dish (chunky silhouette arms).
	var boom := MeshInstance3D.new()
	boom.name = "Boom"
	var boom_mesh := BoxMesh.new()
	boom_mesh.size = Vector3(DISH_RADIUS * 1.35, 0.28, 0.35)
	boom.mesh = boom_mesh
	boom.position = Vector3(0.0, MAST_HEIGHT - 0.15, 0.0)
	root.add_child(boom)

	# Flattened dish bowl (primary silhouette mass).
	var dish := MeshInstance3D.new()
	dish.name = "DishMesh"
	var dish_mesh := SphereMesh.new()
	dish_mesh.radius = DISH_RADIUS
	dish_mesh.height = DISH_FLAT_HEIGHT
	dish_mesh.radial_segments = 24
	dish_mesh.rings = 12
	dish.mesh = dish_mesh
	dish.position = Vector3(0.0, MAST_HEIGHT + 0.15, 0.0)
	# Tip the bowl so side cameras read a dish, not a pancake.
	dish.rotation_degrees = Vector3(28.0, 0.0, 0.0)
	root.add_child(dish)

	# Rim torus: crisp outer edge against hangar fog.
	var rim := MeshInstance3D.new()
	rim.name = "Rim"
	var rim_mesh := TorusMesh.new()
	rim_mesh.inner_radius = RIM_INNER
	rim_mesh.outer_radius = RIM_INNER + RIM_THICKNESS
	rim_mesh.rings = 24
	rim_mesh.ring_segments = 12
	rim.mesh = rim_mesh
	rim.position = Vector3(0.0, MAST_HEIGHT + 0.35, 0.55)
	rim.rotation_degrees = Vector3(28.0, 0.0, 0.0)
	root.add_child(rim)

	# Antenna spike: vertical read above fighter billboards.
	var antenna := MeshInstance3D.new()
	antenna.name = "Antenna"
	var ant_mesh := CylinderMesh.new()
	ant_mesh.top_radius = 0.06
	ant_mesh.bottom_radius = 0.14
	ant_mesh.height = ANTENNA_HEIGHT
	ant_mesh.radial_segments = 8
	antenna.mesh = ant_mesh
	antenna.position = Vector3(0.0, MAST_HEIGHT + DISH_FLAT_HEIGHT * 0.35 + ANTENNA_HEIGHT * 0.5, 0.0)
	root.add_child(antenna)

	var tip := MeshInstance3D.new()
	tip.name = "AntennaTip"
	var tip_mesh := SphereMesh.new()
	tip_mesh.radius = 0.22
	tip_mesh.height = 0.44
	tip.mesh = tip_mesh
	tip.position = Vector3(0.0, MAST_HEIGHT + DISH_FLAT_HEIGHT * 0.35 + ANTENNA_HEIGHT + 0.05, 0.0)
	root.add_child(tip)

	var label := Label3D.new()
	label.name = "DishLabel"
	label.text = "JAMMER DISH"
	label.font_size = LABEL_FONT
	label.outline_size = 12
	label.position = Vector3(0.0, LABEL_Y, 0.0)
	label.billboard = BaseMaterial3D.BILLBOARD_ENABLED
	label.modulate = BONE
	label.outline_modulate = Color(0.10, 0.09, 0.08)
	root.add_child(label)

	return root


static func apply_tint(root: Node3D, live: bool, seized: bool) -> void:
	if root == null or not is_instance_valid(root):
		return
	var primary := BONE
	var accent := RUST
	var glow := 0.14
	var energy := 0.55
	var label_text := "JAMMER"
	if live:
		primary = EMBER
		accent = Color(0.85, 0.42, 0.18)
		glow = 0.38
		energy = 1.15
		label_text = "SEIZE JAMMER"
	elif seized:
		primary = SEIZED_OK
		accent = Color(0.40, 0.55, 0.42)
		glow = 0.18
		energy = 0.7
		label_text = "JAMMER OK"

	for child in root.get_children():
		if child is Label3D:
			child.text = label_text
			child.modulate = primary
			child.outline_modulate = Color(0.10, 0.09, 0.08)
			continue
		if not (child is MeshInstance3D):
			continue
		var mat := StandardMaterial3D.new()
		mat.roughness = 0.92
		mat.metallic = 0.15
		var name := String(child.name)
		match name:
			"GroundRing", "GroundPad":
				mat.albedo_color = accent.darkened(0.25)
				mat.emission_enabled = true
				mat.emission = accent * (glow * 0.7)
				mat.emission_energy_multiplier = energy * 0.7
			"Mast", "Boom":
				mat.albedo_color = GUNMETAL.darkened(0.1)
				mat.emission_enabled = true
				mat.emission = GUNMETAL * 0.08
				mat.emission_energy_multiplier = 0.4
				mat.metallic = 0.35
			"Rim", "Antenna", "AntennaTip":
				mat.albedo_color = accent.lightened(0.05)
				mat.emission_enabled = true
				mat.emission = accent * glow
				mat.emission_energy_multiplier = energy
			_:
				# DishMesh and any unnamed mass: primary tint.
				mat.albedo_color = primary.darkened(0.08)
				mat.emission_enabled = true
				mat.emission = primary * glow
				mat.emission_energy_multiplier = energy
		child.set_surface_override_material(0, mat)
