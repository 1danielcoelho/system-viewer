use crate::{
    app_state::AppState, components::transform::TransformType,
    managers::ECManager,
};

pub struct TransformUpdateSystem {}
impl TransformUpdateSystem {
    pub fn run(&self, _state: &AppState, em: &mut ECManager) {
        let identity: TransformType = cgmath::Transform::one();

        for entity_index in 0..em.get_num_entities() {
            // TODO: This has an indirection
            let parent_index = em.get_parent_index_from_index(entity_index);

            // Note that we go only one parent level: We guarantee that we'll
            // update our transforms in order, so that parents always come before children
            match parent_index {
                Some(parent_index) => {
                    // TODO: Indirection
                    let parent_transform = em.transform[parent_index as usize]
                        .get_world_transform()
                        .clone();
                    em.transform[entity_index as usize].update_world_transform(&parent_transform);
                }
                None => {
                    em.transform[entity_index as usize].update_world_transform(&identity);
                }
            }
        }
    }
}
