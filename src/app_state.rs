use std::{
    collections::HashSet,
    sync::{Arc, Mutex},
};

use egui::Ui;
use web_sys::WebGl2RenderingContext;

use crate::managers::Entity;

pub struct Camera {
    pub pos: cgmath::Point3<f32>,
    pub up: cgmath::Vector3<f32>,
    pub target: cgmath::Point3<f32>,
    pub fov_v: cgmath::Deg<f32>,
    pub near: f32,
    pub far: f32,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum ButtonState {
    Depressed,
    Pressed,
    Handled,
}

pub struct Input {
    pub mouse_x: i32,  // Canvas pixels, (0,0) on top left
    pub mouse_y: i32,  // Canvas pixels, (0,0) on top left
    pub delta_x: i32,  // Since last frame
    pub delta_y: i32,  // Since last frame
    pub over_ui: bool, // Prevents interaction with the scene
    pub m0: ButtonState,
    pub m1: ButtonState,
    pub forward: ButtonState,
    pub left: ButtonState,
    pub right: ButtonState,
    pub back: ButtonState,
    pub up: ButtonState,
    pub down: ButtonState,
}

pub struct AppState {
    pub canvas_height: u32,
    pub canvas_width: u32,
    pub phys_time_ms: f64,
    pub real_time_ms: f64,
    pub phys_delta_time_ms: f64, // Affected by simulation speed
    pub real_delta_time_ms: f64, // Not affected by simulation speed
    pub simulation_speed: f64,
    pub move_speed: f32,
    pub rotate_speed: f32,
    pub light_intensity: f32,
    pub input: Input,
    pub selection: HashSet<Entity>,
    pub camera: Camera,
    pub gl: Option<WebGl2RenderingContext>,
    pub ui: Option<Ui>,
}
impl AppState {
    pub fn new() -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(Self {
            canvas_height: 0,
            canvas_width: 0,
            phys_time_ms: 0.,
            real_time_ms: 0.,
            phys_delta_time_ms: 0.,
            real_delta_time_ms: 0.,
            simulation_speed: 1.,
            move_speed: 5.0,
            rotate_speed: 5.0,
            light_intensity: 0.35,
            input: Input {
                mouse_x: 0,
                mouse_y: 0,
                delta_x: 0,
                delta_y: 0,
                over_ui: false,
                m0: ButtonState::Depressed,
                m1: ButtonState::Depressed,
                forward: ButtonState::Depressed,
                left: ButtonState::Depressed,
                right: ButtonState::Depressed,
                back: ButtonState::Depressed,
                up: ButtonState::Depressed,
                down: ButtonState::Depressed,
            },
            selection: HashSet::new(),
            camera: Camera {
                pos: cgmath::Point3::new(-4.0, -7.0, 8.0),
                up: cgmath::Vector3::new(0.0, 0.0, 1.0),
                target: cgmath::Point3::new(0.0, 0.0, 0.0),
                fov_v: cgmath::Deg(60.0),
                near: 0.1,
                far: 100000.0,
            },
            gl: None,
            ui: None,
        }))
    }
}
