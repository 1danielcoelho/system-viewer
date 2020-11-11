use crate::{
    app_state::AppState, components::transform::TransformType, components::TransformComponent,
    managers::EntityManager,
};

pub struct TransformUpdateSystem {}
impl TransformUpdateSystem {
    pub fn run(
        &self,
        state: &AppState,
        transforms: &mut Vec<TransformComponent>,
        em: &mut EntityManager,
    ) {
        let identity: TransformType = cgmath::Transform::one();

        for entity in 0..em.get_num_entities() {
            if !em.is_live(entity) {
                continue;
            }

            let parent = transforms[entity as usize].parent;            

            // Note that we go only one parent level: We guarantee that we'll
            // update our transforms in order, so that parents always come before children
            match parent {
                Some(parent_entity) => match em.get_entity_index(&parent_entity) {
                    Some(parent_id) => {
                        // TODO: Unnecessary clone here because I can't figure out having multiple refs to same vec...
                        let parent_transform = transforms[parent_id as usize].get_world_transform().clone();
                        transforms[entity as usize].update_world_transform(&parent_transform);
                    }
                    None => {}
                },
                None => {
                    transforms[entity as usize].update_world_transform(&identity);
                }
            }
        }
    }
}
