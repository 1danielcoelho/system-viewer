use crate::{
    components::{MeshComponent, TransformComponent},
    managers::{
        resource::gltf::GltfResource,
        scene::{Entity, Scene, SceneManager},
        ResourceManager,
    },
};
use na::{Quaternion, UnitQuaternion, Vector3};

impl SceneManager {
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
        trans.trans.x = pos[0] as f64;
        trans.trans.y = -pos[2] as f64;
        trans.trans.z = pos[1] as f64;
        trans.rot = UnitQuaternion::new_normalize(Quaternion::new(
            quat[0] as f64,
            -quat[2] as f64,
            quat[1] as f64,
            quat[3] as f64,
        ));
        trans.scale = Vector3::new(scale[0] as f64, scale[1] as f64, scale[2] as f64);

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
}
