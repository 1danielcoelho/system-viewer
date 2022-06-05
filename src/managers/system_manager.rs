use crate::app_state::AppState;
use crate::managers::scene::Scene;
use crate::managers::ResourceManager;
use crate::systems::{PhysicsSystem, RenderingSystem, TransformUpdateSystem};
use crate::GLCTX;

pub struct SystemManager {
    render: RenderingSystem,
    physics: PhysicsSystem,
    trans: TransformUpdateSystem,
}
impl SystemManager {
    pub fn new(res_man: &mut ResourceManager) -> Self {
        return Self {
            render: RenderingSystem::new(res_man),
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

    pub fn resize(&mut self, width: u32, height: u32) {
        GLCTX.with(|ctx| {
            self.render.resize(width, height, ctx);
        });
    }
}
