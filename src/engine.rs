use crate::app_state::AppState;
use crate::managers::scene::SceneManager;
use crate::managers::{
    EventManager, InputManager, InterfaceManager, ResourceManager, SystemManager,
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
    pub fn new() -> Self {
        let mut new_engine = Self {
            scene_man: SceneManager::new(),
            res_man: ResourceManager::new(),
            sys_man: SystemManager::new(),
            event_man: EventManager::new(),
            input_man: InputManager::new(),
            int_man: InterfaceManager::new(),
        };

        new_engine
            .scene_man
            .set_scene("empty", &mut new_engine.res_man, None);

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
        }

        // Draw the UI elements
        self.int_man.end_frame(state, &mut self.scene_man, &mut self.res_man);
    }

    pub fn receive_text(&mut self, url: &str, content_type: &str, text: &str) {
        match content_type {
            "scene_list" => self.receive_scene_list_text(url, text),
            "database" => self.receive_database_text(url, text),
            _ => log::error!(
                "Unexpected content_type for receive_text: '{}'. url: '{}'",
                content_type,
                url
            ),
        }
    }

    pub fn receive_scene_list_text(&mut self, url: &str, text: &str) {
        log::info!(
            "Loading scene descriptions from '{}' (length {})",
            url,
            text.len()
        );

        self.scene_man.receive_scene_description_vec(text).unwrap();
    }

    pub fn receive_database_text(&mut self, url: &str, text: &str) {
        log::info!(
            "Loading database file from '{}' (length {})",
            url,
            text.len()
        );

        self.res_man.load_database_file(url, text).unwrap();
    }

    pub fn receive_bytes(&mut self, url: &str, content_type: &str, data: &mut [u8]) {
        match content_type {
            "texture" => self.receive_texture_bytes(url, data),
            "glb_inject" => self.receive_gltf_bytes(url, data, true),
            "glb_resource" => self.receive_gltf_bytes(url, data, false),
            _ => log::error!(
                "Unexpected content_type for receive bytes: '{}'. url: '{}'",
                content_type,
                url
            ),
        }
    }

    pub fn receive_texture_bytes(&mut self, file_identifier: &str, data: &mut [u8]) {
        log::info!(
            "Loading texture from file '{}' ({} bytes)",
            file_identifier,
            data.len()
        );

        self.res_man.create_texture(file_identifier, data, None);
    }

    pub fn receive_gltf_bytes(&mut self, file_identifier: &str, data: &mut [u8], inject: bool) {
        log::info!(
            "Loading GLTF from file '{}' ({} bytes)",
            file_identifier,
            data.len()
        );

        // TODO: Catch duplicate scenes

        if let Ok((gltf_doc, gltf_buffers, gltf_images)) = gltf::import_slice(data) {
            self.res_man.load_textures_from_gltf(
                file_identifier,
                gltf_doc.textures(),
                &gltf_images,
            );

            let mat_index_to_parsed = self
                .res_man
                .load_materials_from_gltf(file_identifier, gltf_doc.materials());

            self.res_man.load_meshes_from_gltf(
                file_identifier,
                gltf_doc.meshes(),
                &gltf_buffers,
                &mat_index_to_parsed,
            );

            let loaded_scenes = self.scene_man.load_scenes_from_gltf(
                file_identifier,
                gltf_doc.scenes(),
                &self.res_man,
            );

            if inject {
                for scene in loaded_scenes {
                    self.scene_man
                        .inject_scene(&scene, None, &mut self.res_man)
                        .unwrap();
                }
            }
        }
    }
}
