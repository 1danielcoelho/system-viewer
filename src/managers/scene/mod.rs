use super::{resource::gltf_resources::GltfResource, ResourceManager};
use crate::{
    components::{
        light::LightType, LightComponent, MeshComponent, PhysicsComponent, TransformComponent,
    },
    managers::resource::{
        material::{Material, UniformName, UniformValue},
        mesh::Mesh,
        texture::Texture,
    },
    utils::{string::get_unique_name, transform::Transform},
};
use na::{Quaternion, UnitQuaternion, Vector3};
pub use scene::*;
use std::{cell::RefCell, collections::HashMap, rc::Rc};

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

    fn load_gltf_node(
        node: &gltf::Node,
        indent_level: u32,
        file_identifier: &str,
        scene: &mut Scene,
        resources: &ResourceManager,
    ) -> Entity {
        // let indent = "\t".repeat(indent_level as usize);

        let ent: Entity = scene.new_entity(Some(&node.get_identifier(file_identifier)));

        // Transform
        let trans_comp = scene.add_component::<TransformComponent>(ent).unwrap();
        let trans = trans_comp.get_local_transform_mut();
        let (pos, quat, scale) = node.transform().decomposed();
        trans.trans.x = pos[0];
        trans.trans.y = -pos[2];
        trans.trans.z = pos[1];
        trans.rot =
            UnitQuaternion::new_normalize(Quaternion::new(quat[0], -quat[2], quat[1], quat[3]));
        trans.scale = Vector3::new(scale[0], scale[1], scale[2]);

        // Mesh
        // let mut mesh_str = String::new();
        if let Some(mesh) = node.mesh() {
            let mesh_comp = scene.add_component::<MeshComponent>(ent).unwrap();

            let mesh_identifier = mesh.get_identifier(&file_identifier);
            // mesh_str = mesh_identifier.to_owned();
            if let Some(found_mesh) = resources.get_mesh(&mesh_identifier) {
                mesh_comp.set_mesh(Some(found_mesh));
            } else {
                log::error!(
                    "Failed to find mesh '{}' required by node '{}' of scene '{}'",
                    mesh_identifier,
                    node.index(),
                    scene.identifier
                );
            }
        }

        // log::info!(
        //     "{}Node '{}': pos: [{}, {}, {}], rot: [{}, {}, {}, {}], scale: [{}, {}, {}], mesh '{}'",
        //     indent,
        //     node.get_identifier(&file_identifier),
        //     pos[0],
        //     pos[1],
        //     pos[2],
        //     quat[0],
        //     quat[1],
        //     quat[2],
        //     quat[3],
        //     scale[0],
        //     scale[1],
        //     scale[2],
        //     mesh_str
        // );

        // Children
        for child in node.children() {
            let child_ent = SceneManager::load_gltf_node(
                &child,
                indent_level + 1,
                file_identifier,
                scene,
                resources,
            );

            scene.set_entity_parent(ent, child_ent);
        }

        return ent;
    }

    pub fn load_scenes_from_gltf(
        &mut self,
        file_identifier: &str,
        scenes: gltf::iter::Scenes,
        resources: &ResourceManager,
    ) -> Vec<String> {
        let num_scenes = scenes.len();
        log::info!(
            "Loading {} scenes from gltf file '{}':",
            num_scenes,
            file_identifier
        );

        let mut loaded_scene_identifiers: Vec<String> = Vec::new();

        for gltf_scene in scenes {
            let num_nodes = gltf_scene.nodes().len();

            let scene_identifier = gltf_scene.get_identifier(file_identifier);
            let mut scene = self.new_scene(&scene_identifier).unwrap();
            let scene_identifier = scene.identifier.clone();

            log::info!("\tScene '{}': {} root nodes", &scene_identifier, num_nodes);

            scene.reserve_space_for_entities((num_nodes + 1) as u32);

            let root_ent: Entity = scene.new_entity(Some(&scene_identifier));

            for child_node in gltf_scene.nodes() {
                let child_ent = SceneManager::load_gltf_node(
                    &child_node,
                    2,
                    file_identifier,
                    &mut scene,
                    &resources,
                );

                scene.set_entity_parent(root_ent, child_ent);
            }

            loaded_scene_identifiers.push(scene_identifier);
        }

        return loaded_scene_identifiers;
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
        let trans_comp = scene.add_component::<TransformComponent>(sun).unwrap();
        trans_comp.get_local_transform_mut().trans = Vector3::new(0.0, 0.0, 0.0);
        let mesh_comp = scene.add_component::<MeshComponent>(sun).unwrap();
        mesh_comp.set_mesh(res_man.get_or_create_mesh("ico_sphere"));
        mesh_comp.set_material_override(sun_mat.clone(), 0);
        let light_comp = scene.add_component::<LightComponent>(sun).unwrap();
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
        scene.add_component::<TransformComponent>(sun_bary).unwrap();
        let physics = scene.add_component::<PhysicsComponent>(sun_bary).unwrap();
        physics.ang_mom = Vector3::new(0.0, 0.0, 0.25);

        // Planet
        let planet = scene.new_entity(Some("planet"));
        let trans_comp = scene.add_component::<TransformComponent>(planet).unwrap();
        trans_comp.get_local_transform_mut().trans = Vector3::new(10.0, 0.0, 0.0);
        let mesh_comp = scene.add_component::<MeshComponent>(planet).unwrap();
        mesh_comp.set_mesh(res_man.get_or_create_mesh("lat_long_sphere"));
        mesh_comp.set_material_override(planet_mat.clone(), 0);
        scene.set_entity_parent(sun_bary, planet);

        // Planet orbit
        let planet_orbit = scene.new_entity(Some("orbit"));
        let trans_comp = scene
            .add_component::<TransformComponent>(planet_orbit)
            .unwrap();
        trans_comp.get_local_transform_mut().trans = Vector3::new(0.0, 0.0, 0.0);
        trans_comp.get_local_transform_mut().scale = Vector3::new(10.0, 10.0, 10.0);
        let mesh_comp = scene.add_component::<MeshComponent>(planet_orbit).unwrap();
        mesh_comp.set_mesh(res_man.get_or_create_mesh("circle"));

        // Moon orbit center
        let planet_bary = scene.new_entity(Some("center"));
        scene
            .add_component::<TransformComponent>(planet_bary)
            .unwrap();
        let physics = scene
            .add_component::<PhysicsComponent>(planet_bary)
            .unwrap();
        physics.ang_mom = Vector3::new(0.0, 0.0, 0.86);
        scene.set_entity_parent(planet, planet_bary);

        let moon_mat = res_man.instantiate_material("gltf_metal_rough", "moon_mat");
        moon_mat.as_ref().unwrap().borrow_mut().set_uniform_value(
            UniformName::BaseColorFactor,
            UniformValue::Vec4([0.8, 0.8, 0.7, 1.0]),
        );

        // Moon
        let moon = scene.new_entity(Some("moon"));
        let trans_comp = scene.add_component::<TransformComponent>(moon).unwrap();
        trans_comp.get_local_transform_mut().trans = Vector3::new(3.0, 0.0, 0.0);
        trans_comp.get_local_transform_mut().scale = Vector3::new(0.2, 0.2, 0.2);
        let mesh_comp = scene.add_component::<MeshComponent>(moon).unwrap();
        mesh_comp.set_mesh(res_man.get_or_create_mesh("lat_long_sphere"));
        mesh_comp.set_material_override(moon_mat.clone(), 0);
        scene.set_entity_parent(planet_bary, moon);

        // Moon orbit
        let moon_orbit = scene.new_entity(Some("orbit"));
        let trans_comp = scene
            .add_component::<TransformComponent>(moon_orbit)
            .unwrap();
        trans_comp.get_local_transform_mut().trans = Vector3::new(0.0, 0.0, 0.0);
        trans_comp.get_local_transform_mut().scale = Vector3::new(3.0, 3.0, 3.0);
        let mesh_comp = scene.add_component::<MeshComponent>(moon_orbit).unwrap();
        mesh_comp.set_mesh(res_man.get_or_create_mesh("circle"));
        scene.set_entity_parent(planet_bary, moon_orbit);

        // Grid
        let grid = scene.new_entity(Some("grid"));
        let trans_comp = scene.add_component::<TransformComponent>(grid).unwrap();
        trans_comp.get_local_transform_mut().scale = Vector3::new(100000.0, 100000.0, 100000.0);
        let mesh_comp = scene.add_component::<MeshComponent>(grid).unwrap();
        mesh_comp.set_mesh(res_man.get_or_create_mesh("grid"));

        // Axes
        let axes = scene.new_entity(Some("axes"));
        let trans_comp = scene.add_component::<TransformComponent>(axes).unwrap();
        trans_comp.get_local_transform_mut().scale = Vector3::new(300.0, 300.0, 300.0);
        let mesh_comp = scene.add_component::<MeshComponent>(axes).unwrap();
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
        target_transform: Option<Transform>,
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

        // Update the position of the root scene node to the target transform
        if let Some(target_transform) = target_transform {
            let scene_root_trans = injected_scene_copy.transform[0].get_local_transform_mut();
            *scene_root_trans = target_transform;
        }

        // Inject entities into current_scene, and keep track of where they ended up
        current_scene.move_from_other(injected_scene_copy);

        return Ok(());
    }
}
