use crate::components::Component;
use crate::managers::details_ui::DetailsUI;
use crate::managers::resource::body_description::BodyDescription;
use crate::managers::scene::component_storage::ComponentStorage;
use crate::managers::scene::Scene;
use crate::utils::transform::Transform;
use crate::utils::units::Jdn;
use egui::Ui;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OrbitalComponent {
    enabled: bool,

    pub desc: BodyDescription,
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

    fn get_storage(scene: &mut Scene) -> Box<&mut dyn ComponentStorage<Self::ComponentType>> {
        return Box::new(&mut scene.orbital);
    }
}

impl DetailsUI for OrbitalComponent {
    fn draw_details_ui(&mut self, ui: &mut Ui) {
        ui.columns(2, |cols| {
            cols[0].label("Name:");
            cols[1].label(&self.desc.name);
        });
    }
}
