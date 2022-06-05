use crate::managers::scene::Entity;
use crate::utils::camera::Camera;
use crate::utils::log::*;
use crate::utils::web::{local_storage_get, local_storage_set};
use na::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct OpenWindows {
    pub debug: bool,
    pub body_list: bool,
    pub scene_browser: bool,
    pub settings: bool,
    pub controls: bool,
    pub about: bool,
}

impl Default for OpenWindows {
    fn default() -> Self {
        Self {
            debug: false,
            body_list: true,
            scene_browser: true,
            settings: true,
            controls: true,
            about: false,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ButtonState {
    Depressed,
    Pressed,
    Handled,
}
impl Default for ButtonState {
    fn default() -> Self {
        ButtonState::Depressed
    }
}

#[derive(Serialize, Deserialize)]
pub enum ReferenceChange {
    FocusKeepLocation(Entity),
    FocusKeepCoords(Entity),
    Clear,
}

#[derive(Serialize, Deserialize, Default)]
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
    pub f: ButtonState,
    pub g: ButtonState,
    pub esc: ButtonState,

    #[serde(skip)]
    pub modifiers: egui::Modifiers, // We can use this for the rest of the app too

    #[serde(skip)]
    pub egui_events: Vec<egui::Event>,
}

#[derive(Serialize, Deserialize)]
pub struct AppState {
    #[serde(skip)]
    pub pending_reset: bool, // Whether we want the state to be reset to default the next possible time

    // Width and height of the canvas on the screen (clientWidth/clientHeight), not the resolution of the canvas,
    // although we force these to be the same for now anyway
    #[serde(skip)]
    pub canvas_height: u32,
    #[serde(skip)]
    pub canvas_width: u32,

    #[serde(skip)]
    pub start_date: f64,

    #[serde(skip)]
    pub last_frame_s: f64,

    #[serde(skip)]
    pub sim_time_s: f64,

    #[serde(skip)]
    pub real_time_s: f64,

    #[serde(skip)]
    pub sim_delta_time_s: f64, // Affected by simulation speed

    #[serde(skip)]
    pub real_delta_time_s: f64, // Not affected by simulation speed

    #[serde(skip)]
    pub time_of_last_save: f64,

    pub use_skyboxes: bool,
    pub show_grid: bool,
    pub show_axes: bool,
    pub show_points: bool,

    pub pixels_per_point: f32,

    pub frames_per_second_limit: f64,
    pub simulation_speed: f64,
    pub simulation_paused: bool,
    pub move_speed: f64,
    pub rotate_speed: f64,
    pub ev100: f32,

    #[serde(skip)]
    pub input: Input,

    #[serde(skip)]
    pub hovered: Option<Entity>,
    pub selection: Option<Entity>,
    pub camera: Camera,

    #[serde(skip)]
    pub reference_entity: Option<Entity>, // If this is Some, our pos/up/target are wrt. reference_translation
    pub reference_entity_name: Option<String>, // Only used when serializing reference_entity, as actual Entity ids are not expected to be consistent

    #[serde(skip)]
    pub reference_translation: Option<Vector3<f64>>,

    // When we want to change reference, we set the new one here.
    // The transform update system will fixup our pos/up/target to be wrt. to it and move it to reference_entity (setting this
    // to None when done). We need this because our transforms are only finalized (due to physics and stuff) after it runs
    #[serde(skip)]
    pub next_reference_entity: Option<ReferenceChange>,

    // Entity we're currently lerping/pointing to
    #[serde(skip)]
    pub entity_going_to: Option<Entity>,

    pub open_windows: OpenWindows,
    pub last_scene_identifier: String,
}
impl AppState {
    pub fn new() -> Self {
        Self {
            pending_reset: false,
            canvas_height: 0,
            canvas_width: 0,
            start_date: js_sys::Date::now() / 1000.0,
            last_frame_s: 0.0,
            sim_time_s: 0.,
            real_time_s: 0.,
            sim_delta_time_s: 0.,
            real_delta_time_s: 0.,
            time_of_last_save: 0.,
            use_skyboxes: false,
            show_grid: false,
            show_axes: false,
            show_points: true,
            pixels_per_point: 1.0,
            simulation_speed: 1.0,
            simulation_paused: true,
            move_speed: 5.0,
            rotate_speed: 2.0,
            frames_per_second_limit: 120.0,
            ev100: 11.0,
            input: Input::default(),
            hovered: None,
            selection: None,
            camera: Camera {
                pos: Point3::new(10.0, 10.0, 10.0),
                up: Unit::new_unchecked(Vector3::z()),
                target: Point3::new(0.0, 0.0, 0.0),
                fov_v: 60.0,
                near: 0.01,
                far: 100000000.0,
                v: Matrix4::identity(),
                p: Matrix4::identity(),
                v_inv: Matrix4::identity(),
                p_inv: Matrix4::identity(),
            },
            reference_entity: None,
            reference_entity_name: None,
            reference_translation: None,
            next_reference_entity: None,
            entity_going_to: None,
            open_windows: Default::default(),
            last_scene_identifier: String::new(),
        }
    }

    /// Tries fetching our last state from local storage if we can find one.
    /// Just creates a new state otherwise.
    pub fn load_or_new() -> Self {
        if let Some(serialized) = local_storage_get("app_state") {
            match ron::de::from_str::<AppState>(&serialized) {
                Ok(mut state) => {
                    state.start_date = js_sys::Date::now() / 1000.0;
                    return state;
                }
                Err(error) => {
                    error!(
                        LogCat::Io,
                        "Error deserializing app state '{}': '{}'", serialized, error
                    );
                }
            }
        };

        return Self::new();
    }

    pub fn save(&self) {
        let serialized = ron::ser::to_string_pretty(&self, ron::ser::PrettyConfig::new()).unwrap();
        local_storage_set("app_state", &serialized);
    }
}
