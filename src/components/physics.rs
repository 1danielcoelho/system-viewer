use crate::managers::ComponentManager;

use super::{Component, component::ComponentIndex};

pub struct PhysicsComponent {
    enabled: bool,

    pub collision_enabled: bool,
    pub lin_vel: cgmath::Vector3<f32>,
    pub ang_vel: cgmath::Vector3<f32>,
    pub mass: f32,
}
impl PhysicsComponent {
    fn new() -> Self {
        return Self::default();
    }

    fn set_lin_vel(&mut self, new_lin_vel: &cgmath::Vector3<f32>) {
        self.lin_vel = *new_lin_vel;
    }

    fn set_ang_vel(&mut self, new_ang_vel: &cgmath::Vector3<f32>) {
        self.ang_vel = *new_ang_vel;
    }
}
impl Default for PhysicsComponent {
    fn default() -> Self {
        return Self {
            enabled: false,
            collision_enabled: false,
            lin_vel: cgmath::Vector3::new(0.0, 0.0, 0.0),
            ang_vel: cgmath::Vector3::new(0.0, 0.0, 0.0),
            mass: 1.0,
        };
    }
}
impl Component for PhysicsComponent {
    type ComponentType = PhysicsComponent;

    fn get_component_index() -> ComponentIndex {
        return ComponentIndex::Physics;
    }

    fn get_components_vector<'a>(w: &'a mut ComponentManager) -> &'a mut Vec<PhysicsComponent> {
        return &mut w.physics;
    }

    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    fn get_enabled(&mut self) -> bool {
        return self.enabled;
    }
}
