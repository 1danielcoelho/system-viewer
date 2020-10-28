extern crate wasm_bindgen;

use std::sync::{Arc, Mutex};

use app_state::AppState;
use cgmath::{InnerSpace, Vector3};
use components::{MeshComponent, TransformComponent, UIComponent, WidgetType};
use wasm_bindgen::prelude::*;
use winit::{event::Event, event_loop::ControlFlow, platform::web::WindowExtWebSys};
use winit::{event::WindowEvent, window::WindowBuilder};
use winit::{event_loop::EventLoop, platform::web::WindowBuilderExtWebSys};
use world::World;

mod app_state;
mod components;
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

    let mut world = World::new();
    world.res_man.compile_materials(&context);

    let start_ms = js_sys::Date::now();
    let mut last_frame_ms = start_ms;

    // Setup scene
    let entity = world.ent_man.new_entity("cube");
    let trans_comp = world
        .comp_man
        .add_component::<TransformComponent>(entity)
        .unwrap();
    let mesh_comp = world
        .comp_man
        .add_component::<MeshComponent>(entity)
        .unwrap();
    mesh_comp.mesh = world.res_man.generate_mesh("cube", &context);
    mesh_comp.material = world.res_man.get_material("material");

    let ui_entity = world.ent_man.new_entity("test_ui");
    world
        .comp_man
        .add_component::<TransformComponent>(ui_entity);
    let ui_comp = world
        .comp_man
        .add_component::<UIComponent>(ui_entity)
        .unwrap();
    ui_comp.widget_type = WidgetType::TestWidget;

    let app_state: Arc<Mutex<AppState>> = AppState::new();
    app_state.lock().unwrap().gl = Some(context);

    gl_setup::setup_event_handlers(&canvas, app_state.clone());

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll; // Can change this to Wait to pause when no input is given

        match event {
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
                        "Resized to world: {}, h: {}",
                        canvas_width_on_screen,
                        canvas_height_on_screen
                    );
                }

                let now_ms = js_sys::Date::now();

                let app_state_mut = &mut *app_state.lock().unwrap();
                app_state_mut.canvas_height = canvas_height_on_screen;
                app_state_mut.canvas_width = canvas_width_on_screen;
                app_state_mut.time_ms = now_ms - start_ms;
                app_state_mut.delta_time_ms = now_ms - last_frame_ms;

                last_frame_ms = now_ms;

                let cam_forward = ((app_state_mut.camera.target - app_state_mut.camera.pos)
                    as Vector3<f32>)
                    .normalize();
                let cam_right: Vector3<f32> = cam_forward.cross(app_state_mut.camera.up);
                let cam_up: Vector3<f32> = cam_right.cross(cam_forward);

                let mut incr: cgmath::Vector3<f32> = cgmath::Vector3::new(0.0, 0.0, 0.0);
                if app_state_mut.input.forward_down {
                    incr += cam_forward
                        * (app_state_mut.delta_time_ms as f32)
                        * app_state_mut.move_speed;
                }
                if app_state_mut.input.back_down {
                    incr -= cam_forward
                        * (app_state_mut.delta_time_ms as f32)
                        * app_state_mut.move_speed;
                }
                if app_state_mut.input.left_down {
                    incr -=
                        cam_right * (app_state_mut.delta_time_ms as f32) * app_state_mut.move_speed;
                }
                if app_state_mut.input.right_down {
                    incr +=
                        cam_right * (app_state_mut.delta_time_ms as f32) * app_state_mut.move_speed;
                }

                app_state_mut.camera.pos += incr;
                app_state_mut.camera.target += incr;

                world
                    .sys_man
                    .run(&app_state_mut, &mut world.comp_man, &mut world.event_man);

                // Dispatch events
            }

            Event::NewEvents(_) => {}
            Event::DeviceEvent { device_id, event } => {}
            Event::UserEvent(_) => {}
            Event::Suspended => {}
            Event::Resumed => {}
            Event::RedrawEventsCleared => {}
            Event::LoopDestroyed => {}

            // In case the window id doesn't match
            _ => {}
        }
    });
}
