use crate::app_state::AppState;
use crate::components::RigidBodyComponent;
use crate::components::{Component, KinematicComponent, TransformComponent};
use crate::managers::scene::component_storage::ComponentStorage;
use crate::managers::scene::Scene;
use crate::managers::EventReceiver;
use crate::utils::orbits::GRAVITATION_CONSTANT;
use na::*;

pub struct PhysicsSystem {}
impl PhysicsSystem {
    pub fn run(&self, state: &AppState, scene: &mut Scene) {
        if state.simulation_paused || state.simulation_speed == 0.0 {
            return;
        }

        // Load in current transforms
        for (ent, phys) in scene.rigidbody.ent_iter_mut() {
            let trans = scene.transform.get_component(*ent).unwrap();

            phys.trans = trans.get_local_transform().clone();
        }

        // Collect forces
        collect_gravity(scene);

        // Update state vector
        for phys in scene.rigidbody.iter_mut() {
            update_rigidbody(state, phys);
        }

        // Unload new transforms
        for (ent, phys) in scene.rigidbody.ent_iter() {
            let trans = scene.transform.get_component_mut(*ent).unwrap();

            *trans.get_local_transform_mut() = phys.trans.clone();
        }

        // Kinematic components
        // TODO: There's probably a way of doing both at once
        for (ent, kin) in scene.kinematic.ent_iter_mut() {
            let trans = scene.transform.get_component_mut(*ent).unwrap();
            update_kinematic(state, kin, trans);
        }
    }
}
impl EventReceiver for PhysicsSystem {
    fn receive_event(&mut self, _event: crate::managers::Event) {
        todo!()
    }
}

fn collect_gravity(scene: &mut Scene) {
    let phys_comps = scene.rigidbody.get_storage_mut();
    if phys_comps.len() < 2 {
        return;
    }

    for i in 0..phys_comps.len() {
        let this_comp = &phys_comps[i];

        let pos = this_comp.trans.trans;
        let mass = this_comp.mass;

        for j in i + 1..phys_comps.len() {
            let other_comp = &phys_comps[j];

            let delta = other_comp.trans.trans - pos;
            let dist = delta.magnitude();

            // TEMP Safety in case something causes two bodies to be right on top of eachother
            // Should go away once I properly implement collision
            let force: Vector3<f64>;
            if dist < 1E-10 {
                force = Vector3::zeros()
            } else {
                force = delta.normalize() * GRAVITATION_CONSTANT * mass * other_comp.mass
                    / (dist * dist);
            }

            phys_comps[i].force_sum += force;
            phys_comps[j].force_sum += -force;
        }
    }
}

// Applies semi-implicit Euler integration to update `physics` to time t
fn update_rigidbody(state: &AppState, phys_comp: &mut RigidBodyComponent) {
    if !phys_comp.get_enabled() {
        return;
    }

    let dt_s = state.sim_delta_time_s;

    // TODO: What if the object is scaled? Should that affect its linear/rotational motion?

    // TODO: Gyroscopic effects here

    // TODO: Allow applying forces off from the center of mass, generating torque

    // Update momenta
    phys_comp.lin_mom += phys_comp.force_sum * dt_s;
    phys_comp.ang_mom += phys_comp.torque_sum * dt_s;

    // Compute world-space inverse inertia tensor
    let rot_mat = Matrix3::from(phys_comp.trans.rot); // Assumes rot is normalized
    let inv_inertia_world = rot_mat * phys_comp.inv_inertia * rot_mat.transpose();

    // Update velocities
    let lin_vel = phys_comp.lin_mom / phys_comp.mass;
    let ang_vel = inv_inertia_world * phys_comp.ang_mom;
    let ang_vel_q = UnitQuaternion::from_scaled_axis(ang_vel * dt_s);

    // Update position and rotation
    phys_comp.trans.trans += lin_vel * dt_s;
    phys_comp.trans.rot *= ang_vel_q;

    // Clear accumulators
    phys_comp.force_sum = Vector3::new(0.0, 0.0, 0.0);
    phys_comp.torque_sum = Vector3::new(0.0, 0.0, 0.0);

    // Detect collision

    // Solve constraints

    // events?
}

fn update_kinematic(state: &AppState, kin: &KinematicComponent, trans: &mut TransformComponent) {
    let transform = trans.get_local_transform_mut();
    transform.trans += kin.lin_vel * state.sim_delta_time_s;
    transform.rot *= UnitQuaternion::from_scaled_axis(kin.ang_vel * state.sim_delta_time_s);
}
