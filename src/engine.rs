use web_sys::WebGl2RenderingContext;

use crate::{
    app_state::AppState,
    managers::{
        EventManager, InputManager, InterfaceManager, ResourceManager, SceneManager, SystemManager,
    },
};

pub struct Engine {
    pub res_man: ResourceManager,
    pub sys_man: SystemManager,
    pub event_man: EventManager,
    pub input_man: InputManager,
    pub int_man: InterfaceManager,
    pub scene_man: SceneManager,
}
impl Engine {
    pub fn new(gl: WebGl2RenderingContext) -> Self {
        let new_world = Self {
            scene_man: SceneManager::new(),
            res_man: ResourceManager::new(gl),
            sys_man: SystemManager::new(),
            event_man: EventManager::new(),
            input_man: InputManager::new(),
            int_man: InterfaceManager::new(),
        };

        return new_world;
    }

    pub fn update(&mut self, state: &mut AppState) {
        self.input_man.run(state);

        // Startup the UI frame
        self.int_man.begin_frame(state);

        if let Some(scene_mut) = self.scene_man.get_main_scene_mut() {
            self.sys_man.run(state, &mut scene_mut.ent_man);
        }

        // Draw the interface after all systems added in their widgets
        self.int_man.end_frame();
    }
}
