use crate::{
    components::{MeshComponent, OrbitalComponent, TransformComponent},
    managers::{
        scene::{Entity, SceneManager},
        ResourceManager,
    },
    utils::orbits::{elements_to_circle_transform, BodyDescription},
};
use na::Vector3;
use std::collections::HashMap;

impl SceneManager {
    pub fn load_bodies_into_scene(
        &mut self,
        bodies: &Vec<BodyDescription>,
        res_man: &mut ResourceManager,
    ) {
        log::info!("Loading {} bodies...", bodies.len());

        // The idea here is that bodies are only ever parented to barycenters
        // Barycenters will themselves translate, but never rotate or scale, so nested orbits look ok-for-now-I-guess

        let mut id_to_entity: HashMap<u32, Entity> = HashMap::new();
        for body in bodies {
            let parent = {
                if body.reference_id == body.id {
                    None
                } else {
                    // We expect these bodies to be in order, so we should already have
                    // parsed our parent
                    Some(id_to_entity[&body.reference_id])
                }
            };

            let body_ent = self.add_body_entities(body, parent, res_man);
            if let Some(_) = id_to_entity.insert(body.id, body_ent) {
                log::warn!("Body collision when parsing csv for body: '{:#?}'", body);
            }
        }

        log::info!("Loaded {} bodies into the scene", id_to_entity.len());
    }

    /// Adds these entities to the scene
    /// 1) Barycenter/Body entity (+geometry) around the parent barycenter (if available), parented to it (or free body);
    /// 2) Orbit entity (+geometry) around the parent barycenter, parented to it (optional),
    ///
    /// Returns the barycenter/body entity so that we can parent other stuff to it.
    fn add_body_entities(
        &mut self,
        body: &BodyDescription,
        parent_bary: Option<Entity>,
        res_man: &mut ResourceManager,
    ) -> Entity {
        let scene = self.get_main_scene_mut().unwrap();

        log::info!("{:#?}", body);

        // Body
        let body_ent = scene.new_entity(Some(&body.name));
        let trans_comp = scene.add_component::<TransformComponent>(body_ent).unwrap();
        if body.mean_radius.0 > 0.0 {
            // log::info!("Transform for planet body {:#?}", body);

            // trans_comp.get_local_transform_mut().trans =
            //     Vector3::new(1000.0, 0.0, 0.0);

            // Problem here is that the translation is being scaled... something like Pluto has a radius but also an orbit, and we 
            // want it to be scaled so that it's sphere is the correct size, but it's orbit will also be scaled for some reason?

            // If we scale the barycenter, anything we parent to it will be scaled

            trans_comp.get_local_transform_mut().scale = Vector3::new(
                body.mean_radius.0,
                body.mean_radius.0,
                body.mean_radius.0,
            );

            let mesh_comp = scene.add_component::<MeshComponent>(body_ent).unwrap();
            mesh_comp.set_mesh(res_man.get_or_create_mesh("ico_sphere"));

            let orbit_comp = scene.add_component::<OrbitalComponent>(body_ent).unwrap();
            orbit_comp.desc = body.clone();

            if let Some(parent) = parent_bary {
                scene.set_entity_parent(parent, body_ent);
            }
        }

        // Orbit
        if body.orbital_elements.semi_major_axis.0 > 0.0 {
            let orbit_trans = elements_to_circle_transform(&body.orbital_elements);
            // log::info!("Transform for body {:#?}: {:#?}", body, orbit_trans);

            let orbit = scene.new_entity(Some(&(body.name.clone() + "'s orbit")));
            let trans_comp = scene.add_component::<TransformComponent>(orbit).unwrap();
            *trans_comp.get_local_transform_mut() = orbit_trans;

            let mesh_comp = scene.add_component::<MeshComponent>(orbit).unwrap();
            mesh_comp.set_mesh(res_man.get_or_create_mesh("circle"));

            if let Some(parent) = parent_bary {
                scene.set_entity_parent(parent, orbit);
            }
        }

        return body_ent;
    }
}
