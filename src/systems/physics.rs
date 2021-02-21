use crate::app_state::AppState;
use crate::components::Component;
use crate::components::PhysicsComponent;
use crate::managers::scene::component_storage::ComponentStorage;
use crate::managers::scene::Scene;
use crate::managers::EventReceiver;
use crate::utils::orbits::GRAVITATION_CONSTANT;
use na::*;

pub struct PhysicsSystem {}
impl PhysicsSystem {
    pub fn run(&self, state: &AppState, scene: &mut Scene) {
        // Load in current transforms
        for (ent, phys) in scene.physics.ent_iter_mut() {
            let trans = scene.transform.get_component(*ent).unwrap();

            phys.trans = trans.get_local_transform().clone();
        }

        // Collect forces
        collect_gravity(scene);

        // Update state vector
        for phys in scene.physics.iter_mut() {
            update_comp(state, phys);
        }

        // Unload new transforms
        for (ent, phys) in scene.physics.ent_iter() {
            let trans = scene.transform.get_component_mut(*ent).unwrap();

            *trans.get_local_transform_mut() = phys.trans.clone();
        }
    }
}
impl EventReceiver for PhysicsSystem {
    fn receive_event(&mut self, _event: crate::managers::Event) {
        todo!()
    }
}

fn collect_gravity(scene: &mut Scene) {
    let phys_comps = scene.physics.get_storage_mut();
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
            let force: Vector3<f64> =
                delta.normalize() * GRAVITATION_CONSTANT * mass * other_comp.mass / (dist * dist);

            phys_comps[i].force_sum += force;
            phys_comps[j].force_sum += -force;
        }
    }
}

// Applies semi-implicit Euler integration to update `physics` to time t
fn update_comp(state: &AppState, phys_comp: &mut PhysicsComponent) {
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
    let ang_vel_q = Quaternion::new(0.0, ang_vel.x, ang_vel.y, ang_vel.z);

    // Update position and rotation
    phys_comp.trans.trans += lin_vel * dt_s;
    let new_rot = phys_comp.trans.rot.quaternion()
        + 0.5 * ang_vel_q * phys_comp.trans.rot.quaternion() * dt_s; // todo
    phys_comp.trans.rot = UnitQuaternion::new_normalize(new_rot);

    // Clear accumulators
    phys_comp.force_sum = Vector3::new(0.0, 0.0, 0.0);
    phys_comp.torque_sum = Vector3::new(0.0, 0.0, 0.0);

    // Detect collision

    // Solve constraints

    // events?
}
