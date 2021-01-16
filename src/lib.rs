#[macro_use(lazy_static)]
extern crate lazy_static;
extern crate nalgebra as na;
extern crate regex;
extern crate ron;
extern crate serde;
extern crate wasm_bindgen;

use crate::utils::web::local_storage_remove;
use crate::{
    app_state::AppState,
    engine::Engine,
    utils::{
        gl::setup_gl_context,
        web::{force_full_canvas, get_canvas, get_gl_context, setup_event_handlers},
    },
};
use egui::Ui;
use std::cell::RefCell;
use wasm_bindgen::prelude::*;
use web_sys::{HtmlCanvasElement, WebGl2RenderingContext};
use winit::{
    event::Event,
    event_loop::{ControlFlow, EventLoop},
    platform::web::WindowBuilderExtWebSys,
    window::{Window, WindowBuilder},
};

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
pub fn initialize() {
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
        let gl = gl.as_mut().unwrap();

        setup_gl_context(&gl);
    });

    log::info!("Initializing engine...");
    ENGINE.with(|e| {
        let mut e = e.borrow_mut();
        e.replace(Engine::new());
    });
}

#[wasm_bindgen]
pub async fn start_loop() {
    log::info!("Beginning engine loop...");

    let event_loop = EventLoop::new();

    let window = WindowBuilder::new()
        .with_title("Title")
        .with_canvas(Some(get_canvas()))
        .build(&event_loop)
        .expect("Failed to find window!");

    // Every time winit resizes the canvas it manually sets width and height values we must undo
    let canvas = get_canvas();
    force_full_canvas(&canvas);

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll; // Can change this to Wait to pause when no input is given

        match event {
            Event::MainEventsCleared => window.request_redraw(),
            Event::RedrawRequested(id) if id == window.id() => redraw_requested(&window, &canvas),
            _ => {}
        }
    });
}

fn redraw_requested(window: &Window, canvas: &HtmlCanvasElement) {
    STATE.with(|s| {
        if let Ok(mut ref_mut_s) = s.try_borrow_mut() {
            let s = ref_mut_s.as_mut().unwrap();

            update_state(s, window, canvas);

            // Save state to local storage once in a while
            if s.real_time_s - s.time_of_last_save > 3.0 {
                s.save();
                s.time_of_last_save = s.real_time_s;
            }

            ENGINE.with(|e| {
                if let Ok(mut ref_mut_e) = e.try_borrow_mut() {
                    let e = ref_mut_e.as_mut().unwrap();

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

fn update_state(state: &mut AppState, window: &Window, canvas: &HtmlCanvasElement) {
    if state.pending_reset {
        *state = AppState::new();
        local_storage_remove("app_state");
    }

    let canvas_width_on_screen = canvas.client_width() as u32;
    let canvas_height_on_screen = canvas.client_height() as u32;

    // Check if we need to resize
    if window.inner_size().width != canvas_width_on_screen
        || window.inner_size().height != canvas_height_on_screen
    {
        // Sets canvas height and width, unfortunately also setting its style height and width
        window.set_inner_size(winit::dpi::LogicalSize::new(
            canvas_width_on_screen,
            canvas_height_on_screen,
        ));

        force_full_canvas(&canvas);

        log::info!(
            "Resized to w: {}, h: {}",
            canvas_width_on_screen,
            canvas_height_on_screen
        );
    }

    let now_s = (js_sys::Date::now() - state.start_s) / 1000.0;
    let real_delta_s = now_s - state.last_frame_s;
    let phys_delta_s =
        real_delta_s * state.simulation_speed * (!state.simulation_paused as i32 as f64);
    state.last_frame_s = now_s;

    state.canvas_height = canvas_height_on_screen;
    state.canvas_width = canvas_width_on_screen;
    state.sim_time_days += phys_delta_s;
    state.real_time_s += real_delta_s;
    state.sim_delta_time_days = phys_delta_s;
    state.real_delta_time_s = real_delta_s;
}

/** Synchronous function that JS calls to inject bytes data into the engine because we can't await for a JS promise from within the winit engine loop */
#[wasm_bindgen]
pub fn receive_text(url: &str, content_type: &str, text: &str) {
    log::info!(
        "Engine received text from url '{}', content type '{}', length: {}",
        url,
        content_type,
        text.len()
    );

    ENGINE.with(|e| {
        let mut ref_mut = e.borrow_mut();
        let e = ref_mut.as_mut().unwrap();

        e.receive_text(url, content_type, text);
    });
}

/** Synchronous function that JS calls to inject text data into the engine because we can't await for a JS promise from within the winit engine loop */
#[wasm_bindgen]
pub fn receive_bytes(url: &str, content_type: &str, data: &mut [u8]) {
    log::info!(
        "Engine received bytes from url '{}', content type '{}', length: {}",
        url,
        content_type,
        data.len()
    );

    ENGINE.with(|e| {
        let mut ref_mut = e.borrow_mut();
        let e = ref_mut.as_mut().unwrap();

        e.receive_bytes(url, content_type, data);
    });
}

#[wasm_bindgen(module = "/io.js")]
extern "C" {
    pub fn fetch_text(url: &str, content_type: &str);
    pub fn fetch_bytes(url: &str, content_type: &str);
    pub fn prompt_for_text_file(content_type: &str, extension: &str);
    pub fn prompt_for_bytes_file(content_type: &str, extension: &str);
}
