use std::collections::HashMap;

use godot::{
    classes::{
        Area3D, Engine, IPathFollow3D, Material, MeshInstance3D, Path3D, PathFollow3D, QuadMesh,
        StandardMaterial3D, Texture2D, Tween,
        base_material_3d::{BillboardMode, ShadingMode, TextureFilter, TextureParam, Transparency},
        node::ProcessMode,
        notify::{Node3DNotification, NodeNotification},
        tween,
    },
    prelude::*,
    signal::ConnectHandle,
};

use crate::{
    ColorReplace, DamageSource, GLTFImport, color_replace_shader, global_signals::GlobalSignalsImpl,
};

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
#[class(tool, base=Node3D, init)]
struct PlayerSpawner {
    #[export]
    global_signals: Option<Gd<GlobalSignalsImpl>>,

    /// the target node that should contain the spawned player
    #[export]
    container: Option<Gd<Node3D>>,
    spawned_nodes: Vec<Gd<Node>>,
    base: Base<Node3D>,
}

#[godot_api]
impl INode3D for PlayerSpawner {
    fn ready(&mut self) {
        if !Engine::singleton().is_editor_hint() {
            for mut child in self.base_mut().get_children().iter_shared() {
                if child.get_process_mode() == ProcessMode::INHERIT {
                    child.set_process_mode(ProcessMode::DISABLED);
                    if let Ok(mut n3d) = child.try_cast::<Node3D>() {
                        n3d.set_visible(false);
                    }
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

        for mut spawned in self.spawned_nodes.drain(..) {
            spawned.queue_free();
        }

        let mut container = self
            .container
            .clone()
            .unwrap_or_else(|| self.base().get_parent_node_3d().unwrap());
        let children = self.base().get_children();
        for child in children.iter_shared() {
            let mut dup = child.duplicate_node();
            dup.set_process_mode(ProcessMode::INHERIT);
            container.call_deferred("add_child", &[dup.to_variant()]);
            match (child.try_cast::<Node3D>(), dup.clone().try_cast::<Node3D>()) {
                (Ok(c3d), Ok(mut d3d)) => {
                    d3d.set_visible(true);
                    d3d.set_global_transform(c3d.get_global_transform());
                }
                _ => {}
            }
            self.spawned_nodes.push(dup.clone());
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
#[class(tool, base=Node, init)]
struct Enabler {
    #[export]
    #[init(val = true)]
    start_enabled: bool,
    #[export]
    hide_disabled: bool,
    base: Base<Node>,
}

#[godot_api]
impl INode for Enabler {
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
        let mut parent = self.base().get_parent().unwrap();
        parent.set_process_mode(ProcessMode::INHERIT);
        if self.hide_disabled
            && let Ok(mut n3d) = parent.try_cast::<Node3D>()
        {
            n3d.set_visible(true);
        }
        self.signals().enabled().emit();
    }

    #[func]
    fn disable(&mut self) {
        let mut parent = self.base().get_parent().unwrap();
        parent.set_process_mode(ProcessMode::DISABLED);
        if self.hide_disabled
            && let Ok(mut n3d) = parent.try_cast::<Node3D>()
        {
            n3d.set_visible(false);
        }
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

// ===================================================================================================
// ===================================================================================================

#[derive(GodotClass)]
#[class(base=Node, init)]
pub struct CollectiblesGroup {
    items: Vec<Gd<CollectibleHitbox>>,
    total_collected: u8,
    base: Base<Node>,
}

#[godot_api]
impl CollectiblesGroup {
    #[signal]
    pub fn partial_collected(remaining: u8);
    #[signal]
    pub fn all_collected();

    #[func]
    pub fn add_item(&mut self, item: Gd<CollectibleHitbox>) {
        self.items.push(item);
    }
    #[func]
    pub fn try_remove_item(&mut self, item: Gd<CollectibleHitbox>) -> bool {
        match self.items.iter().position(|f| f == &item) {
            Some(pos) => {
                self.items.swap_remove(pos);
                true
            }
            _ => false,
        }
    }

    #[func]
    pub fn item_complete(&mut self, item: Gd<CollectibleHitbox>) {
        if self.try_remove_item(item) {
            godot_print!("collected item and recorded");
            self.total_collected += 1;
            let len = self.items.len() as u8;

            if self.items.is_empty() {
                self.signals().all_collected().emit();
            } else {
                self.signals().partial_collected().emit(len);
            }
        }
    }
}

#[derive(GodotClass)]
#[class(base=Area3D, init)]
pub struct CollectibleHitbox {
    /// the group this collectible is part of. if set, will call methods on the group automatically.
    #[export]
    collection: Option<Gd<CollectiblesGroup>>,
    base: Base<Area3D>,
}

#[godot_api]
impl godot::classes::IArea3D for CollectibleHitbox {
    fn enter_tree(&mut self) {
        let s = self.to_gd();
        if let Some(mut c) = self.collection.clone() {
            c.bind_mut().add_item(s.clone());
        }
        self.base_mut()
            .signals()
            .area_entered()
            .connect_other(&s.clone(), Self::on_area_enter);
    }

    fn exit_tree(&mut self) {
        let s = self.to_gd();
        if let Some(mut c) = self.collection.clone() {
            c.bind_mut().try_remove_item(s.clone());
        }
    }
}

#[godot_api]
impl CollectibleHitbox {
    #[signal]
    fn collected();

    #[func]
    fn on_area_enter(&mut self, _area: Gd<Area3D>) {
        self.base_mut().call_deferred("emit", &[]);
    }

    #[func]
    fn emit(&mut self) {
        let s = self.to_gd();
        if let Some(mut c) = self.collection.clone() {
            c.bind_mut().item_complete(s.clone());
        }
        self.signals().collected().emit();
    }
}

// ===================================================================================================
// ===================================================================================================

/// Lets you replace colors with other colors.
/// Add as a child node of something that has at least one `MeshInstance3D` as a descendant, and affect all of them.
#[derive(GodotClass)]
#[class(base=Node, init, tool)]
pub struct Mesh3DColorReplace {
    /// the list of colors to replace
    #[export]
    #[var(set=set_colors)]
    color_replace: Array<Option<Gd<ColorReplace>>>,
    color_replace_signals: Vec<ConnectHandle>,
    base: Base<Node>,
}

#[godot_api]
impl INode for Mesh3DColorReplace {
    fn exit_tree(&mut self) {
        self.disconnect_all();
        self.deactivate();
    }

    fn on_notification(&mut self, what: NodeNotification) {
        match what {
            NodeNotification::EXTENSION_RELOADED => {
                self.disconnect_all();
                self.connect_all();
            }
            _ => {}
        }
    }
}

#[godot_api]
impl Mesh3DColorReplace {
    fn disconnect_all(&mut self) {
        let signals = &mut self.color_replace_signals;
        for handle in signals.drain(..) {
            if handle.is_connected() {
                handle.disconnect();
            }
        }
    }

    fn connect_all(&mut self) {
        let s = self.to_gd();
        let c = &self.color_replace;
        let signals = &mut self.color_replace_signals;
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
    }

    #[func]
    pub fn set_colors(&mut self, c: Array<Option<Gd<ColorReplace>>>) {
        self.disconnect_all();
        self.color_replace = c
            .iter_shared()
            .map(|x| x.or_else(|| Some(ColorReplace::new_gd())))
            .collect();
        self.connect_all();
        self.base_mut().call_deferred("update_overrides", &[]);
    }

    #[func]
    fn update_overrides(&mut self) {
        let parent = self.base().get_parent();
        if parent.is_none() {
            return;
        }
        let children = parent
            .unwrap()
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
                shader.set_next_pass(Some(new_material).as_ref());
                new_material = shader.upcast::<Material>();
            }
        }
        Some(new_material)
    }

    #[func]
    fn activate(&mut self) {
        self.update_overrides();
    }

    #[func]
    fn deactivate(&mut self) {
        let parent = self.base().get_parent();
        if parent.is_none() {
            return;
        }
        let children = parent
            .unwrap()
            .find_children_ex("")
            .type_("MeshInstance3D")
            .done();
        for child in children.iter_shared() {
            if let Ok(mut mesh) = child.try_cast::<godot::classes::MeshInstance3D>() {
                mesh.set_material_override(Gd::null_arg());
            }
        }
    }
}
// ===================================================================================================
// ===================================================================================================

#[derive(GodotClass)]
#[class(base=Node3D, init)]
pub struct MovementHandler {
    // runtime vars
    start_transform: Transform3D,
    moving: bool,
    target: Option<Gd<Node3D>>,
    target_transform_correction: Transform3D,
    base: Base<Node3D>,
}

#[godot_api]
impl INode3D for MovementHandler {
    fn physics_process(&mut self, _delta: f64) {
        if self.moving {
            return;
        }

        if let Some(target) = self.target.clone() {
            let tform = target.get_global_transform() * self.target_transform_correction;
            self.base_mut()
                .get_parent_node_3d()
                .unwrap()
                .set_global_transform(tform);
        }
    }
}

#[godot_api]
impl MovementHandler {
    #[signal]
    pub fn completed_moving();

    #[func]
    pub fn set_target(&mut self, target: Gd<Node3D>, correction: Transform3D, time: f32) {
        self.moving = true;
        self.start_transform = self
            .base()
            .get_parent_node_3d()
            .unwrap()
            .get_global_transform();
        self.target = Some(target);
        self.target_transform_correction = correction;
        let callable = self.base().callable("interpolate");
        let tween = self
            .base_mut()
            .create_tween()
            .tween_method(&callable, &0.0.to_variant(), &1.0.to_variant(), time as f64)
            .set_trans(tween::TransitionType::LINEAR);

        tween
            .signals()
            .finished()
            .connect_other(self, Self::finish_interpolate);
    }

    #[func]
    fn interpolate(&mut self, weight: f32) {
        let basis = self.start_transform.basis;
        let origin = self.start_transform.origin;
        let tbasis = self.target.as_ref().unwrap().get_global_basis()
            * self.target_transform_correction.basis;
        let torigin = self.target.as_ref().unwrap().get_global_position();
        self.base_mut()
            .get_parent_node_3d()
            .unwrap()
            .set_global_transform(Transform3D {
                basis: basis.slerp(&tbasis, weight),
                origin: origin.lerp(torigin, weight),
            });
    }

    fn finish_interpolate(&mut self) {
        self.moving = false;
        self.signals().completed_moving().emit();
    }
}
