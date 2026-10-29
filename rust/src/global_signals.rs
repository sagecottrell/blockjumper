use crate::{DamageSource, GLTFImport};
use godot::prelude::*;

#[derive(GodotClass)]
#[class(base=Node, tool)]
pub struct GlobalSignalsImpl {
    base: Base<Node>,
}

#[godot_api]
impl INode for GlobalSignalsImpl {
    fn init(base: Base<Node>) -> Self {
        Self { base }
    }
}

#[godot_api]
impl GlobalSignalsImpl {
    #[signal]
    pub fn object_died(obj: Gd<Node3D>, dmg: Option<Gd<DamageSource>>);

    #[func]
    pub fn on_object_died(&mut self, obj: Gd<Node3D>, dmg: Option<Gd<DamageSource>>) {
        self.signals().object_died().emit(&obj, dmg.as_ref());
    }

    #[signal]
    pub fn checkpoint_get(node: Gd<Node3D>);

    #[func]
    pub fn on_checkpoint_get(&mut self, node: Gd<Node3D>) {
        self.signals().checkpoint_get().emit(&node);
    }

    #[signal]
    pub fn object_hit(obj: Gd<Node3D>, amount: Gd<DamageSource>);

    #[func]
    pub fn on_object_hit(&mut self, obj: Gd<Node3D>, amount: Gd<DamageSource>) {
        self.signals().object_hit().emit(&obj, &amount);
    }

    #[func]
    pub fn activate_obj_and_gltf(&mut self, obj: Gd<Node3D>) {
        self.activate_obj_and_gltf_anim(obj, "activate".into());
    }

    #[func]
    pub fn activate_obj_and_gltf_anim(&mut self, mut obj: Gd<Node3D>, anim: StringName) {
        if obj.has_method("activate") {
            obj.call("activate", &[]);
        }
        if let Some(child) = obj.find_child_ex("blockbench_export").done() {
            if let Ok(mut g) = child.try_cast::<GLTFImport>() {
                g.bind_mut().play_animation(anim);
            }
        }
    }
}
