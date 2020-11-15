use std::sync::{Arc, Mutex};

use crate::wasm_bindgen::JsCast;
use crate::{
    app_state::AppState, components::ui::WidgetType, components::MeshComponent,
    components::PhysicsComponent, components::TransformComponent, components::UIComponent,
    engine::Engine, managers::InputManager,
};
use cgmath::Vector3;
use gltf::Gltf;
use wasm_bindgen::prelude::*;
use web_sys::WebGlRenderingContext as GL;
use web_sys::{HtmlCanvasElement, WebGlRenderingContext};
use winit::{
    event::Event,
    event::WindowEvent,
    event_loop::{ControlFlow, EventLoop},
    platform::web::WindowBuilderExtWebSys,
    platform::web::WindowExtWebSys,
    window::{self, WindowBuilder},
};

/** Main interface between javascript and the inner Engine object */
#[wasm_bindgen]
pub struct EngineInterface {
    canvas: HtmlCanvasElement,

    // TODO: This doesn't look like it belongs here
    start_ms: f64,
    last_frame_ms: f64,

    app_state: Arc<Mutex<AppState>>,
    engine: Engine,
}

#[wasm_bindgen]
impl EngineInterface {
    #[wasm_bindgen(constructor)]
    pub fn new(canvas: HtmlCanvasElement) -> Self {
        log::info!("Initializing...");

        let gl: WebGlRenderingContext = canvas
            .get_context("webgl")
            .unwrap()
            .unwrap()
            .dyn_into()
            .unwrap();
        gl.enable(GL::BLEND);
        gl.blend_func(GL::SRC_ALPHA, GL::ONE_MINUS_SRC_ALPHA);
        gl.enable(GL::CULL_FACE);
        gl.cull_face(GL::BACK);
        gl.clear_color(0.0, 0.0, 0.0, 1.0); //RGBA
        gl.clear_depth(1.);

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

        let mut engine = Engine::new();
        engine.res_man.compile_materials(&gl);

        let start_ms = js_sys::Date::now();
        let mut last_frame_ms = 0.0;

        // Setup scene
        let parent = engine.ent_man.new_entity();
        let parent_id = engine.ent_man.get_entity_index(&parent).unwrap();

        let trans_comp = engine
            .comp_man
            .add_component::<TransformComponent>(parent_id)
            .unwrap();
        let phys_comp = engine
            .comp_man
            .add_component::<PhysicsComponent>(parent_id)
            .unwrap();
        phys_comp.ang_mom = Vector3::new(0.0, 0.0, 1.0);
        // phys_comp.lin_mom = Vector3::new(10.0, 0.0, 0.0);
        let mesh_comp = engine
            .comp_man
            .add_component::<MeshComponent>(parent_id)
            .unwrap();
        mesh_comp.mesh = engine.res_man.generate_mesh("cube", &gl);
        mesh_comp.material = engine.res_man.get_material("material");

        let child = engine.ent_man.new_entity();
        let child_id = engine.ent_man.get_entity_index(&child).unwrap();
        engine
            .ent_man
            .set_entity_parent(&parent, &child, &mut engine.comp_man);
        let trans_comp = engine
            .comp_man
            .add_component::<TransformComponent>(child_id)
            .unwrap();
        trans_comp.get_local_transform_mut().disp = Vector3::new(4.0, 0.0, 0.0);
        trans_comp.get_local_transform_mut().scale = 0.5;
        let phys_comp = engine
            .comp_man
            .add_component::<PhysicsComponent>(child_id)
            .unwrap();
        phys_comp.ang_mom = Vector3::new(-1.0, 0.0, 0.0); // This shouldn't do anything
        let mesh_comp = engine
            .comp_man
            .add_component::<MeshComponent>(child_id)
            .unwrap();
        mesh_comp.mesh = engine.res_man.generate_mesh("cube", &gl);
        mesh_comp.material = engine.res_man.get_material("material");

        // let plane = engine.ent_man.new_entity("plane");
        // let trans_comp = engine
        //     .comp_man
        //     .add_component::<TransformComponent>(plane)
        //     .unwrap();
        // trans_comp.transform.scale = 3.0;
        // let mesh_comp = engine
        //     .comp_man
        //     .add_component::<MeshComponent>(plane)
        //     .unwrap();
        // mesh_comp.mesh = engine.res_man.generate_mesh("plane", &gl);
        // mesh_comp.material = engine.res_man.get_material("material");

        let grid = engine.ent_man.new_entity();
        let grid_id = engine.ent_man.get_entity_index(&grid).unwrap();
        let trans_comp = engine
            .comp_man
            .add_component::<TransformComponent>(grid_id)
            .unwrap();
        trans_comp.get_local_transform_mut().scale = 1000.0;
        let mesh_comp = engine
            .comp_man
            .add_component::<MeshComponent>(grid_id)
            .unwrap();
        mesh_comp.mesh = engine.res_man.generate_mesh("grid", &gl);
        mesh_comp.material = engine.res_man.get_material("material");

        let axes = engine.ent_man.new_entity();
        let axes_id = engine.ent_man.get_entity_index(&axes).unwrap();
        let trans_comp = engine
            .comp_man
            .add_component::<TransformComponent>(axes_id)
            .unwrap();
        trans_comp.get_local_transform_mut().scale = 3.0;
        let mesh_comp = engine
            .comp_man
            .add_component::<MeshComponent>(axes_id)
            .unwrap();
        mesh_comp.mesh = engine.res_man.generate_mesh("axes", &gl);
        mesh_comp.material = engine.res_man.get_material("material");

        let ui_entity = engine.ent_man.new_entity();
        let ui_id = engine.ent_man.get_entity_index(&ui_entity).unwrap();
        engine.comp_man.add_component::<TransformComponent>(ui_id);
        let ui_comp = engine.comp_man.add_component::<UIComponent>(ui_id).unwrap();
        ui_comp.widget_type = WidgetType::TestWidget;

        let app_state: Arc<Mutex<AppState>> = AppState::new();
        {
            let mut app_state_mut = &mut *app_state.lock().unwrap();
            app_state_mut.gl = Some(gl);
            app_state_mut.phys_time_ms = 0.0;
            app_state_mut.real_time_ms = 0.0;
        }

        EngineInterface::setup_event_handlers(&canvas, app_state.clone());

        let start_ms = js_sys::Date::now();
        let last_frame_ms = 0.0;

        return EngineInterface {
            canvas,
            engine,
            app_state,
            start_ms,
            last_frame_ms,
        };
    }

    fn setup_event_handlers(canvas: &HtmlCanvasElement, app_state: Arc<Mutex<AppState>>) {
        // mousedown
        {
            let app_state_clone = app_state.clone();
            let canvas_clone = canvas.clone();
            let handler = move |event: web_sys::MouseEvent| {
                let state = &mut *app_state_clone.lock().unwrap();
                match event.button() as i16 {
                    0 => state.input.m0_down = true,

                    // 1 is the mouse wheel click
                    2 => {
                        state.input.m1_down = true;
                        canvas_clone.request_pointer_lock();
                    }
                    _ => {}
                };
            };

            let handler = Closure::wrap(Box::new(handler) as Box<dyn FnMut(_)>);
            canvas
                .add_event_listener_with_callback("mousedown", handler.as_ref().unchecked_ref())
                .expect("Failed to set mousedown event handler");
            handler.forget();
        }

        // mousemove
        {
            let app_state_clone = app_state.clone();
            let handler = move |event: web_sys::MouseEvent| {
                let state = &mut *app_state_clone.lock().unwrap();

                // With pointer lock client_x and client_y don't actually change, so we need movement_*
                if state.input.m1_down {
                    state.input.mouse_x += event.movement_x();
                    state.input.mouse_y += event.movement_y();
                } else {
                    state.input.mouse_x = event.client_x();
                    state.input.mouse_y = event.client_y();
                }
            };

            let handler = Closure::wrap(Box::new(handler) as Box<dyn FnMut(_)>);
            canvas
                .add_event_listener_with_callback("mousemove", handler.as_ref().unchecked_ref())
                .expect("Failed to set mousemove event handler");
            handler.forget();
        }

        // mouseup
        {
            let app_state_clone = app_state.clone();
            let handler = move |event: web_sys::MouseEvent| {
                let state = &mut *app_state_clone.lock().unwrap();
                match event.button() as i16 {
                    0 => state.input.m0_down = false,

                    // 1 is the mouse wheel click
                    2 => {
                        state.input.m1_down = false;

                        // Release pointer lock
                        let window = web_sys::window().unwrap();
                        let doc = window.document().unwrap();
                        doc.exit_pointer_lock();
                    }
                    _ => {}
                };
            };

            let handler = Closure::wrap(Box::new(handler) as Box<dyn FnMut(_)>);
            canvas
                .add_event_listener_with_callback("mouseup", handler.as_ref().unchecked_ref())
                .expect("Failed to set mouseup event handler");
            handler.forget();
        }

        // wheel
        {
            let app_state_clone = app_state.clone();
            let handler = move |event: web_sys::WheelEvent| {
                let state = &mut *app_state_clone.lock().unwrap();

                if event.delta_y() < 0.0 {
                    state.move_speed *= 1.1;
                } else {
                    state.move_speed *= 0.9;
                }
            };

            let handler = Closure::wrap(Box::new(handler) as Box<dyn FnMut(_)>);
            canvas
                .add_event_listener_with_callback("wheel", handler.as_ref().unchecked_ref())
                .expect("Failed to set mouseup event handler");
            handler.forget();
        }

        // keydown
        {
            let app_state_clone = app_state.clone();
            let handler = move |event: web_sys::KeyboardEvent| {
                let state = &mut *app_state_clone.lock().unwrap();
                match (event.code() as String).as_str() {
                    "KeyW" | "ArrowUp" => {
                        state.input.forward_down = true;
                    }
                    "KeyA" | "ArrowLeft" => {
                        state.input.left_down = true;
                    }
                    "KeyS" | "ArrowDown" => {
                        state.input.back_down = true;
                    }
                    "KeyD" | "ArrowRight" => {
                        state.input.right_down = true;
                    }
                    "KeyE" => {
                        state.input.up_down = true;
                    }
                    "KeyQ" => {
                        state.input.down_down = true;
                    }
                    _ => {}
                };
            };

            let handler = Closure::wrap(Box::new(handler) as Box<dyn FnMut(_)>);
            canvas
                .add_event_listener_with_callback("keydown", handler.as_ref().unchecked_ref())
                .expect("Failed to set keydown event handler");
            handler.forget();
        }

        // keyup
        {
            let app_state_clone = app_state.clone();
            let handler = move |event: web_sys::KeyboardEvent| {
                let state = &mut *app_state_clone.lock().unwrap();
                match (event.code() as String).as_str() {
                    "KeyW" | "ArrowUp" => {
                        state.input.forward_down = false;
                    }
                    "KeyA" | "ArrowLeft" => {
                        state.input.left_down = false;
                    }
                    "KeyS" | "ArrowDown" => {
                        state.input.back_down = false;
                    }
                    "KeyD" | "ArrowRight" => {
                        state.input.right_down = false;
                    }
                    "KeyE" => {
                        state.input.up_down = false;
                    }
                    "KeyQ" => {
                        state.input.down_down = false;
                    }
                    _ => {}
                };
            };

            let handler = Closure::wrap(Box::new(handler) as Box<dyn FnMut(_)>);
            canvas
                .add_event_listener_with_callback("keyup", handler.as_ref().unchecked_ref())
                .expect("Failed to set keyup event handler");
            handler.forget();
        }
    }

    #[wasm_bindgen]
    pub fn load_gltf(&mut self, data: &mut [u8]) {
        log::info!("Load_gltf: received {} bytes", data.len());

        let gltf = Gltf::from_slice(data).expect("Failed to load gltf...");
        log::info!("Num meshes: {}", gltf.meshes().len());
    }

    #[wasm_bindgen]
    pub fn begin_loop(mut self) {
        log::info!("Beginning engine loop...");

        let event_loop = EventLoop::new();

        let window = WindowBuilder::new()
            .with_title("Title")
            .with_canvas(Some(self.canvas))
            .build(&event_loop)
            .expect("Failed to find window!");

        // Get a new one as we need to move it into the window builder for some reason
        self.canvas = window.canvas();

        let style = self.canvas.style();
        style
            .set_property_with_priority("width", "100%", "")
            .expect("Failed to set width!");
        style
            .set_property_with_priority("height", "100%", "")
            .expect("Failed to set height!");

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
                    let canvas_width_on_screen = self.canvas.client_width() as u32;
                    let canvas_height_on_screen = self.canvas.client_height() as u32;

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
                        let style = self.canvas.style();
                        style
                            .set_property_with_priority("width", "100%", "")
                            .expect("Failed to set width!");
                        style
                            .set_property_with_priority("height", "100%", "")
                            .expect("Failed to set height!");

                        log::info!(
                            "Resized to engine: {}, h: {}",
                            canvas_width_on_screen,
                            canvas_height_on_screen
                        );
                    }

                    let mut app_state_mut = &mut *self.app_state.lock().unwrap();

                    let now_ms = js_sys::Date::now() - self.start_ms;
                    let real_delta_ms = now_ms - self.last_frame_ms;
                    let phys_delta_ms = real_delta_ms * app_state_mut.simulation_speed;
                    self.last_frame_ms = now_ms;

                    app_state_mut.canvas_height = canvas_height_on_screen;
                    app_state_mut.canvas_width = canvas_width_on_screen;
                    app_state_mut.phys_time_ms += phys_delta_ms;
                    app_state_mut.real_time_ms += real_delta_ms;
                    app_state_mut.phys_delta_time_ms = phys_delta_ms;
                    app_state_mut.real_delta_time_ms = real_delta_ms;

                    // let parent_index = engine.ent_man.get_entity_index(&parent);
                    // let parent_trans_comp: &mut TransformComponent = engine
                    //     .comp_man
                    //     .get_component::<TransformComponent>(parent_index.unwrap())
                    //     .unwrap();
                    // parent_trans_comp.get_local_transform_mut().disp.x +=
                    //     (app_state_mut.phys_delta_time_ms * 0.001) as f32;
                    // parent_trans_comp.get_local_transform_mut().rot = Quaternion::from_axis_angle(
                    //     Vector3::new(0.0 as f32, 0.0 as f32, 1.0 as f32),
                    //     cgmath::Deg((app_state_mut.phys_time_ms / 100.0) as f32),
                    // )
                    // .normalize();

                    // let child_index = engine.ent_man.get_entity_index(&child);
                    // let child_trans_comp: &mut TransformComponent = engine
                    //     .comp_man
                    //     .get_component::<TransformComponent>(child_index.unwrap())
                    //     .unwrap();
                    // child_trans_comp.get_local_transform_mut().disp.x +=
                    //     (app_state_mut.phys_delta_time_ms * 0.001) as f32;
                    // child_trans_comp.get_local_transform_mut().rot = Quaternion::from_axis_angle(
                    //     Vector3::new(1.0 as f32, 0.0 as f32, 0.0 as f32),
                    //     cgmath::Deg((app_state_mut.phys_time_ms / 100.0) as f32),
                    // )
                    // .normalize();

                    self.engine.update(app_state_mut);
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
}
