@tool
extends MultiMeshInstance3D


# Called when the node enters the scene tree for the first time.
func _process(_delta: float) -> void:
	var cubes := get_tree().get_nodes_in_group("Cube")
	
	var transforms := []
	for cube in cubes:
		transforms.append_array(cube._positions())
	
	multimesh.instance_count = transforms.size()
	# Maybe not all of them should be visible at first.
	multimesh.visible_instance_count = transforms.size()

	# Set the transform of the instances.
	for i in multimesh.visible_instance_count:
		multimesh.set_instance_transform(i, transforms[i])
		multimesh.set_instance_custom_data(i, Color(1, 5, 0, 0))
