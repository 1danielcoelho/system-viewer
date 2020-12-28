use super::{
    component::{ComponentStorageType, ComponentType},
    Component,
};
use crate::{
    managers::{details_ui::DetailsUI, scene::Scene},
    utils::transform::Transform,
};
use egui::{Align, Layout, Ui};
use na::{Quaternion, UnitQuaternion, Vector3};

#[derive(Clone)]
pub struct TransformComponent {
    enabled: bool,

    local_transform: Transform,
    world_transform: Transform,
}
impl TransformComponent {
    pub fn new() -> Self {
        return Self::default();
    }

    pub fn get_local_transform(&self) -> &Transform {
        return &self.local_transform;
    }

    pub fn get_local_transform_mut(&mut self) -> &mut Transform {
        return &mut self.local_transform;
    }

    pub fn update_world_transform(&mut self, parent_local_transform: &Transform) {
        self.world_transform = parent_local_transform.concat_clone(&self.local_transform);
    }

    pub fn get_world_transform(&self) -> &Transform {
        return &self.world_transform;
    }
}
impl Default for TransformComponent {
    fn default() -> Self {
        return Self {
            enabled: false,

            local_transform: Transform {
                trans: Vector3::new(0.0, 0.0, 0.0),
                rot: UnitQuaternion::new_unchecked(Quaternion::identity()),
                scale: Vector3::new(1.0, 1.0, 1.0),
            },
            world_transform: Transform {
                trans: Vector3::new(0.0, 0.0, 0.0),
                rot: UnitQuaternion::new_unchecked(Quaternion::identity()),
                scale: Vector3::new(1.0, 1.0, 1.0),
            },
        };
    }
}
impl Component for TransformComponent {
    type ComponentType = TransformComponent;
    const STORAGE_TYPE: ComponentStorageType = ComponentStorageType::Vec;
    const COMPONENT_TYPE: ComponentType = ComponentType::Transform;

    fn get_components_vector<'a>(w: &'a mut Scene) -> Option<&'a mut Vec<TransformComponent>> {
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
        ui.columns(2, |cols| {
            cols[0].label("Pos:");
            cols[1].with_layout(Layout::left_to_right().with_cross_align(Align::Min), |ui| {
                ui.add(egui::DragValue::f32(&mut self.local_transform.trans.x).prefix("x: "));
                ui.add(egui::DragValue::f32(&mut self.local_transform.trans.y).prefix("y: "));
                ui.add(egui::DragValue::f32(&mut self.local_transform.trans.z).prefix("z: "));
            });
        });

        ui.columns(2, |cols| {
            cols[0].label("Rot:");
            cols[1].with_layout(Layout::left_to_right().with_cross_align(Align::Min), |ui| {
                let (mut euler_x, mut euler_y, mut euler_z) =
                    self.local_transform.rot.euler_angles();
                euler_x = euler_x.to_degrees();
                euler_y = euler_y.to_degrees();
                euler_z = euler_z.to_degrees();

                ui.add(
                    egui::DragValue::f32(&mut euler_x)
                        .prefix("x: ")
                        .suffix("deg"),
                );
                ui.add(
                    egui::DragValue::f32(&mut euler_y)
                        .prefix("y: ")
                        .suffix("deg"),
                );
                ui.add(
                    egui::DragValue::f32(&mut euler_z)
                        .prefix("z: ")
                        .suffix("deg"),
                );

                self.local_transform.rot = UnitQuaternion::from_euler_angles(
                    euler_x.to_radians(),
                    euler_y.to_radians(),
                    euler_z.to_radians(),
                );
            });
        });

        ui.columns(2, |cols| {
            cols[0].label("Scale:");
            cols[1].with_layout(Layout::left_to_right().with_cross_align(Align::Min), |ui| {
                ui.add(
                    egui::DragValue::f32(&mut self.local_transform.scale.x)
                        .prefix("x: ")
                        .speed(0.1),
                );
                ui.add(
                    egui::DragValue::f32(&mut self.local_transform.scale.y)
                        .prefix("y: ")
                        .speed(0.1),
                );
                ui.add(
                    egui::DragValue::f32(&mut self.local_transform.scale.z)
                        .prefix("z: ")
                        .speed(0.1),
                );
            });
        });
    }
}
