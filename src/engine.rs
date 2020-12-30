use crate::{
    app_state::AppState,
    components::{MeshComponent, TransformComponent},
    managers::{
        scene::SceneManager, EventManager, InputManager, InterfaceManager, ResourceManager,
        SystemManager,
    },
    utils::orbital_elements::{elements_to_circle_transform, OrbitalElements},
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
            .load_test_scene("test", &mut new_engine.res_man);
        new_engine
            .scene_man
            .set_scene("test", &mut new_engine.res_man);

        return new_engine;
    }

    pub fn update(&mut self, state: &mut AppState) {
        self.input_man.run(state);

        // Startup the UI frame, collecting UI elements
        self.int_man
            .begin_frame(state, &mut self.scene_man, &mut self.res_man);

        if let Some(scene) = self.scene_man.get_main_scene_mut() {
            // Run all systems
            self.sys_man.run(state, scene);

            // Draw the UI elements
            self.int_man.end_frame(state, scene);
        }
    }

    pub fn receive_text(&mut self, url: &str, content_type: &str, text: &str) {
        match content_type {
            "ephemerides" => self.receive_ephemerides_text(url, text),
            "scene" => {
                let scene = self.scene_man.deserialize_scene(text);
                let name = scene.unwrap().identifier.to_owned();

                self.scene_man.set_scene(&name, &mut self.res_man);
            }
            _ => log::error!(
                "Unexpected content_type for receive_text: '{}'. url: '{}'",
                content_type,
                url
            ),
        }
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

    fn receive_ephemerides_text(&mut self, file_name: &str, file_data: &str) {
        log::info!(
            "receive_ephemerides_text, name: {}, data: {}",
            file_name,
            file_data
        );

        // let (elements, body) = parse_ephemerides(file_data)?;
        // log::info!(
        //     "Loaded ephemerides '{}'\n{:#?}\n{:#?}",
        //     file_name,
        //     body,
        //     elements
        // );

        // let orbit_transform = elements_to_circle_transform(&elements);

        // let scene = self
        //
        //     .scene_man
        //     .new_scene(file_name)
        //     .ok_or("Failed to create new scene!")?;

        // let planet_mat = self.res_man.instantiate_material("gltf_metal_rough");
        // planet_mat.as_ref().unwrap().borrow_mut().name = String::from("planet_mat");
        // planet_mat.as_ref().unwrap().borrow_mut().set_uniform_value(
        //     UniformName::BaseColorFactor,
        //     UniformValue::Vec4([0.1, 0.8, 0.2, 1.0]),
        // );

        // // Lat-long sphere
        // let lat_long = scene.new_entity(Some(&body.id));
        // let trans_comp = scene
        //
        //     .add_component::<TransformComponent>(lat_long)
        //     .unwrap();
        // trans_comp.get_local_transform_mut().trans = Vector3::new(10.0, 0.0, 0.0);
        // trans_comp.get_local_transform_mut().scale = Vector3::new(
        //     body.mean_radius as f32,
        //     body.mean_radius as f32,
        //     body.mean_radius as f32,
        // );
        // let mesh_comp = scene
        //
        //     .add_component::<MeshComponent>(lat_long)
        //     .unwrap();
        // mesh_comp.set_mesh(self.res_man.get_or_create_mesh("lat_long_sphere"));
        // mesh_comp.set_material_override(planet_mat.clone(), 0);

        // self.temp_add_ellipse(
        //     file_name,
        //     "first",
        //     &OrbitalElements {
        //         semi_major_axis: 1000.0,
        //         eccentricity: 0.0,
        //         arg_periapsis: 0.0,
        //         inclination: 0.0,
        //         long_asc_node: 0.0,
        //         true_anomaly: 0.0,
        //     },
        // );

        // self.temp_add_ellipse(
        //     file_name,
        //     "second",
        //     &OrbitalElements {
        //         semi_major_axis: 1000.0,
        //         eccentricity: 0.9,
        //         arg_periapsis: 0.0,
        //         inclination: 0.0,
        //         long_asc_node: 0.0,
        //         true_anomaly: 0.0,
        //     },
        // );

        // self.temp_add_ellipse(
        //     file_name,
        //     "third",
        //     &OrbitalElements {
        //         semi_major_axis: 1000.0,
        //         eccentricity: 0.9,
        //         arg_periapsis: 0.0,
        //         inclination: 30.0,
        //         long_asc_node: 0.0,
        //         true_anomaly: 0.0,
        //     },
        // );

        // self.temp_add_ellipse(
        //     file_name,
        //     "third",
        //     &OrbitalElements {
        //         semi_major_axis: 1000.0,
        //         eccentricity: 0.9,
        //         arg_periapsis: 0.0,
        //         inclination: 30.0,
        //         long_asc_node: 45.0,
        //         true_anomaly: 0.0,
        //     },
        // );

        // self.temp_add_ellipse(
        //     file_name,
        //     "fourth",
        //     &OrbitalElements {
        //         semi_major_axis: 1000.0,
        //         eccentricity: 0.9,
        //         arg_periapsis: 30.0,
        //         inclination: 30.0,
        //         long_asc_node: 45.0,
        //         true_anomaly: 0.0,
        //     },
        // );
    }

    fn temp_add_ellipse(&mut self, scene_name: &str, name: &str, elements: &OrbitalElements) {
        let scene = self.scene_man.get_scene_mut(scene_name).unwrap();

        let orbit_transform = elements_to_circle_transform(&elements);
        log::warn!("orbit transform: {:#?}", orbit_transform);

        // Orbit
        let circle = scene.new_entity(Some(&name));
        let trans_comp = scene.add_component::<TransformComponent>(circle).unwrap();
        *trans_comp.get_local_transform_mut() = orbit_transform;
        let mesh_comp = scene.add_component::<MeshComponent>(circle).unwrap();
        mesh_comp.set_mesh(self.res_man.get_or_create_mesh("circle"));
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
