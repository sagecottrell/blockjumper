use std::f32::consts::PI;

use godot::{
    classes::{Input, RayCast3D, tween},
    prelude::*,
};

use crate::{GLTFImport, Platform};

#[derive(GodotClass)]
#[class(base=Node3D)]
struct Player {
    base: Base<Node3D>,

    #[export]
    time_to_move: f64,
    #[export]
    model: Option<Gd<GLTFImport>>,
    #[export(range = (0.0, 90.0, radians_as_degrees))]
    max_angle: f32,

    // onready
    raycast: Option<Gd<RayCast3D>>,

    // runtime vars
    start_transform: Transform3D,
    target_transform: Transform3D,
    moving: bool,
}

#[godot_api]
impl INode3D for Player {
    fn init(base: Base<Node3D>) -> Self {
        Self {
            time_to_move: 0.25,
            model: None,
            raycast: None,
            max_angle: PI / 12.0,
            start_transform: Transform3D::IDENTITY,
            target_transform: Transform3D::IDENTITY,
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
            model
                .look_at_ex(self.base().to_global(direction))
                .up(up)
                .done();

            if let Some(target_transform) = self.test_move() {
                self.moving = true;
                self.start_transform = self.base().get_global_transform();
                self.target_transform = target_transform;
                let callable = self.base().callable("interpolate");
                let time = self.time_to_move;
                let tween = self
                    .base_mut()
                    .create_tween()
                    .tween_method(&callable, &0.0.to_variant(), &1.0.to_variant(), time)
                    .set_trans(tween::TransitionType::LINEAR);

                tween
                    .signals()
                    .finished()
                    .connect_other(self, Player::finish_interpolate);
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
    #[func]
    fn interpolate(&mut self, weight: f32) {
        let basis = self.start_transform.basis;
        let origin = self.start_transform.origin;
        let tbasis = self.target_transform.basis;
        let torigin = self.target_transform.origin;
        self.base_mut().set_global_transform(Transform3D {
            basis: basis.slerp(&tbasis, weight),
            origin: origin.slerp(torigin, weight),
        });
    }

    fn finish_interpolate(&mut self) {
        self.moving = false;
    }

    fn test_move(&mut self) -> Option<Transform3D> {
        match self.raycast.clone() {
            Some(mut raycast) => {
                let up = self.base().get_global_basis().col_b();
                for transform in RAYCAST_TRANSFORMS {
                    raycast.set_position(transform);
                    raycast.force_raycast_update();
                    let collider = raycast.get_collider();
                    if let Some(collider) = collider
                        && let Ok(platform) = collider.try_cast::<Platform>()
                    {
                        let mut target_transform = platform
                            .bind()
                            .get_anchor_transform(raycast.get_collision_point());
                        let is_close =
                            target_transform.basis.col_b().angle_to(up) <= self.max_angle;
                        
                        // rotate the transform until all the directions line up with our current rotation.
                        // this is necessary because the anchor_transform up direction might be aligned with us, but not in any other directions.
                        // the choice of col_a is arbitrary, col_c could also have been used.
                        let a_direction = self.base().get_global_basis().col_a();
                        while target_transform.basis.col_a().dot(a_direction) < 0.5 {
                            target_transform.basis = target_transform.basis.rotated(target_transform.basis.col_b(), PI / 2.0);
                        }
                        if is_close { return Some(target_transform); }
                    }
                }
                None
            }
            _ => None,
        }
    }
}
