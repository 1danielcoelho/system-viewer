use crate::{
    app_state::AppState,
    systems::{InterfaceSystem, RenderingSystem},
};

use super::{ComponentManager, EventManager};

pub struct SystemManager {
    render: RenderingSystem,
    interface: InterfaceSystem,
}
impl SystemManager {
    pub fn new() -> Self {
        return Self {
            render: RenderingSystem {},
            interface: InterfaceSystem::new(),
        };
    }

    // TODO: Make some "context" object that has mut refs to everything and is created every frame
    pub fn run(&mut self, state: &mut AppState, cm: &mut ComponentManager, em: &mut EventManager) {
        self.render.run(state, &cm.transform, &cm.mesh);
        self.interface.run(state, &cm);
    }
}
