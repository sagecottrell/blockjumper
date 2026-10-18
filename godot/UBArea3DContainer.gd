@tool
class_name UBArea3DContainer
extends Node3D


# Called when the node enters the scene tree for the first time.
func _ready() -> void:
	var parent = get_parent()
	if parent is UBMeshInstance3D:
		_do_mesh(parent)
		parent.mesh_data_changed.connect(_on_mesh_changed)

func _on_mesh_changed():
	var parent = get_parent()
	if parent is UBMeshInstance3D:
		_do_mesh(parent)

func _process(_delta):
	if Engine.is_editor_hint():
		var _s = DebugDraw3D.new_scoped_config().set_thickness(0)
		for child in get_children():
			if child is Platform:
				for child2 in child.get_children():
					if child2 is CollisionShape3D:
						var shape = child2.shape
						if shape is CylinderShape3D:
							DebugDraw3D.draw_capsule(child2.global_position, child2.quaternion, shape.radius, shape.height)
				DebugDraw3D.draw_arrow(child.global_position, child.to_global(Vector3.UP))

func _do_mesh(parent: UBMeshInstance3D):
	var data := parent.mesh_data
	var faces := data.faces
	for child in get_children():
		child.queue_free()
	
	for face in faces:
		var vertices : Array[Vector3] = []
		var raw := Array(face.vertices)\
			.map(func(id): return data.positions[id])
		vertices.assign(raw)
		
		match len(vertices):
			4:
				_quad(vertices)
			_:
				continue

func _is_coplanar(vertices: Array[Vector3]) -> bool:
	if len(vertices) == 4:
		var ab = vertices[1] - vertices[0]
		var ac = vertices[2] - vertices[0]
		var ad = vertices[3] - vertices[0]
		return is_zero_approx(ac.cross(ad).dot(ab))
	return false
	
func _quad(v: Array[Vector3]):
	if !_is_coplanar(v):
		return
	
	var side0 := v[3] - v[0]
	var side1 := v[1] - v[0]
	
	var d1 = side1.length()
	var d0 = side0.length()
	side1 = side1.normalized()
	side0 = side0.normalized()
	
	var b := Basis(side1, side0.cross(side1).normalized(), side0)
	
	for i in range(0, d1):
		var x := side1 * (0.5 + i)
		for j in range(0, d0):
			var y := side0 * (0.5 + j)
			var p := x + y + v[0]
			var tform := Transform3D(b, p)
			var area3d := Platform.new()
			area3d.transform = tform
			add_child(area3d)
			_new_collisionshape(area3d)


const COLLISION_CYLINDER = preload("uid://b1o2pb5dfkjbj")

func _new_collisionshape(area3d: Node3D,):
	var cs := CollisionShape3D.new()
	cs.shape = COLLISION_CYLINDER
	area3d.add_child(cs)
