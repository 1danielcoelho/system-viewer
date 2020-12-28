use crate::{
    app_state::AppState,
    managers::scene::Scene,
    systems::{PhysicsSystem, RenderingSystem, TransformUpdateSystem},
};

pub struct SystemManager {
    render: RenderingSystem,
    physics: PhysicsSystem,
    trans: TransformUpdateSystem,
}
impl SystemManager {
    pub fn new() -> Self {
        return Self {
            render: RenderingSystem::new(),
            physics: PhysicsSystem {},
            trans: TransformUpdateSystem {},
        };
    }

    // TODO: Make some "context" object that has mut refs to everything and is created every frame
    pub fn run(&mut self, state: &mut AppState, mut scene: &mut Scene) {
        self.physics.run(state, &mut scene);
        self.trans.run(state, &mut scene);
        self.render.run(state, &mut scene);
    }
}
