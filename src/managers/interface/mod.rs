use crate::{
    app_state::{AppState, ButtonState},
    components::{MeshComponent, OrbitalComponent, TransformComponent},
    managers::{details_ui::DetailsUI, scene::SceneManager, ResourceManager},
    prompt_for_bytes_file, prompt_for_text_file,
    utils::{
        raycasting::{raycast, Ray},
        web::write_string_to_file_prompt,
    },
};
use crate::{managers::scene::Scene, UICTX};
use egui::{
    menu, Align, Button, CollapsingHeader, Frame, Id, LayerId, Layout, Pos2, Response, ScrollArea,
    TextStyle, TopPanel, Ui,
};
use gui_backend::WebInput;
use na::{Matrix4, Point3, Vector3};

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

struct OpenWindows {
    debug: bool,
    scene_man: bool,
}

pub struct InterfaceManager {
    backend: gui_backend::WebBackend,
    web_input: WebInput,
    open_windows: OpenWindows,
    selected_scene_name: String,
}
impl InterfaceManager {
    pub fn new() -> Self {
        return Self {
            backend: gui_backend::WebBackend::new("rustCanvas")
                .expect("Failed to make a web backend for egui"),
            web_input: Default::default(),
            open_windows: OpenWindows {
                debug: true,
                scene_man: false,
            },
            selected_scene_name: String::new(),
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
        raw_input.mouse_pos = Some(Pos2 {
            x: state.input.mouse_x as f32,
            y: state.input.mouse_y as f32,
        });

        // HACK: Currently the UI sets the button state to handled if mouse down happens over it...
        raw_input.mouse_down = state.input.m0 != ButtonState::Depressed;
        raw_input.events.append(&mut state.input.egui_keys);
        raw_input.modifiers = state.input.modifiers;

        self.backend.begin_frame(raw_input);
        let rect = self.backend.ctx.available_rect();

        UICTX.with(|ui| {
            let mut ui = ui.borrow_mut();
            ui.replace(Ui::new(
                self.backend.ctx.clone(),
                LayerId::background(),
                Id::new("interface"),
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
            let ref_mut = ui.borrow_mut();
            let ui = ref_mut.as_ref().unwrap();

            TopPanel::top(Id::new("top panel")).show(&ui.ctx(), |ui| {
                menu::bar(ui, |ui| {
                    menu::menu(ui, "File", |ui| {
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
                    });

                    menu::menu(ui, "Edit", |ui| {
                        if ui.button("Inject GLB...").clicked {
                            prompt_for_bytes_file("glb_inject", ".glb");
                        }

                        if ui.button("Inject orbital elements CSV...").clicked {
                            prompt_for_text_file("csv_inject", ".csv");
                        }
                    });

                    menu::menu(ui, "Window", |ui| {
                        if ui.button("Debug").clicked {
                            self.open_windows.debug = !self.open_windows.debug;
                        }

                        if ui.button("Scene manager").clicked {
                            self.open_windows.scene_man = !self.open_windows.scene_man;
                        }

                        ui.separator();

                        if ui.button("Organize windows").clicked {
                            ui.ctx().memory().reset_areas();
                        }

                        if ui.button("Close all").clicked {
                            self.open_windows.debug = false;
                            self.open_windows.scene_man = false;
                        }
                    });

                    menu::menu(ui, "Tools", |ui| {
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
    }

    fn draw_debug_window(&mut self, state: &mut AppState, scene_man: &mut SceneManager) {
        UICTX.with(|ui| {
            let ref_mut = ui.borrow_mut();
            let ui = ref_mut.as_ref().unwrap();

            let response = egui::Window::new("Debug")
                .open(&mut self.open_windows.debug)
                .show(&ui.ctx(), |ui| {
                    let mut response = ui.columns(2, |cols| {
                        cols[0].label("Simulated seconds since start:");
                        cols[1].label(format!("{:.2}", state.phys_time_ms / 1000.0))
                    });

                    response |= ui.columns(2, |cols| {
                        cols[0].label("Real seconds since start:");
                        cols[1].label(format!("{:.2}", state.real_time_ms / 1000.0))
                    });

                    response |= ui.columns(2, |cols| {
                        cols[0].label("Frames per second:");
                        cols[1].label(format!("{:.2}", 1000.0 / state.real_delta_time_ms))
                    });

                    response |= ui.columns(2, |cols| {
                        cols[0].label("Simulation speed:");
                        cols[1].add(
                            egui::DragValue::f64(&mut state.simulation_speed)
                                .range(-100.0..=100.0)
                                .speed(0.01),
                        )
                    });

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
                        cols[1].add(egui::DragValue::f32(&mut state.camera.near));
                    });

                    ui.columns(2, |cols| {
                        cols[0].label("Far:");
                        cols[1].add(egui::DragValue::f32(&mut state.camera.far));
                    });

                    ui.columns(2, |cols| {
                        cols[0].label("Camera pos:");
                        cols[1].with_layout(
                            Layout::left_to_right().with_cross_align(Align::Min),
                            |ui| {
                                let mut r = ui.add(
                                    egui::DragValue::f32(&mut state.camera.pos.x).prefix("x: "),
                                );
                                r |= ui.add(
                                    egui::DragValue::f32(&mut state.camera.pos.y).prefix("y: "),
                                );
                                r |= ui.add(
                                    egui::DragValue::f32(&mut state.camera.pos.z).prefix("z: "),
                                );
                                r
                            },
                        );
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
                        )
                    });

                    if let Some(selection) = state.selection.iter().next().cloned() {
                        if let Some(scene) = scene_man.get_main_scene_mut() {
                            ui.separator();

                            ui.columns(2, |cols| {
                                cols[0].label("Selected entity:");
                                cols[1].label(format!("{:?}", selection));
                            });

                            ui.columns(2, |cols| {
                                cols[0].label("Name:");
                                cols[1].label(format!(
                                    "{}",
                                    scene.get_entity_name(selection).unwrap_or_default()
                                ));
                            });

                            if let Some(comp) = scene.get_component::<TransformComponent>(selection)
                            {
                                ui.collapsing("Transform component", |ui| {
                                    comp.draw_details_ui(ui);
                                });
                            }

                            if let Some(comp) = scene.get_component::<MeshComponent>(selection) {
                                ui.collapsing("Mesh component", |ui| {
                                    comp.draw_details_ui(ui);
                                });
                            }

                            if let Some(comp) = scene.get_component::<OrbitalComponent>(selection) {
                                ui.collapsing("Orbital component", |ui| {
                                    comp.draw_details_ui(ui);
                                });
                            }
                        }
                    }

                    log::info!("{:?}", response);
                    handle_output!(state, response);
                    return response;
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

                        Frame::dark_canvas(cols[0].style()).show(&mut cols[0], |ui| {
                            ui.set_min_height(ui.available_size().y);

                            ScrollArea::from_max_height(std::f32::INFINITY).show(ui, |ui| {
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
                            CollapsingHeader::new("Selected scene:")
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

                            cols[1].with_layout(Layout::bottom_up(Align::right()), |ui| {
                                ui.horizontal(|ui| {
                                    if ui
                                        .add(
                                            Button::new("Delete")
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
                                            Button::new("Add")
                                                .enabled(self.selected_scene_name != main_name),
                                        )
                                        .clicked
                                    {
                                        scene_man
                                            .inject_scene(&self.selected_scene_name, None, res_man)
                                            .unwrap();
                                    }
                                });
                            });
                        }
                    });
                });

            self.open_windows.scene_man = open_window;

            if let Some(response) = response {
                handle_output!(state, response);
            }
        });
    }
}

fn handle_mouse_on_scene(state: &mut AppState, scene: &mut Scene) {
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
        if let Some(hit) = raycast(&ray, &scene.mesh, &scene.transform) {
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
