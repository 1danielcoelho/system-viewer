use super::{
    component::{ComponentStorageType, ComponentType},
    Component,
};
use crate::managers::{details_ui::DetailsUI, scene::Scene};
use na::{Matrix3, Vector3};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysicsComponent {
    enabled: bool,
    pub collision_enabled: bool,

    // Constants
    pub inv_mass: f64,             // kg
    pub inv_inertia: Matrix3<f64>, // Local space

    // Inputs/computed
    pub force_sum: Vector3<f64>, // Sum of forces being applied to center of mass
    pub torque_sum: Vector3<f64>, // Sum of torque being applied to center of mass

    // State
    pub lin_mom: Vector3<f64>, // kg * m/s
    pub ang_mom: Vector3<f64>, // length is kg * m2 * rad/s, right-hand rule
}
impl PhysicsComponent {
    fn new() -> Self {
        return Self::default();
    }
}
impl Default for PhysicsComponent {
    fn default() -> Self {
        return Self {
            enabled: false,
            collision_enabled: false,
            inv_mass: 1.0,
            inv_inertia: Matrix3::identity(),
            force_sum: Vector3::new(0.0, 0.0, 0.0),
            torque_sum: Vector3::new(0.0, 0.0, 0.0),
            lin_mom: Vector3::new(0.0, 0.0, 0.0),
            ang_mom: Vector3::new(0.0, 0.0, 0.0),
        };
    }
}
impl Component for PhysicsComponent {
    type ComponentType = PhysicsComponent;
    const STORAGE_TYPE: ComponentStorageType = ComponentStorageType::Vec;
    const COMPONENT_TYPE: ComponentType = ComponentType::Physics;

    fn get_components_vector<'a>(w: &'a mut Scene) -> Option<&'a mut Vec<PhysicsComponent>> {
        return Some(&mut w.physics);
    }

    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    fn get_enabled(&mut self) -> bool {
        return self.enabled;
    }
}

impl DetailsUI for PhysicsComponent {}
