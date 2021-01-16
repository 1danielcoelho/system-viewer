use crate::app_state::AppState;
use crate::components::OrbitalComponent;
use crate::components::TransformComponent;
use crate::managers::scene::component_storage::ComponentStorage;
use crate::managers::{scene::Scene, EventReceiver};
use crate::utils::orbits::get_eccentric_anomaly;
use crate::utils::units::{Jdn, J2000_JDN};
use na::Point3;

pub struct OrbitalSystem {}
impl OrbitalSystem {
    pub fn run(&self, state: &AppState, scene: &mut Scene) {
        for (ent, orb) in scene.orbital.iter() {
            let trans = scene.transform.get_component_mut(*ent).unwrap();

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
