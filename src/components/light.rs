use std::collections::HashMap;

use crate::managers::{ECManager, Entity};

use super::{
    component::{ComponentType, ComponentStorageType},
    Component,
};

#[derive(Clone, Copy)]
pub enum LightType {
    Point = 0,
    Directional = 1,
}

#[derive(Clone)]
pub struct LightComponent {
    enabled: bool,

    pub light_type: LightType,
    pub color: cgmath::Vector3<f32>,
    pub intensity: f32,
    pub direction: Option<cgmath::Vector3<f32>>,
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

            light_type: LightType::Point,
            color: cgmath::Vector3::new(0.0, 0.0, 0.0), // Needs to be black so that unused lights don't affect the scene
            intensity: 0.0,
            direction: Some(cgmath::Vector3::new(0.0, 0.0, -1.0)), // Pointing down
        };
    }
}
impl Component for LightComponent {
    type ComponentType = LightComponent;
    const STORAGE_TYPE: ComponentStorageType = ComponentStorageType::HashMap;
    const COMPONENT_TYPE: ComponentType = ComponentType::Light;

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
