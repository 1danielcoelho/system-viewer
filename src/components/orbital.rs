use crate::components::Component;
use crate::managers::details_ui::DetailsUI;
use crate::managers::orbit::OrbitalElements;
use crate::managers::scene::component_storage::ComponentStorage;
use crate::managers::scene::Scene;
use crate::utils::transform::Transform;
use crate::utils::units::Jdn;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OrbitalComponent {
    enabled: bool,

    pub elements: OrbitalElements,
    pub baked_eccentric_anomaly_times: Vec<Jdn>,
    pub circle_to_final_ellipse: Transform<f64>,
}
impl OrbitalComponent {
    #[allow(dead_code)]
    fn new() -> Self {
        return Self::default();
    }
}

impl Component for OrbitalComponent {
    type ComponentType = OrbitalComponent;

    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    fn get_enabled(&self) -> bool {
        return self.enabled;
    }

    fn get_storage(scene: &Scene) -> Box<&dyn ComponentStorage<Self::ComponentType>> {
        return Box::new(&scene.orbital);
    }

    fn get_storage_mut(scene: &mut Scene) -> Box<&mut dyn ComponentStorage<Self::ComponentType>> {
        return Box::new(&mut scene.orbital);
    }
}

impl DetailsUI for OrbitalComponent {}
