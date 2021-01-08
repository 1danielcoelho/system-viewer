use crate::{app_state::ReferenceChange, components::TransformComponent};
use crate::{
    app_state::{AppState, ButtonState},
    managers::scene::Scene,
    utils::transform::Transform,
};
use na::{Matrix4, Point3, Translation3, Unit, Vector3};

pub struct TransformUpdateSystem {}
impl TransformUpdateSystem {
    pub fn run(&self, state: &mut AppState, scene: &mut Scene) {
        let identity = Transform::identity();

        for entity_index in 0..scene.get_num_entities() {
            // TODO: This has an indirection
            let parent_index = scene.get_parent_index_from_index(entity_index);

            // Note that we go only one parent level: We guarantee that we'll
            // update our transforms in order, so that parents always come before children
            match parent_index {
                Some(parent_index) => {
                    // TODO: Indirection, clones TWICE (once inside update_world_transform)
                    let parent_transform = scene.transform[parent_index as usize]
                        .get_world_transform()
                        .clone();
                    scene.transform[entity_index as usize]
                        .update_world_transform(&parent_transform);
                }
                None => {
                    // What
                    scene.transform[entity_index as usize].update_world_transform(&identity);
                }
            }
        }

        rebase_camera_transform(state, scene);

        focus_camera_on_selection(state, scene);
    }
}

/// If we have a `state.camera.next_reference_entity`, this will update the camera `pos`/`target`/`up` to be with respect to
/// that entity's transform instead.
///
/// This function expects that world transforms are finalized.
fn rebase_camera_transform(state: &mut AppState, scene: &mut Scene) {
    let old_entity = &mut state.camera.reference_entity;
    let new_entity = &mut state.camera.next_reference_entity;

    // Don't do anything if asked to changed to the same entity
    if let Some(ReferenceChange::NewEntity(entity)) = new_entity {
        if old_entity.is_some() && *entity == old_entity.unwrap() {
            *new_entity = None;
        }
    }

    if new_entity.is_none() {
        return;
    }
    let new_entity = new_entity.as_ref().unwrap();

    let pos = na::convert::<Point3<f32>, Point3<f64>>(state.camera.pos);
    let target = na::convert::<Point3<f32>, Point3<f64>>(state.camera.target);
    let up = na::convert::<Vector3<f32>, Vector3<f64>>(*state.camera.up);

    let old_to_world = match old_entity.and_then(|e| scene.get_component::<TransformComponent>(e)) {
        Some(old_comp) => Translation3::from(old_comp.get_world_transform().trans).to_homogeneous(),
        None => Matrix4::identity(),
    };

    // From world to new
    let world_to_new = match new_entity {
        ReferenceChange::NewEntity(new_entity) => Translation3::from(
            -scene
                .get_component::<TransformComponent>(*new_entity)
                .unwrap()
                .get_world_transform()
                .trans,
        )
        .to_homogeneous(),
        ReferenceChange::Clear => Matrix4::identity(),
    };

    let trans = world_to_new * old_to_world;

    state.camera.pos = na::convert(trans.transform_point(&pos));
    state.camera.target = na::convert(trans.transform_point(&target));
    state.camera.up = Unit::new_normalize(na::convert(trans.transform_vector(&up)));

    match new_entity {
        ReferenceChange::NewEntity(new_entity) => state.camera.reference_entity = Some(*new_entity),
        ReferenceChange::Clear => state.camera.reference_entity = None,
    };
    state.camera.next_reference_entity = None;
}

fn focus_camera_on_selection(state: &mut AppState, scene: &mut Scene) {
    if state.input.f != ButtonState::Pressed {
        return;
    }

    let target_entity = state.selection.iter().next();
    if target_entity.is_none() {
        return;
    }
    let target_entity = target_entity.unwrap();

    // Set the target entity as our reference first
    if state.camera.reference_entity != Some(*target_entity) {
        state.camera.next_reference_entity = Some(ReferenceChange::NewEntity(*target_entity));
        rebase_camera_transform(state, scene);
    }

    state.camera.pos = Point3::new(100.0, 100.0, 100.0);
    state.camera.target = Point3::new(0.0, 0.0, 0.0);
}
