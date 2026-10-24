use godot::prelude::*;

#[derive(GodotClass)]
#[class(base=Node, tool)]
struct GlobalSignalsImpl {
    base: Base<Node>,
}

#[godot_api]
impl INode for GlobalSignalsImpl {
    fn init(base: Base<Node>) -> Self {
        Self { base }
    }
}

#[godot_api]
impl GlobalSignalsImpl {
    #[signal]
    fn collection_completed();

    #[func]
    fn on_collection_completed(&mut self) {
        self.signals().collection_completed().emit();
    }

    #[signal]
    fn player_died();

    #[signal]
    fn checkpoint_get(node: Gd<Node3D>);

    #[func]
    fn on_checkpoint_get(&mut self, node: Gd<Node3D>) {
        self.signals().checkpoint_get().emit(&node);
    }

    #[signal]
    fn player_hit(amount: u8);
}
