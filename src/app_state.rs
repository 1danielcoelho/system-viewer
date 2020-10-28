use std::sync::{Arc, Mutex};

use web_sys::WebGlRenderingContext;

pub struct AppState {
    pub canvas_height: u32,
    pub canvas_width: u32,
    pub mouse_down: bool,
    pub mouse_x: i32,
    pub mouse_y: i32,
    pub time_ms: f64,
    pub delta_time_ms: f64,
    // TODO: Camera data somehow?
    pub gl: Option<WebGlRenderingContext>,
}
impl AppState {
    pub fn new() -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(Self {
            canvas_height: 0,
            canvas_width: 0,
            mouse_down: false,
            mouse_x: 0,
            mouse_y: 0,
            time_ms: 0.,
            delta_time_ms: 0.,
            gl: None,
        }))
    }
}
