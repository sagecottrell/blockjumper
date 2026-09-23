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

    // onready
    raycast: Option<Gd<RayCast3D>>,

    // runtime vars
    up: Vector3,
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
            up: Vector3::UP,
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

        if let Some(mut model) = self.model.clone() {
            let direction = Vector3::new(direction.x, 0., direction.y).normalized();
            model.look_at(self.base().to_global(direction));
        }

        if let Some(raycast) = self.raycast.clone() {
            let collider = raycast.get_collider();
            if let Some(collider) = collider
                && let Ok(platform) = collider.try_cast::<Platform>()
            {
                let target_transform = platform
                    .bind()
                    .get_anchor_transform(raycast.get_collision_point());
                if target_transform.basis.tdoty(self.up) < 0.7 {
                    return;
                }

                self.moving = true;
                self.start_transform = self.base().get_global_transform();
                self.target_transform = target_transform;
                self.up = target_transform.basis.col_b();
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
}
