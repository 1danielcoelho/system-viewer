use web_sys::WebGl2RenderingContext;

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
    pub fn new(gl: WebGl2RenderingContext) -> Self {
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

        // Prepare the interface man

        if let Some(scene_mut) = self.scene_man.get_main_scene_mut() {
            self.sys_man.run(state, &mut scene_mut.ent_man);
        }

        // Draw the interface after all systems added in their widgets
    }
}
