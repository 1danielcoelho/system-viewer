use crate::components::Component;
use crate::managers::details_ui::DetailsUI;
use crate::utils::transform::Transform;
use egui::Ui;
use na::UnitQuaternion;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TransformComponent {
    local_transform: Transform<f64>,

    #[serde(skip)]
    world_transform: Transform<f64>,
}
impl TransformComponent {
    pub fn new() -> Self {
        return Self::default();
    }

    pub fn get_local_transform(&self) -> &Transform<f64> {
        return &self.local_transform;
    }

    pub fn get_local_transform_mut(&mut self) -> &mut Transform<f64> {
        return &mut self.local_transform;
    }

    pub fn get_world_transform(&self) -> &Transform<f64> {
        return &self.world_transform;
    }

    pub fn get_world_transform_mut(&mut self) -> &mut Transform<f64> {
        return &mut self.world_transform;
    }
}

impl Component for TransformComponent {
    fn get_component_type() -> u64 {
        1
    }
}

impl DetailsUI for TransformComponent {
    fn draw_details_ui(&mut self, ui: &mut Ui) {
        ui.columns(2, |cols| {
            cols[0].label("Pos [Mm]:");
            cols[1].horizontal(|ui| {
                ui.add(egui::DragValue::new(&mut self.local_transform.trans.x).prefix("x: "));
                ui.add(egui::DragValue::new(&mut self.local_transform.trans.y).prefix("y: "));
                ui.add(egui::DragValue::new(&mut self.local_transform.trans.z).prefix("z: "));
            });
        });

        ui.columns(2, |cols| {
            cols[0].label("Rot [deg]:");
            cols[1].horizontal(|ui| {
                let (mut euler_x, mut euler_y, mut euler_z) =
                    self.local_transform.rot.euler_angles();
                euler_x = euler_x.to_degrees();
                euler_y = euler_y.to_degrees();
                euler_z = euler_z.to_degrees();

                ui.add(egui::DragValue::new(&mut euler_x).prefix("x: "));
                ui.add(egui::DragValue::new(&mut euler_y).prefix("y: "));
                ui.add(egui::DragValue::new(&mut euler_z).prefix("z: "));

                self.local_transform.rot = UnitQuaternion::from_euler_angles(
                    euler_x.to_radians(),
                    euler_y.to_radians(),
                    euler_z.to_radians(),
                );
            });
        });

        ui.columns(2, |cols| {
            cols[0].label("Scale:");
            cols[1].horizontal(|ui| {
                ui.add(
                    egui::DragValue::new(&mut self.local_transform.scale.x)
                        .prefix("x: ")
                        .speed(0.1),
                );
                ui.add(
                    egui::DragValue::new(&mut self.local_transform.scale.y)
                        .prefix("y: ")
                        .speed(0.1),
                );
                ui.add(
                    egui::DragValue::new(&mut self.local_transform.scale.z)
                        .prefix("z: ")
                        .speed(0.1),
                );
            });
        });
    }
}
