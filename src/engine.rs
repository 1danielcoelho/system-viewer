use crate::app_state::AppState;
use crate::managers::scene::SceneManager;
use crate::managers::{
    EventManager, InputManager, InterfaceManager, OrbitManager, ResourceManager, SystemManager,
};
use crate::STATE;

pub struct Engine {
    pub res_man: ResourceManager,
    pub sys_man: SystemManager,
    pub event_man: EventManager,
    pub input_man: InputManager,
    pub int_man: InterfaceManager,
    pub scene_man: SceneManager,
    pub orbit_man: OrbitManager,
}
impl Engine {
    pub fn new() -> Self {
        let mut res_man = ResourceManager::new();
        let sys_man = SystemManager::new(&mut res_man);

        let new_engine = Self {
            scene_man: SceneManager::new(),
            res_man,
            sys_man,
            event_man: EventManager::new(),
            input_man: InputManager::new(),
            int_man: InterfaceManager::new(),
            orbit_man: OrbitManager::new(),
        };

        return new_engine;
    }

    pub fn update(&mut self, state: &mut AppState) {
        // Startup the UI frame, collecting UI elements
        self.int_man.begin_frame(state);

        // Run the input manager after begin frame to allow the UI a change to intercept input
        self.input_man.run(state);

        if let Some(scene) = self.scene_man.get_main_scene_mut() {
            // Run all systems
            self.sys_man.run(state, scene);

            // Update serializable state
            // TODO: Find a better place for this?
            state.camera.reference_entity_name = state
                .camera
                .reference_entity
                .and_then(|e| scene.get_entity_name(e))
                .and_then(|s| Some(s.to_owned()));
        }

        // Draw the UI elements
        self.int_man.end_frame(
            state,
            &mut self.scene_man,
            &mut self.res_man,
            &self.orbit_man,
        );
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.sys_man.resize(width, height);
    }

    pub fn receive_text(&mut self, url: &str, content_type: &str, text: &str) {
        match content_type {
            "scene" => self.receive_scene_text(url, text),
            "body_database" | "vectors_database" | "elements_database" => {
                self.receive_database_text(url, content_type, text)
            }
            _ => log::error!(
                "Can't handle text content_type '{}' from url: '{}'",
                content_type,
                url
            ),
        }
    }

    fn receive_scene_text(&mut self, url: &str, text: &str) {
        log::info!("Loading scene from '{}' (length {})", url, text.len());

        self.scene_man.receive_serialized_scene(text);
        self.try_loading_last_scene();
    }

    fn try_loading_last_scene(&mut self) {
        log::info!("Engine has no pending resources. Loading last scene...");

        STATE.with(|s| {
            if let Ok(mut ref_mut_s) = s.try_borrow_mut() {
                let s = ref_mut_s.as_mut().unwrap();

                self.scene_man
                    .load_last_scene(&mut self.res_man, &self.orbit_man, s);
            }
        });
    }

    fn receive_database_text(&mut self, url: &str, content_type: &str, text: &str) {
        log::info!(
            "Loading database file from '{}' (length {})",
            url,
            text.len()
        );

        self.orbit_man.load_database_file(url, content_type, text);
    }

    pub fn receive_bytes(&mut self, url: &str, content_type: &str, data: &mut [u8]) {
        match content_type {
            "cubemap_face" => self.res_man.receive_cubemap_face_file_bytes(url, data),
            "texture" => self.res_man.receive_texture_file_bytes(url, data),
            "gltf" => self.receive_gltf_bytes(url, data),
            _ => log::error!(
                "Can't handle bytes content_type '{}' from url: '{}'",
                content_type,
                url
            ),
        }
    }

    fn receive_gltf_bytes(&mut self, file_identifier: &str, data: &mut [u8]) {
        log::info!(
            "Loading GLTF from file '{}' ({} bytes)",
            file_identifier,
            data.len()
        );

        if let Ok((gltf_doc, gltf_buffers, gltf_images)) = gltf::import_slice(data) {
            self.res_man
                .load_gltf_data(file_identifier, &gltf_doc, &gltf_buffers, &gltf_images);
        }
    }
}
