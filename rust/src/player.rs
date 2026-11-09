use godot::{classes::Input, prelude::*};

use crate::GLTFImport;

#[derive(GodotClass)]
#[class(base=Node3D, init)]
pub struct Player {
    base: Base<Node3D>,

    #[export]
    #[init(val = 0.25)]
    time_to_move: f32,

    #[export]
    model: Option<Gd<GLTFImport>>,

    #[export]
    rotators: Array<Gd<Node3D>>,

    moving: bool,
}

#[godot_api]
impl INode3D for Player {
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

        let d = self.base().to_global(direction);
        for mut rotator in self.rotators.iter_shared() {
            rotator.look_at_ex(d).up(up).done();
        }

        self.signals().rotated().emit();
    }
}

#[godot_api]
impl Player {
    #[signal]
    pub fn rotated();

    #[func]
    pub fn done_moving(&mut self) {
        self.moving = false;
    }

    #[func]
    pub fn start_jump(&mut self) {
        if let Some(mut model) = self.model.clone() {
            let time = self.time_to_move;
            model
                .bind_mut()
                .play_animation_with_length("jump".into(), time);
        }
        self.moving = true;
    }
}
