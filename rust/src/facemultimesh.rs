use godot::{
    classes::{
        IMultiMeshInstance3D, MultiMesh, MultiMeshInstance3D, multi_mesh::TransformFormat,
    }, prelude::*,
};

use crate::{Platform, cube_side_mesh};

#[derive(GodotClass)]
#[class(tool, base=MultiMeshInstance3D)]
struct FaceMultimesh {
    #[export]
    update_live: bool,

    base: Base<MultiMeshInstance3D>,
}

#[godot_api]
impl IMultiMeshInstance3D for FaceMultimesh {
    fn init(base: Base<MultiMeshInstance3D>) -> Self {
        Self {
            base,
            update_live: false,
        }
    }

    fn ready(&mut self) {
        let mut multimesh = MultiMesh::new_gd();
        multimesh
            .set_transform_format(TransformFormat::TRANSFORM_3D);
        multimesh.set_use_custom_data(true);
        multimesh.set_mesh(cube_side_mesh().as_ref());
        multimesh.set_instance_count(1);
        self.base_mut().set_multimesh(Some(multimesh).as_ref());
    }

    fn process(&mut self, _delta: f32) {
        if self.update_live {
            self.do_mm();
        }
    }
}

impl FaceMultimesh {
    fn do_mm(&mut self) {
        let faces = self.base().get_tree().get_nodes_in_group("MMFace");
        let count = faces.len() as i32;
        
        let mut multimesh = self.base_mut().get_multimesh().unwrap();

        if multimesh.get_instance_count() > 0 {
            multimesh.set_instance_count(count);
        }

        for i in 0..count {
            let node = faces.get(i as usize).unwrap().try_cast::<Node3D>().unwrap();
            let parent = node.get_parent().unwrap(); // it's in the tree, it has a parent.
            let data = parent
                .try_cast::<Platform>()
                .ok()
                .and_then(|plt| plt.bind().block_kind.clone())
                .map(|d| d.bind().get_data_for_side(node.get_name().to_string()))
                .unwrap_or_else(|| Vector2i::new(21, 0));
            multimesh
                .set_instance_transform(i, node.get_global_transform());
            multimesh.set_instance_custom_data(
                i,
                Color::from_rgba(data.x as f32, data.y as f32, 0.0, 0.0),
            );
        }
    }
}
