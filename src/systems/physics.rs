use cgmath::{InnerSpace, Matrix, Matrix3, Quaternion, Vector3};

use crate::{
    app_state::AppState,
    components::PhysicsComponent,
    components::{Component, TransformComponent},
    managers::ECManager,
    managers::EventReceiver,
};

pub struct PhysicsSystem {}
impl PhysicsSystem {
    pub fn run(&self, state: &AppState, ent_man: &mut ECManager) {
        for entity_index in 0..ent_man.transform.len() {
            // TODO: Indirection on the hot path...
            if ent_man
                .get_parent_index_from_index(entity_index as u32)
                .is_some()
            {
                continue;
            }

            PhysicsSystem::update(
                state,
                &mut ent_man.transform[entity_index],
                &mut ent_man.physics[entity_index],
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

        let dt = (state.phys_delta_time_ms * 0.001) as f32;

        // TODO: What if the object is scaled? Should that affect its linear/rotational motion?

        // TODO: Gyroscopic effects here

        // TODO: Allow applying forces off from the center of mass, generating torque

        // Update momenta
        phys_comp.lin_mom += phys_comp.force_sum * dt;
        phys_comp.ang_mom += phys_comp.torque_sum * dt;

        // Compute world-space inverse inertia tensor
        let trans = trans_comp.get_local_transform_mut();
        let rot_mat = Matrix3::from(trans.rot); // Assumes rot is normalized
        let inv_inertia_world: Matrix3<f32> = rot_mat * phys_comp.inv_inertia * rot_mat.transpose();

        // Update velocities
        let lin_vel = phys_comp.lin_mom * phys_comp.inv_mass;
        let ang_vel: Vector3<f32> = inv_inertia_world * phys_comp.ang_mom;
        let ang_vel_q: Quaternion<f32> = cgmath::Quaternion::from_sv(0.0, ang_vel);

        // Update position and rotation
        trans.disp += lin_vel * dt;
        trans.rot += 0.5 * ang_vel_q * trans.rot * dt; // todo
        trans.rot = trans.rot.normalize();

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
