use crate::app_state::{AppState, ButtonState, ReferenceChange};
use crate::components::{
    MeshComponent, MetadataComponent, OrbitalComponent, PhysicsComponent, TransformComponent,
};
use crate::managers::details_ui::DetailsUI;
use crate::managers::scene::component_storage::ComponentStorage;
use crate::managers::scene::{Entity, Scene, SceneManager};
use crate::managers::ResourceManager;
use crate::utils::raycasting::{raycast, Ray};
use crate::utils::units::{julian_date_number_to_date, Jdn, J2000_JDN};
use crate::{prompt_for_bytes_file, UICTX};
use gui_backend::WebInput;
use lazy_static::__Deref;
use na::*;
use std::collections::VecDeque;

pub mod details_ui;

const DEBUG: bool = false;

struct OpenWindows {
    debug: bool,
    scene_hierarchy: bool,
    scene_browser: bool,
    settings: bool,
    controls: bool,
    about: bool,
}

pub struct InterfaceManager {
    backend: gui_backend::WebBackend,
    web_input: WebInput,
    open_windows: OpenWindows,
    selected_scene_desc_name: String,

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
                settings: false,
                controls: false,
                about: false,
            },
            selected_scene_desc_name: String::from(""),
            frame_times: vec![16.66; 15].into_iter().collect(),
            time_of_last_update: -2.0,
            last_frame_rate: 60.0, // Optimism
        };
    }

    /// This runs before all systems, and starts collecting all the UI elements we'll draw, as
    /// well as draws the main UI
    pub fn begin_frame(&mut self, state: &mut AppState) {
        self.pre_draw(state);
    }

    /// This runs after all systems, and draws the collected UI elements to the framebuffer
    pub fn end_frame(
        &mut self,
        state: &mut AppState,
        scene_man: &mut SceneManager,
        res_man: &mut ResourceManager,
    ) {
        self.draw_main_ui(state, scene_man, res_man);

        self.draw();

        let mut egui_consuming_pointer: bool = false;
        UICTX.with(|ui| {
            let ui = ui.borrow();
            let ui_ref = ui.as_ref().unwrap();
            let ctx = ui_ref.ctx();

            egui_consuming_pointer = ctx.wants_pointer_input();
        });

        // Always reset the hovered entity even if we won't get to
        // hover new ones: This prevents the tooltip from sticking around if the UI
        // starts covering it (which can also include the label tooltip itself)
        state.hovered = None;
        if let Some(scene) = scene_man.get_main_scene_mut() {
            if !egui_consuming_pointer {
                handle_pointer_on_scene(state, scene);
            }
        }
    }

    fn pre_draw(&mut self, state: &mut AppState) {
        state.input.over_ui = false;

        let mut raw_input = self.web_input.new_frame(egui::Vec2 {
            x: state.canvas_width as f32,
            y: state.canvas_height as f32,
        });

        // If we have pointer lock then we don't really want to use the UI (we're rotating/orbiting/etc.)
        // so don't give the updated mouse position to egui
        let window = web_sys::window().unwrap();
        let doc = window.document().unwrap();
        if let None = doc.pointer_lock_element() {
            raw_input.events.append(&mut state.input.egui_events);
        }
        raw_input.modifiers = state.input.modifiers;
        raw_input.scroll_delta = egui::Vec2 {
            x: state.input.scroll_delta_x as f32,
            y: -state.input.scroll_delta_y as f32,
        };

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

        let mut has_kb: bool = false;
        let mut egui_consuming_pointer: bool = false;

        UICTX.with(|ui| {
            let mut ui = ui.borrow_mut();
            ui.replace(egui::Ui::new(
                self.backend.ctx.clone(),
                egui::LayerId::background(),
                egui::Id::new("interface"),
                rect,
                rect,
            ));

            let ui_ref = ui.as_ref().unwrap();
            has_kb = ui_ref.ctx().wants_keyboard_input();
            egui_consuming_pointer = ui_ref.ctx().wants_pointer_input();
        });

        // Suppress our inputs if egui wants it instead
        {
            if egui_consuming_pointer {
                state.input.scroll_delta_x = 0;
                state.input.scroll_delta_y = 0;
            }

            // Consume keyboard input if egui has keyboard focus, to prevent
            // the input manager from also handling these
            if has_kb {
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
    }

    fn draw(&mut self) {
        // We shouldn't need to raycast against the drawn elements because every widget we draw will optionally
        // also write to AppState if the mouse is over itself
        let (_, paint_jobs) = self.backend.end_frame().unwrap();
        self.backend
            .paint(egui::Rgba::TRANSPARENT, paint_jobs)
            .expect("Failed to paint!");
    }

    fn draw_main_ui(
        &mut self,
        state: &mut AppState,
        scene_man: &mut SceneManager,
        res_man: &mut ResourceManager,
    ) {
        self.draw_main_toolbar(state, scene_man, res_man);

        self.draw_open_windows(state, scene_man, res_man);

        self.draw_pop_ups(state, scene_man);
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

            let old_style = ui.ctx().style().deref().clone();
            let mut style = old_style.clone();

            style.visuals.widgets.noninteractive.bg_fill =
                egui::Color32::from_rgba_unmultiplied(0, 0, 0, 0);
            style.visuals.widgets.noninteractive.bg_stroke.width = 0.0;

            style.visuals.widgets.inactive.bg_fill =
                egui::Color32::from_rgba_unmultiplied(0, 0, 0, 200);
            style.visuals.widgets.inactive.bg_stroke.width = 0.0;

            ui.ctx().set_style(style.clone());

            egui::TopPanel::top(egui::Id::new("top panel")).show(&ui.ctx(), |ui| {
                let num_bodies = scene_man
                    .get_main_scene()
                    .unwrap()
                    .physics
                    .get_num_components();

                let sim_date_str = format!(
                    "{}",
                    julian_date_number_to_date(Jdn(state.sim_time_s / 86400.0 + J2000_JDN.0))
                );

                ui.with_layout(egui::Layout::left_to_right(), |ui| {
                    egui::menu::menu(ui, "⚙", |ui| {
                        egui::Frame::dark_canvas(ui.style())
                            .fill(egui::Color32::from_rgba_unmultiplied(0, 0, 0, 200))
                            .show(ui, |ui| {
                                if ui.button("Reset scene").clicked() {
                                    scene_man.set_scene("empty", res_man, state);
                                    scene_man.set_scene(
                                        &self.selected_scene_desc_name,
                                        res_man,
                                        state,
                                    );
                                }

                                if ui.button("Close scene").clicked() {
                                    scene_man.set_scene("empty", res_man, state);
                                }

                                ui.separator();

                                if ui.button("Scene browser").clicked() {
                                    self.open_windows.scene_browser =
                                        !self.open_windows.scene_browser;
                                }

                                ui.separator();

                                if ui.button("Settings").clicked() {
                                    self.open_windows.settings = !self.open_windows.settings;
                                }

                                if ui.button("Controls").clicked() {
                                    self.open_windows.controls = !self.open_windows.controls;
                                }

                                if ui.button("About").clicked() {
                                    self.open_windows.about = !self.open_windows.about;
                                }

                                if DEBUG {
                                    ui.separator();
                                    ui.separator();
                                    ui.separator();

                                    if ui.button("Inject GLB...").clicked() {
                                        prompt_for_bytes_file("glb_inject", ".glb");
                                    }

                                    ui.separator();

                                    if ui.button("Debug").clicked() {
                                        self.open_windows.debug = !self.open_windows.debug;
                                    }

                                    if ui.button("Organize windows").clicked() {
                                        ui.ctx().memory().reset_areas();
                                    }

                                    if ui.button("Close all windows").clicked() {
                                        self.open_windows.debug = false;
                                        self.open_windows.scene_hierarchy = false;
                                        self.open_windows.about = false;
                                        self.open_windows.scene_browser = false;
                                    }

                                    ui.separator();

                                    if ui
                                        .button("Clear Egui memory")
                                        .on_hover_text("Forget scroll, collapsing headers etc")
                                        .clicked()
                                    {
                                        *ui.ctx().memory() = Default::default();
                                    }

                                    if ui
                                        .button("Reset app state")
                                        .on_hover_text("Clears app state from local storage")
                                        .clicked()
                                    {
                                        state.pending_reset = true;
                                    }
                                }
                            });
                    });

                    if ui
                        .add(
                            egui::Button::new(format!("{:.2} fps", self.last_frame_rate))
                                .text_style(egui::TextStyle::Monospace),
                        )
                        .clicked()
                    {}

                    if ui
                        .add(egui::Button::new(sim_date_str).text_style(egui::TextStyle::Monospace))
                        .clicked()
                    {}

                    ui.horizontal(|ui| {
                        ui.add(
                            egui::DragValue::f64(&mut state.simulation_speed)
                                .speed(0.001)
                                .suffix("x time scale"),
                        );
                    });

                    ui.horizontal(|ui| {
                        ui.add(
                            egui::DragValue::f64(&mut state.move_speed)
                                .speed(0.001)
                                .clamp_range(0.0..=1000000.0)
                                .suffix(" Mm/s velocity"),
                        );
                    });

                    if ui
                        .add(
                            egui::Button::new(format!("{} bodies", num_bodies))
                                .text_style(egui::TextStyle::Monospace),
                        )
                        .clicked()
                    {
                        self.open_windows.scene_hierarchy = !self.open_windows.scene_hierarchy;
                    }

                    ui.horizontal(|ui| {
                        let ref_name = match scene_man.get_main_scene() {
                            Some(scene) => match state.camera.reference_entity {
                                Some(reference) => {
                                    Some(scene.get_entity_name(reference).unwrap_or_default())
                                }
                                None => None,
                            },
                            None => None,
                        };

                        let mut style = ui.ctx().style().deref().clone();
                        style.visuals.widgets.noninteractive.bg_stroke.width = 1.0;

                        style.spacing.window_padding = egui::vec2(0.0, 0.0);

                        style.visuals.widgets.noninteractive.bg_stroke.width = 0.0;
                        style.visuals.widgets.inactive.bg_stroke.width = 0.0;

                        style.visuals.window_shadow.extrusion = 0.0;

                        let mut label_color = match ref_name {
                            Some(_) => egui::Color32::from_rgba_unmultiplied(0, 80, 80, 200),
                            None => style.visuals.widgets.inactive.bg_fill,
                        };

                        let mut text_color = style.visuals.widgets.inactive.fg_stroke.color;

                        let mut button_color = egui::Color32::from_rgba_unmultiplied(
                            ((label_color.r() as f32 * 2.0).round()).clamp(0.0, 255.0) as u8,
                            ((label_color.g() as f32 * 2.0).round()).clamp(0.0, 255.0) as u8,
                            ((label_color.b() as f32 * 2.0).round()).clamp(0.0, 255.0) as u8,
                            200,
                        );

                        // Make focusing button red to indicate the reason we won't be orbiting if we have
                        // alt+click without the target
                        if ref_name.is_none()
                            && state.input.modifiers.alt
                            && state.input.m0 != ButtonState::Depressed
                        {
                            button_color = egui::Color32::from_rgba_unmultiplied(255, 0, 0, 125);
                            label_color = egui::Color32::from_rgba_unmultiplied(255, 0, 0, 25);
                            text_color = egui::Color32::BLACK;
                        }

                        style.visuals.widgets.noninteractive.bg_fill = label_color;
                        style.visuals.widgets.inactive.bg_fill = button_color;

                        egui::Frame::popup(&style.clone()).show(ui, |ui| {
                            ui.label("");

                            ui.add(
                                egui::Label::new(ref_name.unwrap_or("No focus"))
                                    .text_style(egui::TextStyle::Monospace)
                                    .text_color(style.visuals.widgets.inactive.fg_stroke.color),
                            );

                            style.visuals.widgets.noninteractive =
                                old_style.visuals.widgets.noninteractive;
                            style.spacing.window_padding = old_style.spacing.window_padding;
                            ui.set_style(style.clone());
                            ui.ctx().set_style(style.clone());

                            if ui
                                .add(
                                    egui::Button::new("❌")
                                        .enabled(state.camera.reference_entity.is_some())
                                        .text_color(text_color),
                                )
                                .on_hover_text("Stop focusing this body")
                                .clicked()
                            {
                                state.camera.next_reference_entity = Some(ReferenceChange::Clear);
                            }
                        });
                    });
                });
            });

            ui.ctx().set_style(old_style);
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

        self.draw_about_window();
        self.draw_controls_window();
        self.draw_settings_window(state);
        self.draw_scene_browser(state, scene_man, res_man);
    }

    fn draw_settings_window(&mut self, state: &mut AppState) {
        UICTX.with(|ui| {
            let ref_mut = ui.borrow_mut();
            let ui = ref_mut.as_ref().unwrap();

            egui::Window::new("Settings")
                .open(&mut self.open_windows.settings)
                .resizable(false)
                .show(&ui.ctx(), |ui| {
                    egui::Grid::new("settings").show(ui, |ui| {
                        ui.label("Vertical FOV:");
                        ui.add(
                            egui::Slider::f64(&mut state.camera.fov_v, 0.0..=120.0)
                                .text("degrees")
                                .integer(),
                        );
                        ui.end_row();

                        ui.label("Rotation sensitivity:");
                        ui.add(egui::Slider::f64(&mut state.rotate_speed, 0.0..=10.0).text(""));
                        ui.end_row();

                        ui.label("Framerate limit:");
                        ui.add(
                            egui::Slider::f64(&mut state.frames_per_second_limit, 0.5..=120.0)
                                .text(""),
                        );
                        ui.end_row();

                        ui.label("Use skyboxes:");
                        ui.checkbox(&mut state.use_skyboxes, "");
                        ui.end_row();

                        ui.label("Show ecliptic grid:");
                        ui.checkbox(&mut state.show_grid, "");
                        ui.end_row();

                        ui.label("Show coordinate axes:");
                        ui.checkbox(&mut state.show_axes, "");
                        ui.end_row();

                        ui.label("Light intensity multiplier:");
                        ui.add(egui::Slider::f32(&mut state.light_intensity, 0.0..=5.0).text(""));
                        ui.end_row();
                    });
                });
        });
    }

    fn draw_pop_ups(&mut self, state: &mut AppState, scene_man: &mut SceneManager) {
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

                let mut entity_to_focus: Option<ReferenceChange> = None;
                let mut entity_to_go_to: Option<Entity> = None;

                egui::Window::new(name)
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
                                let mut style = ui.ctx().style().deref().clone();
                                let old_bg_fill = style.visuals.widgets.inactive.bg_fill;
                                style.visuals.widgets.inactive.bg_fill =
                                    egui::Color32::from_rgba_unmultiplied(0, 160, 160, 200);
                                ui.set_style(style.clone());

                                let but_res = ui.button("❌").on_hover_text("Stop focusing");
                                if but_res.clicked() {
                                    entity_to_focus = Some(ReferenceChange::Clear);
                                }

                                style.visuals.widgets.inactive.bg_fill = old_bg_fill;
                                ui.set_style(style);
                            } else {
                                let but_res = ui.button("🎥").on_hover_text("Focus");
                                if but_res.clicked() {
                                    entity_to_focus =
                                        Some(ReferenceChange::FocusKeepLocation(*selected_entity));
                                }
                            }

                            let but_res = ui.button("🔍").on_hover_text("Go to");
                            if but_res.clicked() {
                                entity_to_go_to = Some(*selected_entity);
                            }
                        });
                    })
                    .unwrap();

                if let Some(ent) = entity_to_focus {
                    state.camera.next_reference_entity = Some(ent);
                }

                if let Some(ent) = entity_to_go_to {
                    state.camera.entity_going_to = Some(ent);
                }
            }

            if let Some(hovered) = state.hovered {
                if state.selection != state.hovered {
                    if let Some(name) = scene.get_entity_name(hovered) {
                        egui::Window::new(name)
                            // It's important to offset the tooltip here so that egui doesn't
                            // claim the pointer is over any UI. Also without this the cursor is on
                            // top of the text anyway
                            .fixed_pos(egui::Pos2 {
                                x: state.input.mouse_x as f32 + 10.0,
                                y: state.input.mouse_y as f32 + 10.0,
                            })
                            .resizable(false)
                            .scroll(false)
                            .collapsible(false)
                            .title_bar(false)
                            .show(&ui.ctx(), |ui| {
                                ui.label(name);
                            })
                            .unwrap();
                    }
                }
            }
        });
    }

    fn draw_debug_window(&mut self, state: &mut AppState, scene_man: &mut SceneManager) {
        UICTX.with(|ui| {
            let ref_mut = ui.borrow_mut();
            let ui = ref_mut.as_ref().unwrap();

            let frame_rate = self.last_frame_rate;

            egui::Window::new("Debug")
                .open(&mut self.open_windows.debug)
                .show(&ui.ctx(), |ui| {
                    ui.columns(2, |cols| {
                        cols[0].label("Simulation time since reference:");
                        cols[1].label(format!("{:.2} seconds", state.sim_time_s))
                    });

                    ui.columns(2, |cols| {
                        cols[0].label("Simulation date:");
                        cols[1].label(format!(
                            "{}",
                            julian_date_number_to_date(Jdn(
                                state.sim_time_s / 86400.0 + J2000_JDN.0
                            ))
                        ))
                    });

                    ui.columns(2, |cols| {
                        cols[0].label("Real time since start:");
                        cols[1].label(format!("{:.2} s", state.real_time_s))
                    });

                    ui.columns(2, |cols| {
                        cols[0].label("Frames per second:");
                        cols[1].label(format!("{:.2}", frame_rate))
                    });

                    ui.columns(2, |cols| {
                        cols[0].label("Simulation scale:");

                        cols[1].add(egui::DragValue::f64(&mut state.simulation_speed).speed(0.01))
                    });

                    ui.separator();

                    ui.columns(2, |cols| {
                        cols[0].label("Light intensity exponent:");
                        cols[1].add(
                            egui::DragValue::f32(&mut state.light_intensity)
                                .clamp_range(-1000.0..=1000.0)
                                .speed(0.01),
                        )
                    });

                    ui.separator();

                    ui.columns(2, |cols| {
                        cols[0].label("Vertical FOV [deg]:");
                        cols[1].add(
                            egui::DragValue::f64(&mut state.camera.fov_v)
                                .clamp_range(0.1..=120.0)
                                .speed(0.5),
                        )
                    });

                    ui.columns(2, |cols| {
                        cols[0].label("Near [Mm]:");
                        cols[1].add(egui::DragValue::f64(&mut state.camera.near).speed(0.01))
                    });

                    ui.columns(2, |cols| {
                        cols[0].label("Far [Mm]:");
                        cols[1].add(egui::DragValue::f64(&mut state.camera.far))
                    });

                    ui.columns(2, |cols| {
                        cols[0].label("Camera pos [Mm]:");
                        cols[1].horizontal(|ui| {
                            ui.add(egui::DragValue::f64(&mut state.camera.pos.x).prefix("x: "));
                            ui.add(egui::DragValue::f64(&mut state.camera.pos.y).prefix("y: "));
                            ui.add(egui::DragValue::f64(&mut state.camera.pos.z).prefix("z: "));
                        });
                    });

                    if let Some(scene) = scene_man.get_main_scene_mut() {
                        ui.columns(2, |cols| {
                            cols[0].label("Reference:");

                            if let Some(reference) = state.camera.reference_entity {
                                cols[1].horizontal(|ui| {
                                    ui.label(format!(
                                        "{:?}: {}",
                                        reference,
                                        scene.get_entity_name(reference).unwrap_or_default()
                                    ));

                                    let clear_resp =
                                        ui.button("🗑").on_hover_text("Stop focusing this entity");
                                    if clear_resp.clicked() {
                                        state.camera.next_reference_entity =
                                            Some(ReferenceChange::Clear);
                                    }
                                });
                            };
                        });
                    };

                    ui.separator();

                    ui.columns(2, |cols| {
                        cols[0].label("Move speed [???]:");
                        cols[1].add(
                            egui::DragValue::f64(&mut state.move_speed)
                                .clamp_range(1.0..=1000.0)
                                .speed(0.1),
                        )
                    });

                    ui.columns(2, |cols| {
                        cols[0].label("Rotation speed:");
                        cols[1].add(
                            egui::DragValue::f64(&mut state.rotate_speed)
                                .clamp_range(1.0..=10.0)
                                .speed(0.1),
                        )
                    });

                    if let Some(selection) = state.selection.iter().next().cloned() {
                        if let Some(scene) = scene_man.get_main_scene_mut() {
                            ui.separator();

                            ui.columns(2, |cols| {
                                cols[0].label("Selected entity:");
                                cols[1].horizontal(|ui| {
                                    ui.label(format!("{:?}", selection));
                                    let but_res =
                                        ui.button("🎥").on_hover_text("Focus this entity");
                                    if but_res.clicked() {
                                        state.camera.next_reference_entity =
                                            Some(ReferenceChange::FocusKeepLocation(selection));
                                    }
                                })
                            });

                            ui.columns(2, |cols| {
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

                            if let Some(comp) =
                                scene.get_component_mut::<MetadataComponent>(selection)
                            {
                                ui.collapsing("Metadata component", |ui| comp.draw_details_ui(ui));
                            }
                        }
                    }
                });
        });
    }

    fn draw_about_window(&mut self) {
        UICTX.with(|ui| {
            let ref_mut = ui.borrow_mut();
            let ui = ref_mut.as_ref().unwrap();

            egui::Window::new("About")
                .open(&mut self.open_windows.about)
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
        });
    }

    fn draw_controls_window(&mut self) {
        UICTX.with(|ui| {
            let ref_mut = ui.borrow_mut();
            let ui = ref_mut.as_ref().unwrap();

            egui::Window::new("Controls")
                .open(&mut self.open_windows.controls)
                .resizable(false)
                .show(&ui.ctx(), |ui| {
                    egui::Grid::new("controls").show(ui, |ui| {
                        ui.label("Movement:");
                        ui.label("WASDQE, Arrow keys");
                        ui.end_row();

                        ui.label("Select objects");
                        ui.label("Left click");
                        ui.end_row();

                        ui.label("Rotate camera");
                        ui.label("Right-click and drag");
                        ui.end_row();

                        ui.label("Speed up or down");
                        ui.label("Mouse-wheel");
                        ui.end_row();

                        ui.label("Play/pause simulation");
                        ui.label("Spacebar");
                        ui.end_row();

                        ui.label("Focus selected object");
                        ui.label("F");
                        ui.end_row();

                        ui.label("Stop focusing object");
                        ui.label("Esc");
                        ui.end_row();

                        ui.label("Go to selected object");
                        ui.label("G");
                        ui.end_row();

                        ui.label("Orbit focused object");
                        ui.label("Alt + Left-click drag");
                        ui.end_row();

                        ui.label("Zoom to focused object");
                        ui.label("Alt + Mouse-wheel");
                        ui.end_row();
                    });
                });
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

            egui::Window::new("Scene browser")
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
                                for (name, _) in scene_man.descriptions.iter() {
                                    ui.radio_value(
                                        &mut self.selected_scene_desc_name,
                                        name.to_owned(),
                                        {
                                            if name == &main_name {
                                                name.to_owned() + " (active)"
                                            } else {
                                                name.to_owned()
                                            }
                                        },
                                    );
                                }
                            });
                        });

                        let selected_is_active =
                            match scene_man.descriptions.get(&self.selected_scene_desc_name) {
                                Some(desc) => &desc.name == &main_name,
                                None => false,
                            };

                        // HACK: 32.0 is the height of the button. I have no idea how to do this programmatically
                        // Maybe with a bottom up layout, but there is some type of egui crash when I put a ScrollArea
                        // inside another layout, and the ScrollArea spawns a scrollbar
                        let height = cols[1].available_size().y - 32.0;

                        egui::ScrollArea::from_max_height(height).show(&mut cols[1], |ui| {
                            ui.set_min_height(height);

                            if let Some(desc) =
                                scene_man.descriptions.get(&self.selected_scene_desc_name)
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
                                    cols[0].label("Focus");
                                    cols[1].label(
                                        desc.focus.as_ref().unwrap_or(&String::from("No focus")),
                                    );
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
                                if selected_is_active {
                                    if ui.button("   Reset   ").clicked() {
                                        // HACK
                                        scene_man.set_scene("empty", res_man, state);
                                        scene_man.set_scene(
                                            &self.selected_scene_desc_name,
                                            res_man,
                                            state,
                                        );
                                    }
                                } else {
                                    if ui.button("   Open   ").clicked() {
                                        scene_man.set_scene(
                                            &self.selected_scene_desc_name,
                                            res_man,
                                            state,
                                        );
                                    }
                                }
                            });

                            cols[1].with_layout(egui::Layout::left_to_right(), |ui| {
                                if ui
                                    .add(
                                        egui::Button::new("   Close   ")
                                            .enabled(selected_is_active),
                                    )
                                    .on_hover_text("Close this scene")
                                    .clicked()
                                {
                                    scene_man.set_scene("empty", res_man, state);
                                }
                            });
                        });
                    });
                });

            self.open_windows.scene_browser = open_window;
        });
    }

    fn draw_scene_hierarchy_window(&mut self, state: &mut AppState, scene: &Scene) {
        UICTX.with(|ui| {
            let ref_mut = ui.borrow_mut();
            let ui = ref_mut.as_ref().unwrap();

            egui::Window::new("Scene hierarchy")
                .open(&mut self.open_windows.scene_hierarchy)
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
                                    if ui.button(name).clicked() {
                                        state.selection = Some(entity.current);
                                    }
                                }
                            }
                        });
                    });
                });
        });
    }
}

fn handle_pointer_on_scene(state: &mut AppState, scene: &mut Scene) {
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

    let entity =
        raycast(&ray, &scene).and_then(|hit| scene.get_entity_from_index(hit.entity_index));

    let window = web_sys::window().unwrap();
    let doc = window.document().unwrap();
    if let None = doc.pointer_lock_element() {
        if state.input.m0 == ButtonState::Pressed {
            state.selection = entity;
        } else {
            state.hovered = entity;
        }
    }
}
