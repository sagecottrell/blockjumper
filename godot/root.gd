@tool
extends MultiMeshInstance3D

@export var update_live := false
const CUBE_SIDE_MESH = preload("uid://b1msou15xpkdq")

func _ready():
	multimesh = MultiMesh.new()
	multimesh.transform_format = MultiMesh.TRANSFORM_3D
	multimesh.use_custom_data = true
	multimesh.mesh = CUBE_SIDE_MESH
	multimesh.instance_count = 1
	
	_do_mm()

# Called when the node enters the scene tree for the first time.
func _process(_delta: float) -> void:
	if update_live:
		_do_mm()

func _do_mm():
	var faces := get_tree().get_nodes_in_group("MMFace")
	
	if multimesh.instance_count > 0:
		multimesh.instance_count = faces.size()

	# Set the transform of the instances.
	for i in multimesh.instance_count:
		var node := faces[i]
		var parent := node.get_parent()
		var data := Vector2i(21, 0)
		if parent is Cube:
			data = parent.block_kind.get_data_for_side(node.name)
		multimesh.set_instance_transform(i, node.global_transform)
		multimesh.set_instance_custom_data(i, Color(data.x, data.y, 0, 0))
