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
use crate::utils::log::*;
use crate::utils::web::{
    get_canvas, get_gl_context, local_storage_remove, request_animation_frame, request_text,
    setup_event_handlers,
};
use futures::future::join_all;
use std::cell::RefCell;
use std::rc::Rc;
use utils::web::local_storage_set;
use wasm_bindgen::prelude::*;
use web_sys::HtmlCanvasElement;

mod app_state;
mod components;
mod engine;
mod managers;
mod systems;
mod utils;

thread_local! {
    pub static ENGINE: RefCell<Option<Engine>> = RefCell::new(None);
    pub static STATE: RefCell<Option<AppState>> = RefCell::new(None);
    pub static GLCTX: Rc<glow::Context> = Rc::new(get_gl_context());
    pub static UICTX: egui::Context = egui::Context::default();
}

#[wasm_bindgen(start)]
pub fn main_js() {
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
pub async fn start() -> Result<(), JsValue> {
    info!(LogCat::Engine, "Initializing state...");
    STATE.with(|s| {
        let mut s = s.borrow_mut();
        s.replace(AppState::load_or_new());
    });

    info!(LogCat::Engine, "Setting up events...");
    setup_event_handlers();

    info!(LogCat::Engine, "Initializing engine...");
    ENGINE.with(|e| {
        let mut e = e.borrow_mut();
        e.replace(Engine::new());
    });

    #[derive(Copy, Clone)]
    enum AssetType {
        BodyDatabase,
        Scene,
        StateVectors,
        OscElements,
    }
    struct AssetRequest(&'static str, AssetType);
    let requests = vec![
        AssetRequest("public/database/artificial.json", AssetType::BodyDatabase),
        AssetRequest("public/database/asteroids.json", AssetType::BodyDatabase),
        AssetRequest("public/database/comets.json", AssetType::BodyDatabase),
        AssetRequest(
            "public/database/jovian_satellites.json",
            AssetType::BodyDatabase,
        ),
        AssetRequest("public/database/major_bodies.json", AssetType::BodyDatabase),
        AssetRequest(
            "public/database/other_satellites.json",
            AssetType::BodyDatabase,
        ),
        AssetRequest(
            "public/database/saturnian_satellites.json",
            AssetType::BodyDatabase,
        ),
        AssetRequest(
            "public/database/state_vectors.json",
            AssetType::StateVectors,
        ),
        AssetRequest("public/database/osc_elements.json", AssetType::OscElements),
        AssetRequest("public/scenes/earth_centric.ron", AssetType::Scene),
        AssetRequest("public/scenes/full_solar_system.ron", AssetType::Scene),
        AssetRequest("public/scenes/gltf_test.ron", AssetType::Scene),
        AssetRequest("public/scenes/light_test.ron", AssetType::Scene),
        AssetRequest("public/scenes/planet_line_up.ron", AssetType::Scene),
        AssetRequest("public/scenes/planets_and_satellites.ron", AssetType::Scene),
    ];
    struct AssetResponse {
        url: &'static str,
        data: String,
        asset_type: AssetType,
    }

    let promises = join_all(requests.iter().map(|ar| async move {
        match request_text(ar.0).await {
            Ok(res) => Ok(AssetResponse {
                url: ar.0,
                data: res,
                asset_type: ar.1,
            }),
            Err(err) => Err(err),
        }
    }));

    let results: Vec<AssetResponse> = promises
        .await
        .into_iter()
        .collect::<Result<Vec<AssetResponse>, JsValue>>()
        .unwrap();

    ENGINE.with(|e| {
        let mut ref_mut = e.borrow_mut();
        let e = ref_mut.as_mut().unwrap();

        for resp in results.iter() {
            match resp.asset_type {
                AssetType::BodyDatabase => {
                    e.receive_text(resp.url, "body_database", resp.data.as_str());
                }
                AssetType::Scene => {
                    e.receive_text(resp.url, "scene", resp.data.as_str());
                }
                AssetType::StateVectors => {
                    e.receive_text(resp.url, "vectors_database", resp.data.as_str());
                }
                AssetType::OscElements => {
                    e.receive_text(resp.url, "elements_database", resp.data.as_str());
                }
            }
        }

        e.try_loading_last_scene();
    });

    // Summoning ritual courtesy of https://rustwasm.github.io/docs/wasm-bindgen/examples/request-animation-frame.html
    info!(LogCat::Engine, "Beginning request_animation_frame loop...");
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
                    warning!(LogCat::Engine, "Failed to borrow engine for engine update!");
                }
            });
        } else {
            warning!(
                LogCat::Engine,
                "Failed to borrow app state for engine update!"
            );
        }
    });
}

fn serialize_state(state: &mut AppState) {
    state.time_of_last_save = state.real_time_s;
    state.save();

    UICTX.with(|ui| {
        if let Ok(memory_string) = serde_json::to_string(&*ui.memory()) {
            local_storage_set("egui_memory_json", &memory_string)
        } else {
            error!(
                LogCat::Io,
                "Failed to serialize egui state to local storage!"
            );
        }
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

        info!(
            LogCat::Engine,
            "Resized to w: {}, h: {}", canvas_width_on_screen, canvas_height_on_screen
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
