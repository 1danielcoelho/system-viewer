use std::collections::HashMap;

use cgmath::{InnerSpace, UlpsEq, Vector3};

use super::{resource::gltf_resources::GltfResource, ECManager, Entity, ResourceManager};
use crate::components::{
    transform::TransformType, ui::WidgetType, LightComponent, MeshComponent, PhysicsComponent,
    TransformComponent, UIComponent,
};

#[derive(Clone)]
pub struct Scene {
    pub identifier: String,
    pub ent_man: ECManager,
    _private: (),
}
impl Scene {
    fn new(identifier: &str) -> Scene {
        Scene {
            identifier: identifier.to_string(),
            ent_man: ECManager::new(),
            _private: (),
        }
    }
}

pub struct SceneManager {
    main: Option<String>,
    loaded_scenes: HashMap<String, Scene>,
}
impl SceneManager {
    pub fn new() -> Self {
        return Self {
            main: None,
            loaded_scenes: HashMap::new(),
        };
    }

    pub fn get_main_scene(&self) -> Option<&Scene> {
        match self.main.as_ref() {
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
        let indent = "\t".repeat(indent_level as usize);

        let ent: Entity = scene.ent_man.new_entity();

        // Transform
        let trans_comp = scene
            .ent_man
            .add_component::<TransformComponent>(ent)
            .unwrap();
        let trans: &mut TransformType = trans_comp.get_local_transform_mut();
        let (pos, quat, scale) = node.transform().decomposed();
        trans.disp.x = pos[0];
        trans.disp.y = -pos[2];
        trans.disp.z = pos[1];
        trans.rot.v = cgmath::Vector3::new(quat[0], -quat[2], quat[1]);
        trans.rot.s = quat[3];
        trans.rot = trans.rot.normalize();
        trans.scale = scale[0];

        if !scale[0].ulps_eq(&scale[1], f32::EPSILON, f32::default_max_ulps())
            || !scale[0].ulps_eq(&scale[2], f32::EPSILON, f32::default_max_ulps())
        {
            log::warn!(
                "Ignoring non-uniform scale '[{}, {}, {}]' for node '{}' of scene '{}'",
                scale[0],
                scale[1],
                scale[2],
                node.index(),
                scene.identifier
            );
        }

        // Mesh
        let mut mesh_str = String::new();
        if let Some(mesh) = node.mesh() {
            let mesh_comp = scene.ent_man.add_component::<MeshComponent>(ent).unwrap();

            let mesh_identifier = mesh.get_identifier(&file_identifier);
            mesh_str = mesh_identifier.to_owned();
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

            scene.ent_man.set_entity_parent(ent, child_ent);
        }

        return ent;
    }

    pub fn load_scenes_from_gltf(
        &mut self,
        file_identifier: &str,
        scenes: gltf::iter::Scenes,
        resources: &ResourceManager,
    ) {
        let num_scenes = scenes.len();
        log::info!(
            "Loading {} scenes from gltf file '{}':",
            num_scenes,
            file_identifier
        );

        for gltf_scene in scenes {
            let num_nodes = gltf_scene.nodes().len();

            let scene_identifier = gltf_scene.get_identifier(file_identifier);
            let mut scene = Scene::new(&scene_identifier);

            log::info!("\tScene '{}': {} root nodes", scene_identifier, num_nodes);

            scene
                .ent_man
                .reserve_space_for_entities((num_nodes + 1) as u32);

            let root_ent: Entity = scene.ent_man.new_entity();

            for child_node in gltf_scene.nodes() {
                let child_ent = SceneManager::load_gltf_node(
                    &child_node,
                    2,
                    file_identifier,
                    &mut scene,
                    &resources,
                );

                scene.ent_man.set_entity_parent(root_ent, child_ent);
            }

            self.loaded_scenes
                .insert(scene_identifier.to_string(), scene);
        }
    }

    pub fn load_test_scene(&mut self, identifier: &str, res_man: &mut ResourceManager) {
        let mut scene = Scene::new(identifier);

        // Setup scene
        let parent = scene.ent_man.new_entity();
        let trans_comp = scene
            .ent_man
            .add_component::<TransformComponent>(parent)
            .unwrap();
        //trans_comp.get_local_transform_mut().scale = 0.05;
        let phys_comp = scene
            .ent_man
            .add_component::<PhysicsComponent>(parent)
            .unwrap();
        phys_comp.ang_mom = Vector3::new(0.0, 0.0, 1.0);
        // phys_comp.lin_mom = Vector3::new(10.0, 0.0, 0.0);
        let mesh_comp = scene
            .ent_man
            .add_component::<MeshComponent>(parent)
            .unwrap();
        //mesh_comp.set_mesh(res_man.get_or_create_mesh("cube"));
        //mesh_comp.set_material_override(res_man.get_or_create_material("local_normal"), 0);

        let child = scene.ent_man.new_entity();
        scene.ent_man.set_entity_parent(parent, child);
        let trans_comp = scene
            .ent_man
            .add_component::<TransformComponent>(child)
            .unwrap();
        trans_comp.get_local_transform_mut().disp = Vector3::new(4.0, 0.0, 0.0);
        trans_comp.get_local_transform_mut().scale = 0.1;
        let phys_comp = scene
            .ent_man
            .add_component::<PhysicsComponent>(child)
            .unwrap();
        phys_comp.ang_mom = Vector3::new(-1.0, 0.0, 0.0); // This shouldn't do anything
        let mesh_comp = scene.ent_man.add_component::<MeshComponent>(child).unwrap();
        mesh_comp.set_mesh(res_man.get_or_create_mesh("cube"));
        let light_comp = scene
            .ent_man
            .add_component::<LightComponent>(child)
            .unwrap();
        light_comp.color = cgmath::Vector3::new(0.1, 0.8, 0.2);
        light_comp.intensity = 1.0;

        // Lit cube
        // let cube = scene.ent_man.new_entity();
        // let trans_comp = scene
        //     .ent_man
        //     .add_component::<TransformComponent>(cube)
        //     .unwrap();
        // trans_comp.get_local_transform_mut().disp = Vector3::new(0.0, 0.0, 0.0);
        // trans_comp.get_local_transform_mut().scale = 0.8;
        // let mesh_comp = scene.ent_man.add_component::<MeshComponent>(cube).unwrap();
        // mesh_comp.set_mesh(res_man.get_or_create_mesh("cube"));
        // mesh_comp.set_material_override(res_man.get_or_create_material("phong"), 0);

        // let plane = scene.ent_man.new_entity("plane");
        // let trans_comp = scene
        //     .ent_man
        //     .add_component::<TransformComponent>(plane)
        //     .unwrap();
        // trans_comp.transform.scale = 3.0;
        // let mesh_comp = scene
        //     .ent_man
        //     .add_component::<MeshComponent>(plane)
        //     .unwrap();
        // mesh_comp.mesh = scene.res_man.generate_mesh("plane", &gl);
        // mesh_comp.material = scene.res_man.get_material("material");

        let grid = scene.ent_man.new_entity();
        let trans_comp = scene
            .ent_man
            .add_component::<TransformComponent>(grid)
            .unwrap();
        trans_comp.get_local_transform_mut().scale = 1000.0;
        let mesh_comp = scene.ent_man.add_component::<MeshComponent>(grid).unwrap();
        mesh_comp.set_mesh(res_man.get_or_create_mesh("grid"));

        let axes = scene.ent_man.new_entity();
        let trans_comp = scene
            .ent_man
            .add_component::<TransformComponent>(axes)
            .unwrap();
        trans_comp.get_local_transform_mut().scale = 3.0;
        let mesh_comp = scene.ent_man.add_component::<MeshComponent>(axes).unwrap();
        mesh_comp.set_mesh(res_man.get_or_create_mesh("axes"));

        let ui_entity = scene.ent_man.new_entity();
        scene.ent_man.add_component::<TransformComponent>(ui_entity);
        let ui_comp = scene
            .ent_man
            .add_component::<UIComponent>(ui_entity)
            .unwrap();
        ui_comp.widget_type = WidgetType::TestWidget;

        self.loaded_scenes.insert(identifier.to_string(), scene);
    }

    pub fn set_scene(&mut self, identifier: &str) {
        if let Some(main) = &self.main {
            if &main[..] == identifier {
                return;
            }
        };

        if let Some(_found_scene) = self.get_scene(identifier) {
            self.main = Some(identifier.to_string());
        } else {
            log::warn!("Scene with identifier {} not found!", identifier);
        }
    }

    /** Injects the scene with identifier `str` into the current scene, setting its 0th node to transform `target_transform` */
    pub fn inject_scene(
        &mut self,
        identifier: &str,
        target_transform: Option<TransformType>,
    ) -> Result<(), String> {
        // TODO: Will have to copy this data twice to get past the borrow checker, and I'm not sure if it's smart enough to elide it... maybe just use a RefCell for scenes?
        let mut injected_scene_copy = self
            .get_scene(identifier)
            .ok_or(format!("Failed to find scene to inject '{}'", identifier))?
            .clone();

        let current_scene = self
            .get_main_scene_mut()
            .ok_or("Can't inject if we don't have a main scene!")?;

        log::info!(
            "Injecting scene '{}' into current '{}' ({} entities)",
            injected_scene_copy.identifier,
            current_scene.identifier,
            injected_scene_copy.ent_man.get_num_entities()
        );

        // Update the position of the root scene node to the target transform
        if let Some(target_transform) = target_transform {
            let scene_root_trans =
                injected_scene_copy.ent_man.transform[0].get_local_transform_mut();
            *scene_root_trans = target_transform;
        }

        // Inject entities into current_scene, and keep track of where they ended up
        current_scene
            .ent_man
            .move_from_other(injected_scene_copy.ent_man);

        return Ok(());
    }
}
