use std::collections::HashMap;

use godot::{
    classes::{
        AnimationPlayer, Area3D, IArea3D, Material, Mesh, ResourceLoader, ShaderMaterial,
        StandardMaterial3D, base_material_3d::TextureParam, node::ProcessMode,
    },
    prelude::*,
    signal::ConnectHandle,
};

mod components;
mod global_signals;
mod player;

struct MyExtension;

#[gdextension]
unsafe impl ExtensionLibrary for MyExtension {}

// ===================================================================================================
// ===================================================================================================
//

#[derive(GodotClass)]
#[class(base=Resource, init, tool)]
struct ColorReplace {
    #[export]
    #[var(set=set_input)]
    input: Color,
    #[export]
    #[var(set=set_output)]
    output: Color,
    base: Base<Resource>,
}

#[godot_api]
impl ColorReplace {
    #[signal]
    fn on_input();
    #[func]
    fn set_input(&mut self, c: Color) {
        self.input = c;
        self.signals().on_input().emit();
    }

    #[signal]
    fn on_output();
    #[func]
    fn set_output(&mut self, c: Color) {
        self.output = c;
        self.signals().on_output().emit();
    }
}

#[derive(GodotClass)]
#[class(base=Node3D, tool, init)]
struct GLTFImport {
    base: Base<Node3D>,
    anim_lib: Option<Gd<AnimationPlayer>>,

    #[export]
    #[var(set=set_colors)]
    color_replace: Array<Option<Gd<ColorReplace>>>,
    color_replace_signals: Vec<ConnectHandle>,
}

#[godot_api]
impl INode3D for GLTFImport {
    fn ready(&mut self) {
        let anim_lib: Option<Gd<AnimationPlayer>> =
            self.base().find_child("AnimationPlayer").map(|x| x.cast());
        self.anim_lib = anim_lib;
    }
}

#[godot_api]
impl GLTFImport {
    #[func]
    pub fn play_animation(&mut self, anim_name: StringName) {
        self.play_animation_with_scale(anim_name, 1.0);
    }
    #[func]
    pub fn play_animation_with_length(&mut self, anim_name: StringName, length: f32) {
        if let Some(anim) = self.anim_lib.clone() {
            let s = anim.get_animation_library("").unwrap();
            let scale = s.get_animation(&anim_name).unwrap().get_length();
            self.play_animation_with_scale(anim_name, scale / length);
        }
    }
    #[func]
    pub fn play_animation_with_scale(&mut self, anim_name: StringName, scale: f32) {
        if let Some(mut anim) = self.anim_lib.clone() {
            anim.seek(0.);
            anim.set_speed_scale(scale);
            anim.set_current_animation(&anim_name);
            anim.play();
        }
    }

    #[func]
    pub fn play_activate(&mut self) {
        self.play_animation("activate".into());
    }

    #[func]
    pub fn set_colors(&mut self, c: Array<Option<Gd<ColorReplace>>>) {
        let s = self.to_gd();
        let signals = &mut self.color_replace_signals;
        for handle in signals.drain(..) {
            handle.disconnect();
        }
        for c in c.iter_shared() {
            if let Some(c) = c {
                signals.push(
                    c.signals()
                        .on_input()
                        .connect_other(&s, Self::update_overrides),
                );
                signals.push(
                    c.signals()
                        .on_output()
                        .connect_other(&s, Self::update_overrides),
                )
            }
        }
        self.color_replace = c;
        self.update_overrides();
    }

    fn update_overrides(&mut self) {
        let children = self
            .base()
            .find_children_ex("")
            .type_("MeshInstance3D")
            .done();
        let mut mat_map: HashMap<Gd<Material>, Gd<Material>> = HashMap::new();
        for child in children.iter_shared() {
            if let Ok(mut mesh) = child.try_cast::<godot::classes::MeshInstance3D>() {
                let existing_material = mesh.get_mesh().unwrap().surface_get_material(0);
                if let Some(m) = existing_material {
                    let material = match mat_map.get(&m) {
                        Some(mm) => mm.clone(),
                        None => {
                            let new_material = self
                                .create_material(m.clone())
                                .expect("must have gltf material");
                            mat_map.insert(m, new_material.clone());
                            new_material
                        }
                    };

                    mesh.set_material_override(Some(material).as_ref());
                }
            }
        }
    }

    fn create_material(&self, m: Gd<Material>) -> Option<Gd<Material>> {
        let mut new_material = m;
        let albedo = new_material
            .clone()
            .try_cast::<StandardMaterial3D>()
            .ok()?
            .get_texture(TextureParam::ALBEDO)?
            .to_variant();
        for c in self.color_replace.iter_shared() {
            if let Some(c) = c {
                let mut shader = color_replace_shader()?.duplicate_resource();
                shader.set_shader_parameter("albedo_texture", &albedo);
                shader.set_shader_parameter("input_color1", &c.bind().input.to_variant());
                shader.set_shader_parameter("output_color1", &c.bind().output.to_variant());
                new_material = shader.upcast::<Material>();
            }
        }
        Some(new_material)
    }
}

// ===================================================================================================
// ===================================================================================================

#[derive(GodotClass)]
#[class(tool, base=Area3D)]
struct Platform {
    base: Base<Area3D>,
}

#[godot_api]
impl IArea3D for Platform {
    fn init(base: Base<Area3D>) -> Self {
        Self { base }
    }
}

#[godot_api]
impl Platform {
    #[func(virtual)]
    fn get_anchor(&self, _global_collision_point: Vector3) -> Option<Gd<Node3D>> {
        Some(self.to_gd().upcast::<Node3D>())
    }
}

// ===================================================================================================
//
// ===================================================================================================

pub fn cube_side_mesh() -> Option<Gd<Mesh>> {
    try_load("uid://b1msou15xpkdq")
}

pub fn color_replace_shader() -> Option<Gd<ShaderMaterial>> {
    try_load("uid://lyypliprgc46")
}

pub fn try_load<T: GodotClass>(uid: &str) -> Option<Gd<T>>
where
    T: Inherits<Resource>,
{
    ResourceLoader::singleton().load(uid)?.try_cast::<T>().ok()
}

// ===================================================================================================
// Activation Range
// ===================================================================================================

#[derive(GodotClass)]
#[class(base=Area3D)]
struct ActivationRange {
    active: bool,
    base: Base<Area3D>,
}

#[godot_api]
impl IArea3D for ActivationRange {
    fn init(base: Base<Area3D>) -> Self {
        Self {
            base,
            active: false,
        }
    }

    fn ready(&mut self) {
        self.base_mut().set_process_mode(ProcessMode::PAUSABLE);
        self.base_mut().set_monitorable(false);
        let s = self.to_gd();
        self.base_mut()
            .signals()
            .body_entered()
            .connect_other(&s.clone(), Self::on_enter);
        self.base_mut()
            .signals()
            .body_exited()
            .connect_other(&s.clone(), Self::on_exit);
    }
}

#[godot_api]
impl ActivationRange {
    #[signal]
    fn activated();

    #[signal]
    fn deactivated();

    #[func]
    fn on_enter(&mut self, _body: Gd<Node3D>) {
        self.activate();
    }

    #[func]
    fn on_exit(&mut self, _body: Gd<Node3D>) {
        self.deactivate();
    }

    #[func]
    fn activate(&mut self) {
        if self.active {
            return;
        }
        self.active = true;
        self.signals().activated();
    }

    #[func]
    fn deactivate(&mut self) {
        if !self.active {
            return;
        }
        self.active = false;
        self.signals().deactivated();
    }

    #[func]
    fn toggle(&mut self) {
        if self.active {
            self.deactivate();
        } else {
            self.activate();
        }
    }
}

// ===================================================================================================
// ===================================================================================================

#[derive(GodotClass)]
#[class(base=Resource, tool, init)]
pub struct DamageSource {
    #[export]
    #[init(val = 1)]
    amount: u8,

    base: Base<Resource>,
}
