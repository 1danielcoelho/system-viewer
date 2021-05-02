use crate::app_state::AppState;
use crate::managers::scene::Scene;
use crate::managers::ResourceManager;
use crate::systems::{OrbitalSystem, PhysicsSystem, RenderingSystem, TransformUpdateSystem};
use crate::GLCTX;

pub struct SystemManager {
    render: RenderingSystem,
    orbital: OrbitalSystem,
    physics: PhysicsSystem,
    trans: TransformUpdateSystem,
}
impl SystemManager {
    pub fn new(res_man: &mut ResourceManager) -> Self {
        return Self {
            render: RenderingSystem::new(res_man),
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

    pub fn resize(&mut self, width: u32, height: u32) {
        GLCTX.with(|ctx| {
            let ref_mut = ctx.borrow_mut();
            let ctx = ref_mut.as_ref().unwrap();

            self.render.resize(width, height, ctx);
        });
    }
}
