use crate::{
    app_state::AppState,
    components::PhysicsComponent,
    components::{Component, TransformComponent},
    managers::{scene::Scene, EventReceiver},
};
use na::{Matrix3, Quaternion, UnitQuaternion};

pub struct PhysicsSystem {}
impl PhysicsSystem {
    pub fn run(&self, state: &AppState, scene: &mut Scene) {
        for entity_index in 0..scene.transform.len() {
            // TODO: Indirection on the hot path...
            // TODO: Disable physics calculations for child entities for free bodies (rails should be OK though)
            // if scene
            //     .get_parent_index_from_index(entity_index as u32)
            //     .is_some()
            // {
            //     continue;
            // }

            PhysicsSystem::update(
                state,
                &mut scene.transform[entity_index],
                &mut scene.physics[entity_index],
            );
        }
    }

    // Applies semi-implicit Euler integration to update `transform` and `physics` to time t
    pub fn update(
        state: &AppState,
        trans_comp: &mut TransformComponent,
        phys_comp: &mut PhysicsComponent,
    ) {
        if !phys_comp.get_enabled() {
            return;
        }

        let dt = state.sim_delta_time_days * 0.001;

        // TODO: What if the object is scaled? Should that affect its linear/rotational motion?

        // TODO: Gyroscopic effects here

        // TODO: Allow applying forces off from the center of mass, generating torque

        // Update momenta
        phys_comp.lin_mom += phys_comp.force_sum * dt;
        phys_comp.ang_mom += phys_comp.torque_sum * dt;

        // Compute world-space inverse inertia tensor
        let trans = trans_comp.get_local_transform_mut();
        let rot_mat = Matrix3::from(trans.rot); // Assumes rot is normalized
        let inv_inertia_world = rot_mat * phys_comp.inv_inertia * rot_mat.transpose();

        // Update velocities
        let lin_vel = phys_comp.lin_mom * phys_comp.inv_mass;
        let ang_vel = inv_inertia_world * phys_comp.ang_mom;
        let ang_vel_q = Quaternion::new(0.0, ang_vel.x, ang_vel.y, ang_vel.z);

        // Update position and rotation
        trans.trans += lin_vel * dt;
        let new_rot = trans.rot.quaternion() + 0.5 * ang_vel_q * trans.rot.quaternion() * dt; // todo
        trans.rot = UnitQuaternion::new_normalize(new_rot);

        // Clear accumulators?

        // Detect collision

        // Solve constraints

        // events?
    }
}
impl EventReceiver for PhysicsSystem {
    fn receive_event(&mut self, _event: crate::managers::Event) {
        todo!()
    }
}
