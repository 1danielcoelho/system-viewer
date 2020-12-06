use std::collections::HashMap;

use cgmath::{Matrix3, Vector3};

use crate::managers::{ECManager, Entity};

use super::{
    component::{ComponentIndex, ComponentStorageType},
    Component,
};

#[derive(Clone)]
pub struct LightComponent {
    enabled: bool,

    color: cgmath::Vector3<f32>,
    intensity: f32,

    direction: Option<cgmath::Vector3<f32>>,
    max_distance: Option<f32>,
    angle_deg: Option<f32>,
}
impl LightComponent {
    fn new() -> Self {
        return Self::default();
    }
}
impl Default for LightComponent {
    fn default() -> Self {
        return Self {
            enabled: false,
            color: cgmath::Vector3::new(0.0, 0.0, 0.0), // Needs to be black so that unused lights don't affect the scene
            intensity: 0.0,
            direction: Some(cgmath::Vector3::new(0.0, 0.0, -1.0)), // Pointing down
            max_distance: Some(10000.0),
            angle_deg: Some(30.0),
        };
    }
}
impl Component for LightComponent {
    type ComponentType = LightComponent;
    const STORAGE_TYPE: ComponentStorageType = ComponentStorageType::Vec;

    fn get_component_index() -> ComponentIndex {
        return ComponentIndex::Physics;
    }

    fn get_components_map<'a>(
        w: &'a mut ECManager,
    ) -> Option<&'a mut HashMap<Entity, Self::ComponentType>> {
        return Some(&mut w.light);
    }

    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    fn get_enabled(&mut self) -> bool {
        return self.enabled;
    }
}
