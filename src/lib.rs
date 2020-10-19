extern crate wasm_bindgen;

use wasm_bindgen::prelude::*;
use web_sys::WebGlRenderingContext as GL;
use winit::{event::Event, event_loop::ControlFlow, platform::web::WindowExtWebSys};
use winit::{event::WindowEvent, window::WindowBuilder};
use winit::{event_loop::EventLoop, platform::web::WindowBuilderExtWebSys};

#[macro_use]
extern crate lazy_static;

mod app_state;
mod common_funcs;
mod constants;
mod gl_setup;
mod materials;
mod shaders;

#[macro_export]
macro_rules! glc {
    ($ctx:expr, $any:expr) => {
        #[cfg(debug_assertions)]
        while $ctx.get_error() != 0 {} // Not sure why he did this
        $any;
        #[cfg(debug_assertions)]
        while match $ctx.get_error() {
            0 => false,
            err => {
                log::error!("[OpenGL Error] {}", err);
                true
            }
        } {}
    };
}

#[wasm_bindgen(start)]
pub fn main() {
    // std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    console_error_panic_hook::set_once();

    console_log::init_with_level(log::Level::Debug).expect("Unable to initialize console logging");
}

#[wasm_bindgen]
pub fn initialize() {
    let (context, canvas) = gl_setup::initialize_webgl_context().unwrap();

    let event_loop = EventLoop::new();

    let window = WindowBuilder::new()
        .with_title("Title")
        //.with_inner_size(winit::dpi::LogicalSize::new(canvas.client_width(), canvas.client_height()))
        .with_canvas(Some(canvas))
        .build(&event_loop)
        .expect("Failed to find window!");

    let canvas = window.canvas();

    // Restore the canvas to be 100% because the window builder will attempt to set it to some size, and we want it to be driven by layout
    let style = canvas.style();
    style
        .set_property_with_priority("width", "100%", "")
        .expect("Failed to set width!");
    style
        .set_property_with_priority("height", "100%", "")
        .expect("Failed to set height!");

    canvas.set_oncontextmenu(Some(&js_sys::Function::new_with_args(
        "ev",
        r"
                ev.preventDefault();
                return false;
            ",
    )));

    let backend = egui_web::WebBackend::new("rustCanvas").expect("Failed to make a web backend for egui");
    let egui_ctx = egui::Context::new();

    // let app = Box::new(egui::DemoApp::default());
    // let runner = egui_web::AppRunner::new(backend, app).expect("Failed to make app runner");
    // egui_web::run(runner).expect("Failed to run egui");

    let cube = materials::SimpleMaterial::new(&context);

    let start_millis = js_sys::Date::now();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll; // Can change this to Wait to pause when no input is given

        match event {
            //Event::NewEvents(_) => imgui.io_mut().update_delta_time(Duration::from_millis(1)),
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                window_id,
            } if window_id == window.id() => *control_flow = ControlFlow::Exit,

            Event::MainEventsCleared => {
                window.request_redraw();
            }

            Event::RedrawRequested(window_id) if window_id == window.id() => {
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

                    // #HACK: Restore the canvas width/height to 100% so they get driven by the window size
                    let style = canvas.style();
                    style
                        .set_property_with_priority("width", "100%", "")
                        .expect("Failed to set width!");
                    style
                        .set_property_with_priority("height", "100%", "")
                        .expect("Failed to set height!");

                    log::info!(
                        "Resized to w: {}, h: {}",
                        canvas_width_on_screen,
                        canvas_height_on_screen
                    );
                }

                let ctx = &context;

                ctx.viewport(0, 0, canvas_width_on_screen as i32, canvas_height_on_screen as i32);

                let egui_input = egui_ctx.begin_frame();

                app_state::update_dynamic_data(
                    0.0,
                    canvas_height_on_screen as f32,
                    canvas_width_on_screen as f32,
                );
                let curr_state = app_state::get_curr_state();

                // log::info!("App state: {:?}", curr_state);

                glc!(ctx, ctx.clear_color(0.1, 0.1, 0.2, 1.0));
                glc!(ctx, ctx.clear(GL::COLOR_BUFFER_BIT | GL::DEPTH_BUFFER_BIT));

                cube.render(&context, (js_sys::Date::now() - start_millis) as f32, curr_state.canvas_width, curr_state.canvas_height);
            }

            event => {
                // TODO: Handle input events
            }
        }
    });
}
