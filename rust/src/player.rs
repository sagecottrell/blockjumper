use std::f32::consts::PI;

use godot::{
    classes::{Input, RayCast3D},
    prelude::*,
};

use crate::{GLTFImport, Platform};

#[derive(GodotClass)]
#[class(base=Node3D)]
pub struct Player {
    base: Base<Node3D>,

    #[export]
    time_to_move: f32,

    #[export]
    model: Option<Gd<GLTFImport>>,
    #[export(range = (0.0, 90.0, radians_as_degrees))]
    max_angle: f32,

    #[export]
    rotators: Array<Gd<Node3D>>,

    moving: bool,
    // onready
    raycast: Option<Gd<RayCast3D>>,
}

#[godot_api]
impl INode3D for Player {
    fn init(base: Base<Node3D>) -> Self {
        Self {
            time_to_move: 0.25,
            model: None,
            raycast: None,
            max_angle: PI / 12.0,
            rotators: Array::new(),
            moving: false,
            base,
        }
    }

    fn ready(&mut self) {
        let raycast = self.base().try_get_node_as::<RayCast3D>("%RayCast3D");
        self.raycast = raycast;
    }

    fn physics_process(&mut self, _delta: f64) {
        if self.moving {
            return;
        }

        let input = Input::singleton();
        let direction = input.get_vector("left", "right", "forward", "backward");

        if direction.is_zero_approx() {
            return;
        }

        let direction = Vector3::new(direction.x, 0., direction.y).normalized();

        let up = self.base().get_global_basis().col_b();

        if let Some(mut model) = self.model.clone() {
            let d = self.base().to_global(direction);
            for mut rotator in self.rotators.iter_shared() {
                rotator.look_at_ex(d).up(up).done();
            }
            let time = self.time_to_move;
            if let Some((correction, target)) = self.test_move() {
                model
                    .bind_mut()
                    .play_animation_with_length("jump".into(), time);

                self.signals()
                    .selected_target()
                    .emit(&target, correction, time);
                self.moving = true;
            }
        }
    }
}

static RAYCAST_TRANSFORMS: [Vector3; 2] = [
    Vector3 {
        x: 0.0,
        y: 0.5,
        z: 0.0,
    },
    Vector3 {
        x: 0.0,
        y: 0.1,
        z: 0.0,
    },
];

#[godot_api]
impl Player {
    #[signal]
    fn selected_target(target: Gd<Node3D>, correction: Transform3D, time: f32);

    #[func]
    pub fn done_moving(&mut self) {
        self.moving = false;
    }

    fn test_move(&self) -> Option<(Transform3D, Gd<Node3D>)> {
        match self.raycast.clone() {
            Some(mut raycast) => {
                let up_dir = self.base().get_global_basis().col_b();
                for transform in RAYCAST_TRANSFORMS {
                    raycast.set_position(transform);
                    raycast.force_raycast_update();
                    let collider = raycast.get_collider();
                    if let Some(collider) = collider
                        && let Ok(platform) = collider.try_cast::<Platform>()
                    {
                        let target = platform
                            .bind()
                            .get_anchor(raycast.get_collision_point())
                            .unwrap();
                        let is_close =
                            target.get_global_basis().col_b().angle_to(up_dir) <= self.max_angle;

                        if !is_close {
                            continue;
                        }

                        // rotate the transform until all the directions line up with our current rotation.
                        // this is necessary because the anchor_transform up direction might be aligned with us, but not in any other directions.

                        let a_direction = self.base().get_global_basis().col_a();
                        let target_transform = target.get_global_transform();
                        let mut correction = Transform3D::IDENTITY;
                        while (target_transform * correction)
                            .basis
                            .col_a()
                            .dot(a_direction)
                            < 0.5
                        {
                            correction.basis =
                                correction.basis.rotated(correction.basis.col_b(), PI / 2.0);
                        }

                        return Some((correction, target));
                    }
                }
                None
            }
            _ => None,
        }
    }
}
