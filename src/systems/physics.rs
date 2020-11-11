use cgmath::{
    EuclideanSpace, InnerSpace, Matrix, Matrix3, Point3, Quaternion, Rad, Rotation3, Transform,
    Vector3,
};

use crate::{
    app_state::AppState, components::PhysicsComponent, components::TransformComponent,
    managers::EventReceiver,
};

pub struct PhysicsSystem {}
impl PhysicsSystem {
    pub fn run(
        &self,
        state: &AppState,
        transforms: &mut Vec<TransformComponent>,
        physics: &mut Vec<PhysicsComponent>,
    ) {
        for entity in 0..transforms.len() {
            PhysicsSystem::update(state, &mut transforms[entity], &mut physics[entity]);
        }
    }

    // Applies semi-implicit Euler integration to update `transform` and `physics` to time t
    pub fn update(
        state: &AppState,
        trans_comp: &mut TransformComponent,
        phys_comp: &mut PhysicsComponent,
    ) {
        // Only parent entities are subject to physics for now
        if !phys_comp.physics_enabled || trans_comp.parent.is_some() {
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
    fn receive_event(&mut self, event: crate::managers::Event) {
        todo!()
    }
}
