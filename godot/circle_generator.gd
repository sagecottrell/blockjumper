@tool
extends Node3D

@export var scene : PackedScene

# 24 = 2 (a full circle) * 12 (min count due to 15deg max (pi/12))
@export_range(24, 100) var segment_count: int = 24:
	set(value):
		segment_count = value
		redraw()
		
@export_range(1, 10, 0.1) var segment_size: float = 1:
	set(value):
		segment_size = value
		prints('-----', segment_count, segment_size)
		redraw()

@export_range(0, 360, 1, "prefer_slider", "radians_as_degrees") var twist: float = 0:
	set(value):
		twist = value
		redraw()
	
@export_range(0, 1, 0.1, "or_greater") var offset_amount: float = 0:
	set(value):
		offset_amount = value
		redraw()

func redraw():
	var children := get_children(true)
	while children.size() > segment_count:
		children.pop_back().queue_free()
	while children.size() < segment_count:
		var child = scene.instantiate()
		add_child(child)
		children.append(child)
	if segment_count <= 0:
		return
	var theta = 2 * PI / segment_count
	var radius := segment_size / (2 * tan(theta / 2)) + offset_amount
	var phi = twist / segment_count
	for i in segment_count:
		var child = children[i]
		if child is Node3D:
			child.position.x = cos(i * theta) * radius
			child.position.z = sin(i * theta) * radius
			child.rotation = Vector3(0, -i * theta, i * phi)
	
	position.x = -radius
