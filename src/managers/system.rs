use crate::{
    app_state::AppState,
    managers::ECManager,
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
    pub fn run(&mut self, state: &mut AppState, mut ent_man: &mut ECManager) {
        self.physics.run(state, &mut ent_man);
        self.trans.run(state, &mut ent_man);
        self.render.run(state, &mut ent_man);
    }
}
