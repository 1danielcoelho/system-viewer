use super::ResourceManager;
use crate::managers::scene::component_storage::ComponentStorage;
use crate::{
    components::{
        light::LightType, LightComponent, MeshComponent, PhysicsComponent, TransformComponent,
    },
    managers::resource::material::{UniformName, UniformValue},
    utils::{string::get_unique_name, transform::Transform, vec::RemoveItem},
};
use na::Vector3;
use std::collections::HashMap;

pub use scene::*;

pub mod component_storage;
pub mod gltf;
pub mod orbits;
mod scene;
mod serialization;

pub struct SceneManager {
    main: Option<String>,
    loaded_scenes: HashMap<String, Scene>,

    // Used for UI, kept in sync with loaded_scenes
    pub sorted_loaded_scene_names: Vec<String>,
}
impl SceneManager {
    pub fn new() -> Self {
        return Self {
            main: None,
            loaded_scenes: HashMap::new(),
            sorted_loaded_scene_names: Vec::new(),
        };
    }

    pub fn deserialize_scene(&mut self, ron_str: &str) -> Option<&mut Scene> {
        let scene = Scene::deserialize(ron_str);
        if scene.is_err() {
            log::error!(
                "Failed to deserialize scene. Error:\n{}",
                scene.err().unwrap()
            );
            return None;
        }
        let mut scene = scene.unwrap();

        let unique_scene_name = get_unique_name(&scene.identifier, &self.loaded_scenes);

        log::info!(
            "Deserialized scene with name '{}'. New name: '{}'",
            &scene.identifier,
            unique_scene_name
        );
        scene.identifier = unique_scene_name.clone();

        assert!(!self.loaded_scenes.contains_key(&unique_scene_name));

        self.loaded_scenes.insert(unique_scene_name.clone(), scene);
        let scene_in_storage = self.loaded_scenes.get_mut(&unique_scene_name);

        self.sorted_loaded_scene_names.push(unique_scene_name);
        self.sorted_loaded_scene_names.sort();

        return scene_in_storage;
    }

    pub fn new_scene(&mut self, scene_name: &str) -> Option<&mut Scene> {
        let unique_scene_name = get_unique_name(scene_name, &self.loaded_scenes);

        let scene = Scene::new(&unique_scene_name);
        log::info!("Created scene '{}'", unique_scene_name);

        assert!(!self.loaded_scenes.contains_key(&unique_scene_name));

        self.loaded_scenes
            .insert(unique_scene_name.to_string(), scene);
        let scene_in_storage = self.loaded_scenes.get_mut(&unique_scene_name);

        self.sorted_loaded_scene_names.push(unique_scene_name);
        self.sorted_loaded_scene_names.sort();

        return scene_in_storage;
    }

    pub fn get_main_scene_name(&self) -> &Option<String> {
        return &self.main;
    }

    pub fn get_main_scene(&self) -> Option<&Scene> {
        match self.get_main_scene_name() {
            Some(ident_ref) => self.get_scene(ident_ref),
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

    pub fn load_teal_sphere_scene(&mut self, identifier: &str, res_man: &mut ResourceManager) {
        let scene = self.new_scene(&identifier).unwrap();

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
    }

    pub fn load_test_scene(&mut self, identifier: &str, res_man: &mut ResourceManager) {
        let scene = self.new_scene(&identifier).unwrap();

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
    }

    pub fn set_scene(&mut self, identifier: &str, res_man: &mut ResourceManager) {
        if let Some(main) = &self.main {
            if &main[..] == identifier {
                return;
            }
        };

        if let Some(found_scene) = self.get_scene_mut(identifier) {
            res_man.provision_scene_assets(found_scene);
            self.main = Some(identifier.to_string());
        } else {
            log::warn!("Scene with identifier {} not found!", identifier);
        }
    }

    /** Injects the scene with identifier `str` into the current scene, setting its 0th node to transform `target_transform` */
    pub fn inject_scene(
        &mut self,
        identifier: &str,
        target_transform: Option<Transform<f64>>,
        res_man: &mut ResourceManager,
    ) -> Result<(), String> {
        // TODO: Will have to copy this data twice to get past the borrow checker, and I'm not sure if it's smart enough to elide it... maybe just use a RefCell for scenes?
        let mut injected_scene_copy = self
            .get_scene(identifier)
            .ok_or(format!(
                "Failed to find scene to inject '{}'. Available scenes:\n{:#?}",
                identifier,
                self.loaded_scenes.keys()
            ))?
            .clone();

        res_man.provision_scene_assets(&mut injected_scene_copy);

        let current_scene = self
            .get_main_scene_mut()
            .ok_or("Can't inject if we don't have a main scene!")?;

        log::info!(
            "Injecting scene '{}' into current '{}' ({} entities)",
            injected_scene_copy.identifier,
            current_scene.identifier,
            injected_scene_copy.get_num_entities()
        );

        // TODO: This is really not enforced at all, but is usually the case
        let root_entity = injected_scene_copy.get_entity_from_index(0).unwrap();

        // Update the position of the root scene node to the target transform
        if let Some(target_transform) = target_transform {
            let scene_root_trans = injected_scene_copy
                .transform
                .get_component_mut(root_entity)
                .unwrap()
                .get_local_transform_mut();
            *scene_root_trans = target_transform;
        }

        // Inject entities into current_scene, and keep track of where they ended up
        current_scene.move_from_other(injected_scene_copy);

        return Ok(());
    }

    pub fn delete_scene(&mut self, identifier: &str) {
        self.loaded_scenes.remove(identifier);
        self.sorted_loaded_scene_names
            .remove_one_item(&identifier.to_owned());
    }
}
