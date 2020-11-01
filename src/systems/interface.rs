use egui::{Pos2, Ui};
use gui_backend::WebInput;

use crate::{app_state::AppState, components::ui::WidgetType, managers::ComponentManager};

pub struct InterfaceSystem {
    backend: gui_backend::WebBackend,
    web_input: WebInput,
    ui: Option<egui::Ui>,
}
impl InterfaceSystem {
    pub fn new() -> Self {
        return Self {
            backend: gui_backend::WebBackend::new("rustCanvas")
                .expect("Failed to make a web backend for egui"),
            web_input: Default::default(),
            ui: None,
        };
    }

    pub fn run(&mut self, state: &mut AppState, comp_man: &ComponentManager) {
        self.pre_draw(state);
        self.draw(state, comp_man);
    }

    fn pre_draw(&mut self, state: &AppState) {
        let mut raw_input = self.web_input.new_frame(1.0);

        // TODO: Combine these or get rid of one of them?
        raw_input.mouse_pos = Some(Pos2 {
            x: state.input.mouse_x as f32,
            y: state.input.mouse_y as f32,
        });
        raw_input.mouse_down = state.input.m0_down;

        self.ui = Some(self.backend.begin_frame(raw_input));
    }

    fn draw(&mut self, state: &mut AppState, comp_man: &ComponentManager) {
        if self.ui.is_none() {
            return;
        }

        for entity in 0..comp_man.interface.len() {
            InterfaceSystem::draw_widget(self.ui.as_ref().unwrap(), state, entity as u32, comp_man);
        }

        let (_, paint_jobs) = self.backend.end_frame().unwrap();
        self.backend.paint(paint_jobs).expect("Failed to paint!");
    }

    fn draw_widget(ui: &Ui, state: &mut AppState, entity: u32, comp_man: &ComponentManager) {
        let ui_comp = &comp_man.interface[entity as usize];
        match ui_comp.widget_type {
            WidgetType::None => {}
            WidgetType::TestWidget => {
                InterfaceSystem::draw_test_widget(ui, state, entity, comp_man)
            }
        }
    }

    fn draw_test_widget(ui: &Ui, state: &mut AppState, entity: u32, comp_man: &ComponentManager) {
        egui::Window::new("Debug").show(&ui.ctx(), |ui| {
            ui.horizontal(|ui| {
                ui.add(
                    egui::DragValue::f64(&mut state.simulation_speed)
                        .range(-100.0..=100.0)
                        .speed(0.01),
                );
                ui.label(format!("Simulation speed"));
            });

            ui.separator();

            // TODO: Can't use ui.columns due to some bug where everything in column 0 responds at once
            ui.horizontal(|ui| {
                ui.add(
                    egui::DragValue::f32(&mut state.camera.fov_v.0)
                        .range(0.1..=120.0)
                        .speed(0.5),
                );
                ui.label(format!("Vertical FOV (deg)"));
            });

            ui.horizontal(|ui| {
                ui.add(
                    egui::DragValue::f32(&mut state.camera.near)
                        .range(0.1..=19.9)
                        .speed(0.1),
                );
                ui.label(format!("Near"));
            });

            ui.horizontal(|ui| {
                ui.add(egui::DragValue::f32(&mut state.camera.far).range(20.0..=10000.0));
                ui.label(format!("Far"));
            });

            ui.horizontal(|ui| {
                ui.add(
                    egui::DragValue::f32(&mut state.move_speed)
                        .range(1.0..=1000.0)
                        .speed(0.1),
                );
                ui.label(format!("Move speed"));
            });

            ui.horizontal(|ui| {
                ui.add(
                    egui::DragValue::f32(&mut state.rotate_speed)
                        .range(1.0..=10.0)
                        .speed(0.1),
                );
                ui.label(format!("Rotation speed"));
            });

            ui.horizontal(|ui| {
                ui.add(egui::DragValue::f32(&mut state.camera.pos.x).prefix("x: "));
                ui.add(egui::DragValue::f32(&mut state.camera.pos.y).prefix("y: "));
                ui.add(egui::DragValue::f32(&mut state.camera.pos.z).prefix("z: "));
                ui.label(format!("Camera pos"));
            });
        });
    }
}
