@tool
class_name Cube
extends Area3D

@export_range(1, 100) var right : int
@export_range(1, 100) var left : int
@export_range(1, 100) var up : int
@export_range(1, 100) var down : int
@export_range(1, 100) var back : int
@export_range(1, 100) var fwd : int

func _get_anchor(global_collision_point: Vector3) -> Node3D:
	var local := to_local(global_collision_point)
	var axes := [[left, right], [down, up], [fwd, back]]
	var i := local.abs().max_axis_index()
	var b : Marker3D = axes[i][(sign(local[i]) + 1) / 2]
	return b
