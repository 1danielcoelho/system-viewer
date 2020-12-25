use super::{
    component::{ComponentStorageType, ComponentType},
    Component,
};
use crate::managers::{details_ui::DetailsUI, ECManager};
use cgmath::*;
use egui::{Align, Layout, Ui};

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
    const COMPONENT_TYPE: ComponentType = ComponentType::Transform;

    fn get_components_vector<'a>(w: &'a mut ECManager) -> Option<&'a mut Vec<TransformComponent>> {
        return Some(&mut w.transform);
    }

    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    fn get_enabled(&mut self) -> bool {
        return self.enabled;
    }
}

impl DetailsUI for TransformComponent {
    fn draw_details_ui(&mut self, ui: &mut Ui) {
        ui.label("Transform component:");

        ui.columns(2, |cols| {
            cols[0].label("Pos:");
            cols[1].with_layout(Layout::left_to_right().with_cross_align(Align::Min), |ui| {
                ui.add(egui::DragValue::f32(&mut self.local_transform.disp.x).prefix("x: "));
                ui.add(egui::DragValue::f32(&mut self.local_transform.disp.y).prefix("y: "));
                ui.add(egui::DragValue::f32(&mut self.local_transform.disp.z).prefix("z: "));
            });
        });

        ui.columns(2, |cols| {
            cols[0].label("Rot:");
            cols[1].with_layout(Layout::left_to_right().with_cross_align(Align::Min), |ui| {
                let euler: Euler<Rad<f32>> = self.local_transform.rot.into();
                let mut euler: Euler<Deg<f32>> =
                    Euler::new(Deg::from(euler.x), Deg::from(euler.y), Deg::from(euler.z));

                ui.add(
                    egui::DragValue::f32(&mut euler.x.0)
                        .prefix("x: ")
                        .suffix("deg"),
                );
                ui.add(
                    egui::DragValue::f32(&mut euler.y.0)
                        .prefix("y: ")
                        .suffix("deg"),
                );
                ui.add(
                    egui::DragValue::f32(&mut euler.z.0)
                        .prefix("z: ")
                        .suffix("deg"),
                );

                self.local_transform.rot = euler.into();
                self.local_transform.rot.normalize();
            });
        });

        ui.columns(2, |cols| {
            cols[0].label("Scale:");
            cols[1].with_layout(Layout::left_to_right().with_cross_align(Align::Min), |ui| {
                ui.add(egui::DragValue::f32(&mut self.local_transform.scale).speed(0.1));
            });
        });
    }
}
