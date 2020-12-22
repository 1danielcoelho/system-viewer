use std::collections::HashMap;

use cgmath::{InnerSpace, UlpsEq, Vector3};

use super::{resource::gltf_resources::GltfResource, ECManager, Entity, ResourceManager};
use crate::components::{
    light::LightType, transform::TransformType, ui::WidgetType, LightComponent, MeshComponent,
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
        // let indent = "\t".repeat(indent_level as usize);

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
        // let mut mesh_str = String::new();
        if let Some(mesh) = node.mesh() {
            let mesh_comp = scene.ent_man.add_component::<MeshComponent>(ent).unwrap();

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

        // // Parent spinning around Z
        // let parent = scene.ent_man.new_entity();
        // let trans_comp = scene
        //     .ent_man
        //     .add_component::<TransformComponent>(parent)
        //     .unwrap();
        // let phys_comp = scene
        //     .ent_man
        //     .add_component::<PhysicsComponent>(parent)
        //     .unwrap();
        // phys_comp.ang_mom = Vector3::new(0.0, 0.0, 1.0);
        // let mesh_comp = scene
        //     .ent_man
        //     .add_component::<MeshComponent>(parent)
        //     .unwrap();

        // // Light rotating around Z
        // let child = scene.ent_man.new_entity();
        // scene.ent_man.set_entity_parent(parent, child);
        // let trans_comp = scene
        //     .ent_man
        //     .add_component::<TransformComponent>(child)
        //     .unwrap();
        // trans_comp.get_local_transform_mut().disp = Vector3::new(4.0, 0.0, 0.0);
        // trans_comp.get_local_transform_mut().scale = 0.1;
        // let phys_comp = scene
        //     .ent_man
        //     .add_component::<PhysicsComponent>(child)
        //     .unwrap();
        // let mesh_comp = scene.ent_man.add_component::<MeshComponent>(child).unwrap();
        // mesh_comp.set_mesh(res_man.get_or_create_mesh("cube"));
        // let light_comp = scene
        //     .ent_man
        //     .add_component::<LightComponent>(child)
        //     .unwrap();
        // light_comp.color = cgmath::Vector3::new(0.1, 0.8, 0.2);
        // light_comp.intensity = 1.0;

        // // Parent spinning around Y
        // let parent = scene.ent_man.new_entity();
        // let trans_comp = scene
        //     .ent_man
        //     .add_component::<TransformComponent>(parent)
        //     .unwrap();
        // let phys_comp = scene
        //     .ent_man
        //     .add_component::<PhysicsComponent>(parent)
        //     .unwrap();
        // phys_comp.ang_mom = Vector3::new(0.0, 1.0, 0.0);

        // // Light rotating around Y
        // let child = scene.ent_man.new_entity();
        // scene.ent_man.set_entity_parent(parent, child);
        // let trans_comp = scene
        //     .ent_man
        //     .add_component::<TransformComponent>(child)
        //     .unwrap();
        // trans_comp.get_local_transform_mut().disp = Vector3::new(2.0, 0.0, 0.0);
        // trans_comp.get_local_transform_mut().scale = 0.1;
        // let phys_comp = scene
        //     .ent_man
        //     .add_component::<PhysicsComponent>(child)
        //     .unwrap();
        // let mesh_comp = scene.ent_man.add_component::<MeshComponent>(child).unwrap();
        // mesh_comp.set_mesh(res_man.get_or_create_mesh("cube"));
        // let light_comp = scene
        //     .ent_man
        //     .add_component::<LightComponent>(child)
        //     .unwrap();
        // light_comp.color = cgmath::Vector3::new(0.1, 0.1, 0.8);
        // light_comp.intensity = 1.0;

        // Directional light
        let dir_light = scene.ent_man.new_entity();
        let trans_comp = scene
            .ent_man
            .add_component::<TransformComponent>(dir_light)
            .unwrap();
        trans_comp.get_local_transform_mut().disp = Vector3::new(0.2, 0.0, -1.0);
        let light_comp = scene
            .ent_man
            .add_component::<LightComponent>(dir_light)
            .unwrap();
        light_comp.color = cgmath::Vector3::new(1.0, 1.0, 1.0);
        light_comp.intensity = 100.0;
        light_comp.light_type = LightType::Directional;

        // let base_color = res_man
        //     .get_texture("./public/WaterBottle_baseColor.png")
        //     .unwrap();
        // let emissive = res_man
        //     .get_texture("./public/WaterBottle_emissive.png")
        //     .unwrap();
        // let normal = res_man
        //     .get_texture("./public/WaterBottle_normal.png")
        //     .unwrap();
        // let orm = res_man
        //     .get_texture("./public/WaterBottle_occlusionRoughnessMetallic.png")
        //     .unwrap();

        let test_mat1 = res_man.instantiate_material("gltf_metal_rough");
        // test_mat1
        //     .as_ref()
        //     .unwrap()
        //     .borrow_mut()
        //     .set_texture(TextureUnit::BaseColor, Some(base_color));
        // test_mat1
        //     .as_ref()
        //     .unwrap()
        //     .borrow_mut()
        //     .set_texture(TextureUnit::Emissive, Some(emissive));
        // test_mat1
        //     .as_ref()
        //     .unwrap()
        //     .borrow_mut()
        //     .set_texture(TextureUnit::Normal, Some(normal));
        // test_mat1
        //     .as_ref()
        //     .unwrap()
        //     .borrow_mut()
        //     .set_texture(TextureUnit::MetallicRoughness, Some(orm));

        // Plane
        let plane = scene.ent_man.new_entity();
        let trans_comp = scene
            .ent_man
            .add_component::<TransformComponent>(plane)
            .unwrap();
        trans_comp.get_local_transform_mut().disp = Vector3::new(0.0, 0.0, 0.0);
        trans_comp.get_local_transform_mut().scale = 2.0;
        let mesh_comp = scene.ent_man.add_component::<MeshComponent>(plane).unwrap();
        mesh_comp.set_mesh(res_man.get_or_create_mesh("plane"));
        mesh_comp.set_material_override(test_mat1.clone(), 0);

        // Cube
        let cube = scene.ent_man.new_entity();
        let trans_comp = scene
            .ent_man
            .add_component::<TransformComponent>(cube)
            .unwrap();
        trans_comp.get_local_transform_mut().disp = Vector3::new(-4.0, 0.0, 0.0);
        trans_comp.get_local_transform_mut().scale = 1.0;
        let mesh_comp = scene.ent_man.add_component::<MeshComponent>(cube).unwrap();
        mesh_comp.set_mesh(res_man.get_or_create_mesh("cube"));
        mesh_comp.set_material_override(test_mat1.clone(), 0);

        // Ico-sphere
        let ico = scene.ent_man.new_entity();
        let trans_comp = scene
            .ent_man
            .add_component::<TransformComponent>(ico)
            .unwrap();
        trans_comp.get_local_transform_mut().disp = Vector3::new(0.0, 6.0, 0.0);
        trans_comp.get_local_transform_mut().scale = 1.0;
        let mesh_comp = scene.ent_man.add_component::<MeshComponent>(ico).unwrap();
        mesh_comp.set_mesh(res_man.get_or_create_mesh("ico_sphere"));
        mesh_comp.set_material_override(test_mat1.clone(), 0);

        // Lat-long sphere
        let lat_long = scene.ent_man.new_entity();
        let trans_comp = scene
            .ent_man
            .add_component::<TransformComponent>(lat_long)
            .unwrap();
        trans_comp.get_local_transform_mut().disp = Vector3::new(2.0, 6.0, 0.0);
        let mesh_comp = scene
            .ent_man
            .add_component::<MeshComponent>(lat_long)
            .unwrap();
        mesh_comp.set_mesh(res_man.get_or_create_mesh("lat_long_sphere"));
        mesh_comp.set_material_override(test_mat1.clone(), 0);

        // Grid
        let grid = scene.ent_man.new_entity();
        let trans_comp = scene
            .ent_man
            .add_component::<TransformComponent>(grid)
            .unwrap();
        trans_comp.get_local_transform_mut().scale = 1000.0;
        let mesh_comp = scene.ent_man.add_component::<MeshComponent>(grid).unwrap();
        mesh_comp.set_mesh(res_man.get_or_create_mesh("grid"));

        // Axes
        let axes = scene.ent_man.new_entity();
        let trans_comp = scene
            .ent_man
            .add_component::<TransformComponent>(axes)
            .unwrap();
        trans_comp.get_local_transform_mut().scale = 3.0;
        let mesh_comp = scene.ent_man.add_component::<MeshComponent>(axes).unwrap();
        mesh_comp.set_mesh(res_man.get_or_create_mesh("axes"));

        // Debug UI
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
