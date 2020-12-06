use crate::managers::ECManager;

use cgmath::Transform;

use super::{
    component::{ComponentIndex, ComponentStorageType},
    Component,
};

pub type TransformType = cgmath::Decomposed<cgmath::Vector3<f32>, cgmath::Quaternion<f32>>;

#[derive(Clone)]
pub struct TransformComponent {
    enabled: bool,

    local_transform: TransformType,
    world_transform: TransformType,
}
impl TransformComponent {
    pub fn new() -> Self {
        return Self::default();
    }

    pub fn get_local_transform(&self) -> &TransformType {
        return &self.local_transform;
    }

    pub fn get_local_transform_mut(&mut self) -> &mut TransformType {
        return &mut self.local_transform;
    }

    pub fn update_world_transform(&mut self, parent_local_transform: &TransformType) {
        self.world_transform = parent_local_transform.concat(&self.local_transform);
    }

    pub fn get_world_transform(&self) -> &TransformType {
        return &self.world_transform;
    }
}
impl Default for TransformComponent {
    fn default() -> Self {
        return Self {
            enabled: false,

            local_transform: cgmath::Decomposed {
                scale: 1.0,
                disp: cgmath::Vector3::new(0.0, 0.0, 0.0),
                rot: cgmath::Quaternion::new(1.0, 0.0, 0.0, 0.0),
            },
            world_transform: cgmath::Decomposed {
                scale: 1.0,
                disp: cgmath::Vector3::new(0.0, 0.0, 0.0),
                rot: cgmath::Quaternion::new(1.0, 0.0, 0.0, 0.0),
            },
        };
    }
}
impl Component for TransformComponent {
    type ComponentType = TransformComponent;
    const STORAGE_TYPE: ComponentStorageType = ComponentStorageType::Vec;

    fn get_component_index() -> ComponentIndex {
        return ComponentIndex::Transform;
    }

    fn get_components_vector<'a>(
        w: &'a mut ECManager,
    ) -> Option<&'a mut Vec<TransformComponent>> {
        return Some(&mut w.transform);
    }

    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    fn get_enabled(&mut self) -> bool {
        return self.enabled;
    }
}
