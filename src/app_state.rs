use std::sync::Arc;
use std::sync::Mutex;

lazy_static! {
    static ref APP_STATE: Mutex<Arc<AppState>> = Mutex::new(Arc::new(AppState::new()));
}

pub fn update_dynamic_data(time: f32, canvas_height: f32, canvas_width: f32) {
    let mut data = APP_STATE.lock().unwrap();

    *data = Arc::new(AppState {
        canvas_height: canvas_height,
        canvas_width: canvas_width,

        time: time,
        ..*data.clone()
    });
}

pub fn get_curr_state() -> Arc<AppState> {
    APP_STATE.lock().unwrap().clone()
}

#[derive(Debug)]
pub struct AppState {
    pub canvas_height: f32,
    pub canvas_width: f32,
    pub mouse_down: bool,
    pub mouse_x: f32,
    pub mouse_y: f32,
    pub rotation_x_axis: f32,
    pub rotation_y_axis: f32,
    pub time: f32,
}

impl AppState {
    fn new() -> Self {
        Self {
            canvas_height: 0., 
            canvas_width: 0.,
            mouse_down: false,
            mouse_x: -1.,
            mouse_y: -1.,
            rotation_x_axis: -0.5,
            rotation_y_axis: -0.5,
            time: 0.,
        }
    }
}

pub fn update_mouse_down(x: f32, y: f32, is_down: bool) {
    let mut data = APP_STATE.lock().unwrap();
    *data = Arc::new(AppState {
        mouse_down: is_down,
        mouse_x: x,
        mouse_y: data.canvas_height - y,
        ..*data.clone()
    });
}

pub fn update_mouse_position(x: f32, y: f32) {
    let mut data = APP_STATE.lock().unwrap();
    let inverted_y = data.canvas_height - y;
    let x_delta = x - data.mouse_x;
    let y_delta = inverted_y - data.mouse_y;
    let rotation_x_delta = if data.mouse_down {
        std::f32::consts::PI * y_delta / data.canvas_height
    } else {
        0.
    };
    let rotation_y_delta = if data.mouse_down {
        std::f32::consts::PI * x_delta / data.canvas_width
    } else {
        0.
    };

    *data = Arc::new(AppState {
        mouse_x: x,
        mouse_y: inverted_y,
        rotation_x_axis: data.rotation_x_axis + rotation_x_delta,
        rotation_y_axis: data.rotation_y_axis - rotation_y_delta,
        ..*data.clone()
    });
}