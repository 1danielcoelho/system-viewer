use crate::managers::ComponentManager;

use super::{Component, ComponentIndex};

pub type TransformType = cgmath::Decomposed<cgmath::Vector3<f32>, cgmath::Quaternion<f32>>;

pub struct TransformComponent {
    enabled: bool,

    pub transform: TransformType,
    pub parent: u32,
    pub children: Vec<u32>,
}
impl TransformComponent {
    pub fn new() -> Self {
        return Self::default();
    }

    pub fn set_parent(&mut self, new_parent: u32) {
        self.parent = new_parent;
    }
}
impl Default for TransformComponent {
    fn default() -> Self {
        return Self {
            enabled: false,
            transform: cgmath::Decomposed {
                scale: 1.0,
                disp: cgmath::Vector3::new(0.0, 0.0, 0.0),
                rot: cgmath::Quaternion::new(1.0, 0.0, 0.0, 0.0),
            },
            parent: 0,
            children: vec![0],
        };
    }
}
impl Component for TransformComponent {
    type ComponentType = TransformComponent;

    fn get_component_index() -> ComponentIndex {
        return ComponentIndex::Transform;
    }

    fn get_components_vector<'a>(w: &'a mut ComponentManager) -> &'a mut Vec<TransformComponent> {
        return &mut w.transform;
    }

    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    fn get_enabled(&mut self) -> bool {
        return self.enabled;
    }
}
