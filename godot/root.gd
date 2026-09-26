@tool
extends MultiMeshInstance3D


# Called when the node enters the scene tree for the first time.
func _process(_delta: float) -> void:
	var faces := get_tree().get_nodes_in_group("MMFace")
	
	multimesh.instance_count = faces.size()
	# Maybe not all of them should be visible at first.
	multimesh.visible_instance_count = faces.size()

	# Set the transform of the instances.
	for i in multimesh.visible_instance_count:
		multimesh.set_instance_transform(i, faces[i].global_transform)
		multimesh.set_instance_custom_data(i, Color(1, 5, 0, 0))
