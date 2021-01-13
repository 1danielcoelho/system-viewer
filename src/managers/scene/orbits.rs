use crate::components::light::LightType;
use crate::components::{LightComponent, MeshComponent, OrbitalComponent, TransformComponent};
use crate::managers::scene::{Entity, SceneManager};
use crate::managers::ResourceManager;
use crate::utils::orbits::{
    bake_eccentric_anomaly_times, elements_to_circle_transform, BodyDescription, BodyType,
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

        let body_ent = scene.new_entity(Some(&body.name));
        if let Some(parent) = parent_bary {
            scene.set_entity_parent(parent, body_ent);
        }

        let trans_comp = scene.add_component::<TransformComponent>(body_ent);

        // Sphere mesh
        if body.mean_radius.0 > 0.0 {
            let radius = body.mean_radius.0;

            trans_comp.get_local_transform_mut().scale = Vector3::new(radius, radius, radius);

            let mesh_comp = scene.add_component::<MeshComponent>(body_ent);
            mesh_comp.set_mesh(res_man.get_or_create_mesh("ico_sphere"));
            mesh_comp.set_material_override(res_man.get_or_create_material("phong"), 0);
        }

        if body.body_type == BodyType::Star {
            let light_comp = scene.add_component::<LightComponent>(body_ent);
            light_comp.color = Vector3::new(1.0, 1.0, 1.0);
            light_comp.intensity = 5E10;
            light_comp.light_type = LightType::Point;
        }

        // Orbit
        if body.orbital_elements.semi_major_axis.0 > 0.0 {
            let trans = elements_to_circle_transform(&body.orbital_elements);

            let orbit_comp = scene.add_component::<OrbitalComponent>(body_ent);
            orbit_comp.desc = body.clone(); // TODO: I could probably move this in
            orbit_comp.circle_to_final_ellipse = trans.clone();

            // Bake eccentric anomalies into the body
            if body.orbital_elements.sidereal_orbit_period_days > 0.0 {
                const NUM_ANGLES: u32 = 360;

                // Add eccentric anomaly interpolation values
                orbit_comp.baked_eccentric_anomaly_times =
                    bake_eccentric_anomaly_times(&body.orbital_elements, NUM_ANGLES);
            }

            // Orbit mesh entity
            {
                let orbit = scene.new_entity(Some(&(body.name.clone() + "'s orbit")));
                if let Some(parent) = parent_bary {
                    scene.set_entity_parent(parent, orbit);
                }

                let trans_comp = scene.add_component::<TransformComponent>(orbit);
                *trans_comp.get_local_transform_mut() = trans;

                let mesh_comp = scene.add_component::<MeshComponent>(orbit);
                mesh_comp.set_mesh(res_man.get_or_create_mesh("circle"));
            }
        }

        return body_ent;
    }
}
