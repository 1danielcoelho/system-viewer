use crate::{
    app_state::AppState,
    systems::{InterfaceSystem, PhysicsSystem, RenderingSystem, TransformUpdateSystem},
};

use super::{ECManager, EventManager};

pub struct SystemManager {
    render: RenderingSystem,
    interface: InterfaceSystem,
    physics: PhysicsSystem,
    trans: TransformUpdateSystem,
}
impl SystemManager {
    pub fn new() -> Self {
        return Self {
            render: RenderingSystem {},
            interface: InterfaceSystem::new(),
            physics: PhysicsSystem {},
            trans: TransformUpdateSystem {},
        };
    }

    // TODO: Make some "context" object that has mut refs to everything and is created every frame
    pub fn run(&mut self, state: &mut AppState, mut ent_man: &mut ECManager) {
        self.physics.run(state, &mut ent_man);
        self.trans.run(state, &mut ent_man);
        self.render.run(state, &ent_man.transform, &ent_man.mesh);
        self.interface.run(state, &ent_man);
    }
}
