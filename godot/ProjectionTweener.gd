@tool
extends Camera3D

enum Mode {
	Perspective,
	Orthogonal
}

@export var mode : Mode = Mode.Perspective:
	set(value):
		match value:
			Mode.Perspective:
				_to_perspective()
			Mode.Orthogonal:
				_to_pseudo_ortho()
		mode = value

@export var tween_time : float = 1

@export var persp_fov : float
@export var persp_distance : float

var start_fov : float
var start_dist : float

@export_range(1, 100) var distance : float = 3:
	set(value):
		distance = max(value, persp_distance)
		position = position.normalized() * distance
		fov = persp_fov * persp_distance / distance

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
