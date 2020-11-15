use crate::{
    app_state::AppState,
    systems::{InterfaceSystem, PhysicsSystem, RenderingSystem, TransformUpdateSystem},
};

use super::{ComponentManager, EntityManager, EventManager};

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
    pub fn run(
        &mut self,
        state: &mut AppState,
        cm: &mut ComponentManager,
        em: &mut EventManager,
        ent_man: &mut EntityManager,
    ) {
        self.physics
            .run(state, &mut cm.transform, &mut cm.physics, ent_man);
        self.trans.run(state, &mut cm.transform, ent_man);
        self.render.run(state, &cm.transform, &cm.mesh);
        self.interface.run(state, &cm);
    }
}
