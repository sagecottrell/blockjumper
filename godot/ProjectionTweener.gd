@tool
extends Camera3D

enum Mode {
	Perspective,
	Orthogonal
}

@export var mode : Mode = Mode.Perspective:
	set(value):
		if not is_inside_tree():
			mode = value
			return
		match value:
			Mode.Perspective:
				_to_perspective()
			Mode.Orthogonal:
				_to_pseudo_ortho()
		mode = value

@export var tween_time : float = 1

@export var persp_fov : float
@export var persp_distance : float

@export_range(1, 10) var ortho_multiplier : float = 1.0:
	set(value):
		_set_distance(_distance, value)
		ortho_multiplier = value

var start_fov : float
var start_dist : float

var _distance : float
@export_range(1, 100) var distance : float = 3:
	set(value):
		if not is_inside_tree():
			_distance = max(value, persp_distance)
			return
		_set_distance(value)
	get:
		return _distance

func _set_distance(value: float, _ortho_multiplier: float = 0):
	var parent = get_parent_node_3d()
	var rdist = (_distance - persp_distance) * ortho_multiplier + persp_distance
	
	var current_focus := parent.to_local(to_global(Vector3.FORWARD * rdist))
	
	if is_zero_approx(_ortho_multiplier):
		# if not zero, assume the parameter is a new ortho multiplier
		_ortho_multiplier = ortho_multiplier
		
	_distance = max(value, persp_distance)
	rdist = (_distance - persp_distance) * _ortho_multiplier + persp_distance
	
	var direction_from_focus := (position - current_focus).normalized()
	position = current_focus + direction_from_focus * rdist
	fov = persp_fov * persp_distance / _distance

func _to_pseudo_ortho():
	if mode == Mode.Orthogonal:
		return
	var tween := create_tween().set_trans(Tween.TRANS_CIRC)
	tween.tween_property(self, "distance", persp_fov * persp_distance, tween_time)

func _to_perspective():
	if mode == Mode.Perspective:
		return
	var tween := create_tween().set_trans(Tween.TRANS_CIRC)
	tween.tween_property(self, "distance", persp_distance, tween_time)
