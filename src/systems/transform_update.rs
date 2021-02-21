use crate::app_state::AppState;
use crate::app_state::ReferenceChange;
use crate::components::TransformComponent;
use crate::managers::scene::Scene;
use na::*;

pub struct TransformUpdateSystem {}
impl TransformUpdateSystem {
    pub fn run(&self, state: &mut AppState, scene: &mut Scene) {
        concatenate_parent_transforms(scene);

        update_reference_translation(state, scene);

        handle_reference_changes(state, scene);

        handle_go_to(state, scene);
    }
}

fn concatenate_parent_transforms(scene: &mut Scene) {
    for entity_index in 0..scene.get_num_entities() {
        let parent_trans = scene
            .get_parent_index_from_index(entity_index)
            .and_then(|parent_index| scene.transform.get_component_from_index(parent_index))
            .and_then(|parent_trans| Some(parent_trans.get_world_transform().clone()));

        if let Some(ent_trans) = scene.transform.get_component_from_index_mut(entity_index) {
            match parent_trans {
                Some(mut trans) => {
                    trans.concat(ent_trans.get_local_transform());
                    *ent_trans.get_world_transform_mut() = trans;
                }
                None => {
                    *ent_trans.get_world_transform_mut() = ent_trans.get_local_transform().clone()
                }
            };
        }
    }
}

/// Fetch the intended reference entity and store on the state its translation directly.
/// This so that the camera and other consumers don't have to all poke around the scene to find it
/// Plus this way it is always up to date and a single consistent value throughout all uses
fn update_reference_translation(state: &mut AppState, scene: &mut Scene) {
    if state.camera.reference_entity.is_none() {
        state.camera.reference_translation = None;
        return;
    }

    let ref_ent = state.camera.reference_entity.unwrap();

    let trans = scene
        .get_component::<TransformComponent>(ref_ent)
        .and_then(|c| Some(c.get_world_transform().trans));

    if trans.is_none() {
        log::warn!(
            "Found no transform component for tracked entity '{:?}'",
            ref_ent
        );
    }

    state.camera.reference_translation = trans;
}

/// If we have a `state.camera.next_reference_entity`, this will update the camera `pos`/`target`/`up` to be with respect to
/// that entity's transform instead.
///
/// This function expects that world transforms are finalized, and that referen_translation is
/// up-to-date
fn handle_reference_changes(state: &mut AppState, scene: &mut Scene) {
    let old_entity = &mut state.camera.reference_entity;
    let new_entity = &mut state.camera.next_reference_entity;

    // Don't do anything if asked to changed to the same entity
    if let Some(ReferenceChange::TrackKeepLocation(entity)) = new_entity {
        if old_entity.is_some() && *entity == old_entity.unwrap() {
            *new_entity = None;
        }
    }
    if let Some(ReferenceChange::TrackKeepCoords(entity)) = new_entity {
        if old_entity.is_some() && *entity == old_entity.unwrap() {
            *new_entity = None;
        }
    }
    if new_entity.is_none() {
        return;
    }

    let new_entity = new_entity.as_ref().unwrap();
    match new_entity {
        ReferenceChange::TrackKeepLocation(new_entity) => {
            let old_to_world = match state.camera.reference_translation {
                Some(old_trans) => Translation3::from(old_trans).to_homogeneous(),
                None => Matrix4::identity(),
            };

            let world_to_new = Translation3::from(
                -scene
                    .get_component::<TransformComponent>(*new_entity)
                    .unwrap()
                    .get_world_transform()
                    .trans,
            )
            .to_homogeneous();

            let trans = world_to_new * old_to_world;

            state.camera.pos = trans.transform_point(&state.camera.pos);
            state.camera.up = Unit::new_normalize(trans.transform_vector(&state.camera.up));
            state.camera.target = Point3::new(0.0, 0.0, 0.0);
            state.camera.reference_entity = Some(*new_entity);
        }
        ReferenceChange::TrackKeepCoords(new_entity) => {
            state.camera.reference_entity = Some(*new_entity);
        }
        ReferenceChange::Clear => {
            let old_to_world = match state.camera.reference_translation {
                Some(old_trans) => Translation3::from(old_trans).to_homogeneous(),
                None => Matrix4::identity(),
            };

            state.camera.pos = old_to_world.transform_point(&state.camera.pos);
            state.camera.up = Unit::new_normalize(old_to_world.transform_vector(&state.camera.up));
            state.camera.target = old_to_world.transform_point(&state.camera.target);
            state.camera.reference_entity = None;
        }
    };

    // If we changed reference, we need to cache the reference translation again
    // so that other calculations done this frame are correct
    update_reference_translation(state, scene);

    state
        .camera
        .update_transforms(state.canvas_width as f64 / state.canvas_height as f64);

    state.camera.next_reference_entity = None;
}

fn handle_go_to(state: &mut AppState, scene: &mut Scene) {
    if state.camera.entity_going_to.is_none() {
        return;
    }
    let target_entity = state.camera.entity_going_to.unwrap();

    let transform = scene
        .get_component::<TransformComponent>(target_entity)
        .unwrap()
        .get_world_transform();

    let offset = transform.scale.mean() * 2.0;

    let mut target_pos = transform.trans;
    let mut camera_pos = target_pos + Vector3::new(offset, offset, offset);

    if let Some(reference_trans) = state.camera.reference_translation {
        target_pos -= reference_trans;
        camera_pos -= reference_trans;
    }

    state.camera.pos = Point3::from(camera_pos);
    state.camera.target = Point3::from(target_pos);
    state.camera.entity_going_to = None;
}
