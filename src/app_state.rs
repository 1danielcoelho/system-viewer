use crate::managers::scene::Entity;
use na::*;
use std::collections::HashSet;

pub enum ReferenceChange {
    NewEntity(Entity),
    Clear,
}

pub struct Camera {
    pub pos: Point3<f32>,
    pub up: Unit<Vector3<f32>>,
    pub target: Point3<f32>,
    pub fov_v: f32,
    pub near: f32,
    pub far: f32,
    pub reference_entity: Option<Entity>, // If this is Some, our pos/up/target are wrt. the entity's transform

    // When we want to change reference, we set the new one here.
    // The transform update system will fixup our pos/up/target to be wrt. to it and move it to reference_entity (setting this
    // to None when done). We need this because our transforms are only finalized (due to physics and stuff) after it runs
    pub next_reference_entity: Option<ReferenceChange>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum ButtonState {
    Depressed,
    Pressed,
    Handled,
}

pub struct Input {
    pub mouse_x: i32, // Canvas pixels, (0,0) on top left
    pub mouse_y: i32, // Canvas pixels, (0,0) on top left
    pub delta_x: i32, // Since last frame
    pub delta_y: i32, // Since last frame
    pub scroll_delta_x: i32,
    pub scroll_delta_y: i32,
    pub over_ui: bool, // Prevents interaction with the scene
    pub m0: ButtonState,
    pub m1: ButtonState,
    pub forward: ButtonState,
    pub left: ButtonState,
    pub right: ButtonState,
    pub back: ButtonState,
    pub up: ButtonState,
    pub down: ButtonState,
    pub spacebar: ButtonState,

    pub modifiers: egui::Modifiers, // We can use this for the rest of the app too
    pub egui_keys: Vec<egui::Event>, // Mostly for typing into UI
}

pub struct AppState {
    pub canvas_height: u32,
    pub canvas_width: u32,
    pub start_s: f64,
    pub last_frame_s: f64,
    pub sim_time_days: f64,
    pub real_time_s: f64,
    pub sim_delta_time_days: f64, // Affected by simulation speed
    pub real_delta_time_s: f64,   // Not affected by simulation speed
    pub simulation_speed: f64,
    pub simulation_paused: bool, 
    pub move_speed: f32,
    pub rotate_speed: f32,
    pub light_intensity: f32,
    pub input: Input,
    pub selection: HashSet<Entity>,
    pub camera: Camera,
}
impl AppState {
    pub fn new() -> Self {
        Self {
            canvas_height: 0,
            canvas_width: 0,
            start_s: 0.0,
            last_frame_s: 0.0,
            sim_time_days: 0.,
            real_time_s: 0.,
            sim_delta_time_days: 0.,
            real_delta_time_s: 0.,
            simulation_speed: 1.,
            simulation_paused: false,
            move_speed: 10000.0,
            rotate_speed: 5.0,
            light_intensity: 0.35,
            input: Input {
                mouse_x: 0,
                mouse_y: 0,
                delta_x: 0,
                delta_y: 0,
                scroll_delta_x: 0,
                scroll_delta_y: 0,
                over_ui: false,
                m0: ButtonState::Depressed,
                m1: ButtonState::Depressed,
                forward: ButtonState::Depressed,
                left: ButtonState::Depressed,
                right: ButtonState::Depressed,
                back: ButtonState::Depressed,
                up: ButtonState::Depressed,
                down: ButtonState::Depressed,
                spacebar: ButtonState::Depressed,
                modifiers: egui::Modifiers {
                    alt: false,
                    ctrl: false,
                    shift: false,
                    mac_cmd: false,
                    command: false,
                },
                egui_keys: Vec::new(),
            },
            selection: HashSet::new(),
            camera: Camera {
                pos: Point3::new(-34874.89, 144281.34, 614.11206),
                up: Unit::new_unchecked(Vector3::z()),
                target: Point3::new(0.0, 0.0, 0.0),
                fov_v: 60.0,
                near: 5.0,
                far: 100000000.0,
                reference_entity: None,
                next_reference_entity: None,
            },
        }
    }
}
