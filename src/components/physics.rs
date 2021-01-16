use crate::components::Component;
use crate::managers::scene::component_storage::ComponentStorage;
use crate::managers::{details_ui::DetailsUI, scene::Scene};
use crate::utils::transform::Transform;
use na::{Matrix3, Vector3};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysicsComponent {
    enabled: bool,
    pub collision_enabled: bool,

    // Constants
    /// Kg
    pub mass: f64,
    /// Local space
    pub inv_inertia: Matrix3<f64>, 

    // Inputs/computed
    /// kg * Mm/s^2 
    pub force_sum: Vector3<f64>, 
    pub torque_sum: Vector3<f64>,

    // State
    /// kg * Mm/s
    pub lin_mom: Vector3<f64>, 
    /// length is kg * Mm^2 * rad/s, right-hand rule
    pub ang_mom: Vector3<f64>, 
    pub trans: Transform<f64>,
}

impl PhysicsComponent {
    #[allow(dead_code)]
    fn new() -> Self {
        return Self::default();
    }
}

impl Default for PhysicsComponent {
    fn default() -> Self {
        return Self {
            enabled: false,
            collision_enabled: false,
            mass: 1.0,
            inv_inertia: Matrix3::identity(),
            force_sum: Vector3::new(0.0, 0.0, 0.0),
            torque_sum: Vector3::new(0.0, 0.0, 0.0),
            lin_mom: Vector3::new(0.0, 0.0, 0.0),
            ang_mom: Vector3::new(0.0, 0.0, 0.0),
            trans: Transform::identity(),
        };
    }
}

impl Component for PhysicsComponent {
    type ComponentType = PhysicsComponent;

    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    fn get_enabled(&self) -> bool {
        return self.enabled;
    }

    fn get_storage(scene: &mut Scene) -> Box<&mut dyn ComponentStorage<Self::ComponentType>> {
        return Box::new(&mut scene.physics);
    }
}

impl DetailsUI for PhysicsComponent {}
