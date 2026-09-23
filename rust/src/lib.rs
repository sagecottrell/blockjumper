use godot::{classes::{Area3D, IArea3D}, prelude::*};

mod player;

struct MyExtension;

#[gdextension]
unsafe impl ExtensionLibrary for MyExtension {}


#[derive(GodotClass)]
#[class(base=Node3D)]
struct GLTFImport {
    base: Base<Node3D>
}


#[godot_api]
impl INode3D for GLTFImport {
    fn init(base: Base<Node3D>) -> Self {
        Self {base}
    }
}






#[derive(GodotClass)]
#[class(base=Area3D)]
struct Platform {
    base: Base<Area3D>
}


#[godot_api]
impl IArea3D for Platform {
    fn init(base: Base<Area3D>) -> Self {
        Self {base}
    }
}


#[godot_api]
impl Platform {
    #[func(virtual)]
    fn get_anchor_transform(&self, _global_collision_point: Vector3) -> Transform3D {
        self.base().get_global_transform()
    }
}
