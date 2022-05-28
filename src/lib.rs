// Prevent some weirdness with std::f32::clamp when using use::na::*;
#![allow(unstable_name_collisions)]

#[macro_use(lazy_static)]
extern crate lazy_static;
extern crate nalgebra as na;
extern crate regex;
extern crate ron;
extern crate serde;
extern crate wasm_bindgen;

use crate::app_state::AppState;
use crate::engine::Engine;
use crate::utils::web::{
    get_canvas, get_gl_context, local_storage_remove, request_animation_frame, request_text,
    setup_event_handlers,
};
use egui::Ui;
use futures::future::join_all;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use web_sys::{HtmlCanvasElement, WebGl2RenderingContext};

mod app_state;
mod components;
mod engine;
mod managers;
mod systems;
mod utils;

// Note that even though these are pub, we can't really use them spaghettily from within
// the engine as they're mut borrowed in engine update, so it's not so bad.
// This is mostly used so that we can affect the engine from JS callbacks.
// Also, having the webgl context in here is actually safer, as there is no guarantee two random
// callers pulled it from the canvas at the same time
thread_local! {
    pub static ENGINE: RefCell<Option<Engine>> = RefCell::new(None);
    pub static STATE: RefCell<Option<AppState>> = RefCell::new(None);
    pub static GLCTX: RefCell<Option<WebGl2RenderingContext>> = RefCell::new(None);
    pub static UICTX: RefCell<Option<Ui>> = RefCell::new(None);
}

#[wasm_bindgen(start)]
pub fn main_js() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Debug).expect("Unable to initialize console logging");
}

#[wasm_bindgen]
pub async fn start() -> Result<(), JsValue> {
    log::info!("Initializing state...");
    STATE.with(|s| {
        let mut s = s.borrow_mut();
        s.replace(AppState::load_or_new());
    });

    log::info!("Initializing canvas...");
    let canvas = get_canvas();
    setup_event_handlers(&canvas);

    log::info!("Initializing WebGl rendering context...");
    GLCTX.with(|gl| {
        let mut gl = gl.borrow_mut();
        gl.replace(get_gl_context(&canvas));
    });

    log::info!("Initializing engine...");
    ENGINE.with(|e| {
        let mut e = e.borrow_mut();
        e.replace(Engine::new());
    });

    let body_databases = vec![
        "public/database/artificial.json",
        "public/database/asteroids.json",
        "public/database/comets.json",
        "public/database/jovian_satellites.json",
        "public/database/major_bodies.json",
        "public/database/other_satellites.json",
        "public/database/saturnian_satellites.json",
        // "public/database/state_vectors.json",
        // "public/database/osc_elements.json",
    ];

    let body_database_results: Vec<String> =
        join_all(body_databases.iter().map(|url| request_text(url)))
            .await
            .into_iter()
            .collect::<Result<Vec<String>, JsValue>>()
            .unwrap();

    ENGINE.with(|e| {
        let mut ref_mut = e.borrow_mut();
        let e = ref_mut.as_mut().unwrap();

        for it in body_databases.iter().zip(body_database_results.iter()) {
            let (url, text) = it;
            e.receive_text(url, "body_database", text.as_str());
        }
    });

    // TODO: Load scenes

    // fetch_required_text("public/database/artificial.json", "body_database");
    // fetch_required_text("public/database/asteroids.json", "body_database");
    // fetch_required_text("public/database/comets.json", "body_database");
    // fetch_required_text("public/database/jovian_satellites.json", "body_database");
    // fetch_required_text("public/database/major_bodies.json", "body_database");
    // fetch_required_text("public/database/other_satellites.json", "body_database");
    // fetch_required_text("public/database/saturnian_satellites.json", "body_database");
    // fetch_required_text("public/database/state_vectors.json", "vectors_database");
    // fetch_required_text("public/database/osc_elements.json", "elements_database");

    // fetch_required_text("public/scenes/auto_load_manifest.txt", "auto_load_manifest");

    // Summoning ritual curtesy of https://rustwasm.github.io/docs/wasm-bindgen/examples/request-animation-frame.html
    log::info!("Beginning request_animation_frame loop...");
    let f = Rc::new(RefCell::new(None));
    let g = f.clone();
    *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
        redraw_requested();

        request_animation_frame(f.borrow().as_ref().unwrap());
    }) as Box<dyn FnMut()>));
    request_animation_frame(g.borrow().as_ref().unwrap());

    Ok(())
}

fn redraw_requested() {
    STATE.with(|s| {
        if let Ok(mut ref_mut_s) = s.try_borrow_mut() {
            let s = ref_mut_s.as_mut().unwrap();

            let canvas = get_canvas();

            let state_result = update_state(s, &canvas);
            if state_result == UpdateStateResult::NoDraw {
                return;
            }

            // Save state to local storage once in a while
            if s.real_time_s - s.time_of_last_save > 3.0 {
                serialize_state(s);
            }

            ENGINE.with(|e| {
                if let Ok(mut ref_mut_e) = e.try_borrow_mut() {
                    let e = ref_mut_e.as_mut().unwrap();

                    // Update our main framebuffer
                    if let UpdateStateResult::ResizeDraw(width, height) = state_result {
                        e.resize(width, height);
                    };

                    e.update(s);
                } else {
                    log::warn!("Failed to borrow engine for engine update!");
                }
            });
        } else {
            log::warn!("Failed to borrow app state for engine update!");
        }
    });
}

fn serialize_state(state: &mut AppState) {
    state.time_of_last_save = state.real_time_s;
    state.save();

    UICTX.with(|ui| {
        let ui = ui.borrow();
        let ui_ref = ui.as_ref().unwrap();
        let ctx = ui_ref.ctx();

        gui_backend::save_memory(ctx);
    });
}

#[derive(PartialEq)]
enum UpdateStateResult {
    NoDraw,
    Draw,
    ResizeDraw(u32, u32),
}

fn update_state(state: &mut AppState, canvas: &HtmlCanvasElement) -> UpdateStateResult {
    if state.pending_reset {
        *state = AppState::new();
        local_storage_remove("app_state");
    }

    let canvas_width_on_screen = canvas.client_width() as u32;
    let canvas_height_on_screen = canvas.client_height() as u32;

    let mut resized = false;
    if canvas.width() as u32 != canvas_width_on_screen || canvas.height() != canvas_height_on_screen
    {
        // Sets the actual resolution of the canvas in pixels
        canvas.set_width(canvas_width_on_screen);
        canvas.set_height(canvas_height_on_screen);

        log::info!(
            "Resized to w: {}, h: {}",
            canvas_width_on_screen,
            canvas_height_on_screen
        );

        // We'll need to resize framebuffers and stuff
        resized = true;
    }

    state.canvas_height = canvas_height_on_screen;
    state.canvas_width = canvas_width_on_screen;

    let now_s = js_sys::Date::now() / 1000.0 - state.start_date;
    let real_delta_s = now_s - state.last_frame_s;

    // Framerate limiter
    if real_delta_s < 1.0 / state.frames_per_second_limit {
        return UpdateStateResult::NoDraw;
    }

    let sim_delta_s =
        real_delta_s * state.simulation_speed * (!state.simulation_paused as i32 as f64);
    state.last_frame_s = now_s;
    state.sim_time_s += sim_delta_s;
    state.real_time_s += real_delta_s;
    state.sim_delta_time_s = sim_delta_s;
    state.real_delta_time_s = real_delta_s;

    if resized {
        return UpdateStateResult::ResizeDraw(state.canvas_width, state.canvas_height);
    } else {
        return UpdateStateResult::Draw;
    }
}
