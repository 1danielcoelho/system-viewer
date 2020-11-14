extern crate wasm_bindgen;

use std::sync::{Arc, Mutex};

use app_state::AppState;
use cgmath::{Basis3, Deg, InnerSpace, MetricSpace, Rotation, Rotation3, Vector3};
use components::{
    ui::WidgetType, MeshComponent, PhysicsComponent, TransformComponent, UIComponent,
};
use gltf::Gltf;
use managers::InputManager;
use wasm_bindgen::prelude::*;
use winit::{event::Event, event_loop::ControlFlow, platform::web::WindowExtWebSys};
use winit::{event::WindowEvent, window::WindowBuilder};
use winit::{event_loop::EventLoop, platform::web::WindowBuilderExtWebSys};
use world::World;

mod app_state;
mod components;
mod gl_setup;
mod managers;
mod systems;
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
pub fn load_gltf(data: &mut [u8]) {
    log::info!("received {} bytes", data.len());

    let gltf = Gltf::from_slice(data).expect("Failed to load gltf...");
    log::info!("Num meshes: {}", gltf.meshes().len());
}

#[wasm_bindgen]
pub fn initialize() {
    let (context, canvas) = gl_setup::initialize_webgl_context().unwrap();

    let event_loop = EventLoop::new();

    let window = WindowBuilder::new()
        .with_title("Title")
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
        r"ev.preventDefault();return false;",
    )));

    let mut world = World::new();
    world.res_man.compile_materials(&context);

    let start_ms = js_sys::Date::now();
    let mut last_frame_ms = 0.0;

    // Setup scene
    let parent = world.ent_man.new_entity();
    let parent_id = world.ent_man.get_entity_index(&parent).unwrap();

    let trans_comp = world
        .comp_man
        .add_component::<TransformComponent>(parent_id)
        .unwrap();
    let phys_comp = world
        .comp_man
        .add_component::<PhysicsComponent>(parent_id)
        .unwrap();
    phys_comp.ang_mom = Vector3::new(0.0, 0.0, 1.0);
    let mesh_comp = world
        .comp_man
        .add_component::<MeshComponent>(parent_id)
        .unwrap();
    mesh_comp.mesh = world.res_man.generate_mesh("cube", &context);
    mesh_comp.material = world.res_man.get_material("material");

    let child = world.ent_man.new_entity();
    let child_id = world.ent_man.get_entity_index(&child).unwrap();
    world
        .ent_man
        .set_entity_parent(&parent, &child, &mut world.comp_man);
    let trans_comp = world
        .comp_man
        .add_component::<TransformComponent>(child_id)
        .unwrap();
    trans_comp.get_local_transform_mut().disp = Vector3::new(4.0, 0.0, 0.0);
    trans_comp.get_local_transform_mut().scale = 0.5;
    let phys_comp = world
        .comp_man
        .add_component::<PhysicsComponent>(child_id)
        .unwrap();
    phys_comp.ang_mom = Vector3::new(-1.0, 0.0, 0.0);
    let mesh_comp = world
        .comp_man
        .add_component::<MeshComponent>(child_id)
        .unwrap();
    mesh_comp.mesh = world.res_man.generate_mesh("cube", &context);
    mesh_comp.material = world.res_man.get_material("material");

    // let plane = world.ent_man.new_entity("plane");
    // let trans_comp = world
    //     .comp_man
    //     .add_component::<TransformComponent>(plane)
    //     .unwrap();
    // trans_comp.transform.scale = 3.0;
    // let mesh_comp = world
    //     .comp_man
    //     .add_component::<MeshComponent>(plane)
    //     .unwrap();
    // mesh_comp.mesh = world.res_man.generate_mesh("plane", &context);
    // mesh_comp.material = world.res_man.get_material("material");

    let grid = world.ent_man.new_entity();
    let grid_id = world.ent_man.get_entity_index(&grid).unwrap();
    let trans_comp = world
        .comp_man
        .add_component::<TransformComponent>(grid_id)
        .unwrap();
    trans_comp.get_local_transform_mut().scale = 1000.0;
    let mesh_comp = world
        .comp_man
        .add_component::<MeshComponent>(grid_id)
        .unwrap();
    mesh_comp.mesh = world.res_man.generate_mesh("grid", &context);
    mesh_comp.material = world.res_man.get_material("material");

    let axes = world.ent_man.new_entity();
    let axes_id = world.ent_man.get_entity_index(&axes).unwrap();
    let trans_comp = world
        .comp_man
        .add_component::<TransformComponent>(axes_id)
        .unwrap();
    trans_comp.get_local_transform_mut().scale = 3.0;
    let mesh_comp = world
        .comp_man
        .add_component::<MeshComponent>(axes_id)
        .unwrap();
    mesh_comp.mesh = world.res_man.generate_mesh("axes", &context);
    mesh_comp.material = world.res_man.get_material("material");

    let ui_entity = world.ent_man.new_entity();
    let ui_id = world.ent_man.get_entity_index(&ui_entity).unwrap();
    world.comp_man.add_component::<TransformComponent>(ui_id);
    let ui_comp = world.comp_man.add_component::<UIComponent>(ui_id).unwrap();
    ui_comp.widget_type = WidgetType::TestWidget;

    let app_state: Arc<Mutex<AppState>> = AppState::new();
    {
        let mut app_state_mut = &mut *app_state.lock().unwrap();
        app_state_mut.gl = Some(context);
        app_state_mut.phys_time_ms = last_frame_ms;
        app_state_mut.real_time_ms = last_frame_ms;
    }

    InputManager::setup_event_handlers(&canvas, app_state.clone());

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

                let mut app_state_mut = &mut *app_state.lock().unwrap();

                let now_ms = js_sys::Date::now() - start_ms;
                let real_delta_ms = now_ms - last_frame_ms;
                let phys_delta_ms = real_delta_ms * app_state_mut.simulation_speed;
                last_frame_ms = now_ms;

                app_state_mut.canvas_height = canvas_height_on_screen;
                app_state_mut.canvas_width = canvas_width_on_screen;
                app_state_mut.phys_time_ms += phys_delta_ms;
                app_state_mut.real_time_ms += real_delta_ms;
                app_state_mut.phys_delta_time_ms = phys_delta_ms;
                app_state_mut.real_delta_time_ms = real_delta_ms;

                world.update(app_state_mut);
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
