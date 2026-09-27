use std::f32::consts::PI;

use godot::{
    classes::{
        Camera3D, Control, DirectionalLight3D, EditorInspectorPlugin, EditorPlugin, IControl,
        IEditorInspectorPlugin, IEditorPlugin, MultiMesh, MultiMeshInstance3D, SubViewport, SubViewportContainer, control::LayoutPreset,
        multi_mesh::TransformFormat,
    },
    prelude::*,
    signal::ConnectHandle,
};

use crate::cube_side_mesh;

#[derive(GodotClass)]
#[class(tool, base=Resource)]
pub(crate) struct BlockDefinition {
    base: Base<Resource>,

    #[export]
    #[var(set=set_var_default)]
    default: Vector2i,

    #[export]
    #[var(set=set_var_up)]
    up: Vector2i,

    #[export]
    #[var(set=set_var_down)]
    down: Vector2i,
}

#[godot_api]
impl IResource for BlockDefinition {
    fn init(base: Base<Resource>) -> Self {
        Self {
            default: Vector2i::new(0, 2),
            up: Vector2i::default(),
            down: Vector2i::default(),
            base,
        }
    }
}

#[godot_api]
impl BlockDefinition {
    #[signal]
    fn on_change_mesh();

    #[func]
    fn set_var_default(&mut self, default: Vector2i) {
        self.default = default;
        self.trigger_on_change();
    }

    #[func]
    fn set_var_up(&mut self, up: Vector2i) {
        self.up = up;
        self.trigger_on_change();
    }

    #[func]
    fn set_var_down(&mut self, down: Vector2i) {
        self.down = down;
        self.trigger_on_change();
    }

    fn trigger_on_change(&mut self) {
        self.signals().on_change_mesh().emit();
    }

    #[func]
    pub fn get_data_for_side(&self, name: String) -> Vector2i {
        match name.as_str() {
            "+y" => self.get_var_up(),
            "-y" => self.get_var_down(),
            _ => self.default,
        }
    }

    #[func]
    fn get_var_up(&self) -> Vector2i {
        if self.up == Vector2i::default() {
            self.default
        } else {
            self.up
        }
    }

    #[func]
    fn get_var_down(&self) -> Vector2i {
        if self.down == Vector2i::default() {
            self.default
        } else {
            self.down
        }
    }
}


/// ==================================
/// ==================================
/// Editor Plugin
/// ==================================
/// ==================================

#[derive(GodotClass)]
#[class(tool, base=EditorPlugin)]
struct BlockDefinitionEditorPlugin {
    plugin: Option<Gd<BlockDefinitionEditorInspectorPlugin>>,

    base: Base<EditorPlugin>,
}

#[godot_api]
impl IEditorPlugin for BlockDefinitionEditorPlugin {
    fn init(base: Base<EditorPlugin>) -> Self {
        Self { plugin: None, base }
    }

    fn enter_tree(&mut self) {
        let plugin = BlockDefinitionEditorInspectorPlugin::new_gd();
        self.plugin = Some(plugin.clone());
        self.base_mut()
            .add_inspector_plugin(Some(plugin.clone()).as_ref());
    }

    fn exit_tree(&mut self) {
        if let Some(plugin) = self.plugin.clone() {
            self.base_mut()
                .remove_inspector_plugin(Some(plugin).as_ref());
        }
    }
}

#[derive(GodotClass)]
#[class(tool, base=EditorInspectorPlugin)]
struct BlockDefinitionEditorInspectorPlugin {
    base: Base<EditorInspectorPlugin>,
}

#[godot_api]
impl IEditorInspectorPlugin for BlockDefinitionEditorInspectorPlugin {
    fn init(base: Base<EditorInspectorPlugin>) -> Self {
        Self { base }
    }

    fn can_handle(&self, object: Option<Gd<Object>>) -> bool {
        object
            .map(|x| x.try_cast::<BlockDefinition>().is_ok())
            .unwrap_or(false)
    }

    fn parse_begin(&mut self, object: Option<Gd<Object>>) {
        self.base_mut().add_custom_control(
            Some({
                let mut label = BlockDefResource3DPreviewControl::new_alloc();
                label
                    .bind_mut()
                    .set_resource(object.map(|x| x.try_cast::<BlockDefinition>().unwrap()));

                label
            })
            .as_ref(),
        );
    }
}

#[derive(GodotClass)]
#[class(tool, base=Control)]
struct BlockDefResource3DPreviewControl {
    mesh_instance: Option<Gd<MultiMesh>>,
    connect_handle: Option<ConnectHandle>,
    resource: Option<Gd<BlockDefinition>>,
    base: Base<Control>,
}

#[godot_api]
impl IControl for BlockDefResource3DPreviewControl {
    fn init(base: Base<Control>) -> Self {
        Self {
            mesh_instance: None,
            connect_handle: None,
            resource: None,
            base,
        }
    }
}

#[godot_api]
impl BlockDefResource3DPreviewControl {
    #[func]
    fn set_resource(&mut self, resource: Option<Gd<BlockDefinition>>) {
        self.resource = resource.clone();

        self.base_mut()
            .set_custom_minimum_size(Vector2::new(0., 200.));

        let mut sub_viewport_container = SubViewportContainer::new_alloc();
        sub_viewport_container.set_stretch(true);
        sub_viewport_container.set_anchors_and_offsets_preset(LayoutPreset::FULL_RECT);

        let mut sub_viewport = SubViewport::new_alloc();
        sub_viewport.set_use_own_world_3d(true);

        let mut camera = Camera3D::new_alloc();
        camera.set_position(Vector3::new(0.5, 2., 0.));
        camera.rotate_x(-PI / 2.);

        let mut dir_light = DirectionalLight3D::new_alloc();
        dir_light.set_transform(camera.get_transform());

        let mut mesh_instance = MultiMeshInstance3D::new_alloc();

        let mut multimesh = MultiMesh::new_gd();
        multimesh.set_use_custom_data(true);
        multimesh.set_transform_format(TransformFormat::TRANSFORM_3D);
        multimesh.set_mesh(cube_side_mesh().as_ref());
        multimesh.set_instance_count(6);
        self.mesh_instance = Some(multimesh.clone());

        mesh_instance.set_multimesh(Some(multimesh).as_ref());
        sub_viewport.add_child(&mesh_instance.upcast::<Node>());
        sub_viewport.add_child(&dir_light.upcast::<Node>());
        sub_viewport.add_child(&camera.upcast::<Node>());
        sub_viewport_container.add_child(&sub_viewport.upcast::<Node>());
        self.base_mut()
            .add_child(&sub_viewport_container.upcast::<Node>());

        if let Some(handle) = self.connect_handle.take() {
            handle.disconnect();
        }

        if let Some(ref mut resource) = resource.clone() {
            self.set_mesh();

            let handle = resource
                .bind_mut()
                .signals()
                .on_change_mesh()
                .connect_other(self, Self::set_mesh);
            self.connect_handle = Some(handle);
        }
    }

    fn set_mesh(&mut self) {
        if let Some(mi) = &mut self.mesh_instance
            && let Some(resource) = &self.resource
        {
            let default = resource.bind().default;
            mi.set_instance_transform(
                0,
                Transform3D::default().translated(Vector3::new(0., 0., -1.)),
            );
            mi.set_instance_transform(
                1,
                Transform3D::default().translated(Vector3::new(-1., 0., 0.)),
            );
            mi.set_instance_transform(
                2,
                Transform3D::default().translated(Vector3::new(0., 0., 0.)),
            );
            mi.set_instance_transform(
                3,
                Transform3D::default().translated(Vector3::new(1., 0., 0.)),
            );
            mi.set_instance_transform(
                4,
                Transform3D::default().translated(Vector3::new(2., 0., 0.)),
            );
            mi.set_instance_transform(
                5,
                Transform3D::default().translated(Vector3::new(0., 0., 1.)),
            );

            let up = resource.bind().get_var_up();
            let down = resource.bind().get_var_down();

            mi.set_instance_custom_data(0, Color::from_rgba(up.x as f32, up.y as f32, 0., 0.));
            mi.set_instance_custom_data(
                1,
                Color::from_rgba(default.x as f32, default.y as f32, 0., 0.),
            );
            mi.set_instance_custom_data(
                2,
                Color::from_rgba(default.x as f32, default.y as f32, 0., 0.),
            );
            mi.set_instance_custom_data(
                3,
                Color::from_rgba(default.x as f32, default.y as f32, 0., 0.),
            );
            mi.set_instance_custom_data(
                4,
                Color::from_rgba(default.x as f32, default.y as f32, 0., 0.),
            );
            mi.set_instance_custom_data(5, Color::from_rgba(down.x as f32, down.y as f32, 0., 0.));
        }
    }
}
