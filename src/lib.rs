extern crate wasm_bindgen;

use app_state::AppState;
use components::{MeshComponent, PhysicsComponent, TransformComponent};
use egui::Pos2;
use gui_backend::WebInput;
use wasm_bindgen::prelude::*;
use web_sys::WebGlRenderingContext as GL;
use winit::{event::Event, event_loop::ControlFlow, platform::web::WindowExtWebSys};
use winit::{event::WindowEvent, window::WindowBuilder};
use winit::{event_loop::EventLoop, platform::web::WindowBuilderExtWebSys};
use world::World;

#[macro_use]
extern crate lazy_static;

mod app_state;
mod common_funcs;
mod components;
mod constants;
mod entity;
mod events;
mod gl_setup;
mod materials;
mod mesh;
mod object;
mod resources;
mod shaders;
mod systems;
mod texture;
mod world;

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

pub fn native_pixels_per_point() -> f32 {
    let pixels_per_point = web_sys::window().unwrap().device_pixel_ratio() as f32;
    if pixels_per_point > 0.0 && pixels_per_point.is_finite() {
        pixels_per_point
    } else {
        1.0
    }
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

    let mut backend =
        gui_backend::WebBackend::new("rustCanvas").expect("Failed to make a web backend for egui");
    let mut web_input: WebInput = Default::default();

    let cube = materials::SimpleMaterial::new(&context);

    let start_ms = js_sys::Date::now();
    let mut last_frame_ms = start_ms;

    let world = World::new();
    let w: &mut World = &mut world.borrow_mut();

    let cube_mesh = w.res_man.generate_mesh("cube", &context);

    let entity = w.ent_man.new_entity("cube");
    let trans_comp = w
        .comp_man
        .add_component::<TransformComponent>(entity)
        .unwrap();
    let mesh_comp = w.comp_man.add_component::<MeshComponent>(entity).unwrap();
    mesh_comp.mesh = cube_mesh;

    let app_state = AppState::new();
    app_state.gl = Some(context);

    //let trans_comp = mut_world.comp_man.add_component::<TransformComponent>().unwrap();

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

                let now_ms = js_sys::Date::now();
                let mut raw_input = web_input.new_frame(1.0);

                let app_state_mut = &mut app_state;
                app_state_mut.canvas_height = canvas_height_on_screen;
                app_state_mut.canvas_width = canvas_width_on_screen;
                app_state.time_ms = now_ms - start_ms;
                app_state.delta_time_ms = now_ms - last_frame_ms;
                app_state.ui = Some(backend.begin_frame(raw_input));

                last_frame_ms = now_ms;

                // TODO: Combine these or get rid of one of them?
                raw_input.mouse_pos = Some(Pos2 {
                    x: app_state.mouse_x,
                    y: app_state.mouse_y,
                });
                raw_input.mouse_down = app_state.mouse_down;

                let mut s = String::from("test");
                let mut value = 0.0;
                egui::Window::new("Debug").show(&app_state.ui.unwrap().ctx(), |ui| {
                    ui.label(format!("Hello, world {}", 123));
                    if ui.button("Save").clicked {
                        log::info!("Save!");
                    }
                    ui.text_edit(&mut s);
                    ui.add(egui::Slider::f32(&mut value, 0.0..=1.0).text("float"));
                });

                let (_, paint_jobs) = backend.end_frame().unwrap();
                backend.paint(paint_jobs).expect("Failed to paint!");
            }

            _ => {
                // TODO: Handle input events
            }
        }
    });
}
