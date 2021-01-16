use crate::app_state::AppState;
use crate::components::Component;
use crate::components::PhysicsComponent;
use crate::managers::scene::component_storage::ComponentStorage;
use crate::managers::scene::Scene;
use crate::managers::EventReceiver;
use na::{Matrix3, Quaternion, UnitQuaternion};

pub struct PhysicsSystem {}
impl PhysicsSystem {
    pub fn run(&self, state: &AppState, scene: &mut Scene) {
        // Load in current transforms
        for (ent, phys) in scene.physics.ent_iter_mut() {
            let trans = scene.transform.get_component(*ent).unwrap();

            phys.trans = trans.get_local_transform().clone(); 
        }

        // Run physics system
        for phys in scene.physics.iter_mut() {
            PhysicsSystem::update(state, phys);
        }

        // Unload new transforms
        for (ent, phys) in scene.physics.ent_iter() {
            let trans = scene.transform.get_component_mut(*ent).unwrap();

            *trans.get_local_transform_mut() = phys.trans.clone();
        }
    }

    // Applies semi-implicit Euler integration to update `physics` to time t
    pub fn update(state: &AppState, phys_comp: &mut PhysicsComponent) {
        if !phys_comp.get_enabled() {
            return;
        }
        
        let dt = state.sim_delta_time_days * 86400.0;

        // TODO: What if the object is scaled? Should that affect its linear/rotational motion?

        // TODO: Gyroscopic effects here

        // TODO: Allow applying forces off from the center of mass, generating torque

        // Update momenta
        phys_comp.lin_mom += phys_comp.force_sum * dt;
        phys_comp.ang_mom += phys_comp.torque_sum * dt;

        // Compute world-space inverse inertia tensor
        let rot_mat = Matrix3::from(phys_comp.trans.rot); // Assumes rot is normalized
        let inv_inertia_world = rot_mat * phys_comp.inv_inertia * rot_mat.transpose();

        // Update velocities
        let lin_vel = phys_comp.lin_mom * phys_comp.inv_mass;
        let ang_vel = inv_inertia_world * phys_comp.ang_mom;
        let ang_vel_q = Quaternion::new(0.0, ang_vel.x, ang_vel.y, ang_vel.z);

        // Update position and rotation
        phys_comp.trans.trans += lin_vel * dt;
        let new_rot = phys_comp.trans.rot.quaternion()
            + 0.5 * ang_vel_q * phys_comp.trans.rot.quaternion() * dt; // todo
        phys_comp.trans.rot = UnitQuaternion::new_normalize(new_rot);

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
