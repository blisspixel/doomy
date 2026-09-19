extends Node3D
class_name ArenaCover

## Builds the arena's cover from the solids the server sends in MapInfo.
##
## The cover used to be hand-placed boxes in the scene file, duplicating a list
## of rectangles that the server also held. That is the same class of bug that
## produced the ninety degree facing mismatch and the silently dropped jump
## field: two copies of one shape, and nothing making them agree. When the
## arena doubled, the server's cover moved and the scene's did not, so
## fighters spawned in open ground that still looked like it had walls in it.
##
## Now there is one copy. The server owns the geometry, sends it once on join
## and again when the map changes, and this builds what it is told. What you
## can hide behind and what actually blocks a shot cannot disagree.

const WALL_HEIGHT: float = 4.5
const LOW_THRESHOLD: float = 1.0

var _built_for: int = -1

func _ready() -> void:
	name = "ArenaCover"

## Rebuild from a MapInfo payload. Cheap to call again; it only rebuilds when
## the map actually changed, because the server resends on rotation.
func apply_map_info(info: Dictionary) -> void:
	var map_id: int = int(info.get("map_id", -1))
	if map_id == _built_for:
		return
	_built_for = map_id
	for child in get_children():
		child.queue_free()
	var solids: Array = info.get("solids", [])
	for entry in solids:
		if typeof(entry) == TYPE_DICTIONARY:
			_add_solid(entry as Dictionary)

func _add_solid(solid: Dictionary) -> void:
	var min_x: float = float(solid.get("min_x", 0.0))
	var max_x: float = float(solid.get("max_x", 0.0))
	var min_z: float = float(solid.get("min_z", 0.0))
	var max_z: float = float(solid.get("max_z", 0.0))
	var size_x: float = absf(max_x - min_x)
	var size_z: float = absf(max_z - min_z)
	if size_x <= 0.0 or size_z <= 0.0:
		return

	# A wide, thin solid is a wall you shoot over; a chunky one is a pillar you
	# hide behind. Height follows from the footprint so the shape reads as what
	# it does without the server having to describe it.
	var thin: bool = minf(size_x, size_z) < LOW_THRESHOLD
	var height: float = 1.6 if thin else WALL_HEIGHT

	var mesh: BoxMesh = BoxMesh.new()
	mesh.size = Vector3(size_x, height, size_z)

	var node: MeshInstance3D = MeshInstance3D.new()
	node.mesh = mesh
	node.position = Vector3((min_x + max_x) * 0.5, height * 0.5, (min_z + max_z) * 0.5)
	node.material_override = _material(thin)
	add_child(node)

static func _material(thin: bool) -> StandardMaterial3D:
	var mat: StandardMaterial3D = StandardMaterial3D.new()
	mat.albedo_color = Color(0.29, 0.25, 0.22) if thin else Color(0.34, 0.30, 0.26)
	mat.roughness = 0.95
	mat.texture_filter = BaseMaterial3D.TEXTURE_FILTER_NEAREST
	return mat
