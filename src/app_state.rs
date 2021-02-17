use crate::managers::scene::Entity;
use crate::utils::web::{local_storage_get, local_storage_set};
use na::*;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Serialize, Deserialize, PartialEq, Clone, Copy)]
pub enum SimulationScale {
    Seconds,
    Days,
    Years,
}
impl SimulationScale {
    pub fn to_str(&self) -> &str {
        match *self {
            SimulationScale::Seconds => "seconds / real second",
            SimulationScale::Days => "days / real second",
            SimulationScale::Years => "years / real second",
        }
    }
}

#[derive(Serialize, Deserialize)]
pub enum ReferenceChange {
    NewEntity(Entity),
    Clear,
}

#[derive(Serialize, Deserialize)]
pub struct Camera {
    pub pos: Point3<f64>,
    pub up: Unit<Vector3<f64>>,
    pub target: Point3<f64>,
    pub fov_v: f64,
    pub near: f64,
    pub far: f64,

    #[serde(skip)]
    pub reference_entity: Option<Entity>, // If this is Some, our pos/up/target are wrt. reference_translation
    #[serde(skip)]
    pub reference_translation: Option<Vector3<f64>>,

    // When we want to change reference, we set the new one here.
    // The transform update system will fixup our pos/up/target to be wrt. to it and move it to reference_entity (setting this
    // to None when done). We need this because our transforms are only finalized (due to physics and stuff) after it runs
    #[serde(skip)]
    pub next_reference_entity: Option<ReferenceChange>,

    // Calculated once per frame after inputs are accounted for
    #[serde(skip)]
    pub v: Matrix4<f64>,
    #[serde(skip)]
    pub p: Matrix4<f64>,
    #[serde(skip)]
    pub v_inv: Matrix4<f64>,
    #[serde(skip)]
    pub p_inv: Matrix4<f64>,
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

    #[serde(skip)]
    pub modifiers: egui::Modifiers, // We can use this for the rest of the app too

    #[serde(skip)]
    pub egui_keys: Vec<egui::Event>, // Mostly for typing into UI
}

#[derive(Serialize, Deserialize)]
pub struct AppState {
    #[serde(skip)]
    pub pending_reset: bool, // Whether we want the state to be reset to default the next possible time

    #[serde(skip)]
    pub canvas_height: u32,

    #[serde(skip)]
    pub canvas_width: u32,

    #[serde(skip)]
    pub start_s: f64,

    #[serde(skip)]
    pub last_frame_s: f64,

    #[serde(skip)]
    pub sim_time_days: f64,

    #[serde(skip)]
    pub real_time_s: f64,

    #[serde(skip)]
    pub sim_delta_time_days: f64, // Affected by simulation speed

    #[serde(skip)]
    pub real_delta_time_s: f64, // Not affected by simulation speed

    #[serde(skip)]
    pub time_of_last_save: f64,

    pub simulation_scale: SimulationScale,
    pub simulation_speed: f64,
    pub simulation_paused: bool,
    pub move_speed: f64,
    pub rotate_speed: f64,
    pub light_intensity: f32,

    #[serde(skip)]
    pub input: Input,

    #[serde(skip)]
    pub hovered: HashSet<Entity>,
    pub selection: HashSet<Entity>,
    pub camera: Camera,
}
impl AppState {
    pub fn new() -> Self {
        Self {
            pending_reset: false,
            canvas_height: 0,
            canvas_width: 0,
            start_s: js_sys::Date::now(),
            last_frame_s: 0.0,
            sim_time_days: 0.,
            real_time_s: 0.,
            sim_delta_time_days: 0.,
            real_delta_time_s: 0.,
            time_of_last_save: 0.,
            simulation_scale: SimulationScale::Seconds,
            simulation_speed: 1. / 86400.0,
            simulation_paused: true,
            move_speed: 5.0,
            rotate_speed: 2.0,
            light_intensity: 1.0,
            input: Input::default(),
            hovered: HashSet::new(),
            selection: HashSet::new(),
            camera: Camera {
                pos: Point3::new(10.0, 10.0, 10.0),
                up: Unit::new_unchecked(Vector3::z()),
                target: Point3::new(0.0, 0.0, 0.0),
                fov_v: 60.0,
                near: 1.0,
                far: 100000000.0,
                reference_entity: None,
                reference_translation: None,
                next_reference_entity: None,
                v: Matrix4::identity(),
                p: Matrix4::identity(),
                v_inv: Matrix4::identity(),
                p_inv: Matrix4::identity(),
            },
        }
    }

    /// Tries fetching our last state from local storage if we can find one.
    /// Just creates a new state otherwise.
    pub fn load_or_new() -> Self {
        if let Some(serialized) = local_storage_get("app_state") {
            match ron::de::from_str::<AppState>(&serialized) {
                Ok(mut state) => {
                    state.start_s = js_sys::Date::now();
                    return state;
                }
                Err(error) => {
                    log::error!(
                        "Error deserializing app state '{}': '{}'",
                        serialized,
                        error
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
