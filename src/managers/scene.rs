use std::collections::HashMap;

use cgmath::{InnerSpace, UlpsEq, Vector3};

use super::{ComponentManager, Entity, EntityManager, resource::gltf_resources::GltfResource, ResourceManager};
use crate::components::{
    transform::TransformType, ui::WidgetType, MeshComponent, PhysicsComponent, TransformComponent,
    UIComponent,
};

#[derive(Clone)]
pub struct Scene {
    pub identifier: String,
    pub ent_man: EntityManager,
    pub comp_man: ComponentManager,
    _private: (),
}
impl Scene {
    fn new(identifier: &str) -> Scene {
        Scene {
            identifier: identifier.to_string(),
            ent_man: EntityManager::new(),
            comp_man: ComponentManager::new(),
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
        let ent_index = scene.ent_man.get_entity_index(&ent).unwrap();

        // Transform
        let trans_comp = scene
            .comp_man
            .add_component::<TransformComponent>(ent_index)
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
            let mesh_comp = scene
                .comp_man
                .add_component::<MeshComponent>(ent_index)
                .unwrap();

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

            scene
                .ent_man
                .set_entity_parent(&ent, &child_ent, &mut scene.comp_man);
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

                scene
                    .ent_man
                    .set_entity_parent(&root_ent, &child_ent, &mut scene.comp_man);
            }

            self.loaded_scenes
                .insert(scene_identifier.to_string(), scene);
        }
    }

    pub fn load_test_scene(&mut self, identifier: &str, res_man: &mut ResourceManager) {
        let mut scene = Scene::new(identifier);

        // Setup scene
        let parent = scene.ent_man.new_entity();
        let parent_id = scene.ent_man.get_entity_index(&parent).unwrap();
        let trans_comp = scene
            .comp_man
            .add_component::<TransformComponent>(parent_id)
            .unwrap();
        //trans_comp.get_local_transform_mut().scale = 0.05;
        let phys_comp = scene
            .comp_man
            .add_component::<PhysicsComponent>(parent_id)
            .unwrap();
        phys_comp.ang_mom = Vector3::new(0.0, 0.0, 1.0);
        // phys_comp.lin_mom = Vector3::new(10.0, 0.0, 0.0);
        let mesh_comp = scene
            .comp_man
            .add_component::<MeshComponent>(parent_id)
            .unwrap();
        mesh_comp.set_mesh(res_man.get_or_create_mesh("cube"));
        //mesh_comp.set_material_override(res_man.get_or_create_material("local_normal"), 0);

        let child = scene.ent_man.new_entity();
        let child_id = scene.ent_man.get_entity_index(&child).unwrap();
        scene
            .ent_man
            .set_entity_parent(&parent, &child, &mut scene.comp_man);
        let trans_comp = scene
            .comp_man
            .add_component::<TransformComponent>(child_id)
            .unwrap();
        trans_comp.get_local_transform_mut().disp = Vector3::new(4.0, 0.0, 0.0);
        trans_comp.get_local_transform_mut().scale = 0.5;
        let phys_comp = scene
            .comp_man
            .add_component::<PhysicsComponent>(child_id)
            .unwrap();
        phys_comp.ang_mom = Vector3::new(-1.0, 0.0, 0.0); // This shouldn't do anything
        let mesh_comp = scene
            .comp_man
            .add_component::<MeshComponent>(child_id)
            .unwrap();
        mesh_comp.set_mesh(res_man.get_or_create_mesh("cube"));

        // let plane = scene.ent_man.new_entity("plane");
        // let trans_comp = scene
        //     .comp_man
        //     .add_component::<TransformComponent>(plane)
        //     .unwrap();
        // trans_comp.transform.scale = 3.0;
        // let mesh_comp = scene
        //     .comp_man
        //     .add_component::<MeshComponent>(plane)
        //     .unwrap();
        // mesh_comp.mesh = scene.res_man.generate_mesh("plane", &gl);
        // mesh_comp.material = scene.res_man.get_material("material");

        let grid = scene.ent_man.new_entity();
        let grid_id = scene.ent_man.get_entity_index(&grid).unwrap();
        let trans_comp = scene
            .comp_man
            .add_component::<TransformComponent>(grid_id)
            .unwrap();
        trans_comp.get_local_transform_mut().scale = 1000.0;
        let mesh_comp = scene
            .comp_man
            .add_component::<MeshComponent>(grid_id)
            .unwrap();
        mesh_comp.set_mesh(res_man.get_or_create_mesh("grid"));

        let axes = scene.ent_man.new_entity();
        let axes_id = scene.ent_man.get_entity_index(&axes).unwrap();
        let trans_comp = scene
            .comp_man
            .add_component::<TransformComponent>(axes_id)
            .unwrap();
        trans_comp.get_local_transform_mut().scale = 3.0;
        let mesh_comp = scene
            .comp_man
            .add_component::<MeshComponent>(axes_id)
            .unwrap();
        mesh_comp.set_mesh(res_man.get_or_create_mesh("axes"));

        let ui_entity = scene.ent_man.new_entity();
        let ui_id = scene.ent_man.get_entity_index(&ui_entity).unwrap();
        scene.comp_man.add_component::<TransformComponent>(ui_id);
        let ui_comp = scene.comp_man.add_component::<UIComponent>(ui_id).unwrap();
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
        // Will have to copy this data twice to get past the borrow checker, and I'm not sure if it's smart enough to elide it... maybe just use a RefCell for scenes?
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
                injected_scene_copy.comp_man.transform[0].get_local_transform_mut();
            *scene_root_trans = target_transform;
        }

        // Inject entities into current_scene, and keep track of where they ended up
        let remapped_indices = current_scene.ent_man.move_from_other(
            injected_scene_copy.ent_man,
            &mut injected_scene_copy.comp_man,
        );

        // Move components from injected_scene_copy according to the remapped indices
        current_scene
            .comp_man
            .move_from_other(injected_scene_copy.comp_man, &remapped_indices);

        return Ok(());
    }
}
