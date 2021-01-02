use crate::{
    components::{MeshComponent, OrbitalComponent, TransformComponent},
    managers::{
        scene::{Entity, SceneManager},
        ResourceManager,
    },
    utils::{
        orbits::{
            bake_eccentric_anomaly_times, elements_to_circle_transform, BodyDescription,
            GRAVITATION_CONSTANT,
        },
        units::{Jdn, Rad},
    },
};
use na::Vector3;
use std::{collections::HashMap, f64::consts::PI};

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

        let grav_constant = parent_bary
            .and_then(|p| scene.get_component::<OrbitalComponent>(p))
            .and_then(|c| Some(GRAVITATION_CONSTANT * c.desc.mass));

        let mut parent_name = String::new();
        if let Some(parent_bary) = parent_bary {
            parent_name = scene.get_entity_name(parent_bary).unwrap().to_owned();
        }

        continue here
        log::info!("parent grav constant for {}: {:?}. Parent bary: {}", body.name, grav_constant, parent_name);

        // Body
        let body_ent = scene.new_entity(Some(&body.name));
        let trans_comp = scene.add_component::<TransformComponent>(body_ent).unwrap();
        if body.mean_radius.0 > 0.0 {
            trans_comp.get_local_transform_mut().scale =
                Vector3::new(body.mean_radius.0, body.mean_radius.0, body.mean_radius.0);

            let mesh_comp = scene.add_component::<MeshComponent>(body_ent).unwrap();
            mesh_comp.set_mesh(res_man.get_or_create_mesh("ico_sphere"));

            let orbit_comp = scene.add_component::<OrbitalComponent>(body_ent).unwrap();
            orbit_comp.desc = body.clone();

            if let Some(grav_constant) = grav_constant {
                let orbital_period =
                    2.0 * PI / (grav_constant / body.orbital_elements.semi_major_axis.0).sqrt();

                log::info!("Body {} has a period of {} days", body.name, orbital_period);

                const NUM_ANGLES: u32 = 360;

                // Add angles for eccentric anomaly interpolation
                orbit_comp
                    .baked_eccentric_anomaly_times
                    .resize((NUM_ANGLES + 1) as usize, Jdn(0.0));
                let incr = 2.0 * PI / NUM_ANGLES as f64;
                for i in 0..=NUM_ANGLES {
                    orbit_comp.baked_eccentric_anomaly_angles[i as usize] = Rad(i as f64 * incr);
                }

                // Add eccentric anomaly interpolation values
                orbit_comp.baked_eccentric_anomaly_times = bake_eccentric_anomaly_times(
                    &body.orbital_elements,
                    orbital_period,
                    NUM_ANGLES,
                );
            }

            if let Some(parent) = parent_bary {
                scene.set_entity_parent(parent, body_ent);
            }
        }

        // Orbit
        if body.orbital_elements.semi_major_axis.0 > 0.0 {
            let orbit_trans = elements_to_circle_transform(&body.orbital_elements);

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
