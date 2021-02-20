use crate::app_state::{AppState, ButtonState, ReferenceChange, SimulationScale};
use crate::components::{MeshComponent, OrbitalComponent, PhysicsComponent, TransformComponent};
use crate::managers::details_ui::DetailsUI;
use crate::managers::scene::component_storage::ComponentStorage;
use crate::managers::scene::{Entity, Scene, SceneManager};
use crate::managers::ResourceManager;
use crate::utils::raycasting::{raycast, Ray};
use crate::utils::units::{julian_date_number_to_date, Jdn, J2000_JDN};
use crate::{prompt_for_bytes_file, UICTX};
use egui::Layout;
use gui_backend::WebInput;
use lazy_static::__Deref;
use na::*;
use std::collections::VecDeque;

pub mod details_ui;

const DEBUG: bool = true;

#[macro_export]
macro_rules! handle_output {
    ($s:ident, $e:expr) => {{
        let result = $e;
        handle_output_func($s, result);
    }};
}

pub fn handle_output_func(state: &mut AppState, output: egui::Response) {
    if output.hovered {
        state.input.over_ui = true;

        if state.input.m0 == ButtonState::Pressed {
            state.input.m0 = ButtonState::Handled;
        }
    }

    if output.has_kb_focus {
        if state.input.forward == ButtonState::Pressed {
            state.input.forward = ButtonState::Handled;
        }

        if state.input.left == ButtonState::Pressed {
            state.input.left = ButtonState::Handled;
        }

        if state.input.right == ButtonState::Pressed {
            state.input.right = ButtonState::Handled;
        }

        if state.input.back == ButtonState::Pressed {
            state.input.back = ButtonState::Handled;
        }

        if state.input.up == ButtonState::Pressed {
            state.input.up = ButtonState::Handled;
        }

        if state.input.down == ButtonState::Pressed {
            state.input.down = ButtonState::Handled;
        }
    }
}

struct OpenWindows {
    debug: bool,
    scene_hierarchy: bool,
    scene_browser: bool,
    about: bool,
}

pub struct InterfaceManager {
    backend: gui_backend::WebBackend,
    web_input: WebInput,
    open_windows: OpenWindows,
    selected_scene_desc_index: u32,

    frame_times: VecDeque<f64>,
    time_of_last_update: f64,
    last_frame_rate: f64,
}
impl InterfaceManager {
    pub fn new() -> Self {
        return Self {
            backend: gui_backend::WebBackend::new("rustCanvas")
                .expect("Failed to make a web backend for egui"),
            web_input: Default::default(),
            open_windows: OpenWindows {
                debug: false,
                scene_hierarchy: false,
                scene_browser: true,
                about: false,
            },
            selected_scene_desc_index: 0,
            frame_times: vec![16.66; 15].into_iter().collect(),
            time_of_last_update: -2.0,
            last_frame_rate: 60.0, // Optimism
        };
    }

    /**
     * This runs before all systems, and starts collecting all the UI elements we'll draw, as
     * well as draws the main UI
     */
    pub fn begin_frame(&mut self, state: &mut AppState) {
        self.pre_draw(state);
    }

    /** This runs after all systems, and draws the collected UI elements to the framebuffer */
    pub fn end_frame(
        &mut self,
        state: &mut AppState,
        scene_man: &mut SceneManager,
        res_man: &mut ResourceManager,
    ) {
        self.draw_main_ui(state, scene_man, res_man);

        self.draw();

        if let Some(scene) = scene_man.get_main_scene_mut() {
            if !state.input.over_ui && state.input.m1 == ButtonState::Depressed {
                handle_mouse_on_scene(state, scene);
            }
        }
    }

    fn pre_draw(&mut self, state: &mut AppState) {
        state.input.over_ui = false;

        // TODO: Fill in more data in raw_input
        let mut raw_input = self.web_input.new_frame();
        raw_input.mouse_pos = Some(egui::Pos2 {
            x: state.input.mouse_x as f32,
            y: state.input.mouse_y as f32,
        });

        // HACK: Currently the UI sets the button state to handled if mouse down happens over it...
        raw_input.mouse_down = state.input.m0 != ButtonState::Depressed;
        raw_input.events.append(&mut state.input.egui_keys);
        raw_input.modifiers = state.input.modifiers;

        self.backend.begin_frame(raw_input);
        let rect = self.backend.ctx.available_rect();

        // Always record our new frame times
        self.frame_times.pop_back();
        self.frame_times.push_front(state.real_delta_time_s);

        // Update framerate display only once a second or else it's too hard to read
        if state.real_time_s - self.time_of_last_update > 1.0 {
            self.time_of_last_update = state.real_time_s;

            let new_frame_rate: f64 =
                1.0 / (self.frame_times.iter().sum::<f64>() / (self.frame_times.len() as f64));
            self.last_frame_rate = new_frame_rate;
        }

        UICTX.with(|ui| {
            let mut ui = ui.borrow_mut();
            ui.replace(egui::Ui::new(
                self.backend.ctx.clone(),
                egui::LayerId::background(),
                egui::Id::new("interface"),
                rect,
                rect,
            ));
        });
    }

    fn draw(&mut self) {
        // We shouldn't need to raycast against the drawn elements because every widget we draw will optionally
        // also write to AppState if the mouse is over itself
        let (_, paint_jobs) = self.backend.end_frame().unwrap();
        self.backend.paint(paint_jobs).expect("Failed to paint!");
    }

    fn draw_main_ui(
        &mut self,
        state: &mut AppState,
        scene_man: &mut SceneManager,
        res_man: &mut ResourceManager,
    ) {
        self.draw_pop_ups(state, scene_man, res_man);

        self.draw_main_toolbar(state, scene_man, res_man);

        self.draw_open_windows(state, scene_man, res_man);
    }

    fn draw_main_toolbar(
        &mut self,
        state: &mut AppState,
        scene_man: &mut SceneManager,
        res_man: &mut ResourceManager,
    ) {
        UICTX.with(|ui| {
            let mut ref_mut = ui.borrow_mut();
            let ui = ref_mut.as_mut().unwrap();

            let mut style = ui.ctx().style().deref().clone();
            let old_fill = style.visuals.widgets.noninteractive.bg_fill;
            let old_stroke = style.visuals.widgets.noninteractive.bg_stroke.width;

            style.visuals.widgets.noninteractive.bg_fill =
                egui::Srgba::from_rgba_unmultiplied(255, 0, 0, 0);
            style.visuals.widgets.noninteractive.bg_stroke.width = 0.0;
            ui.ctx().set_style(style);

            let response = egui::TopPanel::top(egui::Id::new("top panel"))
                .show(&ui.ctx(), |ui| {
                    let num_bodies = scene_man
                        .get_main_scene()
                        .unwrap()
                        .physics
                        .get_num_components();

                    let sim_date_str = format!(
                        "{}",
                        julian_date_number_to_date(Jdn(state.sim_time_days + J2000_JDN.0))
                    );

                    ui.with_layout(egui::Layout::left_to_right(), |ui| {
                        egui::menu::menu(ui, "⚙", |ui| {
                            let mut total_res = ui.button("Reset scene");
                            if total_res.clicked {}

                            let res = ui.button("Scene browser");
                            if res.clicked {
                                self.open_windows.scene_browser = !self.open_windows.scene_browser;
                            }
                            total_res |= res;

                            ui.separator();

                            let res = ui.button("About");
                            if res.clicked {
                                self.open_windows.about = !self.open_windows.about;
                            }
                            total_res |= res;

                            if DEBUG {
                                ui.separator();
                                ui.separator();

                                let res = ui.button("New");
                                if res.clicked {
                                    let new_scene_name = scene_man
                                        .new_scene("New scene")
                                        .unwrap()
                                        .identifier
                                        .clone();
                                    scene_man.set_scene(&new_scene_name, res_man, Some(state));
                                }
                                total_res |= res;

                                total_res |= ui.button("Open");

                                total_res |= ui.button("Save");

                                ui.separator();

                                let res = ui.button("Close");
                                if res.clicked {
                                    let new_scene_name = scene_man
                                        .new_scene("New scene")
                                        .unwrap()
                                        .identifier
                                        .clone();
                                    scene_man.set_scene(&new_scene_name, res_man, Some(state));
                                }
                                total_res |= res;

                                let res = ui.button("Inject GLB...");
                                if res.clicked {
                                    prompt_for_bytes_file("glb_inject", ".glb");
                                }
                                total_res |= res;

                                ui.separator();

                                let res = ui.button("Debug");
                                if res.clicked {
                                    self.open_windows.debug = !self.open_windows.debug;
                                }
                                total_res |= res;

                                let res = ui.button("Scene hierarchy");
                                if res.clicked {
                                    self.open_windows.scene_hierarchy =
                                        !self.open_windows.scene_hierarchy;
                                }
                                total_res |= res;

                                ui.separator();

                                let res = ui.button("Organize windows");
                                if res.clicked {
                                    ui.ctx().memory().reset_areas();
                                }
                                total_res |= res;

                                let res = ui.button("Close all");
                                if res.clicked {
                                    self.open_windows.debug = false;
                                    self.open_windows.scene_hierarchy = false;
                                    self.open_windows.about = false;
                                    self.open_windows.scene_browser = false;
                                }
                                total_res |= res;

                                ui.separator();

                                let res = ui
                                    .button("Clear Egui memory")
                                    .on_hover_text("Forget scroll, collapsing headers etc");
                                if res.clicked {
                                    *ui.ctx().memory() = Default::default();
                                }
                                total_res |= res;

                                let res = ui
                                    .button("Reset app state")
                                    .on_hover_text("Clears app state from local storage");
                                if res.clicked {
                                    state.pending_reset = true;
                                }
                                total_res |= res;
                            }

                            handle_output!(state, total_res);
                        });

                        if ui
                            .add(
                                egui::Button::new(format!("{:.2} fps", self.last_frame_rate))
                                    .text_style(egui::TextStyle::Monospace),
                            )
                            .clicked
                        {
                            log::info!("Clicked on clock!");
                        }

                        let mut sim_speed_in_units = match state.simulation_scale {
                            SimulationScale::Seconds => state.simulation_speed * 86400.0,
                            SimulationScale::Days => state.simulation_speed,
                            SimulationScale::Years => state.simulation_speed / 365.0,
                        };

                        ui.horizontal(|ui| {
                            ui.add(
                                egui::DragValue::f64(&mut sim_speed_in_units)
                                    .speed(0.001)
                                    .suffix("x speed"),
                            );
                        });

                        state.simulation_speed = match state.simulation_scale {
                            SimulationScale::Seconds => sim_speed_in_units / 86400.0,
                            SimulationScale::Days => sim_speed_in_units,
                            SimulationScale::Years => sim_speed_in_units * 365.0,
                        };

                        if ui
                            .add(
                                egui::Button::new(sim_date_str)
                                    .text_style(egui::TextStyle::Monospace),
                            )
                            .clicked
                        {
                            log::info!("Clicked on clock!");
                        }

                        if ui
                            .add(
                                egui::Button::new(format!("{} bodies", num_bodies))
                                    .text_style(egui::TextStyle::Monospace),
                            )
                            .clicked
                        {
                            self.open_windows.scene_hierarchy = !self.open_windows.scene_hierarchy;
                        }

                        ui.horizontal(|ui| {
                            let ref_name = match scene_man.get_main_scene() {
                                Some(scene) => match state.camera.reference_entity {
                                    Some(reference) => {
                                        scene.get_entity_name(reference).unwrap_or_default()
                                    }
                                    None => "Not tracking",
                                },
                                None => "No scene!",
                            };

                            let clear_resp = ui
                                .add(
                                    egui::Button::new("🗑")
                                        .enabled(state.camera.reference_entity.is_some()),
                                )
                                .on_hover_text("Stop tracking this body");
                            if clear_resp.clicked {
                                state.camera.next_reference_entity = Some(ReferenceChange::Clear);
                            }

                            ui.label(ref_name);
                        });
                    });
                })
                .1;
            handle_output!(state, response);

            let mut style = ui.ctx().style().deref().clone();
            style.visuals.widgets.noninteractive.bg_fill = old_fill;
            style.visuals.widgets.noninteractive.bg_stroke.width = old_stroke;
            ui.ctx().set_style(style);
        });
    }

    fn draw_open_windows(
        &mut self,
        state: &mut AppState,
        scene_man: &mut SceneManager,
        res_man: &mut ResourceManager,
    ) {
        self.draw_debug_window(state, scene_man);

        if let Some(main_scene) = scene_man.get_main_scene() {
            self.draw_scene_hierarchy_window(state, main_scene);
        }

        self.draw_about_window(state);
        self.draw_scene_browser(state, scene_man, res_man);
    }

    fn draw_pop_ups(
        &mut self,
        state: &mut AppState,
        scene_man: &mut SceneManager,
        res_man: &mut ResourceManager,
    ) {
        let scene = scene_man.get_main_scene();
        if scene.is_none() {
            return;
        }
        let scene = scene.unwrap();

        UICTX.with(|ui| {
            let ref_mut = ui.borrow_mut();
            let ui = ref_mut.as_ref().unwrap();

            let mut cam_pos = state.camera.pos;
            let mut cam_target = state.camera.target;
            if let Some(reference) = state.camera.reference_translation {
                cam_pos += reference;
                cam_target += reference;
            }

            let world_to_ndc = state.camera.p * state.camera.v;

            // If we have up locked to +Z, we need to calculate the real up vector
            let forward = (cam_target - cam_pos).normalize();
            let right = forward.cross(&state.camera.up).normalize();

            let mut response: Option<egui::Response> = None;

            for selected_entity in &state.selection {
                let name = scene.get_entity_name(*selected_entity);
                if name.is_none() {
                    continue;
                }
                let name = name.unwrap();

                let trans = scene
                    .get_component::<TransformComponent>(*selected_entity)
                    .and_then(|c| Some(c.get_world_transform()));
                if trans.is_none() {
                    continue;
                }
                let trans = trans.unwrap();

                // I think there should be a much simpler way of finding the label position using trig,
                // but I couldn't do it without trashing precision
                let scale = (trans.scale.x + trans.scale.y + trans.scale.z) / 3.0;
                let mut obj_to_cam = cam_pos - Point3::from(trans.trans);
                let distance = obj_to_cam.magnitude();
                obj_to_cam = obj_to_cam.normalize();

                // Have to enforce all models are within a radius 1 sphere to use this...
                let ang_dir_to_tangent = (scale / distance).acos();

                // Note that this axis is only equal camera up if the object is directly ahead. In most
                // cases this is slightly different than cam up
                let axis = obj_to_cam.cross(&right).normalize();

                let rotation = na::Rotation3::new(axis * ang_dir_to_tangent);

                let obj_to_tang = rotation.transform_vector(&obj_to_cam) * scale * 1.1;
                let tang_point = Point3::from(trans.trans) + obj_to_tang;

                let obj_v = state.camera.v.transform_point(&Point3::from(trans.trans));

                let mut ndc = world_to_ndc.transform_point(&tang_point);

                // If it's behind us we have to flip this to keep it showing on the same side
                if obj_v.z > 0.0 {
                    ndc.x *= -1.0;
                    ndc.y *= -1.0;
                    ndc.z = 0.0;

                    // Make sure it's pushed out of the NDC box so that it never shows in the middle of the screen
                    // if it's behind us, but always on the edge
                    let ndc_dir = ndc.coords.normalize();
                    ndc += ndc_dir * 2.0;
                }

                let mut canvas_x = (state.canvas_width as f64 * (ndc.x + 1.0) / 2.0) as i32 + 1;
                let mut canvas_y = (state.canvas_height as f64 * (1.0 - ndc.y) / 2.0) as i32 + 1;

                // TODO: Figure out how (if possible) to do this with egui...
                let expected_width = 60;
                let expected_height = 42;
                let margin = 20;

                canvas_x = canvas_x
                    .max(margin)
                    .min((state.canvas_width - expected_width) as i32);
                canvas_y = canvas_y
                    .max(margin + expected_height / 2)
                    .min((state.canvas_height - expected_height as u32) as i32);

                let mut entity_to_track: Option<ReferenceChange> = None;
                let mut entity_to_go_to: Option<Entity> = None;

                let label_response = egui::Window::new(name)
                    .fixed_pos(egui::Pos2 {
                        x: canvas_x as f32,
                        y: canvas_y as f32 - 20.0, // TODO: Find actual size
                    })
                    .resizable(false)
                    .scroll(false)
                    .show(&ui.ctx(), |ui| {
                        ui.label(format!("Distance: {:.3} Mm", distance));

                        ui.horizontal(|ui| {
                            if state.camera.reference_entity == Some(*selected_entity) {
                                let but_res = ui.button("🗑").on_hover_text("Clear tracking");
                                if but_res.clicked {
                                    entity_to_track = Some(ReferenceChange::Clear);
                                }
                            } else {
                                let but_res = ui.button("🎥").on_hover_text("Track");
                                if but_res.clicked {
                                    entity_to_track =
                                        Some(ReferenceChange::Track(*selected_entity));
                                }
                            }

                            let but_res = ui.button("🔍").on_hover_text("Go to");
                            if but_res.clicked {
                                entity_to_go_to = Some(*selected_entity);
                            }
                        });
                    })
                    .unwrap();

                state.camera.next_reference_entity = entity_to_track;
                state.camera.entity_going_to = entity_to_go_to;

                match response {
                    Some(ref mut response) => *response |= label_response,
                    None => response = Some(label_response),
                };
            }

            for hovered_entity in &state.hovered {
                if state.selection.contains(hovered_entity) {
                    continue;
                }

                let name = scene.get_entity_name(*hovered_entity);
                if name.is_none() {
                    continue;
                }
                let name = name.unwrap();

                egui::Window::new(name)
                    .fixed_pos(egui::Pos2 {
                        x: state.input.mouse_x as f32,
                        y: state.input.mouse_y as f32,
                    })
                    .resizable(false)
                    .scroll(false)
                    .collapsible(false)
                    .show(&ui.ctx(), |ui| {});
            }

            if let Some(response) = response {
                handle_output!(state, response);
            }
        });
    }

    fn draw_debug_window(&mut self, state: &mut AppState, scene_man: &mut SceneManager) {
        UICTX.with(|ui| {
            let ref_mut = ui.borrow_mut();
            let ui = ref_mut.as_ref().unwrap();

            let frame_rate = self.last_frame_rate;

            let response = egui::Window::new("Debug")
                .open(&mut self.open_windows.debug)
                .show(&ui.ctx(), |ui| {
                    let mut response = ui.columns(2, |cols| {
                        cols[0].label("Simulation time since reference:");
                        cols[1].label(format!("{:.2} days", state.sim_time_days))
                    });

                    response |= ui.columns(2, |cols| {
                        cols[0].label("Simulation date:");
                        cols[1].label(format!(
                            "{}",
                            julian_date_number_to_date(Jdn(state.sim_time_days + J2000_JDN.0))
                        ))
                    });

                    response |= ui.columns(2, |cols| {
                        cols[0].label("Real time since start:");
                        cols[1].label(format!("{:.2} s", state.real_time_s))
                    });

                    response |= ui.columns(2, |cols| {
                        cols[0].label("Frames per second:");
                        cols[1].label(format!("{:.2}", frame_rate))
                    });

                    let mut sim_scale = state.simulation_scale;
                    let mut sim_speed_in_units = match state.simulation_scale {
                        SimulationScale::Seconds => state.simulation_speed * 86400.0,
                        SimulationScale::Days => state.simulation_speed,
                        SimulationScale::Years => state.simulation_speed / 365.0,
                    };

                    response |= ui.columns(2, |cols| {
                        cols[0].label("Simulation scale:");

                        cols[1]
                            .horizontal(|ui| {
                                ui.add(egui::DragValue::f64(&mut sim_speed_in_units).speed(0.01));

                                egui::combo_box(
                                    ui,
                                    egui::Id::new("Simulation scale"),
                                    state.simulation_scale.to_str(),
                                    |ui| {
                                        ui.selectable_value(
                                            &mut sim_scale,
                                            SimulationScale::Years,
                                            SimulationScale::Years.to_str(),
                                        );
                                        ui.selectable_value(
                                            &mut sim_scale,
                                            SimulationScale::Days,
                                            SimulationScale::Days.to_str(),
                                        );
                                        ui.selectable_value(
                                            &mut sim_scale,
                                            SimulationScale::Seconds,
                                            SimulationScale::Seconds.to_str(),
                                        );
                                    },
                                );
                            })
                            .1
                    });

                    state.simulation_scale = sim_scale;
                    state.simulation_speed = match sim_scale {
                        SimulationScale::Seconds => sim_speed_in_units / 86400.0,
                        SimulationScale::Days => sim_speed_in_units,
                        SimulationScale::Years => sim_speed_in_units * 365.0,
                    };

                    ui.separator();

                    response |= ui.columns(2, |cols| {
                        cols[0].label("Light intensity exponent:");
                        cols[1].add(
                            egui::DragValue::f32(&mut state.light_intensity)
                                .range(-1000.0..=1000.0)
                                .speed(0.01),
                        )
                    });

                    ui.separator();

                    response |= ui.columns(2, |cols| {
                        cols[0].label("Vertical FOV [deg]:");
                        cols[1].add(
                            egui::DragValue::f64(&mut state.camera.fov_v)
                                .range(0.1..=120.0)
                                .speed(0.5),
                        )
                    });

                    response |= ui.columns(2, |cols| {
                        cols[0].label("Near [Mm]:");
                        cols[1].add(egui::DragValue::f64(&mut state.camera.near).speed(0.01))
                    });

                    response |= ui.columns(2, |cols| {
                        cols[0].label("Far [Mm]:");
                        cols[1].add(egui::DragValue::f64(&mut state.camera.far))
                    });

                    response |= ui.columns(2, |cols| {
                        cols[0].label("Camera pos [Mm]:");
                        cols[1]
                            .horizontal(|ui| {
                                let mut r = ui.add(
                                    egui::DragValue::f64(&mut state.camera.pos.x).prefix("x: "),
                                );
                                r |= ui.add(
                                    egui::DragValue::f64(&mut state.camera.pos.y).prefix("y: "),
                                );
                                r |= ui.add(
                                    egui::DragValue::f64(&mut state.camera.pos.z).prefix("z: "),
                                );
                                r
                            })
                            .1
                    });

                    if let Some(scene) = scene_man.get_main_scene_mut() {
                        response |= ui.columns(2, |cols| {
                            let mut res = cols[0].label("Reference:");

                            if let Some(reference) = state.camera.reference_entity {
                                res |= cols[1]
                                    .horizontal(|ui| {
                                        let mut r = ui.label(format!(
                                            "{:?}: {}",
                                            reference,
                                            scene.get_entity_name(reference).unwrap_or_default()
                                        ));

                                        let clear_resp = ui
                                            .button("🗑")
                                            .on_hover_text("Stop tracking this entity");
                                        if clear_resp.clicked {
                                            state.camera.next_reference_entity =
                                                Some(ReferenceChange::Clear);
                                        }

                                        r |= clear_resp;
                                        r
                                    })
                                    .1;
                            };

                            res
                        });
                    };

                    ui.separator();

                    response |= ui.columns(2, |cols| {
                        cols[0].label("Move speed [???]:");
                        cols[1].add(
                            egui::DragValue::f64(&mut state.move_speed)
                                .range(1.0..=1000.0)
                                .speed(0.1),
                        )
                    });

                    response |= ui.columns(2, |cols| {
                        cols[0].label("Rotation speed:");
                        cols[1].add(
                            egui::DragValue::f64(&mut state.rotate_speed)
                                .range(1.0..=10.0)
                                .speed(0.1),
                        )
                    });

                    if let Some(selection) = state.selection.iter().next().cloned() {
                        if let Some(scene) = scene_man.get_main_scene_mut() {
                            ui.separator();

                            response |= ui.columns(2, |cols| {
                                cols[0].label("Selected entity:");
                                cols[1]
                                    .horizontal(|ui| {
                                        ui.label(format!("{:?}", selection));
                                        let but_res =
                                            ui.button("🎥").on_hover_text("Track this entity");
                                        if but_res.clicked {
                                            state.camera.next_reference_entity =
                                                Some(ReferenceChange::Track(selection));
                                        }

                                        but_res
                                    })
                                    .1
                            });

                            response |= ui.columns(2, |cols| {
                                cols[0].label("Name:");
                                cols[1].label(format!(
                                    "{}",
                                    scene.get_entity_name(selection).unwrap_or_default()
                                ))
                            });

                            // TODO: Make this more generic
                            if let Some(comp) =
                                scene.get_component_mut::<TransformComponent>(selection)
                            {
                                ui.collapsing("Transform component", |ui| comp.draw_details_ui(ui));
                            }

                            if let Some(comp) = scene.get_component_mut::<MeshComponent>(selection)
                            {
                                ui.collapsing("Mesh component", |ui| comp.draw_details_ui(ui));
                            }

                            if let Some(comp) =
                                scene.get_component_mut::<OrbitalComponent>(selection)
                            {
                                ui.collapsing("Orbital component", |ui| comp.draw_details_ui(ui));
                            }

                            if let Some(comp) =
                                scene.get_component_mut::<PhysicsComponent>(selection)
                            {
                                ui.collapsing("Physics component", |ui| comp.draw_details_ui(ui));
                            }
                        }
                    }

                    handle_output!(state, response);
                });

            if let Some(response) = response {
                handle_output!(state, response);
            }
        });
    }

    fn draw_about_window(&mut self, state: &mut AppState) {
        UICTX.with(|ui| {
            let ref_mut = ui.borrow_mut();
            let ui = ref_mut.as_ref().unwrap();

            let mut open_window = self.open_windows.about;

            let response = egui::Window::new("About")
                .open(&mut open_window)
                .scroll(false)
                .resizable(false)
                .fixed_size(egui::vec2(400.0, 400.0))
                .show(&ui.ctx(), |ui| {
                    ui.label("This is a simple, custom N-body simulation 3D engine written for the web.");
                    ui.label("\nIt uses simple semi-implicit Euler integration to calculate the effect of gravity at each timestep.\nInitial J2000 state vectors were collected from NASA's HORIZONS system and JPL's Small-Body Database Search Engine, and when required evolved to J2000 using the mean orbital elements (e.g. for asteroids).");
                    ui.label("\nIt is fully written in Rust (save for some glue Javascript code), and compiled to WebAssembly via wasm_bindgen, which includes WebGL2 bindings.");
                    ui.label("The 3D engine uses a data-oriented entity component system in order to maximize performance of batch physics calculations, and the Egui immediate mode GUI library, also written in pure Rust.");
                    ui.horizontal_wrapped_for_text(egui::TextStyle::Body, |ui| {
                        ui.label("\nProject home page:");
                        ui.hyperlink("https://github.com/1danielcoelho/system-viewer");
                    });
                });

            self.open_windows.about = open_window;

            if let Some(response) = response {
                handle_output!(state, response);
            }
        });
    }

    fn draw_scene_browser(
        &mut self,
        state: &mut AppState,
        scene_man: &mut SceneManager,
        res_man: &mut ResourceManager,
    ) {
        UICTX.with(|ui| {
            let ref_mut = ui.borrow_mut();
            let ui = ref_mut.as_ref().unwrap();

            let mut open_window = self.open_windows.scene_browser;

            let response = egui::Window::new("Scene browser")
                .open(&mut open_window)
                .scroll(false)
                .resizable(false)
                .fixed_size(egui::vec2(600.0, 300.0))
                .show(&ui.ctx(), |ui| {
                    ui.columns(2, |cols| {
                        cols[0].set_min_height(300.0);
                        cols[1].set_min_height(300.0);

                        let main_name = scene_man.get_main_scene().unwrap().identifier.clone();

                        egui::Frame::dark_canvas(cols[0].style()).show(&mut cols[0], |ui| {
                            ui.set_min_height(ui.available_size().y);

                            egui::ScrollArea::from_max_height(std::f32::INFINITY).show(ui, |ui| {
                                for (index, scene) in scene_man.descriptions.0.iter().enumerate() {
                                    ui.radio_value(
                                        &mut self.selected_scene_desc_index,
                                        index as u32,
                                        {
                                            if &scene.name == &main_name {
                                                scene.name.to_owned() + " (active)"
                                            } else {
                                                scene.name.to_owned()
                                            }
                                        },
                                    );
                                }
                            });
                        });

                        let selected_is_active = match scene_man
                            .descriptions
                            .0
                            .get(self.selected_scene_desc_index as usize)
                        {
                            Some(desc) => &desc.name == &main_name,
                            None => false,
                        };

                        // HACK: 32.0 is the height of the button. I have no idea how to do this programmatically
                        // Maybe with a bottom up layout, but there is some type of egui crash when I put a ScrollArea
                        // inside another layout, and the ScrollArea spawns a scrollbar
                        let height = cols[1].available_size().y - 32.0;

                        egui::ScrollArea::from_max_height(height).show(&mut cols[1], |ui| {
                            ui.set_min_height(height);

                            if let Some(desc) = scene_man
                                .descriptions
                                .0
                                .get(self.selected_scene_desc_index as usize)
                            {
                                ui.columns(2, |cols| {
                                    cols[0].label("Name:");
                                    cols[1].label(&desc.name);
                                });

                                ui.columns(2, |cols| {
                                    cols[0].label("Description:");
                                    cols[1].label(&desc.description);
                                });

                                ui.columns(2, |cols| {
                                    cols[0].label("Time:");
                                    cols[1].label(&desc.time);
                                });

                                ui.columns(2, |cols| {
                                    cols[0].label("Simulation scale:");
                                    cols[1].label(format!("{}", &desc.simulation_scale));
                                });

                                ui.columns(2, |cols| {
                                    cols[0].label("Reference");
                                    cols[1].label(&desc.reference);
                                });

                                let mut num_bodies = 0;
                                for (_, bodies) in desc.bodies.iter() {
                                    num_bodies += bodies.len();
                                }

                                ui.columns(2, |cols| {
                                    cols[0].label("Bodies:");
                                    cols[1].label(format!("{}", num_bodies));
                                });
                            }
                        });

                        cols[1].separator();

                        // TODO: There has to be a simpler way of just centering two buttons on that space
                        cols[1].columns(2, |cols| {
                            cols[0].with_layout(egui::Layout::right_to_left(), |ui| {
                                if ui.button("   Open   ").clicked {
                                    let name = &scene_man.descriptions.0
                                        [self.selected_scene_desc_index as usize]
                                        .name
                                        .clone();
                                    scene_man.set_scene(name, res_man, Some(state));
                                }
                            });

                            cols[1].with_layout(egui::Layout::left_to_right(), |ui| {
                                let close_resp = ui
                                    .add(
                                        egui::Button::new("   Close   ")
                                            .enabled(selected_is_active),
                                    )
                                    .on_hover_text("Close this scene");
                                if close_resp.clicked {
                                    let name = &scene_man.descriptions.0
                                        [self.selected_scene_desc_index as usize]
                                        .name
                                        .clone();
                                    scene_man.delete_scene(name);
                                }
                            });
                        });
                    });
                });

            self.open_windows.scene_browser = open_window;

            if let Some(response) = response {
                handle_output!(state, response);
            }
        });
    }

    fn draw_scene_hierarchy_window(&mut self, state: &mut AppState, scene: &Scene) {
        UICTX.with(|ui| {
            let ref_mut = ui.borrow_mut();
            let ui = ref_mut.as_ref().unwrap();

            let mut open_window = self.open_windows.scene_hierarchy;

            let response = egui::Window::new("Scene hierarchy")
                .open(&mut open_window)
                .scroll(false)
                .resizable(true)
                .default_size(egui::vec2(300.0, 400.0))
                .show(&ui.ctx(), |ui| {
                    ui.set_min_height(300.0);

                    egui::Frame::dark_canvas(ui.style()).show(ui, |ui| {
                        ui.set_min_height(ui.available_size().y);

                        egui::ScrollArea::from_max_height(std::f32::INFINITY).show(ui, |ui| {
                            for entity in scene.get_entity_entries() {
                                if !entity.live {
                                    continue;
                                }

                                if let Some(name) = &entity.name {
                                    if ui.button(name).clicked {
                                        state.selection.clear();
                                        state.selection.insert(entity.current);
                                    }
                                }
                            }
                        });
                    });
                });

            self.open_windows.scene_hierarchy = open_window;

            if let Some(response) = response {
                handle_output!(state, response);
            }
        });
    }
}

fn handle_mouse_on_scene(state: &mut AppState, scene: &mut Scene) {
    let end_world = state.camera.canvas_to_world(
        state.input.mouse_x,
        state.input.mouse_y,
        state.canvas_width,
        state.canvas_height,
    );

    let mut start_world = state.camera.pos.clone();
    if let Some(reference) = state.camera.reference_translation {
        start_world += reference;
    }

    let ray = Ray {
        start: start_world,
        direction: (end_world - start_world).normalize(),
    };

    let entity = if let Some(hit) = raycast(&ray, &scene) {
        scene.get_entity_from_index(hit.entity_index)
    } else {
        None
    };

    state.hovered.clear();

    if state.input.m0 == ButtonState::Pressed {
        state.selection.clear();

        if entity.is_some() {
            state.selection.insert(entity.unwrap());
        }
    } else {
        if entity.is_some() {
            state.hovered.insert(entity.unwrap());
        }
    }
}
