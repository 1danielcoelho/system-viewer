use crate::app_state::{AppState, ButtonState, ReferenceChange, SimulationScale};
use crate::components::{MeshComponent, OrbitalComponent, TransformComponent};
use crate::managers::details_ui::DetailsUI;
use crate::managers::scene::component_storage::ComponentStorage;
use crate::managers::scene::{Scene, SceneManager};
use crate::managers::ResourceManager;
use crate::utils::raycasting::{raycast, Ray};
use crate::utils::units::{julian_date_number_to_date, Jdn, J2000_JDN};
use crate::utils::web::write_string_to_file_prompt;
use crate::{prompt_for_bytes_file, prompt_for_text_file, UICTX};
use egui::Srgba;
use gui_backend::WebInput;
use lazy_static::__Deref;
use na::{Matrix4, Point3, Translation3, Vector3};
use std::borrow::BorrowMut;
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
    scene_man: bool,
    scene_hierarchy: bool,
    scene_browser: bool,
    about: bool,
}

pub struct InterfaceManager {
    backend: gui_backend::WebBackend,
    web_input: WebInput,
    open_windows: OpenWindows,
    selected_scene_name: String,

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
                scene_man: false,
                scene_hierarchy: false,
                scene_browser: true,
                about: false,
            },
            selected_scene_name: String::new(),
            frame_times: vec![16.66; 15].into_iter().collect(),
            time_of_last_update: -2.0,
            last_frame_rate: 60.0, // Optimism
        };
    }

    /**
     * This runs before all systems, and starts collecting all the UI elements we'll draw, as
     * well as draws the main UI
     */
    pub fn begin_frame(
        &mut self,
        state: &mut AppState,
        scene_man: &mut SceneManager,
        res_man: &mut ResourceManager,
    ) {
        self.pre_draw(state);

        self.draw_main_ui(state, scene_man, res_man);
    }

    /** This runs after all systems, and draws the collected UI elements to the framebuffer */
    pub fn end_frame(&mut self, state: &mut AppState, scene: &mut Scene) {
        self.draw();

        if !state.input.over_ui && state.input.m1 == ButtonState::Depressed {
            handle_mouse_on_scene(state, scene);
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

            egui::TopPanel::top(egui::Id::new("top panel")).show(&ui.ctx(), |ui| {
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
                        if ui.button("Reset scene").clicked {}

                        if ui.button("Scene browser").clicked {
                            self.open_windows.scene_browser = !self.open_windows.scene_browser;
                        }

                        ui.separator();

                        if ui.button("About").clicked {
                            self.open_windows.about = !self.open_windows.about;
                        }

                        if DEBUG {
                            ui.separator();
                            ui.separator();

                            if ui.button("New").clicked {
                                let new_scene_name =
                                    scene_man.new_scene("New scene").unwrap().identifier.clone();
                                scene_man.set_scene(&new_scene_name, res_man);
                            }

                            if ui.button("Open").clicked {
                                prompt_for_text_file("scene", ".ron");
                            }

                            if ui.button("Save").clicked {
                                if let Some(scene) = scene_man.get_main_scene() {
                                    let ser_str = scene.serialize();
                                    write_string_to_file_prompt(
                                        &format!("{}.ron", &scene.identifier),
                                        &ser_str,
                                    );
                                } else {
                                    log::warn!("Clicked Save but no scene is currently loaded!");
                                }
                            }

                            ui.separator();

                            if ui.button("Close").clicked {
                                let new_scene_name =
                                    scene_man.new_scene("New scene").unwrap().identifier.clone();
                                scene_man.set_scene(&new_scene_name, res_man);
                            }

                            if ui.button("Inject GLB...").clicked {
                                prompt_for_bytes_file("glb_inject", ".glb");
                            }

                            if ui.button("Inject orbital elements CSV...").clicked {
                                prompt_for_text_file("csv_inject", ".csv");
                            }

                            ui.separator();

                            if ui.button("Debug").clicked {
                                self.open_windows.debug = !self.open_windows.debug;
                            }

                            if ui.button("Scene manager").clicked {
                                self.open_windows.scene_man = !self.open_windows.scene_man;
                            }

                            if ui.button("Scene hierarchy").clicked {
                                self.open_windows.scene_hierarchy =
                                    !self.open_windows.scene_hierarchy;
                            }

                            ui.separator();

                            if ui.button("Organize windows").clicked {
                                ui.ctx().memory().reset_areas();
                            }

                            if ui.button("Close all").clicked {
                                self.open_windows.debug = false;
                                self.open_windows.scene_man = false;
                                self.open_windows.scene_hierarchy = false;
                            }

                            ui.separator();

                            if ui
                                .button("Clear Egui memory")
                                .on_hover_text("Forget scroll, collapsing headers etc")
                                .clicked
                            {
                                *ui.ctx().memory() = Default::default();
                            }

                            if ui
                                .button("Reset app state")
                                .on_hover_text("Clears app state from local storage")
                                .clicked
                            {
                                state.pending_reset = true;
                            }
                        }
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
                        .add(egui::Button::new(sim_date_str).text_style(egui::TextStyle::Monospace))
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
            });

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
        self.draw_scene_manager_window(state, scene_man, res_man);

        if let Some(main_scene) = scene_man.get_main_scene() {
            self.draw_scene_hierarchy_window(state, main_scene);
        }

        self.draw_about_window(state);
        self.draw_scene_browser(state, scene_man, res_man);
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
                        cols[0].label("Vertical FOV (deg):");
                        cols[1].add(
                            egui::DragValue::f64(&mut state.camera.fov_v)
                                .range(0.1..=120.0)
                                .speed(0.5),
                        )
                    });

                    response |= ui.columns(2, |cols| {
                        cols[0].label("Near:");
                        cols[1].add(egui::DragValue::f64(&mut state.camera.near).speed(0.01))
                    });

                    response |= ui.columns(2, |cols| {
                        cols[0].label("Far:");
                        cols[1].add(egui::DragValue::f64(&mut state.camera.far))
                    });

                    response |= ui.columns(2, |cols| {
                        cols[0].label("Camera pos:");
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
                        cols[0].label("Move speed:");
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
                                                Some(ReferenceChange::NewEntity(selection));
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

                            if let Some(comp) = scene.get_component::<TransformComponent>(selection)
                            {
                                ui.collapsing("Transform component", |ui| comp.draw_details_ui(ui));
                            }

                            if let Some(comp) = scene.get_component::<MeshComponent>(selection) {
                                ui.collapsing("Mesh component", |ui| comp.draw_details_ui(ui));
                            }

                            if let Some(comp) = scene.get_component::<OrbitalComponent>(selection) {
                                ui.collapsing("Orbital component", |ui| comp.draw_details_ui(ui));
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

    fn draw_scene_manager_window(
        &mut self,
        state: &mut AppState,
        scene_man: &mut SceneManager,
        res_man: &mut ResourceManager,
    ) {
        UICTX.with(|ui| {
            let ref_mut = ui.borrow_mut();
            let ui = ref_mut.as_ref().unwrap();

            let mut open_window = self.open_windows.scene_man;

            let response = egui::Window::new("Scene manager")
                .open(&mut open_window)
                .scroll(false)
                .resizable(false)
                .fixed_size(egui::vec2(600.0, 300.0))
                .show(&ui.ctx(), |ui| {
                    ui.columns(2, |cols| {
                        cols[0].set_min_height(300.0);
                        cols[1].set_min_height(300.0);

                        let main_name = scene_man
                            .get_main_scene_name()
                            .as_ref()
                            .and_then(|s| Some(s.clone()))
                            .unwrap_or_default();

                        egui::Frame::dark_canvas(cols[0].style()).show(&mut cols[0], |ui| {
                            ui.set_min_height(ui.available_size().y);

                            egui::ScrollArea::from_max_height(std::f32::INFINITY).show(ui, |ui| {
                                if self.selected_scene_name.is_empty() {
                                    self.selected_scene_name = main_name.clone();
                                }
                                for scene in scene_man.sorted_loaded_scene_names.iter() {
                                    ui.radio_value(&mut self.selected_scene_name, scene.clone(), {
                                        if scene == &main_name {
                                            scene.to_owned() + " (active)"
                                        } else {
                                            scene.clone()
                                        }
                                    });
                                }
                            });
                        });

                        if let Some(scene) = scene_man.get_scene(&self.selected_scene_name) {
                            egui::CollapsingHeader::new("Selected scene:")
                                .default_open(true)
                                .show(&mut cols[1], |ui| {
                                    ui.columns(2, |cols| {
                                        cols[0].label("Name:");
                                        cols[1].label(&scene.identifier);
                                    });

                                    ui.columns(2, |cols| {
                                        cols[0].label("Entities:");
                                        cols[1].label(format!("{}", scene.get_num_entities()));
                                    });
                                });

                            cols[1].with_layout(
                                egui::Layout::bottom_up(egui::Align::right()),
                                |ui| {
                                    ui.horizontal(|ui| {
                                        if ui
                                            .add(
                                                egui::Button::new("Delete")
                                                    .enabled(self.selected_scene_name != main_name),
                                            )
                                            .clicked
                                        {
                                            scene_man.delete_scene(&self.selected_scene_name);
                                        }

                                        if ui.button("Load").clicked {
                                            scene_man.set_scene(&self.selected_scene_name, res_man);
                                        }

                                        if ui
                                            .add(
                                                egui::Button::new("Add")
                                                    .enabled(self.selected_scene_name != main_name),
                                            )
                                            .clicked
                                        {
                                            scene_man
                                                .inject_scene(
                                                    &self.selected_scene_name,
                                                    None,
                                                    res_man,
                                                )
                                                .unwrap();
                                        }
                                    });
                                },
                            );
                        }
                    });
                });

            self.open_windows.scene_man = open_window;

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

                        let main_name = scene_man
                            .get_main_scene_name()
                            .as_ref()
                            .and_then(|s| Some(s.clone()))
                            .unwrap_or_default();

                        egui::Frame::dark_canvas(cols[0].style()).show(&mut cols[0], |ui| {
                            ui.set_min_height(ui.available_size().y);

                            egui::ScrollArea::from_max_height(std::f32::INFINITY).show(ui, |ui| {
                                if self.selected_scene_name.is_empty() {
                                    self.selected_scene_name = main_name.clone();
                                }
                                for scene in scene_man.sorted_loaded_scene_names.iter() {
                                    ui.radio_value(&mut self.selected_scene_name, scene.clone(), {
                                        if scene == &main_name {
                                            scene.to_owned() + " (active)"
                                        } else {
                                            scene.clone()
                                        }
                                    });
                                }
                            });
                        });

                        if let Some(scene) = scene_man.get_scene(&self.selected_scene_name) {
                            egui::CollapsingHeader::new("Selected scene:")
                                .default_open(true)
                                .show(&mut cols[1], |ui| {
                                    ui.columns(2, |cols| {
                                        cols[0].label("Name:");
                                        cols[1].label(&scene.identifier);
                                    });

                                    ui.columns(2, |cols| {
                                        cols[0].label("Bodies:");
                                        cols[1].label(format!("{}", scene.get_num_entities()));
                                    });

                                    ui.columns(2, |cols| {
                                        cols[0].label("Description:");
                                        cols[1].label("This is where the description would be. It would say something like \" Hey this is a solar system simulation at J2000 where you can see where Oumuamua was coming from and so on\"");
                                    });
                                });

                            cols[1].with_layout(
                                egui::Layout::bottom_up(egui::Align::Center),
                                |ui| {
                                    if ui.button("   Open   ").clicked {
                                        scene_man.set_scene(&self.selected_scene_name, res_man);
                                    }
                                },
                            );
                        }
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
    let p = Matrix4::new_perspective(
        state.canvas_width as f64 / state.canvas_height as f64,
        state.camera.fov_v.to_radians() as f64,
        state.camera.near as f64,
        state.camera.far as f64,
    );

    let v = Matrix4::look_at_rh(&state.camera.pos, &state.camera.target, &state.camera.up);

    let reference = match state.camera.reference_entity {
        Some(reference) => {
            let trans = scene
                .get_component::<TransformComponent>(reference)
                .unwrap()
                .get_world_transform()
                .trans;

            Translation3::new(trans.x as f64, trans.y as f64, trans.z as f64).to_homogeneous()
        }
        None => Matrix4::identity(),
    };

    let ndc_to_world: Matrix4<f64> =
        reference * v.try_inverse().unwrap() * p.try_inverse().unwrap();

    let ndc_near_pos = Point3::from(Vector3::new(
        -1.0 + 2.0 * state.input.mouse_x as f64 / (state.canvas_width - 1) as f64,
        1.0 - 2.0 * state.input.mouse_y as f64 / (state.canvas_height - 1) as f64,
        -1.0,
    ));

    let end_world = ndc_to_world.transform_point(&ndc_near_pos);
    let start_world = reference.transform_point(&state.camera.pos);

    let ray = Ray {
        start: start_world,
        direction: (end_world - start_world).normalize(),
    };

    if state.input.m0 == ButtonState::Pressed {
        if let Some(hit) = raycast(&ray, &scene) {
            if let Some(entity) = scene.get_entity_from_index(hit.entity_index) {
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
