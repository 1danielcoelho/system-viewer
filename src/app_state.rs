use std::sync::{Arc, Mutex};

use web_sys::WebGlRenderingContext;

pub struct Camera {
    pub pos: cgmath::Point3<f32>,
    pub up: cgmath::Vector3<f32>,
    pub target: cgmath::Point3<f32>,
    pub fov_v: cgmath::Deg<f32>,
    pub near: f32,
    pub far: f32,
}

pub struct Input {
    pub mouse_x: i32,
    pub mouse_y: i32,
    pub delta_x: i32, // since last frame
    pub delta_y: i32, // since last frame
    pub m0_down: bool,
    pub m1_down: bool,
    pub forward_down: bool,
    pub left_down: bool,
    pub right_down: bool,
    pub back_down: bool,
    pub up_down: bool,
    pub down_down: bool, // ugh
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
    pub input: Input,
    pub camera: Camera,
    pub gl: Option<WebGlRenderingContext>,
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
            input: Input {
                mouse_x: 0,
                mouse_y: 0,
                delta_x: 0,
                delta_y: 0,
                m0_down: false,
                m1_down: false,
                forward_down: false,
                left_down: false,
                right_down: false,
                back_down: false,
                up_down: false,
                down_down: false,
            },
            camera: Camera {
                pos: cgmath::Point3::new(0.0, -5.0, 0.0),
                up: cgmath::Vector3::new(0.0, 0.0, 1.0),
                target: cgmath::Point3::new(0.0, 0.0, 0.0),
                fov_v: cgmath::Deg(80.0),
                near: 0.1,
                far: 100000.0,
            },
            gl: None,
        }))
    }
}
