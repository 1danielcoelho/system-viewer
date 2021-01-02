use crate::{
    app_state::AppState,
    managers::scene::Scene,
    systems::{OrbitalSystem, PhysicsSystem, RenderingSystem, TransformUpdateSystem},
};

pub struct SystemManager {
    render: RenderingSystem,
    orbital: OrbitalSystem,
    physics: PhysicsSystem,
    trans: TransformUpdateSystem,
}
impl SystemManager {
    pub fn new() -> Self {
        return Self {
            render: RenderingSystem::new(),
            orbital: OrbitalSystem {},
            physics: PhysicsSystem {},
            trans: TransformUpdateSystem {},
        };
    }

    // TODO: Make some "context" object that has mut refs to everything and is created every frame
    pub fn run(&mut self, state: &mut AppState, mut scene: &mut Scene) {
        self.orbital.run(state, &mut scene);
        self.physics.run(state, &mut scene);
        self.trans.run(state, &mut scene);
        self.render.run(state, &mut scene);
    }
}
