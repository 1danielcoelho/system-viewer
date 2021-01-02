use crate::{
    app_state::AppState,
    components::OrbitalComponent,
    components::{Component, TransformComponent},
    managers::{scene::Scene, EventReceiver},
    utils::{
        orbits::{elements_to_ellipse_rotation_transform, get_eccentric_anomaly},
        transform::Transform,
        units::{Jdn, J2000_JDN},
    },
};
use na::{Matrix3, Point3, Quaternion, UnitQuaternion, Vector3};

pub struct OrbitalSystem {}
impl OrbitalSystem {
    pub fn run(&self, state: &AppState, scene: &mut Scene) {
        for (ent, orb) in scene.orbital.iter() {
            let index = scene.get_entity_index(*ent).unwrap();
            let trans = &mut scene.transform[index as usize];

            OrbitalSystem::update(state, trans, orb);
        }
    }

    // Applies semi-implicit Euler integration to update `transform` and `physics` to time t
    pub fn update(
        state: &AppState,
        trans_comp: &mut TransformComponent,
        orbit_comp: &OrbitalComponent,
    ) {
        let current_time = Jdn(state.sim_time_days + J2000_JDN.0);

        let eccentric_anomaly = get_eccentric_anomaly(
            current_time,
            orbit_comp.desc.orbital_elements.sidereal_orbit_period_days,
            &orbit_comp.baked_eccentric_anomaly_angles,
            &orbit_comp.baked_eccentric_anomaly_times,
        );

        let orb = &orbit_comp.desc.orbital_elements;
        let b = orb.semi_major_axis.0 * (1.0 - orb.eccentricity.powi(2));

        let planar_x = orb.semi_major_axis.0 * eccentric_anomaly.0.cos();
        let planar_y = b * eccentric_anomaly.0.sin();

        let transform = elements_to_ellipse_rotation_transform(orb).concat_clone(&Transform {
            trans: Vector3::new(-orb.semi_major_axis.0 * orb.eccentricity, 0.0, 0.0),
            ..Transform::identity()
        });

        trans_comp.get_local_transform_mut().trans = transform
            .transform_point(&Point3::new(planar_x, planar_y, 0.0))
            .coords;
    }
}
impl EventReceiver for OrbitalSystem {
    fn receive_event(&mut self, _event: crate::managers::Event) {
        todo!()
    }
}
