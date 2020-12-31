use crate::{app_state::AppState, managers::scene::Scene, utils::transform::Transform};

pub struct TransformUpdateSystem {}
impl TransformUpdateSystem {
    pub fn run(&self, _state: &AppState, scene: &mut Scene) {
        let identity = Transform::identity();

        for entity_index in 0..scene.get_num_entities() {
            // TODO: This has an indirection
            let parent_index = scene.get_parent_index_from_index(entity_index);

            // Note that we go only one parent level: We guarantee that we'll
            // update our transforms in order, so that parents always come before children
            match parent_index {
                Some(parent_index) => {
                    // TODO: Indirection
                    let parent_transform = scene.transform[parent_index as usize]
                        .get_world_transform()
                        .clone();
                    scene.transform[entity_index as usize]
                        .update_world_transform(&parent_transform);
                }
                None => {
                    scene.transform[entity_index as usize].update_world_transform(&identity);
                }
            }
        }
    }
}
