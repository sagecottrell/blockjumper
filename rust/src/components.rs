use godot::{
    classes::{
        Engine, MeshInstance3D, QuadMesh, StandardMaterial3D, Texture2D,
        base_material_3d::{BillboardMode, ShadingMode, TextureFilter, TextureParam, Transparency},
        notify::Node3DNotification,
    },
    prelude::*,
};

use crate::GLTFImport;

#[derive(GodotClass)]
#[class(base=Node3D, internal, tool)]
pub struct EditorDebugIcon {
    pub icon: Option<Gd<Texture2D>>,
    base: Base<Node3D>,
}

#[godot_api]
impl INode3D for EditorDebugIcon {
    fn init(base: Base<Node3D>) -> Self {
        Self { base, icon: None }
    }

    fn ready(&mut self) {
        if !Engine::singleton().is_editor_hint() {
            return;
        }
        if let Some(icon) = self.icon.clone() {
            let mut mesh_instance = MeshInstance3D::new_alloc();
            let mut mesh = QuadMesh::new_gd();
            let mut material = StandardMaterial3D::new_gd();
            material.set_texture(TextureParam::ALBEDO, &icon);
            material.set_transparency(Transparency::ALPHA);
            material.set_texture_filter(TextureFilter::NEAREST);
            material.set_billboard_mode(BillboardMode::ENABLED);
            material.set_shading_mode(ShadingMode::UNSHADED);
            mesh.set_material(&material);
            mesh_instance.set_mesh(&mesh);
            self.base_mut().add_child(&mesh_instance);
        }
    }
}

// ==========================================================================================
// // ==========================================================================================

#[derive(GodotClass)]
#[class(base=Node3D, tool)]
pub struct HeavyDutyButton {
    /// the animation to use for the "press" action
    #[export]
    press_anim_name: StringName,

    #[export]
    release_anim_name: StringName,

    #[export]
    toggle_on_press: bool,

    pressed: u8,

    base: Base<Node3D>,
}

#[godot_api]
impl INode3D for HeavyDutyButton {
    fn init(base: Base<Node3D>) -> Self {
        Self {
            base,
            release_anim_name: "release".into(),
            press_anim_name: "press".into(),
            pressed: 0,
            toggle_on_press: false,
        }
    }

    fn ready(&mut self) {
        self.base_mut().update_configuration_warnings();
        self.play_animation();
    }

    fn get_configuration_warnings(&self) -> PackedArray<GString> {
        if self.get_gltf().is_some() {
            PackedStringArray::from(&[])
        } else {
            PackedStringArray::from(&["child must be GLTFImport".to_godot()])
        }
    }

    fn on_notification(&mut self, what: Node3DNotification) {
        match what {
            Node3DNotification::CHILD_ORDER_CHANGED => {
                self.base_mut().update_configuration_warnings();
            }
            _ => {}
        }
    }
}

#[godot_api]
impl HeavyDutyButton {
    fn get_gltf(&self) -> Option<Gd<GLTFImport>> {
        if let Some(child) = self.base().get_child(0) {
            child.try_cast::<GLTFImport>().ok()
        } else {
            None
        }
    }

    fn play_animation(&mut self) {
        if let Some(mut child) = self.get_gltf() {
            if self.pressed > 0 {
                child
                    .bind_mut()
                    .play_animation(self.press_anim_name.clone());
            } else {
                child
                    .bind_mut()
                    .play_animation(self.release_anim_name.clone());
            }
        }
    }

    #[signal]
    fn pressed();

    #[signal]
    fn released();

    fn press(&mut self) {
        self.play_animation();
        self.signals().pressed().emit();
    }

    #[func]
    fn press_force(&mut self, weight: u8) {
        self.pressed = weight;
        self.press();
    }

    fn release(&mut self) {
        self.play_animation();
        self.signals().released().emit();
    }

    #[func]
    fn release_force(&mut self) {
        self.pressed = 0;
        self.release();
    }

    #[func]
    fn on_pressed(&mut self) {
        if self.toggle_on_press && self.pressed > 0 {
            self.release_force();
            return;
        }
        let is_pressed = self.pressed > 0;
        self.pressed = self.pressed.saturating_add(1);
        if !is_pressed {
            self.press();
        }
    }

    #[func]
    fn on_released(&mut self) {
        if self.toggle_on_press {
            return;
        }
        let is_pressed = self.pressed > 0;
        self.pressed = self.pressed.saturating_sub(1);
        if is_pressed {
            self.release();
        }
    }
}
