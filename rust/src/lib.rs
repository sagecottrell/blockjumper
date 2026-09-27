use godot::{
    classes::{Area3D, IArea3D, ResourceLoader, Mesh}, prelude::*,
};

mod blockdef;
mod facemultimesh;
mod player;

struct MyExtension;

#[gdextension]
unsafe impl ExtensionLibrary for MyExtension {}

#[derive(GodotClass)]
#[class(base=Node3D)]
struct GLTFImport {
    base: Base<Node3D>,
}

#[godot_api]
impl INode3D for GLTFImport {
    fn init(base: Base<Node3D>) -> Self {
        Self { base }
    }
}

#[derive(GodotClass)]
#[class(tool, base=Area3D)]
struct Platform {
    #[export]
    block_kind: Option<Gd<blockdef::BlockDefinition>>,
    base: Base<Area3D>,
}

#[godot_api]
impl IArea3D for Platform {
    fn init(base: Base<Area3D>) -> Self {
        Self {
            block_kind: None,
            base,
        }
    }
}

#[godot_api]
impl Platform {
    #[func(virtual)]
    fn get_anchor_transform(&self, _global_collision_point: Vector3) -> Transform3D {
        self.base().get_global_transform()
    }
}



pub fn cube_side_mesh() -> Option<Gd<Mesh>> {
    ResourceLoader::singleton()
        .load("uid://b1msou15xpkdq")?
        .try_cast::<Mesh>()
        .ok()
}