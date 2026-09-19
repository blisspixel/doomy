extends RefCounted
class_name JammerDishBuilder
## Solo Broadcast jammer dish world silhouette (client-only).
## Pixel-3D / Rock & Roll Racing saturated scrap palette:
## bone / gunmetal / rust / ember. No neon flood.
## Sized and lit so stranger eyes cannot miss it at spectator
## follow (~12m) and tip overview (~36m). Soft-touch seize radius
## stays 3.0m on the server; visual mass is intentionally larger.

# Horizontal dish bowl. Gate tests use these floors.
const DISH_RADIUS: float = 5.5
const DISH_FLAT_HEIGHT: float = 2.4
const MAST_HEIGHT: float = 4.6
const MAST_BOTTOM_RADIUS: float = 1.35
const MAST_TOP_RADIUS: float = 0.75
const RING_RADIUS: float = 6.2
const RING_HEIGHT: float = 0.28
const RIM_INNER: float = 4.85
const RIM_THICKNESS: float = 0.55
const ANTENNA_HEIGHT: float = 3.2
const LABEL_Y: float = 10.0
const LABEL_FONT: int = 160
const BANNER_WIDTH: float = 7.5
const BANNER_HEIGHT: float = 1.6

# Headless size gates (world meters).
const MIN_DIAMETER: float = 10.0
const MIN_STACK_HEIGHT: float = 8.5

# Stranger-eye footprint gates (75deg FOV, 1280x720).
const FOLLOW_DISTANCE_M: float = 12.0
const OVERVIEW_DISTANCE_M: float = 36.0
const REF_FOV_DEG: float = 75.0
const REF_VIEWPORT_W: float = 1280.0
const MIN_FOLLOW_SPAN_PX: float = 280.0
const MIN_OVERVIEW_SPAN_PX: float = 95.0

const BONE := Color(0.96, 0.90, 0.72)
const GUNMETAL := Color(0.55, 0.58, 0.62)
const RUST := Color(0.78, 0.34, 0.16)
const EMBER := Color(1.0, 0.62, 0.18)
const SEIZED_OK := Color(0.52, 0.78, 0.48)
const BANNER_INK := Color(0.08, 0.07, 0.06)


static func diameter() -> float:
	return DISH_RADIUS * 2.0


static func stack_height() -> float:
	# Mast top + dish half-height + antenna tip (label sits above; mesh stack).
	return MAST_HEIGHT + (DISH_FLAT_HEIGHT * 0.5) + ANTENNA_HEIGHT


static func projected_span_px(world_span_m: float, distance_m: float, fov_deg: float = REF_FOV_DEG, viewport_px: float = REF_VIEWPORT_W) -> float:
	# Horizontal span in pixels for a pinhole camera aimed at the subject.
	var half_fov := deg_to_rad(fov_deg) * 0.5
	var denom := 2.0 * distance_m * tan(half_fov)
	if denom <= 0.0001:
		return 0.0
	return (world_span_m / denom) * viewport_px


static func follow_span_px() -> float:
	return projected_span_px(diameter(), FOLLOW_DISTANCE_M)


static func overview_span_px() -> float:
	return projected_span_px(diameter(), OVERVIEW_DISTANCE_M)


static func build() -> Node3D:
	var root := Node3D.new()
	root.name = "JammerDish"

	# Hazard ground ring: large seize-read footprint, rust grit.
	var ring := MeshInstance3D.new()
	ring.name = "GroundRing"
	var ring_mesh := CylinderMesh.new()
	ring_mesh.top_radius = RING_RADIUS
	ring_mesh.bottom_radius = RING_RADIUS + 0.25
	ring_mesh.height = RING_HEIGHT
	ring_mesh.radial_segments = 28
	ring.mesh = ring_mesh
	# Root sits at Snapshot y~0.35; drop ring to floor.
	ring.position = Vector3(0.0, -0.28, 0.0)
	root.add_child(ring)

	# Inner pad fill so the ring reads as a pad, not a hoop only.
	var pad := MeshInstance3D.new()
	pad.name = "GroundPad"
	var pad_mesh := CylinderMesh.new()
	pad_mesh.top_radius = RING_RADIUS * 0.78
	pad_mesh.bottom_radius = RING_RADIUS * 0.84
	pad_mesh.height = RING_HEIGHT * 0.6
	pad_mesh.radial_segments = 24
	pad.mesh = pad_mesh
	pad.position = Vector3(0.0, -0.2, 0.0)
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
	mast.position = Vector3(0.0, MAST_HEIGHT * 0.5 - 0.15, 0.0)
	root.add_child(mast)

	# Cross-boom yoke under the dish (chunky silhouette arms).
	var boom := MeshInstance3D.new()
	boom.name = "Boom"
	var boom_mesh := BoxMesh.new()
	boom_mesh.size = Vector3(DISH_RADIUS * 1.45, 0.45, 0.55)
	boom.mesh = boom_mesh
	boom.position = Vector3(0.0, MAST_HEIGHT - 0.1, 0.0)
	root.add_child(boom)

	# Flattened dish bowl (primary silhouette mass).
	var dish := MeshInstance3D.new()
	dish.name = "DishMesh"
	var dish_mesh := SphereMesh.new()
	dish_mesh.radius = DISH_RADIUS
	dish_mesh.height = DISH_FLAT_HEIGHT
	dish_mesh.radial_segments = 28
	dish_mesh.rings = 14
	dish.mesh = dish_mesh
	dish.position = Vector3(0.0, MAST_HEIGHT + 0.25, 0.0)
	# Tip the bowl so side cameras read a dish, not a pancake.
	dish.rotation_degrees = Vector3(32.0, 0.0, 0.0)
	root.add_child(dish)

	# Rim torus: thick outer edge against hangar fog / dark racks.
	var rim := MeshInstance3D.new()
	rim.name = "Rim"
	var rim_mesh := TorusMesh.new()
	rim_mesh.inner_radius = RIM_INNER
	rim_mesh.outer_radius = RIM_INNER + RIM_THICKNESS
	rim_mesh.rings = 28
	rim_mesh.ring_segments = 14
	rim.mesh = rim_mesh
	rim.position = Vector3(0.0, MAST_HEIGHT + 0.55, 0.85)
	rim.rotation_degrees = Vector3(32.0, 0.0, 0.0)
	root.add_child(rim)

	# Antenna spike: vertical read above fighter billboards.
	var antenna := MeshInstance3D.new()
	antenna.name = "Antenna"
	var ant_mesh := CylinderMesh.new()
	ant_mesh.top_radius = 0.1
	ant_mesh.bottom_radius = 0.22
	ant_mesh.height = ANTENNA_HEIGHT
	ant_mesh.radial_segments = 8
	antenna.mesh = ant_mesh
	antenna.position = Vector3(0.0, MAST_HEIGHT + DISH_FLAT_HEIGHT * 0.35 + ANTENNA_HEIGHT * 0.5, 0.0)
	root.add_child(antenna)

	var tip := MeshInstance3D.new()
	tip.name = "AntennaTip"
	var tip_mesh := SphereMesh.new()
	tip_mesh.radius = 0.38
	tip_mesh.height = 0.76
	tip.mesh = tip_mesh
	tip.position = Vector3(0.0, MAST_HEIGHT + DISH_FLAT_HEIGHT * 0.35 + ANTENNA_HEIGHT + 0.08, 0.0)
	root.add_child(tip)

	# High-contrast banner plate behind the label so the words punch at 36m.
	var banner := MeshInstance3D.new()
	banner.name = "LabelBanner"
	var banner_mesh := BoxMesh.new()
	banner_mesh.size = Vector3(BANNER_WIDTH, BANNER_HEIGHT, 0.12)
	banner.mesh = banner_mesh
	banner.position = Vector3(0.0, LABEL_Y, 0.0)
	root.add_child(banner)

	var label := Label3D.new()
	label.name = "DishLabel"
	label.text = "SEIZE JAMMER"
	label.font_size = LABEL_FONT
	label.outline_size = 22
	label.position = Vector3(0.0, LABEL_Y, 0.18)
	label.billboard = BaseMaterial3D.BILLBOARD_ENABLED
	label.modulate = BONE
	label.outline_modulate = Color(0.05, 0.04, 0.03)
	label.no_depth_test = true
	label.pixel_size = 0.012
	root.add_child(label)

	# Default to live tint so a freshly built dish is never material-less / dark.
	apply_tint(root, true, false)
	return root


static func apply_tint(root: Node3D, live: bool, seized: bool) -> void:
	if root == null or not is_instance_valid(root):
		return
	var primary := BONE
	var accent := RUST
	var glow := 0.55
	var energy := 2.4
	var label_text := "JAMMER"
	if live:
		primary = EMBER
		accent = Color(0.95, 0.42, 0.12)
		glow = 0.95
		energy = 4.2
		label_text = "SEIZE JAMMER"
	elif seized:
		primary = SEIZED_OK
		accent = Color(0.42, 0.62, 0.40)
		glow = 0.45
		energy = 2.0
		label_text = "JAMMER OK"

	for child in root.get_children():
		if child is Label3D:
			child.text = label_text
			child.modulate = primary
			child.outline_modulate = Color(0.05, 0.04, 0.03)
			child.billboard = BaseMaterial3D.BILLBOARD_ENABLED
			child.no_depth_test = true
			continue
		if not (child is MeshInstance3D):
			continue
		var mat := StandardMaterial3D.new()
		# Unshaded so dark hangar ambient cannot swallow the silhouette.
		mat.shading_mode = BaseMaterial3D.SHADING_MODE_UNSHADED
		mat.roughness = 1.0
		mat.metallic = 0.0
		var name := String(child.name)
		match name:
			"GroundRing":
				mat.albedo_color = accent
				mat.emission_enabled = true
				mat.emission = accent
				mat.emission_energy_multiplier = energy * 0.85
			"GroundPad":
				mat.albedo_color = primary.lightened(0.15)
				mat.emission_enabled = true
				mat.emission = primary * 0.7
				mat.emission_energy_multiplier = energy * 0.7
			"Mast", "Boom":
				mat.albedo_color = GUNMETAL
				mat.emission_enabled = true
				mat.emission = GUNMETAL * 0.35
				mat.emission_energy_multiplier = energy * 0.45
			"Rim", "Antenna", "AntennaTip":
				mat.albedo_color = accent.lightened(0.12)
				mat.emission_enabled = true
				mat.emission = accent
				mat.emission_energy_multiplier = energy
			"LabelBanner":
				mat.albedo_color = BANNER_INK
				mat.emission_enabled = true
				mat.emission = BANNER_INK
				mat.emission_energy_multiplier = 1.2
				# Billboard the banner plate with the label.
				mat.billboard_mode = BaseMaterial3D.BILLBOARD_ENABLED
			_:
				# DishMesh and any unnamed mass: primary tint.
				mat.albedo_color = primary
				mat.emission_enabled = true
				mat.emission = primary * glow
				mat.emission_energy_multiplier = energy
		child.set_surface_override_material(0, mat)
