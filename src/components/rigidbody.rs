use crate::components::Component;
use crate::managers::scene::component_storage::ComponentStorage;
use crate::managers::{details_ui::DetailsUI, scene::Scene};
use crate::utils::transform::Transform;
use na::{Matrix3, Vector3};
use nalgebra::UnitQuaternion;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RigidBodyComponent {
    enabled: bool,
    pub collision_enabled: bool,

    // Constants
    /// Kg
    pub mass: f64,
    /// Local space
    pub inv_inertia: Matrix3<f64>,

    // Inputs/computed
    /// kg * Mm/s^2
    pub force_sum: Vector3<f64>,
    pub torque_sum: Vector3<f64>,

    // State
    /// kg * Mm/s
    pub lin_mom: Vector3<f64>,
    /// length is kg * Mm^2 * rad/s, right-hand rule
    pub ang_mom: Vector3<f64>,
    pub trans: Transform<f64>,
}

impl RigidBodyComponent {
    #[allow(dead_code)]
    fn new() -> Self {
        return Self::default();
    }
}

impl Default for RigidBodyComponent {
    fn default() -> Self {
        return Self {
            enabled: false,
            collision_enabled: false,
            mass: 1.0,
            inv_inertia: Matrix3::identity(),
            force_sum: Vector3::new(0.0, 0.0, 0.0),
            torque_sum: Vector3::new(0.0, 0.0, 0.0),
            lin_mom: Vector3::new(0.0, 0.0, 0.0),
            ang_mom: Vector3::new(0.0, 0.0, 0.0),
            trans: Transform::identity(),
        };
    }
}

impl Component for RigidBodyComponent {
    type ComponentType = RigidBodyComponent;

    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    fn get_enabled(&self) -> bool {
        return self.enabled;
    }

    fn get_storage(scene: &Scene) -> Box<&dyn ComponentStorage<Self::ComponentType>> {
        return Box::new(&scene.rigidbody);
    }

    fn get_storage_mut(scene: &mut Scene) -> Box<&mut dyn ComponentStorage<Self::ComponentType>> {
        return Box::new(&mut scene.rigidbody);
    }
}

impl DetailsUI for RigidBodyComponent {
    fn draw_details_ui(&mut self, ui: &mut egui::Ui) {
        ui.columns(2, |cols| {
            cols[0].label("Collision enabled:");
            cols[1].checkbox(&mut self.collision_enabled, "");
        });

        ui.columns(2, |cols| {
            cols[0].label("Mass [kg]:");
            cols[1].add(egui::DragValue::new(&mut self.mass));
        });

        ui.collapsing("Last transform:", |ui| {
            ui.columns(2, |cols| {
                cols[0].label("Pos [Mm]:");
                cols[1].horizontal(|ui| {
                    ui.add(egui::DragValue::new(&mut self.trans.trans.x).prefix("x: "));
                    ui.add(egui::DragValue::new(&mut self.trans.trans.y).prefix("y: "));
                    ui.add(egui::DragValue::new(&mut self.trans.trans.z).prefix("z: "));
                });
            });

            ui.columns(2, |cols| {
                cols[0].label("Rot [deg]:");
                cols[1].horizontal(|ui| {
                    let (mut euler_x, mut euler_y, mut euler_z) = self.trans.rot.euler_angles();
                    euler_x = euler_x.to_degrees();
                    euler_y = euler_y.to_degrees();
                    euler_z = euler_z.to_degrees();

                    ui.add(egui::DragValue::new(&mut euler_x).prefix("x: "));
                    ui.add(egui::DragValue::new(&mut euler_y).prefix("y: "));
                    ui.add(egui::DragValue::new(&mut euler_z).prefix("z: "));

                    self.trans.rot = UnitQuaternion::from_euler_angles(
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
                        egui::DragValue::new(&mut self.trans.scale.x)
                            .prefix("x: ")
                            .speed(0.1),
                    );
                    ui.add(
                        egui::DragValue::new(&mut self.trans.scale.y)
                            .prefix("y: ")
                            .speed(0.1),
                    );
                    ui.add(
                        egui::DragValue::new(&mut self.trans.scale.z)
                            .prefix("z: ")
                            .speed(0.1),
                    );
                });
            });
        });

        let mut vel_x = self.lin_mom.x / self.mass;
        let mut vel_y = self.lin_mom.y / self.mass;
        let mut vel_z = self.lin_mom.z / self.mass;

        ui.columns(2, |cols| {
            cols[0].label("Linear velocity [Mm/s]:");
            cols[1].horizontal(|ui| {
                ui.add(egui::DragValue::new(&mut vel_x).prefix("x: "));
                ui.add(egui::DragValue::new(&mut vel_y).prefix("y: "));
                ui.add(egui::DragValue::new(&mut vel_z).prefix("z: "));
            });
        });

        self.lin_mom.x = vel_x * self.mass;
        self.lin_mom.y = vel_y * self.mass;
        self.lin_mom.z = vel_z * self.mass;

        ui.columns(2, |cols| {
            cols[0].label("Angular momentum [kg Mm^2 rad/s]:");
            cols[1].horizontal(|ui| {
                ui.add(egui::DragValue::new(&mut self.ang_mom.x).prefix("x: "));
                ui.add(egui::DragValue::new(&mut self.ang_mom.y).prefix("y: "));
                ui.add(egui::DragValue::new(&mut self.ang_mom.z).prefix("z: "));
            });
        });
    }
}
