use super::ResourceManager;
use crate::app_state::{AppState, ReferenceChange};
use crate::components::light::LightType;
use crate::components::{LightComponent, MeshComponent, PhysicsComponent, TransformComponent};
use crate::managers::orbit::{BodyDescription, BodyInstanceDescription, StateVector};
use crate::managers::resource::material::{UniformName, UniformValue};
use crate::managers::resource::texture::TextureUnit;
use crate::managers::scene::description::SceneDescription;
use crate::managers::scene::orbits::{add_body_instance_entities, fetch_default_motion_if_needed};
use crate::managers::OrbitManager;
use crate::utils::orbits::OBLIQUITY_OF_ECLIPTIC;
use crate::utils::string::get_unique_name;
use crate::utils::units::J2000_JDN;
use crate::STATE;
use na::*;
pub use scene::*;
use std::collections::HashMap;

pub mod component_storage;
pub mod description;
pub mod orbits;
mod scene;

pub struct SceneManager {
    main: Option<String>,
    loaded_scenes: HashMap<String, Scene>,
    pub descriptions: HashMap<String, SceneDescription>,

    // If this is larger than zero we have scene fetch requests
    // in flight, and may receive a new scene any second
    pub num_scenes_expected: u32,
}
impl SceneManager {
    pub fn new() -> Self {
        let mut new_man = Self {
            main: None,
            loaded_scenes: HashMap::new(),
            descriptions: HashMap::new(),
            num_scenes_expected: 0,
        };

        // Don't actually call set_scene here as we don't want to trigger all that stuff
        new_man.new_scene("empty");
        new_man.main = Some(String::from("empty"));

        return new_man;
    }

    /// For SceneManager usage. Consumers should just `set_scene` instead, and the scenes
    /// will be created automatically when needed
    pub(super) fn new_scene(&mut self, scene_name: &str) -> Option<&mut Scene> {
        let unique_scene_name = get_unique_name(scene_name, &self.loaded_scenes);

        let scene = Scene::new(&unique_scene_name);
        log::info!("Created scene '{}'", unique_scene_name);

        assert!(!self.loaded_scenes.contains_key(&unique_scene_name));

        self.loaded_scenes
            .insert(unique_scene_name.to_string(), scene);
        let scene_in_storage = self.loaded_scenes.get_mut(&unique_scene_name);

        return scene_in_storage;
    }

    /// Sets the current scene to one with `identifier`. Will load/create the scene
    /// if we can/know how to. This includes constructing new `Scene`s from `SceneDescription`s
    pub fn set_scene(
        &mut self,
        identifier: &str,
        res_man: &mut ResourceManager,
        orbit_man: &OrbitManager,
        state: &mut AppState,
    ) {
        if let Some(main) = &self.main {
            if main.as_str() == identifier {
                return;
            }
        };

        if !self.loaded_scenes.contains_key(identifier) {
            self.load_scene(identifier, res_man, orbit_man, state);
        }

        // Don't start resetting/unloading stuff if we have nowhere else to go
        // If this failed to load I'm sure we'll have another error message about it too
        if self.get_scene(identifier).is_none() {
            return;
        }

        // Discard previous scene and reset state
        if let Some(main) = self.main.clone() {
            self.delete_scene(&main);
        }
        state.selection = None;
        state.hovered = None;

        // Set new scene
        self.main = Some(identifier.to_string());
        let main_scene = self.get_main_scene().unwrap();

        // Check if we have a description for that scene (they should have same name)
        if let Some(desc) = self.descriptions.get(identifier) {
            state.simulation_speed = desc.simulation_scale;

            // Camera target (already wrt. reference)
            // If the identifier is the same as last_scene we're loading from state, so keep our camera transform
            let mut need_go_to: bool = false;
            if state.last_scene_identifier.as_str() != identifier {
                if desc.camera_pos.is_some()
                    && desc.camera_target.is_some()
                    && desc.camera_up.is_some()
                {
                    state.camera.pos = desc.camera_pos.unwrap();
                    state.camera.up = desc.camera_up.unwrap();
                    state.camera.target = desc.camera_target.unwrap();
                } else {
                    need_go_to = true;
                }

                if let Some(focus) = &desc.focus {
                    // Ugh.. this shouldn't be too often though
                    for (entity, component) in main_scene.metadata.iter() {
                        if let Some(id) = component.get_metadata("body_id") {
                            if id == focus {
                                state.camera.next_reference_entity =
                                    Some(ReferenceChange::FocusKeepCoords(*entity));

                                if need_go_to {
                                    state.camera.entity_going_to = Some(*entity);
                                }

                                log::info!(
                                    "Setting initial focused entity to '{:?}'",
                                    main_scene.get_entity_name(*entity)
                                );
                                break;
                            }
                        }
                    }
                }
            }

            // TODO: Proper handling of time (would involve rolling simulation/orbits forward/back
            // to match target time, for now let's just all sit at J2000)
            assert!(desc.time == String::from("J2000"));
            state.sim_time_s = 0.0;
        }

        // Do this last because we'll check whether we should goto something or keep our state
        // by comparing against this
        state.last_scene_identifier = identifier.to_string();
    }

    fn load_scene(
        &mut self,
        identifier: &str,
        res_man: &mut ResourceManager,
        orbit_man: &OrbitManager,
        state: &mut AppState,
    ) -> &mut Scene {
        match identifier {
            "test" => self.load_test_scene(res_man, state),
            "teal" => self.load_teal_sphere_scene(res_man),
            "planetarium" => self.load_planetarium_scene(res_man),
            _ => self.load_scene_from_desc(identifier, res_man, orbit_man, state),
        }
    }

    pub fn load_last_scene(
        &mut self,
        res_man: &mut ResourceManager,
        orbit_man: &OrbitManager,
        state: &mut AppState,
    ) {
        let identifier = state.last_scene_identifier.clone();

        if !identifier.is_empty() {
            if self.descriptions.contains_key(&identifier) {
                log::info!("Trying to load last scene '{}'", identifier);
                self.set_scene(&identifier, res_man, orbit_man, state);
            } else {
                log::warn!(
                    "Failed to find a description for last loaded scene '{}'",
                    identifier
                );
            }
        }
    }

    /// Completely unloads the scene with `identifier`. The next time `set_scene` is called with
    /// `identifier`, it may need to be re-constructed
    pub fn delete_scene(&mut self, identifier: &str) {
        // Never delete the empty scene as that is our "fallback" scene
        if identifier == "empty" {
            return;
        }

        if let Some(main_name) = self.main.as_ref() {
            if main_name == identifier {
                self.main = Some(String::from("empty"));
            }
        }

        log::info!(
            "Unloading scene '{}'. Scene is now '{:?}'",
            identifier,
            self.main
        );

        self.loaded_scenes.remove(identifier);
    }

    pub fn get_main_scene(&self) -> Option<&Scene> {
        match self.main.as_ref() {
            Some(ident_ref) => self.get_scene(&ident_ref),
            None => None,
        }
    }

    pub fn get_main_scene_mut(&mut self) -> Option<&mut Scene> {
        match self.main.as_ref() {
            Some(ident_ref) => self.loaded_scenes.get_mut(ident_ref),
            None => None,
        }
    }

    pub fn get_scene(&self, identifier: &str) -> Option<&Scene> {
        return self.loaded_scenes.get(identifier);
    }

    pub fn get_scene_mut(&mut self, identifier: &str) -> Option<&mut Scene> {
        return self.loaded_scenes.get_mut(identifier);
    }

    pub fn receive_serialized_scene(&mut self, serialized: &str) {
        let new_desc: Result<SceneDescription, String> = ron::de::from_str(serialized)
            .map_err(|e| format!("RON deserialization error:\n{}", e).to_owned());
        if let Err(e) = new_desc {
            log::error!("{}", e);
            return;
        }
        let new_desc = new_desc.unwrap();

        log::info!("Loaded new scene description '{}'", new_desc.name);

        let name = new_desc.name.clone();
        self.descriptions.insert(name, new_desc);
    }

    fn load_teal_sphere_scene(&mut self, res_man: &mut ResourceManager) -> &mut Scene {
        let scene = self.new_scene("teal_sphere").unwrap();

        // Floor
        let planet = scene.new_entity(Some("floor"));
        let trans_comp = scene.add_component::<TransformComponent>(planet);
        trans_comp.get_local_transform_mut().trans = Vector3::new(0.0, 0.0, -2.0);
        trans_comp.get_local_transform_mut().scale = Vector3::new(50.0, 50.0, 1.0);
        let mesh_comp = scene.add_component::<MeshComponent>(planet);
        mesh_comp.set_mesh(res_man.get_or_create_mesh("plane"));
        mesh_comp.set_material_override(res_man.get_or_create_material("phong"), 0);

        // Cube
        let planet = scene.new_entity(Some("cube"));
        let trans_comp = scene.add_component::<TransformComponent>(planet);
        trans_comp.get_local_transform_mut().trans = Vector3::new(0.0, 0.0, 0.0);
        let mesh_comp = scene.add_component::<MeshComponent>(planet);
        mesh_comp.set_mesh(res_man.get_or_create_mesh("ico_sphere"));
        mesh_comp.set_material_override(res_man.get_or_create_material("phong"), 0);

        // Light material
        let light_color: [f32; 3] = [0.0, 1.0, 1.0];
        let light_mat = res_man.instantiate_material("gltf_metal_rough", "mega_sun_mat");
        light_mat
            .as_ref()
            .unwrap()
            .borrow_mut()
            .set_uniform_value(UniformName::EmissiveFactor, UniformValue::Vec3(light_color));

        // Light center
        let light_center = scene.new_entity(Some("light_center"));
        scene.add_component::<TransformComponent>(light_center);
        let physics = scene.add_component::<PhysicsComponent>(light_center);
        physics.ang_mom = Vector3::new(0.0, 0.0, 300.0);

        // Light
        let light_ent = scene.new_entity(Some("Mega-sun"));
        let trans_comp = scene.add_component::<TransformComponent>(light_ent);
        trans_comp.get_local_transform_mut().trans = Vector3::new(5.0, 0.0, 0.0);
        trans_comp.get_local_transform_mut().scale = Vector3::new(0.2, 0.2, 0.2);
        let mesh_comp = scene.add_component::<MeshComponent>(light_ent);
        mesh_comp.set_mesh(res_man.get_or_create_mesh("lat_long_sphere"));
        mesh_comp.set_material_override(light_mat.clone(), 0);
        let light_comp = scene.add_component::<LightComponent>(light_ent);
        light_comp.color = Vector3::new(light_color[0], light_color[1], light_color[2]);
        light_comp.intensity = 100000.0;
        light_comp.light_type = LightType::Point;
        scene.set_entity_parent(light_center, light_ent);

        // Axes
        let axes = scene.new_entity(Some("axes"));
        let trans_comp = scene.add_component::<TransformComponent>(axes);
        trans_comp.get_local_transform_mut().scale = Vector3::new(10.0, 10.0, 10.0);
        let mesh_comp = scene.add_component::<MeshComponent>(axes);
        mesh_comp.set_mesh(res_man.get_or_create_mesh("axes"));

        return scene;
    }

    fn load_test_scene(
        &mut self,
        res_man: &mut ResourceManager,
        state: &mut AppState,
    ) -> &mut Scene {
        let scene = self.new_scene("test").unwrap();

        if state.use_skyboxes {
            scene.skybox_mesh = res_man.get_or_create_mesh("quad");
            scene.skybox_trans = Some(
                Rotation3::<f64>::from_axis_angle(
                    &Vector3::x_axis(),
                    OBLIQUITY_OF_ECLIPTIC.to_rad().0,
                )
                .to_homogeneous(),
            );
            scene.skybox_mat = res_man.instantiate_material("skybox", "test_skybox");
            scene.skybox_mat.as_ref().unwrap().borrow_mut().set_texture(
                TextureUnit::BaseColor,
                res_man.get_or_request_texture("starmap_16k", true),
            );
        }

        let sun_color: [f32; 3] = [1.0, 1.0, 0.8];
        let sun_mat = res_man.instantiate_material("gltf_metal_rough", "vert_sun_mat");
        sun_mat
            .as_ref()
            .unwrap()
            .borrow_mut()
            .set_uniform_value(UniformName::EmissiveFactor, UniformValue::Vec3(sun_color));

        let sun_color2: [f32; 3] = [1.0, 0.3, 0.1];
        let sun_mat2 = res_man.instantiate_material("gltf_metal_rough", "vert_sun_mat");
        sun_mat2
            .as_ref()
            .unwrap()
            .borrow_mut()
            .set_uniform_value(UniformName::EmissiveFactor, UniformValue::Vec3(sun_color2));

        let ent = scene.new_entity(Some("sun"));
        let trans_comp = scene.add_component::<TransformComponent>(ent);
        trans_comp.get_local_transform_mut().trans = Vector3::new(0.0, 0.0, 5.0);
        trans_comp.get_local_transform_mut().scale = Vector3::new(0.1, 0.1, 0.1);
        let mesh_comp = scene.add_component::<MeshComponent>(ent);
        mesh_comp.set_mesh(res_man.get_or_create_mesh("lat_long_sphere"));
        mesh_comp.set_material_override(sun_mat.clone(), 0);
        let light_comp = scene.add_component::<LightComponent>(ent);
        light_comp.color = Vector3::new(sun_color[0], sun_color[1], sun_color[2]);
        light_comp.intensity = 20.0;
        light_comp.light_type = LightType::Point;

        let ent = scene.new_entity(Some("sun2"));
        let trans_comp = scene.add_component::<TransformComponent>(ent);
        trans_comp.get_local_transform_mut().trans = Vector3::new(0.0, 0.0, -5.0);
        trans_comp.get_local_transform_mut().scale = Vector3::new(0.1, 0.1, 0.1);
        let mesh_comp = scene.add_component::<MeshComponent>(ent);
        mesh_comp.set_mesh(res_man.get_or_create_mesh("lat_long_sphere"));
        mesh_comp.set_material_override(sun_mat2.clone(), 0);
        let light_comp = scene.add_component::<LightComponent>(ent);
        light_comp.color = Vector3::new(sun_color2[0], sun_color2[1], sun_color2[2]);
        light_comp.intensity = 15.0;
        light_comp.light_type = LightType::Point;

        let ent = scene.new_entity(Some("sphere"));
        let trans_comp = scene.add_component::<TransformComponent>(ent);
        trans_comp.get_local_transform_mut().trans = Vector3::new(0.0, 0.0, 0.0);
        trans_comp.get_local_transform_mut().scale = Vector3::new(1.0, 1.0, 1.0);
        let mesh_comp = scene.add_component::<MeshComponent>(ent);
        mesh_comp.set_mesh(res_man.get_or_create_mesh("ico_sphere"));
        mesh_comp.set_material_override(res_man.get_or_create_material("phong"), 0);

        let ent = scene.new_entity(Some("cube"));
        let trans_comp = scene.add_component::<TransformComponent>(ent);
        trans_comp.get_local_transform_mut().trans = Vector3::new(5.0, 0.0, 0.0);
        trans_comp.get_local_transform_mut().scale = Vector3::new(1.0, 1.0, 1.0);
        let mesh_comp = scene.add_component::<MeshComponent>(ent);
        mesh_comp.set_mesh(res_man.get_or_create_mesh("cube"));
        mesh_comp.set_material_override(res_man.get_or_create_material("phong"), 0);

        // let mut rng = rand::thread_rng();
        // for _ in 0..500 {
        //     let ent = scene.new_entity(Some("box"));

        //     let pos = Vector3::new(rng.gen_range(-10.0..10.0), rng.gen_range(-10.0..10.0), 0.0);

        //     let trans_comp = scene.add_component::<TransformComponent>(ent);
        //     trans_comp.get_local_transform_mut().trans = pos;
        //     trans_comp.get_local_transform_mut().scale = Vector3::new(0.1, 0.1, 0.1);
        //     let mesh_comp = scene.add_component::<MeshComponent>(ent);
        //     mesh_comp.set_mesh(res_man.get_or_create_mesh("ico_sphere"));
        //     mesh_comp.set_material_override(res_man.get_or_create_material("phong"), 0);
        //     let phys_comp = scene.add_component::<PhysicsComponent>(ent);
        //     phys_comp.mass = 1E20;
        //     phys_comp.force_sum = pos
        //         .cross(&Vector3::z())
        //         .scale(1E13 * rng.gen_range(0.0..10.0));
        // }

        // Grid
        if state.show_grid {
            let grid = scene.new_entity(Some("grid"));
            let trans_comp = scene.add_component::<TransformComponent>(grid);
            trans_comp.get_local_transform_mut().scale = Vector3::new(10.0, 10.0, 10.0);
            let mesh_comp = scene.add_component::<MeshComponent>(grid);
            mesh_comp.set_mesh(res_man.get_or_create_mesh("grid"));
        }

        // Axes
        if state.show_axes {
            let axes = scene.new_entity(Some("axes"));
            let trans_comp = scene.add_component::<TransformComponent>(axes);
            trans_comp.get_local_transform_mut().trans = Vector3::new(0.001, 0.001, 0.001);
            trans_comp.get_local_transform_mut().scale = Vector3::new(5.0, 5.0, 5.0);
            let mesh_comp = scene.add_component::<MeshComponent>(axes);
            mesh_comp.set_mesh(res_man.get_or_create_mesh("axes"));
        }

        return scene;
    }

    fn load_planetarium_scene(&mut self, res_man: &mut ResourceManager) -> &mut Scene {
        let scene = self.new_scene("planetarium").unwrap();

        let sun_mat = res_man.instantiate_material("gltf_metal_rough", "sun_mat");
        sun_mat.as_ref().unwrap().borrow_mut().set_uniform_value(
            UniformName::EmissiveFactor,
            UniformValue::Vec3([1.0, 0.7, 0.0]),
        );

        // Sun
        let sun = scene.new_entity(Some("sun"));
        let trans_comp = scene.add_component::<TransformComponent>(sun);
        trans_comp.get_local_transform_mut().trans = Vector3::new(0.0, 0.0, 0.0);
        trans_comp.get_local_transform_mut().scale = Vector3::new(1.0, 1.0, 1.0);
        let mesh_comp = scene.add_component::<MeshComponent>(sun);
        mesh_comp.set_mesh(res_man.get_or_create_mesh("lat_long_sphere"));
        mesh_comp.set_material_override(sun_mat.clone(), 0);
        let light_comp = scene.add_component::<LightComponent>(sun);
        light_comp.color = Vector3::new(1.0, 1.0, 1.0);
        light_comp.intensity = 10000000.0;
        light_comp.light_type = LightType::Point;

        let planet_mat = res_man.instantiate_material("gltf_metal_rough", "planet_mat");
        planet_mat.as_ref().unwrap().borrow_mut().set_uniform_value(
            UniformName::BaseColorFactor,
            UniformValue::Vec4([0.1, 0.8, 0.2, 1.0]),
        );

        // System center so the sun doesn't have to rotate for the planet to orbit
        let sun_bary = scene.new_entity(Some("sun_barycenter"));
        scene.add_component::<TransformComponent>(sun_bary);
        let physics = scene.add_component::<PhysicsComponent>(sun_bary);
        physics.ang_mom = Vector3::new(0.0, 0.0, 470.0);

        // Planet
        let planet = scene.new_entity(Some("planet"));
        let trans_comp = scene.add_component::<TransformComponent>(planet);
        trans_comp.get_local_transform_mut().trans = Vector3::new(10.0, 0.0, 0.0);
        let mesh_comp = scene.add_component::<MeshComponent>(planet);
        mesh_comp.set_mesh(res_man.get_or_create_mesh("lat_long_sphere"));
        mesh_comp.set_material_override(planet_mat.clone(), 0);
        scene.set_entity_parent(sun_bary, planet);

        // Planet orbit
        let planet_orbit = scene.new_entity(Some("orbit"));
        let trans_comp = scene.add_component::<TransformComponent>(planet_orbit);
        trans_comp.get_local_transform_mut().trans = Vector3::new(0.0, 0.0, 0.0);
        trans_comp.get_local_transform_mut().scale = Vector3::new(10.0, 10.0, 10.0);
        let mesh_comp = scene.add_component::<MeshComponent>(planet_orbit);
        mesh_comp.set_mesh(res_man.get_or_create_mesh("circle"));

        // Moon orbit center
        let planet_bary = scene.new_entity(Some("center"));
        scene.add_component::<TransformComponent>(planet_bary);
        let physics = scene.add_component::<PhysicsComponent>(planet_bary);
        physics.ang_mom = Vector3::new(0.0, 0.0, 720.0);
        scene.set_entity_parent(planet, planet_bary);

        let moon_mat = res_man.instantiate_material("gltf_metal_rough", "moon_mat");
        moon_mat.as_ref().unwrap().borrow_mut().set_uniform_value(
            UniformName::BaseColorFactor,
            UniformValue::Vec4([0.8, 0.8, 0.7, 1.0]),
        );

        // Moon
        let moon = scene.new_entity(Some("moon"));
        let trans_comp = scene.add_component::<TransformComponent>(moon);
        trans_comp.get_local_transform_mut().trans = Vector3::new(3.0, 0.0, 0.0);
        trans_comp.get_local_transform_mut().scale = Vector3::new(0.2, 0.2, 0.2);
        let mesh_comp = scene.add_component::<MeshComponent>(moon);
        mesh_comp.set_mesh(res_man.get_or_create_mesh("lat_long_sphere"));
        mesh_comp.set_material_override(moon_mat.clone(), 0);
        scene.set_entity_parent(planet_bary, moon);

        // Moon orbit
        let moon_orbit = scene.new_entity(Some("orbit"));
        let trans_comp = scene.add_component::<TransformComponent>(moon_orbit);
        trans_comp.get_local_transform_mut().trans = Vector3::new(0.0, 0.0, 0.0);
        trans_comp.get_local_transform_mut().scale = Vector3::new(3.0, 3.0, 3.0);
        let mesh_comp = scene.add_component::<MeshComponent>(moon_orbit);
        mesh_comp.set_mesh(res_man.get_or_create_mesh("circle"));
        scene.set_entity_parent(planet_bary, moon_orbit);

        // Outer system center
        let outer_bary = scene.new_entity(Some("outer_barycenter"));
        scene.add_component::<TransformComponent>(outer_bary);
        let physics = scene.add_component::<PhysicsComponent>(outer_bary);
        physics.ang_mom = Vector3::new(0.0, 0.0, -80.0);

        // Outer planet 1
        let mat = res_man.instantiate_material("gltf_metal_rough", "outer_mat_1");
        mat.as_ref().unwrap().borrow_mut().set_uniform_value(
            UniformName::BaseColorFactor,
            UniformValue::Vec4([0.8, 0.9, 0.5, 1.0]),
        );
        let planet = scene.new_entity(Some("planet"));
        let trans_comp = scene.add_component::<TransformComponent>(planet);
        trans_comp.get_local_transform_mut().trans = Vector3::new(30.0, 0.0, 0.0);
        let mesh_comp = scene.add_component::<MeshComponent>(planet);
        mesh_comp.set_mesh(res_man.get_or_create_mesh("lat_long_sphere"));
        mesh_comp.set_material_override(mat.clone(), 0);
        scene.set_entity_parent(outer_bary, planet);

        // Outer planet 2
        let mat = res_man.instantiate_material("gltf_metal_rough", "outer_mat_2");
        mat.as_ref().unwrap().borrow_mut().set_uniform_value(
            UniformName::BaseColorFactor,
            UniformValue::Vec4([0.9, 0.6, 0.8, 1.0]),
        );
        let planet = scene.new_entity(Some("planet"));
        let trans_comp = scene.add_component::<TransformComponent>(planet);
        trans_comp.get_local_transform_mut().trans = Vector3::new(0.0, 30.0, 0.0);
        let mesh_comp = scene.add_component::<MeshComponent>(planet);
        mesh_comp.set_mesh(res_man.get_or_create_mesh("lat_long_sphere"));
        mesh_comp.set_material_override(mat.clone(), 0);
        scene.set_entity_parent(outer_bary, planet);

        // Outer planet 3
        let mat = res_man.instantiate_material("gltf_metal_rough", "outer_mat_3");
        mat.as_ref().unwrap().borrow_mut().set_uniform_value(
            UniformName::BaseColorFactor,
            UniformValue::Vec4([0.9, 0.9, 0.9, 1.0]),
        );
        let planet = scene.new_entity(Some("planet"));
        let trans_comp = scene.add_component::<TransformComponent>(planet);
        trans_comp.get_local_transform_mut().trans = Vector3::new(0.0, -30.0, 0.0);
        let mesh_comp = scene.add_component::<MeshComponent>(planet);
        mesh_comp.set_mesh(res_man.get_or_create_mesh("lat_long_sphere"));
        mesh_comp.set_material_override(mat.clone(), 0);
        scene.set_entity_parent(outer_bary, planet);

        // Outer planet 4
        let mat = res_man.instantiate_material("gltf_metal_rough", "outer_mat_4");
        mat.as_ref().unwrap().borrow_mut().set_uniform_value(
            UniformName::BaseColorFactor,
            UniformValue::Vec4([0.2, 0.3, 0.1, 1.0]),
        );
        let planet = scene.new_entity(Some("planet"));
        let trans_comp = scene.add_component::<TransformComponent>(planet);
        trans_comp.get_local_transform_mut().trans = Vector3::new(-30.0, 0.0, 0.0);
        let mesh_comp = scene.add_component::<MeshComponent>(planet);
        mesh_comp.set_mesh(res_man.get_or_create_mesh("lat_long_sphere"));
        mesh_comp.set_material_override(mat.clone(), 0);
        scene.set_entity_parent(outer_bary, planet);

        // Outer orbit
        let outer_orbit = scene.new_entity(Some("orbit"));
        let trans_comp = scene.add_component::<TransformComponent>(outer_orbit);
        trans_comp.get_local_transform_mut().trans = Vector3::new(0.0, 0.0, 0.0);
        trans_comp.get_local_transform_mut().scale = Vector3::new(30.0, 30.0, 30.0);
        let mesh_comp = scene.add_component::<MeshComponent>(outer_orbit);
        mesh_comp.set_mesh(res_man.get_or_create_mesh("circle"));

        let counter_sun_color: [f32; 3] = [0.0, 0.0, 1.0];
        let counter_sun_mat = res_man.instantiate_material("gltf_metal_rough", "counter_sun_mat");
        counter_sun_mat
            .as_ref()
            .unwrap()
            .borrow_mut()
            .set_uniform_value(
                UniformName::EmissiveFactor,
                UniformValue::Vec3(counter_sun_color),
            );

        // Counter-sun barycenter
        let counter_sun_bary = scene.new_entity(Some("counter_sun_barycenter"));
        scene.add_component::<TransformComponent>(counter_sun_bary);
        let physics = scene.add_component::<PhysicsComponent>(counter_sun_bary);
        physics.ang_mom = Vector3::new(0.0, 0.0, -75.0);

        // Counter-sun
        let counter_sun = scene.new_entity(Some("Counter-sun"));
        let trans_comp = scene.add_component::<TransformComponent>(counter_sun);
        trans_comp.get_local_transform_mut().trans = Vector3::new(15.0, 0.0, 0.0);
        let mesh_comp = scene.add_component::<MeshComponent>(counter_sun);
        mesh_comp.set_mesh(res_man.get_or_create_mesh("lat_long_sphere"));
        mesh_comp.set_material_override(counter_sun_mat.clone(), 0);
        let light_comp = scene.add_component::<LightComponent>(counter_sun);
        light_comp.color = Vector3::new(
            counter_sun_color[0],
            counter_sun_color[1],
            counter_sun_color[2],
        );
        light_comp.intensity = 10000000.0;
        light_comp.light_type = LightType::Point;
        scene.set_entity_parent(counter_sun_bary, counter_sun);

        let mega_sun_color: [f32; 3] = [0.0, 1.0, 1.0];
        let mega_sun_mat = res_man.instantiate_material("gltf_metal_rough", "mega_sun_mat");
        mega_sun_mat
            .as_ref()
            .unwrap()
            .borrow_mut()
            .set_uniform_value(
                UniformName::EmissiveFactor,
                UniformValue::Vec3(mega_sun_color),
            );

        // Mega-sun barycenter
        let mega_sun_bary = scene.new_entity(Some("mega_sun_barycenter"));
        scene.add_component::<TransformComponent>(mega_sun_bary);
        let physics = scene.add_component::<PhysicsComponent>(mega_sun_bary);
        physics.ang_mom = Vector3::new(0.0, 0.0, 133.0);

        // Mega-sun
        let mega_sun = scene.new_entity(Some("Mega-sun"));
        let trans_comp = scene.add_component::<TransformComponent>(mega_sun);
        trans_comp.get_local_transform_mut().trans = Vector3::new(35.0, 0.0, 0.0);
        trans_comp.get_local_transform_mut().scale = Vector3::new(3.0, 3.0, 3.0);
        let mesh_comp = scene.add_component::<MeshComponent>(mega_sun);
        mesh_comp.set_mesh(res_man.get_or_create_mesh("lat_long_sphere"));
        mesh_comp.set_material_override(mega_sun_mat.clone(), 0);
        let light_comp = scene.add_component::<LightComponent>(mega_sun);
        light_comp.color = Vector3::new(mega_sun_color[0], mega_sun_color[1], mega_sun_color[2]);
        light_comp.intensity = 100000000.0;
        light_comp.light_type = LightType::Point;
        scene.set_entity_parent(mega_sun_bary, mega_sun);

        let vert_sun_color: [f32; 3] = [1.0, 0.0, 1.0];
        let vert_sun_mat = res_man.instantiate_material("gltf_metal_rough", "vert_sun_mat");
        vert_sun_mat
            .as_ref()
            .unwrap()
            .borrow_mut()
            .set_uniform_value(
                UniformName::EmissiveFactor,
                UniformValue::Vec3(vert_sun_color),
            );

        // Vertical-sun bary
        let vert_sun_bary = scene.new_entity(Some("vertical_sun_barycenter"));
        scene.add_component::<TransformComponent>(vert_sun_bary);
        let physics = scene.add_component::<PhysicsComponent>(vert_sun_bary);
        physics.ang_mom = Vector3::new(150.0, 0.0, 0.0);

        // Vertical-sun
        let vert_sun = scene.new_entity(Some("Vertical-sun"));
        let trans_comp = scene.add_component::<TransformComponent>(vert_sun);
        trans_comp.get_local_transform_mut().trans = Vector3::new(0.0, 5.0, 0.0);
        let mesh_comp = scene.add_component::<MeshComponent>(vert_sun);
        mesh_comp.set_mesh(res_man.get_or_create_mesh("lat_long_sphere"));
        mesh_comp.set_material_override(vert_sun_mat.clone(), 0);
        let light_comp = scene.add_component::<LightComponent>(vert_sun);
        light_comp.color = Vector3::new(vert_sun_color[0], vert_sun_color[1], vert_sun_color[2]);
        light_comp.intensity = 10000000.0;
        light_comp.light_type = LightType::Point;
        scene.set_entity_parent(vert_sun_bary, vert_sun);

        // Grid
        let grid = scene.new_entity(Some("grid"));
        let trans_comp = scene.add_component::<TransformComponent>(grid);
        trans_comp.get_local_transform_mut().scale = Vector3::new(1000000.0, 1000000.0, 1000000.0);
        let mesh_comp = scene.add_component::<MeshComponent>(grid);
        mesh_comp.set_mesh(res_man.get_or_create_mesh("grid"));

        // Axes
        let axes = scene.new_entity(Some("axes"));
        let trans_comp = scene.add_component::<TransformComponent>(axes);
        trans_comp.get_local_transform_mut().scale = Vector3::new(10000.0, 10000.0, 10000.0);
        let mesh_comp = scene.add_component::<MeshComponent>(axes);
        mesh_comp.set_mesh(res_man.get_or_create_mesh("axes"));

        return scene;
    }

    fn load_scene_from_desc(
        &mut self,
        identifier: &str,
        res_man: &mut ResourceManager,
        orbit_man: &OrbitManager,
        state: &mut AppState,
    ) -> &mut Scene {
        let desc = self.descriptions.get(identifier).cloned().unwrap();

        // Make Rust happy
        // TODO: Remove this cloning
        let name = desc.name.clone();
        let mut instance_descs: Vec<BodyInstanceDescription> = desc.bodies.clone();
        let time = J2000_JDN;

        let scene = self.new_scene(&name).unwrap();

        // Bodies
        for instance_desc in instance_descs.iter_mut() {
            let mut body_desc: Option<BodyDescription> = None;
            let mut default_state_vector: Option<StateVector> = None;

            if let Some(source) = instance_desc.source.clone() {
                let slash_pos = source.rfind("/").unwrap();
                let db_name = &source[..slash_pos]; // e.g. "major_bodies"
                let body_id = &source[slash_pos + 1..]; // e.g. "399"

                // Wildcard
                if let Some(num_str) = body_id.strip_prefix("*") {
                    let mut limit: Option<usize> = None;

                    if num_str.len() > 0 {
                        let parsed = num_str.parse::<usize>();
                        if parsed.is_err() {
                            log::warn!(
                                "Failed to parse the wildcard number in instance desc source '{}'",
                                source
                            );
                            continue;
                        }
                        limit = Some(parsed.unwrap());
                    }

                    let bodies = orbit_man.get_n_bodies(db_name, limit);

                    log::info!(
                        "Found {} wildcard bodies from db '{}'",
                        bodies.len(),
                        db_name
                    );

                    for body in bodies {
                        let default_state_vector = fetch_default_motion_if_needed(
                            body.id.as_ref().unwrap(),
                            orbit_man,
                            time,
                        );

                        add_body_instance_entities(
                            scene,
                            time,
                            Some(body),
                            &instance_desc,
                            default_state_vector,
                            res_man,
                        );
                    }
                // Just a single body with a source
                } else {
                    body_desc = orbit_man.get_body(db_name, body_id).ok().cloned();
                    default_state_vector = fetch_default_motion_if_needed(body_id, orbit_man, time);
                }
            }

            // Create body entity and components (even if we have no actual body we may need it
            // in order to parent other things to)
            add_body_instance_entities(
                scene,
                time,
                body_desc.as_ref(),
                &instance_desc,
                default_state_vector,
                res_man,
            );
        }

        // Grid
        if state.show_grid {
            let grid = scene.new_entity(Some("grid"));
            let trans_comp = scene.add_component::<TransformComponent>(grid);
            trans_comp.get_local_transform_mut().scale =
                Vector3::new(1000000.0, 1000000.0, 1000000.0);
            let mesh_comp = scene.add_component::<MeshComponent>(grid);
            mesh_comp.set_mesh(res_man.get_or_create_mesh("grid"));
        }

        // Axes
        if state.show_axes {
            let axes = scene.new_entity(Some("axes"));
            let trans_comp = scene.add_component::<TransformComponent>(axes);
            trans_comp.get_local_transform_mut().scale = Vector3::new(10000.0, 10000.0, 10000.0);
            let mesh_comp = scene.add_component::<MeshComponent>(axes);
            mesh_comp.set_mesh(res_man.get_or_create_mesh("axes"));
        }

        // Skybox
        if state.use_skyboxes {
            scene.skybox_mesh = res_man.get_or_create_mesh("quad");
            scene.skybox_trans = Some(
                Rotation3::<f64>::from_axis_angle(
                    &Vector3::x_axis(),
                    OBLIQUITY_OF_ECLIPTIC.to_rad().0,
                )
                .to_homogeneous(),
            );
            scene.skybox_mat = res_man.instantiate_material("skybox", "test_skybox");
            scene.skybox_mat.as_ref().unwrap().borrow_mut().set_texture(
                TextureUnit::BaseColor,
                res_man.get_or_request_texture("starmap_16k", true),
            );
        }

        // Points
        if state.show_points {
            scene.points_mesh = res_man.get_or_create_mesh("points");
            scene.points_mat = res_man.get_or_create_material("default_points");
        }

        return scene;
    }
}
