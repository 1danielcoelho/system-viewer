use std::collections::HashMap;

use cgmath::Vector3;

use super::{ComponentManager, EntityManager, ResourceManager};
use crate::components::{
    ui::WidgetType, MeshComponent, PhysicsComponent, TransformComponent, UIComponent,
};

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

    pub fn load_scenes_from_gltf(
        &mut self,
        _gltf: gltf::iter::Scenes,
        _resources: &ResourceManager,
    ) {
    }

    pub fn load_test_scene(&mut self, identifier: &str, res_man: &mut ResourceManager) {
        let mut scene = Scene::new(identifier);

        // Setup scene
        let parent = scene.ent_man.new_entity();
        let parent_id = scene.ent_man.get_entity_index(&parent).unwrap();

        let _trans_comp = scene
            .comp_man
            .add_component::<TransformComponent>(parent_id)
            .unwrap();
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
}
