use crate::app_state::{AppState, ButtonState};
use crate::managers::scene::Scene;
use crate::{app_state::ReferenceChange, components::TransformComponent};
use na::{Matrix4, Point3, Translation3, Unit};

pub struct TransformUpdateSystem {}
impl TransformUpdateSystem {
    pub fn run(&self, state: &mut AppState, scene: &mut Scene) {
        concatenate_parent_transforms(scene);

        update_reference_translation(state, scene);

        rebase_camera_transform(state, scene);

        focus_camera_on_selection(state, scene);
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
    state.camera.reference_translation = if let Some(ref_ent) = state.camera.reference_entity {
        Some(
            scene
                .get_component::<TransformComponent>(ref_ent)
                .unwrap()
                .get_world_transform()
                .trans,
        )
    } else {
        None
    };
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

    let old_to_world = match old_entity.and_then(|e| scene.get_component::<TransformComponent>(e)) {
        Some(old_comp) => Translation3::from(old_comp.get_world_transform().trans).to_homogeneous(),
        None => Matrix4::identity(),
    };

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

    state.camera.pos = trans.transform_point(&state.camera.pos);
    state.camera.target = trans.transform_point(&state.camera.target);
    state.camera.up = Unit::new_normalize(trans.transform_vector(&state.camera.up));

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
