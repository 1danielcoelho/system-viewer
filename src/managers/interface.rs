use crate::app_state::{AppState, ButtonState};
use egui::{Id, LayerId, Pos2, Response, Ui};
use gui_backend::WebInput;
use web_sys::WebGl2RenderingContext;

type GL = WebGl2RenderingContext;

macro_rules! handle_output {
    ($s:ident, $e:expr) => {{
        let result = $e;
        handle_output_func($s, result);
    }};
}

fn handle_output_func(state: &mut AppState, output: Response) {
    if output.clicked && state.input.m0 == ButtonState::Pressed {
        state.input.m0 = ButtonState::Handled;
    }
}

pub struct InterfaceManager {
    backend: gui_backend::WebBackend,
    web_input: WebInput,
}
impl InterfaceManager {
    pub fn new() -> Self {
        return Self {
            backend: gui_backend::WebBackend::new("rustCanvas")
                .expect("Failed to make a web backend for egui"),
            web_input: Default::default(),
        };
    }

    pub fn begin_frame(&mut self, state: &mut AppState) {
        self.pre_draw(state);

        InterfaceManager::draw_main_ui(state);
    }

    pub fn end_frame(&mut self) {
        let (_, paint_jobs) = self.backend.end_frame().unwrap();
        self.backend.paint(paint_jobs).expect("Failed to paint!");
    }

    fn pre_draw(&mut self, state: &mut AppState) {
        let mut raw_input = self.web_input.new_frame();

        // TODO: Combine these or get rid of one of them?
        raw_input.mouse_pos = Some(Pos2 {
            x: state.input.mouse_x as f32,
            y: state.input.mouse_y as f32,
        });
        raw_input.mouse_down = state.input.m0 == ButtonState::Pressed;

        self.backend.begin_frame(raw_input);
        let rect = self.backend.ctx.available_rect();

        state.ui = Some(Ui::new(
            self.backend.ctx.clone(),
            LayerId::background(),
            Id::new("interface"),
            rect,
            rect,
        ));
    }

    fn draw_main_ui(state: &mut AppState) {
        // TODO: Draw menus and toolbars and stuff

        draw_test_widget(state);
    }
}

fn draw_test_widget(state: &mut AppState) {
    // Have to take ui out of state or else we'll have aliasing issues passing ui into the closure below
    let ui = state.ui.take();

    egui::Window::new("Debug").show(&ui.as_ref().unwrap().ctx(), |ui| {
        ui.columns(2, |cols| {
            handle_output!(state, cols[0].label("Simulated seconds since start:"));
            handle_output!(
                state,
                cols[1].label(format!("{:.2}", state.phys_time_ms / 1000.0))
            );
        });

        ui.columns(2, |cols| {
            handle_output!(state, cols[0].label("Real seconds since start:"));
            handle_output!(
                state,
                cols[1].label(format!("{:.2}", state.real_time_ms / 1000.0))
            );
        });

        ui.columns(2, |cols| {
            handle_output!(state, cols[0].label("Frames per second:"));
            handle_output!(
                state,
                cols[1].label(format!("{:.2}", 1000.0 / state.real_delta_time_ms))
            );
        });

        ui.columns(2, |cols| {
            handle_output!(state, cols[0].label("Simulation speed:"));
            handle_output!(
                state,
                cols[1].add(
                    egui::DragValue::f64(&mut state.simulation_speed)
                        .range(-100.0..=100.0)
                        .speed(0.01),
                )
            );
        });

        ui.separator();

        ui.columns(2, |cols| {
            handle_output!(state, cols[0].label("Light intensity exponent:"));
            handle_output!(
                state,
                cols[1].add(
                    egui::DragValue::f32(&mut state.light_intensity)
                        .range(-1000.0..=1000.0)
                        .speed(0.01),
                )
            );
        });

        ui.separator();

        ui.columns(2, |cols| {
            handle_output!(state, cols[0].label("Vertical FOV (deg):"));
            handle_output!(
                state,
                cols[1].add(
                    egui::DragValue::f32(&mut state.camera.fov_v.0)
                        .range(0.1..=120.0)
                        .speed(0.5),
                )
            );
        });

        ui.columns(2, |cols| {
            handle_output!(state, cols[0].label("Near:"));
            handle_output!(
                state,
                cols[1].add(
                    egui::DragValue::f32(&mut state.camera.near)
                        .range(0.01..=19.9)
                        .speed(0.01),
                )
            );
        });

        ui.columns(2, |cols| {
            handle_output!(state, cols[0].label("Far:"));
            handle_output!(
                state,
                cols[1].add(egui::DragValue::f32(&mut state.camera.far).range(20.0..=10000.0))
            );
        });

        ui.columns(2, |cols| {
            handle_output!(state, cols[0].label("Camera pos:"));
            handle_output!(
                state,
                cols[1].add(egui::DragValue::f32(&mut state.camera.pos.x).prefix("x: "))
            );
            handle_output!(
                state,
                cols[1].add(egui::DragValue::f32(&mut state.camera.pos.y).prefix("y: "))
            );
            handle_output!(
                state,
                cols[1].add(egui::DragValue::f32(&mut state.camera.pos.z).prefix("z: "))
            );
        });

        ui.separator();

        ui.columns(2, |cols| {
            handle_output!(state, cols[0].label("Move speed:"));
            handle_output!(
                state,
                cols[1].add(
                    egui::DragValue::f32(&mut state.move_speed)
                        .range(1.0..=1000.0)
                        .speed(0.1),
                )
            );
        });

        ui.columns(2, |cols| {
            handle_output!(state, cols[0].label("Rotation speed:"));
            handle_output!(
                state,
                cols[1].add(
                    egui::DragValue::f32(&mut state.rotate_speed)
                        .range(1.0..=10.0)
                        .speed(0.1),
                )
            );
        });
    });

    state.ui = ui;
}
