use crate::components::Component;
use crate::managers::details_ui::DetailsUI;
use na::Vector3;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KinematicComponent {
    pub lin_vel: Vector3<f64>, // Mm/s
    pub ang_vel: Vector3<f64>, // rad/s
}

impl KinematicComponent {
    #[allow(dead_code)]
    fn new() -> Self {
        return Self::default();
    }
}

impl Default for KinematicComponent {
    fn default() -> Self {
        return Self {
            lin_vel: Vector3::new(0.0, 0.0, 0.0),
            ang_vel: Vector3::new(0.0, 0.0, 0.0),
        };
    }
}

impl Component for KinematicComponent {
    fn get_component_type() -> u64 {
        32
    }
}

impl DetailsUI for KinematicComponent {
    fn draw_details_ui(&mut self, ui: &mut egui::Ui) {
        ui.columns(2, |cols| {
            cols[0].label("Linear velocity [Mm/s]:");
            cols[1].horizontal(|ui| {
                ui.add(egui::DragValue::new(&mut self.lin_vel.x).prefix("x: "));
                ui.add(egui::DragValue::new(&mut self.lin_vel.y).prefix("y: "));
                ui.add(egui::DragValue::new(&mut self.lin_vel.z).prefix("z: "));
            });
        });

        ui.columns(2, |cols| {
            cols[0].label("Angular momentum [kg Mm^2 rad/s]:");
            cols[1].horizontal(|ui| {
                ui.add(egui::DragValue::new(&mut self.ang_vel.x).prefix("x: "));
                ui.add(egui::DragValue::new(&mut self.ang_vel.y).prefix("y: "));
                ui.add(egui::DragValue::new(&mut self.ang_vel.z).prefix("z: "));
            });
        });
    }
}
