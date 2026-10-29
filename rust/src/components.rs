use godot::{
    classes::{
        Area3D, Engine, IPathFollow3D, MeshInstance3D, Path3D, PathFollow3D, QuadMesh,
        StandardMaterial3D, Texture2D, Tween,
        base_material_3d::{BillboardMode, ShadingMode, TextureFilter, TextureParam, Transparency},
        node::ProcessMode,
        notify::Node3DNotification,
    },
    prelude::*,
};

use crate::{DamageSource, GLTFImport, global_signals::GlobalSignalsImpl, player::Player};

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

// =====================================================================================
// // =====================================================================================

#[derive(GodotClass)]
#[class(tool, base=Node3D)]
struct PlayerSpawner {
    #[export]
    #[var(set=set_player_scene)]
    player_scene: Option<Gd<PackedScene>>,
    #[export]
    global_signals: Option<Gd<GlobalSignalsImpl>>,

    /// the target node that should contain the spawned player
    #[export]
    container: Option<Gd<Node3D>>,
    player: Option<Gd<Player>>,
    base: Base<Node3D>,
}

#[godot_api]
impl INode3D for PlayerSpawner {
    fn init(base: Base<Node3D>) -> Self {
        Self {
            base,
            player_scene: None,
            global_signals: None,
            player: None,
            container: None,
        }
    }

    fn ready(&mut self) {
        if !Engine::singleton().is_editor_hint() {
            for mut child in self.base_mut().get_children().iter_shared() {
                if child.get_process_mode() == ProcessMode::INHERIT {
                    child.set_process_mode(ProcessMode::DISABLED);
                }
            }
        }
    }
}

#[godot_api]
impl PlayerSpawner {
    #[func]
    fn spawn(&mut self) {
        if Engine::singleton().is_editor_hint() {
            return;
        }

        if let Some(mut existing) = self.player.clone() {
            existing.queue_free();
            self.player = None;
        }

        if let Some(scene) = self.player_scene.clone() {
            let mut instance: Gd<Player> = scene.instantiate_as();
            self.container
                .clone()
                .unwrap_or_else(|| self.base().get_parent_node_3d().unwrap())
                .call_deferred("add_child", &[instance.clone().to_variant()]);
            instance.set_global_transform(self.base().get_global_transform());
            for child in self.base_mut().get_children().iter_shared() {
                let mut dup = child.duplicate_node();
                dup.set_process_mode(ProcessMode::INHERIT);
                instance.call_deferred("add_child", &[dup.to_variant()]);
            }
            self.player = Some(instance);
        }
    }

    #[func]
    fn set_player_scene(&mut self, scene: Option<Gd<PackedScene>>) {
        self.player_scene = scene.clone();
        if Engine::singleton().is_editor_hint() {
            if let Some(mut child) = self.base_mut().find_child_ex("player").owned(false).done() {
                child.queue_free();
            }
            if let Some(scene) = scene.clone() {
                let instance: Gd<Player> = scene.instantiate_as();
                self.base_mut().add_child(&instance.upcast::<Node>());
            }
        }
    }
}

//
// =====================================================================================
// // =====================================================================================
#[derive(GodotClass)]
#[class(tool, base=PathFollow3D)]
struct TweenController {
    #[export]
    forward_speed: f32,
    #[export]
    wait_one: f32,
    #[export]
    back_on_disable: bool,
    #[export]
    back_speed: f32,
    #[export]
    wait_two: f32,
    #[export]
    loops: i32,

    #[export]
    #[var(set=set_start_activated)]
    start_activated: bool,

    tween: Option<Gd<Tween>>,

    base: Base<PathFollow3D>,
}

#[godot_api]
impl IPathFollow3D for TweenController {
    fn init(base: Base<PathFollow3D>) -> Self {
        Self {
            base,
            forward_speed: 1.,
            wait_one: 0.,
            back_on_disable: false,
            back_speed: -1.,
            wait_two: -1.0,
            loops: -1,
            start_activated: false,
            tween: None,
        }
    }

    fn ready(&mut self) {
        if self.start_activated {
            self.activate();
        }
    }

    fn on_notification(&mut self, what: Node3DNotification) {
        match what {
            Node3DNotification::EDITOR_PRE_SAVE => {
                self.reset();
            }
            _ => {}
        }
    }
}

#[godot_api]
impl TweenController {
    #[signal]
    fn activated();

    #[signal]
    fn deactivated();

    #[func]
    fn interpolate(&mut self, weight: f32) {
        self.base_mut().set_progress_ratio(weight);
    }

    #[func]
    fn set_start_activated(&mut self, start_active: bool) {
        self.start_activated = start_active;
        if Engine::singleton().is_editor_hint() {
            if start_active {
                self.activate();
            } else {
                self.reset();
            }
        }
    }

    #[func]
    fn activate(&mut self) {
        self.clean_tween();
        let mut tween = self.base_mut().create_tween();
        self.forward(&mut tween);
        if !self.back_on_disable {
            self.backward(&mut tween, 1.0);
            if self.loops < 0 {
                tween.set_loops();
            } else if self.loops > 0 {
                tween.set_loops_ex().loops(self.loops).done();
            }
        }
        self.tween = Some(tween);
        self.signals().activated().emit();
    }

    #[func]
    fn deactivate(&mut self) {
        self.clean_tween();
        if !self.back_on_disable {
            return;
        }

        let mut tween = self.base_mut().create_tween();
        self.backward(&mut tween, self.base().get_progress_ratio());
        self.tween = Some(tween);
        self.signals().deactivated().emit();
    }

    #[func]
    fn reset(&mut self) {
        self.clean_tween();
        self.interpolate(0.0);
    }

    fn clean_tween(&mut self) {
        if let Some(mut tween) = self.tween.clone()
            && tween.is_valid()
        {
            tween.kill();
        }
        self.tween = None;
    }

    fn get_curve_len(&self) -> Option<f32> {
        Some(
            self.base()
                .get_parent_node_3d()?
                .try_cast::<Path3D>()
                .ok()?
                .get_curve()?
                .get_baked_length(),
        )
    }

    fn forward(&self, tween: &mut Gd<Tween>) {
        let interp = self.base().callable("interpolate");
        let one = 1.0.to_variant();
        let zero = self.base().get_progress_ratio().to_variant();
        if let Some(len) = self.get_curve_len() {
            let time = ((len - self.base().get_progress()) / self.forward_speed) as f64;
            tween.tween_method(&interp, &zero, &one, time);

            if self.wait_one > 0. {
                tween.tween_interval(self.wait_one as f64);
            }
        }
    }

    fn backward(&self, tween: &mut Gd<Tween>, ratio: f32) {
        let interp = self.base().callable("interpolate");
        let one = ratio.to_variant();
        let zero = 0.0.to_variant();
        if let Some(len) = self.get_curve_len() {
            let mut back_speed = self.back_speed;
            if back_speed == -1.0 {
                back_speed = self.forward_speed;
            }
            if back_speed > 0. {
                let dist = len * ratio;
                let time = (dist / back_speed) as f64;
                tween.tween_method(&interp, &one, &zero, time);
            }

            let mut wait_two = self.wait_two;
            if wait_two < 0. {
                wait_two = self.wait_one;
            }
            if wait_two > 0. {
                tween.tween_interval(wait_two as f64);
            }
        }
    }
}

#[derive(GodotClass)]
#[class(tool, base=Node)]
struct Enabler {
    #[export]
    start_enabled: bool,
    base: Base<Node>,
}

#[godot_api]
impl INode for Enabler {
    fn init(base: Base<Node>) -> Self {
        Self {
            base,
            start_enabled: true,
        }
    }

    fn ready(&mut self) {
        if self.start_enabled {
            self.enable();
        } else {
            self.disable();
        }
    }
}

#[godot_api]
impl Enabler {
    #[signal]
    fn enabled();

    #[signal]
    fn disabled();

    #[func]
    fn enable(&mut self) {
        self.base()
            .get_parent()
            .unwrap()
            .set_process_mode(ProcessMode::INHERIT);
        self.signals().enabled().emit();
    }

    #[func]
    fn disable(&mut self) {
        self.base()
            .get_parent()
            .unwrap()
            .set_process_mode(ProcessMode::DISABLED);
        self.signals().disabled().emit();
    }
}

#[derive(GodotClass)]
#[class(base=Area3D, init)]
pub struct Hitbox {
    #[export]
    trigger_checkpoint: bool,
    base: Base<Area3D>,
}

#[godot_api]
impl Hitbox {
    #[signal]
    pub fn hit(dmg: Option<Gd<DamageSource>>);

    #[func]
    pub fn on_hit(&mut self, dmg: Option<Gd<DamageSource>>) {
        self.signals().hit().emit(dmg.as_ref());
    }

    #[signal]
    pub fn checkpoint(cp: Gd<Node3D>);

    /// emits the checkpoint signal if `trigger_checkpoint` is true
    #[func]
    pub fn on_checkpoint(&mut self, cp: Gd<Node3D>) {
        if !self.trigger_checkpoint {
            return;
        }
        self.signals().checkpoint().emit(&cp);
    }
}

#[derive(GodotClass)]
#[class(base=Node, init)]
pub struct Teleport {
    #[export(range=(0., 1.))]
    #[init(val = 0.01)]
    noop_if_close: f32,
    base: Base<Node>,
}

#[godot_api]
impl Teleport {
    #[func]
    pub fn teleport_parent_to(&mut self, to: Gd<Node3D>) {
        if let Ok(mut parent) = self.base().get_parent().unwrap().try_cast::<Node3D>() {
            if parent
                .get_global_position()
                .distance_to(to.get_global_position())
                < self.noop_if_close
            {
                return;
            }
            parent.set_global_transform(to.get_global_transform());
            self.signals().teleported().emit(&to);
        }
    }

    #[signal]
    pub fn teleported(to: Gd<Node3D>);
}

// ===================================================================================================
// ===================================================================================================

#[derive(GodotClass)]
#[class(base=Area3D, init)]
pub struct EnemyHurtbox {
    #[export]
    damage: Option<Gd<DamageSource>>,
    base: Base<Area3D>,
}

#[godot_api]
impl godot::classes::IArea3D for EnemyHurtbox {
    fn ready(&mut self) {
        let s = self.to_gd();
        self.base_mut()
            .signals()
            .area_entered()
            .connect_other(&s.clone(), Self::on_area_enter);
    }
}

#[godot_api]
impl EnemyHurtbox {
    #[func]
    pub fn on_area_enter(&mut self, area: Gd<Area3D>) {
        if let Ok(mut hitbox) = area.try_cast::<Hitbox>() {
            hitbox.bind_mut().on_hit(self.damage.clone());
        }
    }
}
