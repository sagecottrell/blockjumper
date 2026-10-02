use godot::{
    classes::{AnimationPlayer, Area3D, IArea3D, Mesh, ResourceLoader},
    prelude::*,
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
    anim_lib: Option<Gd<AnimationPlayer>>,
}

#[godot_api]
impl INode3D for GLTFImport {
    fn init(base: Base<Node3D>) -> Self {
        Self {
            base,
            anim_lib: None,
        }
    }

    fn ready(&mut self) {
        let anim_lib: Gd<AnimationPlayer> =
            self.base().find_child("AnimationPlayer").unwrap().cast();
        self.anim_lib = Some(anim_lib);
    }
}

#[godot_api]
impl GLTFImport {
    pub fn play_animation(&mut self, anim_name: StringName) {
        self.play_animation_with_length(anim_name, 1.0);
    }
    pub fn play_animation_with_length(&mut self, anim_name: StringName, length: f32) {
        if let Some(anim) = self.anim_lib.clone() {
            let s = anim.get_animation_library("").unwrap();
            let scale = s.get_animation(&anim_name).unwrap().get_length();
            self.play_animation_with_scale(anim_name, scale / length);
        }
    }
    pub fn play_animation_with_scale(&mut self, anim_name: StringName, scale: f32) {
        if let Some(mut anim) = self.anim_lib.clone() {
            anim.seek(0.);
            anim.set_speed_scale(scale);
            anim.set_current_animation(&anim_name);
            anim.play();
        }
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
    fn get_anchor(&self, _global_collision_point: Vector3) -> Option<Gd<Node3D>> {
        Some(self.to_gd().upcast::<Node3D>())
    }
}

pub fn cube_side_mesh() -> Option<Gd<Mesh>> {
    ResourceLoader::singleton()
        .load("uid://b1msou15xpkdq")?
        .try_cast::<Mesh>()
        .ok()
}
