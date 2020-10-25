use egui::Ui;
use web_sys::WebGlRenderingContext;

pub struct AppState {
    pub canvas_height: u32,
    pub canvas_width: u32,
    pub mouse_down: bool,
    pub mouse_x: f32,
    pub mouse_y: f32,
    pub time_ms: f64,
    pub delta_time_ms: f64,
    // TODO: Camera data somehow?

    pub gl: Option<WebGlRenderingContext>,
}
impl AppState {
    pub fn new() -> Self {
        Self {
            canvas_height: 0, 
            canvas_width: 0,
            mouse_down: false,
            mouse_x: -1.,
            mouse_y: -1.,
            time_ms: 0.,
            delta_time_ms: 0.,
            gl: None,
        }
    }
}
