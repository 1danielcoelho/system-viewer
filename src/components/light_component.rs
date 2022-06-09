use crate::components::Component;
use crate::managers::details_ui::DetailsUI;
use na::Vector3;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum LightType {
    Point = 0,
    Directional = 1,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LightComponent {
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
            light_type: LightType::Point,
            color: Vector3::new(0.0, 0.0, 0.0), // Needs to be black so that unused lights don't affect the scene
            intensity: 0.0,
            direction: Some(Vector3::new(0.0, 0.0, -1.0)), // Pointing down
        };
    }
}
impl Component for LightComponent {
    fn get_component_type() -> u64 {
        16
    }
}
impl DetailsUI for LightComponent {
    fn draw_details_ui(&mut self, _ui: &mut egui::Ui) {
        todo!();
    }
}
