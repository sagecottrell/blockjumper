@tool
class_name Cube
extends Platform

@export var pos_x : Marker3D
@export var neg_x : Marker3D
@export var pos_y : Marker3D
@export var neg_y : Marker3D
@export var pos_z : Marker3D
@export var neg_z : Marker3D

func _positions() -> Array[Transform3D]:
	var axes := [neg_x, pos_x, neg_y, pos_y, neg_z, pos_z]
	return axes.map(func(x): return x.global_transform)

func _get_anchor_transform(global_collision_point: Vector3) -> Transform3D:
	var local := to_local(global_collision_point)
	var axes := [[neg_x, pos_x], [neg_y, pos_y], [neg_z, pos_z]]
	var i := local.abs().max_axis_index()
	var b : Marker3D = axes[i][(sign(local[i]) + 1) / 2]
	return b.global_transform
