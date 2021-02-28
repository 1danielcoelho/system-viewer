use std::borrow::BorrowMut;

use web_sys::console::info;

use crate::app_state::AppState;
use crate::fetch_text;
use crate::managers::scene::SceneManager;
use crate::managers::{
    EventManager, InputManager, InterfaceManager, ResourceManager, SystemManager,
};
use crate::STATE;

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
        let new_engine = Self {
            scene_man: SceneManager::new(),
            res_man: ResourceManager::new(),
            sys_man: SystemManager::new(),
            event_man: EventManager::new(),
            input_man: InputManager::new(),
            int_man: InterfaceManager::new(),
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
        }

        // Draw the UI elements
        self.int_man
            .end_frame(state, &mut self.scene_man, &mut self.res_man);
    }

    pub fn receive_text(&mut self, url: &str, content_type: &str, text: &str) {
        match content_type {
            "auto_load_manifest" => self.receive_auto_load_manifest_text(url, text),
            "scene" => self.receive_scene_text(url, text),
            "body_database" | "vectors_database" | "elements_database" => {
                self.receive_database_text(url, content_type, text)
            }
            _ => log::error!(
                "Unexpected content_type for receive_text: '{}'. url: '{}'",
                content_type,
                url
            ),
        }
    }

    fn receive_auto_load_manifest_text(&mut self, url: &str, text: &str) {
        log::info!(
            "Loading auto load manifest from '{}' (length {})",
            url,
            text.len()
        );

        let files: Vec<&str> = text.lines().collect();
        self.scene_man.num_scenes_expected = files.len() as u32;
        log::info!(
            "Expecting {} new scenes",
            self.scene_man.num_scenes_expected
        );

        for file in files.iter() {
            fetch_text(&("public/scenes/".to_owned() + file), "scene");
        }
    }

    fn receive_scene_text(&mut self, url: &str, text: &str) {
        log::info!("Loading scene from '{}' (length {})", url, text.len());

        self.scene_man.num_scenes_expected -= 1;
        self.scene_man.receive_serialized_scene(text);

        if self.scene_man.num_scenes_expected == 0 {
            STATE.with(|s| {
                if let Ok(mut ref_mut_s) = s.try_borrow_mut() {
                    let s = ref_mut_s.as_mut().unwrap();
                    let s_ref = s.borrow_mut();

                    let identifier = s_ref.last_scene_identifier.clone();

                    if !identifier.is_empty() {
                        self.scene_man
                            .set_scene(&identifier, &mut self.res_man, s_ref);
                    }
                }
            });
        }
    }

    fn receive_database_text(&mut self, url: &str, content_type: &str, text: &str) {
        log::info!(
            "Loading database file from '{}' (length {})",
            url,
            text.len()
        );

        self.res_man.load_database_file(url, content_type, text);
    }

    pub fn receive_bytes(&mut self, url: &str, content_type: &str, data: &mut [u8]) {
        match content_type {
            "cubemap_face" => self.res_man.receive_cubemap_face_file_bytes(url, data),
            "texture" => self.res_man.receive_texture_file_bytes(url, data),
            "glb_inject" => self.receive_gltf_bytes(url, data, true),
            "glb_resource" => self.receive_gltf_bytes(url, data, false),
            _ => log::error!(
                "Unexpected content_type for receive bytes: '{}'. url: '{}'",
                content_type,
                url
            ),
        }
    }

    fn receive_gltf_bytes(&mut self, file_identifier: &str, data: &mut [u8], inject: bool) {
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
