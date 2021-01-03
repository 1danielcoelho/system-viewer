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

    pub fn update(
        state: &AppState,
        trans_comp: &mut TransformComponent,
        orbit_comp: &OrbitalComponent,
    ) {
        let current_time = Jdn(state.sim_time_days + J2000_JDN.0);

        let eccentric_anomaly = get_eccentric_anomaly(
            current_time,
            orbit_comp.desc.orbital_elements.sidereal_orbit_period_days,
            &orbit_comp.baked_eccentric_anomaly_times,
        );

        trans_comp.get_local_transform_mut().trans = orbit_comp
            .circle_to_final_ellipse
            .transform_point(&Point3::new(
                eccentric_anomaly.0.cos(),
                eccentric_anomaly.0.sin(),
                0.0,
            ))
            .coords;
    }
}
impl EventReceiver for OrbitalSystem {
    fn receive_event(&mut self, _event: crate::managers::Event) {
        todo!()
    }
}
