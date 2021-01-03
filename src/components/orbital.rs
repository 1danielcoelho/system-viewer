use super::{
    component::{ComponentStorageType, ComponentType},
    Component,
};
use crate::{
    managers::{
        details_ui::DetailsUI,
        scene::{Entity, Scene},
    },
    utils::{orbits::BodyDescription, transform::Transform, units::Jdn},
};
use egui::Ui;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OrbitalComponent {
    enabled: bool,

    pub desc: BodyDescription,
    pub baked_eccentric_anomaly_times: Vec<Jdn>,
    pub circle_to_final_ellipse: Transform<f64>,
}
impl OrbitalComponent {
    fn new() -> Self {
        return Self::default();
    }
}

impl Component for OrbitalComponent {
    type ComponentType = OrbitalComponent;
    const STORAGE_TYPE: ComponentStorageType = ComponentStorageType::HashMap;
    const COMPONENT_TYPE: ComponentType = ComponentType::Orbital;

    fn get_components_map<'a>(
        w: &'a mut Scene,
    ) -> Option<&'a mut HashMap<Entity, Self::ComponentType>> {
        return Some(&mut w.orbital);
    }

    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    fn get_enabled(&mut self) -> bool {
        return self.enabled;
    }
}

impl DetailsUI for OrbitalComponent {
    fn draw_details_ui(&mut self, ui: &mut Ui) {
        ui.columns(2, |cols| {
            cols[0].label("Name:");
            cols[1].label(&self.desc.name);
        });

        ui.columns(2, |cols| {
            cols[0].label("Id:");
            cols[1].label(self.desc.id.to_string());
        });

        ui.columns(2, |cols| {
            cols[0].label("Reference id:");
            cols[1].label(self.desc.reference_id.to_string());
        });

        ui.columns(2, |cols| {
            cols[0].label("Mass:");
            cols[1].label(self.desc.mass.to_string());
        });
    }
}
