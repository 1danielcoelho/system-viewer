use crate::{
    app_state::AppState,
    systems::{InterfaceSystem, PhysicsSystem, RenderingSystem, TransformUpdateSystem},
};

use super::ECManager;

pub struct SystemManager {
    render: RenderingSystem,
    interface: InterfaceSystem,
    physics: PhysicsSystem,
    trans: TransformUpdateSystem,
}
impl SystemManager {
    pub fn new() -> Self {
        return Self {
            render: RenderingSystem::new(),
            interface: InterfaceSystem::new(),
            physics: PhysicsSystem {},
            trans: TransformUpdateSystem {},
        };
    }

    // TODO: Make some "context" object that has mut refs to everything and is created every frame
    pub fn run(&mut self, state: &mut AppState, mut ent_man: &mut ECManager) {
        self.interface.begin_frame(state, &ent_man);

        self.physics.run(state, &mut ent_man);
        self.trans.run(state, &mut ent_man);
        self.render.run(state, &mut ent_man);

        self.interface.end_frame();
    }
}
