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

        // TODO: Figure out how to do this using the same ent_man ref...

        // Startup the UI frame
        self.int_man.begin_frame(
            state,
            self.scene_man
                .get_main_scene_mut()
                .and_then(|s| Some(&mut s.ent_man)),
        );

        let ent_man = self
            .scene_man
            .get_main_scene_mut()
            .and_then(|s| Some(&mut s.ent_man));

        // Run all systems if we have a scene
        if let Some(ent_man) = ent_man {
            self.sys_man.run(state, ent_man);
        }

        // Draw the UI, also handling mouse interaction if we have a scene
        self.int_man.end_frame(
            state,
            self.scene_man
                .get_main_scene_mut()
                .and_then(|s| Some(&mut s.ent_man)),
        );
    }
}
