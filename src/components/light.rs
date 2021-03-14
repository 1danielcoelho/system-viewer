use crate::components::Component;
use crate::managers::details_ui::DetailsUI;
use crate::managers::scene::component_storage::ComponentStorage;
use crate::managers::scene::Scene;
use na::Vector3;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum LightType {
    Point = 0,
    Directional = 1,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LightComponent {
    enabled: bool,

    pub light_type: LightType,
    pub color: Vector3<f32>,
    pub intensity: f32, // Candela for point/spot lights; Lux for for directional lights
    pub direction: Option<Vector3<f32>>,
}
impl LightComponent {
    #[allow(dead_code)]
    fn new() -> Self {
        return Self::default();
    }
}
impl Default for LightComponent {
    fn default() -> Self {
        return Self {
            enabled: false,

            light_type: LightType::Point,
            color: Vector3::new(0.0, 0.0, 0.0), // Needs to be black so that unused lights don't affect the scene
            intensity: 0.0,
            direction: Some(Vector3::new(0.0, 0.0, -1.0)), // Pointing down
        };
    }
}
impl Component for LightComponent {
    type ComponentType = LightComponent;

    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    fn get_enabled(&self) -> bool {
        return self.enabled;
    }

    fn get_storage(scene: &Scene) -> Box<&dyn ComponentStorage<Self::ComponentType>> {
        return Box::new(&scene.light);
    }

    fn get_storage_mut(scene: &mut Scene) -> Box<&mut dyn ComponentStorage<Self::ComponentType>> {
        return Box::new(&mut scene.light);
    }
}
impl DetailsUI for LightComponent {}
