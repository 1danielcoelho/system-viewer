use web_sys::WebGlRenderingContext;

use crate::{
    app_state::AppState,
    managers::{EventManager, InputManager, ResourceManager, SceneManager, SystemManager},
};

pub struct Engine {
    pub res_man: ResourceManager,
    pub sys_man: SystemManager,
    pub event_man: EventManager,
    pub in_man: InputManager,
    pub scene_man: SceneManager,
}
impl Engine {
    pub fn new(gl: WebGlRenderingContext) -> Self {
        let new_world = Self {
            scene_man: SceneManager::new(),
            res_man: ResourceManager::new(gl),
            sys_man: SystemManager::new(),
            event_man: EventManager::new(),
            in_man: InputManager::new(),
        };

        return new_world;
    }

    pub fn update(&mut self, state: &mut AppState) {
        self.in_man.run(state);

        if let Some(scene_mut) = self.scene_man.get_main_scene_mut() {
            self.sys_man.run(state, &mut scene_mut.ent_man);
        }
    }
}
