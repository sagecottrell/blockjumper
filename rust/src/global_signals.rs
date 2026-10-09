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
}
