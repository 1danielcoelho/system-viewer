use crate::app_state::{AppState, ButtonState, ReferenceChange};
use crate::components::{MeshComponent, MetadataComponent, RigidBodyComponent, TransformComponent};
use crate::managers::details_ui::DetailsUI;
use crate::managers::scene::component_storage::ComponentStorage;
use crate::managers::scene::{Entity, Scene, SceneManager};
use crate::managers::{OrbitManager, ResourceManager};
use crate::utils::log::*;
use crate::utils::raycasting::{raycast, Ray};
use crate::utils::units::{julian_date_number_to_date, Jdn, J2000_JDN};
use crate::utils::web::{
    get_document, is_local_storage_enabled, local_storage_clear, local_storage_enable,
    local_storage_get,
};
use crate::{GLCTX, UICTX};
use egui::Widget;
use lazy_static::__Deref;
use na::*;
use std::collections::VecDeque;

const DEBUG: bool = true;

pub struct InterfaceManager {
    painter: egui_glow::Painter,

    selected_scene_desc_name: String,
    body_list_filter: String,

    frame_times: VecDeque<f64>,
    time_of_last_update: f64,
    last_frame_rate: f64,

    local_storage_ok: bool,
}
impl InterfaceManager {
    pub fn new() -> Self {
        return GLCTX.with(|glctx| {
            return UICTX.with(|uictx| {
                let new_man = Self {
                    painter: egui_glow::Painter::new(glctx.clone(), None, "").unwrap(),
                    selected_scene_desc_name: String::from(""),
                    body_list_filter: String::from(""),
                    frame_times: vec![16.66; 15].into_iter().collect(),
                    time_of_last_update: -2.0,
                    last_frame_rate: 60.0,
                    local_storage_ok: is_local_storage_enabled(),
                };

                if !new_man.local_storage_ok {
                    local_storage_clear();
                }

                // Egui currently looks like ass on web because the text is too thin and
                // not pixel perfect, so here we make things brighter and thicker to hide it
                let mut visuals = egui::Visuals::dark();
                visuals.collapsing_header_frame = true;
                visuals.override_text_color = Some(egui::Color32::LIGHT_GRAY);
                visuals.widgets.active.bg_stroke.width = 2.0;
                visuals.widgets.active.fg_stroke.width = 2.0;
                visuals.widgets.hovered.bg_stroke.width = 2.0;
                visuals.widgets.hovered.fg_stroke.width = 2.0;
                visuals.widgets.inactive.bg_stroke.width = 2.0;
                visuals.widgets.inactive.fg_stroke.width = 2.0;
                visuals.widgets.noninteractive.bg_stroke.width = 2.0;
                visuals.widgets.noninteractive.fg_stroke.width = 2.0;
                visuals.widgets.open.bg_stroke.width = 2.0;
                visuals.widgets.open.fg_stroke.width = 2.0;
                uictx.set_visuals(visuals);

                // let style: egui::Style = (*uictx.style()).clone();
                // uictx.set_style(style);

                info!(LogCat::Io, "Loading egui state...");
                if new_man.local_storage_ok {
                    if let Some(memory_string) = local_storage_get("egui_memory_json") {
                        if let Ok(memory) = serde_json::from_str(&memory_string) {
                            *uictx.memory() = memory;
                        } else {
                            error!(
                                LogCat::Io,
                                "Failed to load egui state from memory string {}", memory_string
                            );
                        }
                    }
                }

                return new_man;
            });
        });
    }

    /// This runs before all systems, and starts collecting all the UI elements we'll draw, as
    /// well as draws the main UI
    pub fn begin_frame(&mut self, state: &mut AppState) {
        state.input.over_ui = false;

        // Lets fill this in and give it to egui
        let mut new_input = egui::RawInput::default();
        new_input.screen_rect = Some(egui::Rect {
            min: egui::Pos2::new(0.0, 0.0),
            max: egui::Pos2::new(state.canvas_width as f32, state.canvas_height as f32),
        });

        // If we have pointer lock then we don't really want to use the UI (we're rotating/orbiting/etc.)
        // so don't give the updated mouse position to egui
        let doc = get_document();
        if let None = doc.pointer_lock_element() {
            new_input.events.append(&mut state.input.egui_events);
        }
        new_input.modifiers = state.input.modifiers;

        UICTX.with(|uictx| {
            uictx.begin_frame(new_input);

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

            let has_kb: bool = uictx.wants_keyboard_input();
            let egui_consuming_pointer: bool = uictx.wants_pointer_input();

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
        });
    }

    /// This runs after all systems, and draws the collected UI elements to the framebuffer
    pub fn end_frame(
        &mut self,
        state: &mut AppState,
        scene_man: &mut SceneManager,
        res_man: &mut ResourceManager,
        orbit_man: &OrbitManager, // TODO: This shouldn't be done this way, man
    ) {
        self.draw_main_ui(state, scene_man, res_man, orbit_man);

        UICTX.with(|uictx| {
            let output = uictx.end_frame();

            let clipped_primitives = uictx.tessellate(output.shapes);

            // TODO: pixels_per_point
            self.painter.paint_and_update_textures(
                [state.canvas_width, state.canvas_height],
                state.pixels_per_point,
                &clipped_primitives,
                &output.textures_delta,
            );

            // Always reset the hovered entity even if we won't get to
            // hover new ones: This prevents the tooltip from sticking around if the UI
            // starts covering it (which can also include the label tooltip itself)
            state.hovered = None;
            if let Some(scene) = scene_man.get_main_scene_mut() {
                if !uictx.wants_pointer_input() {
                    handle_pointer_on_scene(state, scene);
                }
            }
        });
    }

    fn draw_main_ui(
        &mut self,
        state: &mut AppState,
        scene_man: &mut SceneManager,
        res_man: &mut ResourceManager,
        orbit_man: &OrbitManager,
    ) {
        self.draw_main_toolbar(state, scene_man, res_man, orbit_man);

        self.draw_open_windows(state, scene_man, res_man, orbit_man);

        self.draw_pop_ups(state, scene_man);
    }

    fn draw_main_toolbar(
        &mut self,
        state: &mut AppState,
        scene_man: &mut SceneManager,
        res_man: &mut ResourceManager,
        orbit_man: &OrbitManager,
    ) {
        UICTX.with(|uictx| {
            let old_style = uictx.style().deref().clone();
            let mut style = old_style.clone();

            style.visuals.widgets.noninteractive.bg_fill =
                egui::Color32::from_rgba_unmultiplied(0, 0, 0, 0);
            style.visuals.widgets.noninteractive.bg_stroke.width = 0.0;

            style.visuals.widgets.inactive.bg_fill =
                egui::Color32::from_rgba_unmultiplied(0, 0, 0, 200);
            style.visuals.widgets.inactive.bg_stroke.width = 0.0;

            uictx.set_style(style.clone());

            egui::TopBottomPanel::top(egui::Id::new("top panel")).show(&uictx, |ui| {
                let num_bodies = scene_man
                    .get_main_scene()
                    .unwrap()
                    .rigidbody
                    .get_num_components();

                let sim_date_str = format!(
                    "{}",
                    julian_date_number_to_date(Jdn(state.sim_time_s / 86400.0 + J2000_JDN.0))
                );

                ui.with_layout(egui::Layout::left_to_right(), |ui| {
                    ui.menu_button("⚙", |ui| {
                        egui::Frame::dark_canvas(ui.style())
                            .fill(egui::Color32::from_rgba_unmultiplied(0, 0, 0, 200))
                            .show(ui, |ui| {
                                if ui.button("Reset scene").clicked() {
                                    scene_man.set_scene("empty", res_man, orbit_man, state);
                                    scene_man.set_scene(
                                        &self.selected_scene_desc_name,
                                        res_man,
                                        orbit_man,
                                        state,
                                    );
                                }

                                if ui.button("Close scene").clicked() {
                                    scene_man.set_scene("empty", res_man, orbit_man, state);
                                }

                                ui.separator();

                                if ui.button("Scene browser").clicked() {
                                    state.open_windows.scene_browser =
                                        !state.open_windows.scene_browser;
                                }

                                ui.separator();

                                if ui.button("Settings").clicked() {
                                    state.open_windows.settings = !state.open_windows.settings;
                                }

                                if ui.button("Controls").clicked() {
                                    state.open_windows.controls = !state.open_windows.controls;
                                }

                                if ui.button("About").clicked() {
                                    state.open_windows.about = !state.open_windows.about;
                                }

                                if DEBUG {
                                    ui.separator();

                                    if ui.button("Debug").clicked() {
                                        state.open_windows.debug = !state.open_windows.debug;
                                    }

                                    if ui.button("Organize windows").clicked() {
                                        uictx.memory().reset_areas();
                                    }

                                    if ui.button("Close all windows").clicked() {
                                        state.open_windows.debug = false;
                                        state.open_windows.body_list = false;
                                        state.open_windows.about = false;
                                        state.open_windows.scene_browser = false;
                                        state.open_windows.controls = false;
                                        state.open_windows.settings = false;
                                    }

                                    ui.separator();

                                    if ui
                                        .button("Clear Egui memory")
                                        .on_hover_text("Forget scroll, collapsing headers etc")
                                        .clicked()
                                    {
                                        *uictx.memory() = Default::default();
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
                        .add(egui::Button::new(
                            egui::RichText::new(format!("{:.2} fps", self.last_frame_rate))
                                .monospace(),
                        ))
                        .clicked()
                    {}

                    if ui
                        .add(egui::Button::new(
                            egui::RichText::new(sim_date_str).monospace(),
                        ))
                        .clicked()
                    {}

                    ui.horizontal(|ui| {
                        ui.add(
                            egui::DragValue::new(&mut state.simulation_speed)
                                .speed(1.0)
                                .suffix("x time scale"),
                        );
                    });

                    ui.horizontal(|ui| {
                        ui.add(
                            egui::DragValue::new(&mut state.move_speed)
                                .speed(0.001)
                                .clamp_range::<f64>(0.0..=1000000.0)
                                .suffix(" Mm/s velocity"),
                        );
                    });

                    if ui
                        .add(egui::Button::new(
                            egui::RichText::new(format!("{} bodies", num_bodies)).monospace(),
                        ))
                        .clicked()
                    {
                        state.open_windows.body_list = !state.open_windows.body_list;
                    }

                    ui.horizontal(|ui| {
                        let ref_name = match scene_man.get_main_scene() {
                            Some(scene) => match state.reference_entity {
                                Some(reference) => {
                                    Some(scene.get_entity_name(reference).unwrap_or_default())
                                }
                                None => None,
                            },
                            None => None,
                        };

                        let mut style = uictx.style().deref().clone();
                        style.visuals.widgets.noninteractive.bg_stroke.width = 1.0;

                        style.spacing.window_margin = egui::style::Margin {
                            left: 0.0,
                            right: 0.0,
                            top: 0.0,
                            bottom: 0.0,
                        };

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

                            ui.add(egui::Label::new(
                                egui::RichText::new(ref_name.unwrap_or("No focus"))
                                    .monospace()
                                    .color(style.visuals.widgets.inactive.fg_stroke.color),
                            ));

                            style.visuals.widgets.noninteractive =
                                old_style.visuals.widgets.noninteractive;
                            style.spacing.window_margin = old_style.spacing.window_margin;
                            ui.set_style(style.clone());
                            uictx.set_style(style.clone());

                            if ui
                                .add_enabled(
                                    state.reference_entity.is_some(),
                                    egui::Button::new(egui::RichText::new("❌").color(text_color)),
                                )
                                .on_hover_text("Stop focusing this body")
                                .clicked()
                            {
                                state.next_reference_entity = Some(ReferenceChange::Clear);
                            }
                        });
                    });
                });
            });

            uictx.set_style(old_style);
        });
    }

    fn draw_open_windows(
        &mut self,
        state: &mut AppState,
        scene_man: &mut SceneManager,
        res_man: &mut ResourceManager,
        orbit_man: &OrbitManager,
    ) {
        self.draw_debug_window(state, scene_man);

        if let Some(main_scene) = scene_man.get_main_scene() {
            self.draw_body_list_window(state, main_scene);
        }

        self.draw_about_window(state);
        self.draw_controls_window(state);
        self.draw_settings_window(state);
        self.draw_scene_browser(state, scene_man, res_man, orbit_man);
    }

    fn draw_settings_window(&mut self, state: &mut AppState) {
        UICTX.with(|uictx| {
            let mut open_window = state.open_windows.settings;

            egui::Window::new("Settings")
                .open(&mut open_window)
                .resizable(false)
                .show(&uictx, |ui| {
                    egui::Grid::new("settings").show(ui, |ui| {
                        ui.label("Vertical FOV:");
                        ui.add(
                            egui::Slider::new(&mut state.camera.fov_v, 0.0..=120.0)
                                .text("degrees")
                                .integer(),
                        );
                        ui.end_row();

                        ui.label("Rotation sensitivity:");
                        ui.add(egui::Slider::new(&mut state.rotate_speed, 0.0..=10.0).text(""));
                        ui.end_row();

                        ui.label("Framerate limit:");
                        ui.add(
                            egui::Slider::new(&mut state.frames_per_second_limit, 0.5..=120.0)
                                .text("fps"),
                        );
                        ui.end_row();

                        ui.label("Pixels per point:");
                        ui.add(
                            egui::Slider::new(&mut state.pixels_per_point, 0.1..=10.0)
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

                        ui.label("Show points:");
                        ui.checkbox(&mut state.show_points, "");
                        ui.end_row();

                        ui.label("EV100:");
                        ui.add(egui::Slider::new(&mut state.ev100, -20.0..=20.0).text(""));
                        ui.end_row();

                        ui.label("Allow Local Storage:");
                        if ui.checkbox(&mut self.local_storage_ok, "").on_hover_text("Allow usage of localStorage for storing session data like app state, window state and last loaded scene.").clicked() {

                            if self.local_storage_ok {
                                info!(LogCat::Ui, "Allowing usage of local storage");
                                local_storage_enable();
                            } else {
                                info!(LogCat::Ui, "Stopping usage and clearing local storage");
                                local_storage_clear();
                            }
                        }
                        ui.end_row();
                    });
                });

            state.open_windows.settings = open_window;
        });
    }

    fn draw_pop_ups(&mut self, state: &mut AppState, scene_man: &mut SceneManager) {
        let scene = scene_man.get_main_scene();
        if scene.is_none() {
            return;
        }
        let scene = scene.unwrap();

        UICTX.with(|uictx| {
            let mut cam_pos = state.camera.pos;
            let mut cam_target = state.camera.target;
            if let Some(reference) = state.reference_translation {
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
                // TODO: Use last_drawn_position or whatever it is used to draw points to draw this
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
                    .show(&uictx, |ui| {
                        ui.label(format!("Distance: {:.3} Mm", distance));

                        ui.horizontal(|ui| {
                            if state.reference_entity == Some(*selected_entity) {
                                let mut style = uictx.style().deref().clone();
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
                    state.next_reference_entity = Some(ent);
                }

                if let Some(ent) = entity_to_go_to {
                    state.entity_going_to = Some(ent);
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
                            .collapsible(false)
                            .title_bar(false)
                            .show(&uictx, |ui| {
                                ui.label(name);
                            })
                            .unwrap();
                    }
                }
            }
        });
    }

    fn draw_debug_window(&mut self, state: &mut AppState, scene_man: &mut SceneManager) {
        UICTX.with(|uictx| {
            let frame_rate = self.last_frame_rate;
            let mut open_window = state.open_windows.debug;

            egui::Window::new("Debug")
                .open(&mut open_window)
                .show(&uictx, |ui| {
                    egui::ScrollArea::vertical()
                        .max_height(std::f32::INFINITY)
                        .show(ui, |ui| {
                            egui::Grid::new("controls").show(ui, |ui| {
                                ui.label("Simulation time since reference:");
                                ui.label(format!("{:.2} seconds", state.sim_time_s));
                                ui.end_row();

                                ui.label("Simulation date:");
                                ui.label(format!(
                                    "{}",
                                    julian_date_number_to_date(Jdn(
                                        state.sim_time_s / 86400.0 + J2000_JDN.0
                                    ))
                                ));
                                ui.end_row();

                                ui.label("Real time since start:");
                                ui.label(format!("{:.2} s", state.real_time_s));
                                ui.end_row();

                                ui.label("Frames per second:");
                                ui.label(format!("{:.2}", frame_rate));
                                ui.end_row();

                                ui.label("Simulation scale:");
                                ui.add(
                                    egui::DragValue::new(&mut state.simulation_speed).speed(0.01),
                                );
                                ui.end_row();

                                ui.separator();
                                ui.separator();
                                ui.end_row();

                                ui.label("EV100:");
                                ui.add(
                                    egui::DragValue::new(&mut state.ev100)
                                        .clamp_range(-20.0..=20.0)
                                        .speed(0.01),
                                );
                                ui.end_row();

                                ui.separator();
                                ui.separator();
                                ui.end_row();

                                ui.label("Vertical FOV [deg]:");
                                ui.add(
                                    egui::DragValue::new(&mut state.camera.fov_v)
                                        .clamp_range(0.1..=120.0)
                                        .speed(0.5),
                                );
                                ui.end_row();

                                ui.label("Near [Mm]:");
                                ui.add(egui::DragValue::new(&mut state.camera.near).speed(0.01));
                                ui.end_row();

                                ui.label("Far [Mm]:");
                                ui.add(egui::DragValue::new(&mut state.camera.far));
                                ui.end_row();

                                // Guarantee valid values even if we manually typed garbage in
                                state.camera.near = state.camera.near.max(0.0001);
                                state.camera.far = state.camera.far.max(state.camera.near + 0.0001);

                                ui.label("Camera pos [Mm]:");
                                ui.horizontal(|ui| {
                                    ui.add(
                                        egui::DragValue::new(&mut state.camera.pos.x).prefix("x: "),
                                    );
                                    ui.add(
                                        egui::DragValue::new(&mut state.camera.pos.y).prefix("y: "),
                                    );
                                    ui.add(
                                        egui::DragValue::new(&mut state.camera.pos.z).prefix("z: "),
                                    );
                                });
                                ui.end_row();

                                if let Some(scene) = scene_man.get_main_scene_mut() {
                                    ui.label("Reference:");

                                    if let Some(reference) = state.reference_entity {
                                        ui.horizontal(|ui| {
                                            ui.label(format!(
                                                "{:?}: {}",
                                                reference,
                                                scene
                                                    .get_entity_name(reference)
                                                    .unwrap_or_default()
                                            ));

                                            let clear_resp = ui
                                                .button("🗑")
                                                .on_hover_text("Stop focusing this entity");
                                            if clear_resp.clicked() {
                                                state.next_reference_entity =
                                                    Some(ReferenceChange::Clear);
                                            }
                                        });
                                    };
                                    ui.end_row();
                                };

                                ui.separator();
                                ui.separator();
                                ui.end_row();

                                ui.label("Move speed [???]:");
                                ui.add(
                                    egui::DragValue::new(&mut state.move_speed)
                                        .clamp_range(1.0..=1000.0)
                                        .speed(0.1),
                                );
                                ui.end_row();

                                ui.label("Rotation speed:");
                                ui.add(
                                    egui::DragValue::new(&mut state.rotate_speed)
                                        .clamp_range(1.0..=10.0)
                                        .speed(0.1),
                                );
                                ui.end_row();
                            });

                            ui.separator();

                            if let Some(selection) = state.selection.iter().next().cloned() {
                                if let Some(scene) = scene_man.get_main_scene_mut() {
                                    ui.label("Selected entity:");
                                    ui.horizontal(|ui| {
                                        ui.label(format!("{:?}", selection));
                                        let but_res =
                                            ui.button("🎥").on_hover_text("Focus this entity");
                                        if but_res.clicked() {
                                            state.next_reference_entity =
                                                Some(ReferenceChange::FocusKeepLocation(selection));
                                        }
                                    });
                                    ui.end_row();

                                    ui.label("Name:");
                                    ui.label(format!(
                                        "{}",
                                        scene.get_entity_name(selection).unwrap_or_default()
                                    ));
                                    ui.end_row();

                                    if let Some(children) = scene.get_entity_children(selection) {
                                        if children.len() > 0 {
                                            ui.collapsing("Children", |ui| {
                                                for child in children {
                                                    let but_child = ui.button(
                                                        scene
                                                            .get_entity_name(*child)
                                                            .unwrap_or_default(),
                                                    );
                                                    if but_child.clicked() {
                                                        state.selection = Some(*child);
                                                    }
                                                }
                                            });
                                        }
                                    }
                                    ui.end_row();

                                    // TODO: Make this more generic
                                    if let Some(comp) =
                                        scene.get_component_mut::<TransformComponent>(selection)
                                    {
                                        ui.collapsing("Transform component", |ui| {
                                            comp.draw_details_ui(ui)
                                        });
                                    }

                                    if let Some(comp) =
                                        scene.get_component_mut::<MeshComponent>(selection)
                                    {
                                        ui.collapsing("Mesh component", |ui| {
                                            comp.draw_details_ui(ui)
                                        });
                                    }

                                    if let Some(comp) =
                                        scene.get_component_mut::<RigidBodyComponent>(selection)
                                    {
                                        ui.collapsing("RigidBody component", |ui| {
                                            comp.draw_details_ui(ui)
                                        });
                                    }

                                    if let Some(comp) =
                                        scene.get_component_mut::<MetadataComponent>(selection)
                                    {
                                        ui.collapsing("Metadata component", |ui| {
                                            comp.draw_details_ui(ui)
                                        });
                                    }
                                }
                            }
                        });
                });

            state.open_windows.debug = open_window;
        });
    }

    fn draw_about_window(&mut self, state: &mut AppState) {
        UICTX.with(|uictx| {
            egui::Window::new("About")
                .open(&mut state.open_windows.about)
                .resizable(false)
                .fixed_size(egui::vec2(400.0, 400.0))
                .show(&uictx, |ui| {
                    ui.vertical_centered_justified(|ui|{
                        ui.label("\nSystem Viewer\nv.0.2\nAuthor: Daniel Coelho\n");
                    });
                    ui.label("This is a simple, custom N-body simulation 3D engine written for the web.");
                    ui.label("\nIt uses simple semi-implicit Euler integration to calculate the effect of gravity at each timestep.\nInitial J2000 state vectors were collected from NASA's HORIZONS system and JPL's Small-Body Database Search Engine, and when required evolved to J2000 using the mean orbital elements (e.g. for asteroids).");
                    ui.label("\nIt is fully written in Rust (save for some glue Javascript code), and compiled to WebAssembly via wasm_bindgen, which includes WebGL2 bindings.");
                    ui.label("The 3D engine uses a data-oriented entity component system in order to maximize performance of batch physics calculations, and the Egui immediate mode GUI library, also written in pure Rust.");
                    ui.horizontal_wrapped(|ui| {
                        ui.label("\nProject home page:");
                        ui.hyperlink("https://github.com/1danielcoelho/system-viewer");
                    });
                });
        });
    }

    fn draw_controls_window(&mut self, state: &mut AppState) {
        UICTX.with(|uictx| {
            egui::Window::new("Controls")
                .open(&mut state.open_windows.controls)
                .resizable(false)
                .show(&uictx, |ui| {
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
        orbit_man: &OrbitManager,
    ) {
        UICTX.with(|uictx| {
            let mut open_window = state.open_windows.scene_browser;

            egui::Window::new("Scene browser")
                .open(&mut open_window)
                .resizable(false)
                .fixed_size(egui::vec2(600.0, 300.0))
                .show(&uictx, |ui| {
                    ui.columns(2, |cols| {
                        cols[0].set_min_height(300.0);
                        cols[1].set_min_height(300.0);

                        let main_name = scene_man.get_main_scene().unwrap().identifier.clone();

                        egui::Frame::dark_canvas(cols[0].style())
                            .inner_margin(egui::style::Margin::same(5.0))
                            .show(&mut cols[0], |ui| {
                                ui.set_min_height(ui.available_size().y);

                                egui::ScrollArea::vertical()
                                    .max_height(std::f32::INFINITY)
                                    .show(ui, |ui| {
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

                        egui::ScrollArea::vertical()
                            .max_height(height)
                            .show(&mut cols[1], |ui| {
                                ui.set_min_height(height);

                                egui::Frame::none()
                                    .inner_margin(egui::style::Margin::same(5.0))
                                    .show(ui, |ui| {
                                        if let Some(desc) = scene_man
                                            .descriptions
                                            .get(&self.selected_scene_desc_name)
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
                                                cols[1]
                                                    .label(format!("{}", &desc.simulation_scale));
                                            });

                                            ui.columns(2, |cols| {
                                                cols[0].label("Focus");
                                                cols[1].label(
                                                    desc.focus
                                                        .as_ref()
                                                        .unwrap_or(&String::from("No focus")),
                                                );
                                            });

                                            ui.columns(2, |cols| {
                                                cols[0].label("Bodies:");
                                                cols[1].label(format!("{}", desc.bodies.len()));
                                            });
                                        }
                                    });
                            });

                        cols[1].separator();

                        // TODO: There has to be a simpler way of just centering two buttons on that space
                        cols[1].columns(2, |cols| {
                            cols[0].with_layout(egui::Layout::right_to_left(), |ui| {
                                if selected_is_active {
                                    if ui.button("   Reset   ").clicked() {
                                        // HACK
                                        scene_man.set_scene("empty", res_man, orbit_man, state);
                                        scene_man.set_scene(
                                            &self.selected_scene_desc_name,
                                            res_man,
                                            orbit_man,
                                            state,
                                        );
                                    }
                                } else {
                                    if ui.button("   Open   ").clicked() {
                                        scene_man.set_scene(
                                            &self.selected_scene_desc_name,
                                            res_man,
                                            orbit_man,
                                            state,
                                        );
                                    }
                                }
                            });

                            cols[1].with_layout(egui::Layout::left_to_right(), |ui| {
                                if ui
                                    .add_enabled(
                                        selected_is_active,
                                        egui::Button::new("   Close   "),
                                    )
                                    .on_hover_text("Close this scene")
                                    .clicked()
                                {
                                    scene_man.set_scene("empty", res_man, orbit_man, state);
                                }
                            });
                        });
                    });
                });

            state.open_windows.scene_browser = open_window;
        });
    }

    fn draw_body_list_window(&mut self, state: &mut AppState, scene: &Scene) {
        UICTX.with(|uictx| {
            let mut open_window = state.open_windows.body_list;

            egui::Window::new("Body list")
                .open(&mut open_window)
                .resizable(true)
                .default_size(egui::vec2(300.0, 400.0))
                .show(&uictx, |ui| {
                    ui.set_min_height(300.0);

                    ui.horizontal(|ui| {
                        ui.label("Search: ");
                        egui::TextEdit::singleline(&mut self.body_list_filter)
                            .desired_width(f32::INFINITY)
                            .ui(ui);
                    });

                    let filter_lower = self.body_list_filter.to_lowercase();

                    egui::Frame::dark_canvas(ui.style()).show(ui, |ui| {
                        ui.set_min_height(ui.available_size().y);

                        egui::ScrollArea::vertical()
                            .max_height(std::f32::INFINITY)
                            .auto_shrink([false, false])
                            .show(ui, |ui| {
                                egui::Frame::none()
                                    .inner_margin(egui::style::Margin::same(5.0))
                                    .show(ui, |ui| {
                                        for entity in scene.get_entity_entries() {
                                            if !entity.live {
                                                continue;
                                            }

                                            if scene
                                                .get_component::<RigidBodyComponent>(entity.current)
                                                .is_none()
                                            {
                                                continue;
                                            }

                                            if let Some(name) = &entity.name {
                                                if name.to_lowercase().contains(&filter_lower) {
                                                    if ui.button(name).clicked() {
                                                        state.selection = Some(entity.current);
                                                    }
                                                }
                                            }
                                        }
                                    });
                            });
                    });
                });

            state.open_windows.body_list = open_window;
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
    if let Some(reference) = state.reference_translation {
        start_world += reference;
    }

    let ray = Ray {
        start: start_world,
        direction: (end_world - start_world).normalize(),
    };

    let mut entity =
        raycast(&ray, &scene).and_then(|hit| scene.get_entity_from_index(hit.entity_index));

    // If we hit e.g. Saturn's rings we want to select Saturn itself, as that will contain all the useful stuff
    if let Some(valid_entity) = entity {
        entity = Some(scene.get_entity_ancestor(valid_entity));
    }

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
