use self::details_ui::DetailsUI;
use super::ECManager;

use crate::{
    app_state::{AppState, ButtonState},
    components::{MeshComponent, TransformComponent},
    engine_interface::prompt_for_text_file,
    utils::{
        raycasting::{raycast, Ray},
        web::write_string_to_file_prompt,
    },
};
use egui::{menu, Align, Button, Id, LayerId, Layout, Pos2, Response, TextStyle, TopPanel, Ui};
use gui_backend::WebInput;
use js_sys::encode_uri_component;
use na::*;
use web_sys::{HtmlElement, WebGl2RenderingContext};

type GL = WebGl2RenderingContext;

pub mod details_ui;

#[macro_export]
macro_rules! handle_output {
    ($s:ident, $e:expr) => {{
        let result = $e;
        handle_output_func($s, result);
    }};
}

pub fn handle_output_func(state: &mut AppState, output: Response) {
    if output.hovered {
        state.input.over_ui = true;

        if state.input.m0 == ButtonState::Pressed {
            state.input.m0 = ButtonState::Handled;
        }
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

    /**
     * This runs before all systems, and starts collecting all the UI elements we'll draw, as
     * well as draws the main UI
     */
    pub fn begin_frame(&mut self, state: &mut AppState, ent_man: Option<&mut ECManager>) {
        self.pre_draw(state);

        draw_main_ui(state, ent_man);
    }

    /** This runs after all systems, and draws the collected UI elements to the framebuffer */
    pub fn end_frame(&mut self, state: &mut AppState, ent_man: Option<&mut ECManager>) {
        self.draw();

        if let Some(ent_man) = ent_man {
            if !state.input.over_ui && state.input.m1 == ButtonState::Depressed {
                handle_mouse_on_scene(state, ent_man);
            }
        }
    }

    fn pre_draw(&mut self, state: &mut AppState) {
        state.input.over_ui = false;

        // TODO: Fill in more data in raw_input
        let mut raw_input = self.web_input.new_frame();
        raw_input.mouse_pos = Some(Pos2 {
            x: state.input.mouse_x as f32,
            y: state.input.mouse_y as f32,
        });

        // HACK: Currently the UI sets the button state to handled if mouse down happens over it...
        raw_input.mouse_down = state.input.m0 != ButtonState::Depressed;

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

    fn draw(&mut self) {
        // We shouldn't need to raycast against the drawn elements because every widget we draw will optionally
        // also write to AppState if the mouse is over itself
        let (_, paint_jobs) = self.backend.end_frame().unwrap();
        self.backend.paint(paint_jobs).expect("Failed to paint!");
    }
}

fn draw_main_ui(state: &mut AppState, ent_man: Option<&mut ECManager>) {
    draw_main_toolbar(state);

    draw_test_widget(state, ent_man);
}

fn handle_mouse_on_scene(state: &mut AppState, ent_man: &mut ECManager) {
    let p = Matrix4::new_perspective(
        state.canvas_width as f32 / state.canvas_height as f32,
        state.camera.fov_v.to_radians(),
        state.camera.near,
        state.camera.far,
    );
    let v = Matrix4::look_at_rh(&state.camera.pos, &state.camera.target, &state.camera.up);

    let ndc_near_pos = Point3::from(Vector3::new(
        -1.0 + 2.0 * state.input.mouse_x as f32 / (state.canvas_width - 1) as f32,
        1.0 - 2.0 * state.input.mouse_y as f32 / (state.canvas_height - 1) as f32,
        -1.0,
    ));

    let ndc_to_world: Matrix4<f32> = v.try_inverse().unwrap() * p.try_inverse().unwrap();

    let world_pos = ndc_to_world.transform_point(&ndc_near_pos);

    let ray = Ray {
        start: state.camera.pos,
        direction: (world_pos - state.camera.pos).normalize(),
    };

    if state.input.m0 == ButtonState::Pressed {
        if let Some(hit) = raycast(&ray, &ent_man.mesh, &ent_man.transform) {
            if let Some(entity) = ent_man.get_entity_from_index(hit.entity_index) {
                state.selection.clear();
                state.selection.insert(entity);
            }
        } else {
            state.selection.clear();
        }

        // log::info!("Raycast hit: {:?}", rayhit);

        // let ui = state.ui.take();
        // let response = egui::Window::new("Test")
        //     .fixed_pos(Pos2 {
        //         x: state.input.mouse_x as f32,
        //         y: state.input.mouse_y as f32,
        //     })
        //     .show(&ui.as_ref().unwrap().ctx(), |ui| {});

        // state.ui = ui;
    }
}

fn draw_main_toolbar(state: &mut AppState) {
    let ui = state.ui.take();

    TopPanel::top(Id::new("top panel")).show(&ui.as_ref().unwrap().ctx(), |ui| {
        menu::bar(ui, |ui| {
            menu::menu(ui, "File", |ui| {
                if ui.button("New").clicked {
                    log::info!("New");
                }

                if ui.button("Open").clicked {
                    prompt_for_text_file("ephemerides");
                }

                if ui.button("Save").clicked {
                    write_string_to_file_prompt("test.json", "my test data");
                }

                ui.separator();

                if ui.button("Close").clicked {
                    log::info!("Close");
                }
            });

            menu::menu(ui, "Edit", |ui| {
                if ui.button("New object...").clicked {
                    log::info!("New object");
                }
            });

            menu::menu(ui, "Tools", |ui| {
                if ui.button("Organize windows").clicked {
                    ui.ctx().memory().reset_areas();
                }

                if ui
                    .button("Clear Egui memory")
                    .on_hover_text("Forget scroll, collapsing headers etc")
                    .clicked
                {
                    *ui.ctx().memory() = Default::default();
                }
            });

            let time = state.real_time_ms / 1000.0;
            let time = format!(
                "{:02}:{:02}:{:02}.{:02}",
                (time % (24.0 * 60.0 * 60.0) / 3600.0).floor(),
                (time % (60.0 * 60.0) / 60.0).floor(),
                (time % 60.0).floor(),
                (time % 1.0 * 100.0).floor()
            );

            ui.with_layout(Layout::right_to_left(), |ui| {
                if ui
                    .add(Button::new(time).text_style(TextStyle::Monospace))
                    .clicked
                {
                    log::info!("Clicked on clock!");
                }
            });
        });
    });

    state.ui = ui;
}

fn draw_test_widget(state: &mut AppState, ent_man: Option<&mut ECManager>) {
    // Have to take ui out of state or else we'll have aliasing issues passing ui into the closure below
    let ui = state.ui.take();

    let response = egui::Window::new("Debug").show(&ui.as_ref().unwrap().ctx(), |ui| {
        ui.columns(2, |cols| {
            cols[0].label("Simulated seconds since start:");
            cols[1].label(format!("{:.2}", state.phys_time_ms / 1000.0));
        });

        ui.columns(2, |cols| {
            cols[0].label("Real seconds since start:");
            cols[1].label(format!("{:.2}", state.real_time_ms / 1000.0));
        });

        ui.columns(2, |cols| {
            cols[0].label("Frames per second:");
            cols[1].label(format!("{:.2}", 1000.0 / state.real_delta_time_ms));
        });

        ui.columns(2, |cols| {
            cols[0].label("Simulation speed:");
            cols[1].add(
                egui::DragValue::f64(&mut state.simulation_speed)
                    .range(-100.0..=100.0)
                    .speed(0.01),
            );
        });

        ui.separator();

        ui.columns(2, |cols| {
            cols[0].label("Light intensity exponent:");
            cols[1].add(
                egui::DragValue::f32(&mut state.light_intensity)
                    .range(-1000.0..=1000.0)
                    .speed(0.01),
            );
        });

        ui.separator();

        ui.columns(2, |cols| {
            cols[0].label("Vertical FOV (deg):");
            cols[1].add(
                egui::DragValue::f32(&mut state.camera.fov_v)
                    .range(0.1..=120.0)
                    .speed(0.5),
            );
        });

        ui.columns(2, |cols| {
            cols[0].label("Near:");
            cols[1].add(
                egui::DragValue::f32(&mut state.camera.near)
                    .range(0.01..=19.9)
                    .speed(0.01),
            );
        });

        ui.columns(2, |cols| {
            cols[0].label("Far:");
            cols[1].add(egui::DragValue::f32(&mut state.camera.far));
        });

        ui.columns(2, |cols| {
            cols[0].label("Camera pos:");
            cols[1].with_layout(Layout::left_to_right().with_cross_align(Align::Min), |ui| {
                ui.add(egui::DragValue::f32(&mut state.camera.pos.x).prefix("x: "));
                ui.add(egui::DragValue::f32(&mut state.camera.pos.y).prefix("y: "));
                ui.add(egui::DragValue::f32(&mut state.camera.pos.z).prefix("z: "));
            });
        });

        ui.separator();

        ui.columns(2, |cols| {
            cols[0].label("Move speed:");
            cols[1].add(
                egui::DragValue::f32(&mut state.move_speed)
                    .range(1.0..=1000.0)
                    .speed(0.1),
            );
        });

        ui.columns(2, |cols| {
            cols[0].label("Rotation speed:");
            cols[1].add(
                egui::DragValue::f32(&mut state.rotate_speed)
                    .range(1.0..=10.0)
                    .speed(0.1),
            );
        });

        if let Some(selection) = state.selection.iter().next().cloned() {
            ui.separator();

            ui.columns(2, |cols| {
                cols[0].label("Selected entity:");
                cols[1].label(format!("{:?}", selection));
            });

            if let Some(ent_man) = ent_man {
                ui.columns(2, |cols| {
                    cols[0].label("Name:");
                    cols[1].label(format!(
                        "{}",
                        ent_man.get_entity_name(selection).unwrap_or_default()
                    ));
                });

                if let Some(comp) = ent_man.get_component::<TransformComponent>(selection) {
                    ui.collapsing("Transform component", |ui| {
                        comp.draw_details_ui(ui);
                    });
                }

                if let Some(comp) = ent_man.get_component::<MeshComponent>(selection) {
                    ui.collapsing("Mesh component", |ui| {
                        comp.draw_details_ui(ui);
                    });
                }
            }
        }
    });

    if let Some(response) = response {
        handle_output!(state, response);
    }

    state.ui = ui;
}
