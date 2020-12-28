use crate::{
    app_state::AppState,
    components::MeshComponent,
    engine::Engine,
    managers::{
        resource::material::{UniformName, UniformValue},
        Scene,
    },
    utils::{
        orbital_elements::{elements_to_circle_transform, parse_ephemerides, OrbitalElements},
        transform::Transform,
    },
};
use crate::{app_state::ButtonState, components::TransformComponent};
use na::{Quaternion, UnitQuaternion, Vector3};
use std::{
    cell::RefCell,
    sync::{Arc, Mutex},
};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{HtmlCanvasElement, WebGl2RenderingContext};
use winit::{
    event::Event,
    event::WindowEvent,
    event_loop::{ControlFlow, EventLoop},
    platform::web::WindowBuilderExtWebSys,
    platform::web::WindowExtWebSys,
    window::WindowBuilder,
};

type GL = WebGl2RenderingContext;

thread_local! {
    pub static EI: RefCell<EngineInterface> = RefCell::new(EngineInterface::new());
}

fn get_canvas() -> HtmlCanvasElement {
    let window = web_sys::window().expect("no global `window` exists");
    let document = window.document().expect("should have a document on window");
    let el = document.get_element_by_id("rustCanvas").unwrap();
    let canvas: HtmlCanvasElement = el.dyn_into().unwrap();
    return canvas;
}

pub struct EngineInterface {
    // TODO: This doesn't look like it belongs here
    start_ms: f64,
    last_frame_ms: f64,

    app_state: Arc<Mutex<AppState>>,
    engine: Engine,
}

impl EngineInterface {
    pub fn new() -> Self {
        log::info!("Initializing...");

        let canvas = get_canvas();

        let gl: WebGl2RenderingContext = canvas
            .get_context("webgl2")
            .unwrap()
            .unwrap()
            .dyn_into()
            .unwrap();

        // Setup webgl context
        gl.enable(GL::BLEND);
        gl.blend_func(GL::SRC_ALPHA, GL::ONE_MINUS_SRC_ALPHA);
        gl.enable(GL::CULL_FACE);
        gl.cull_face(GL::BACK);
        gl.clear_color(0.0, 0.0, 0.0, 1.0); //RGBA
        gl.clear_depth(1.);

        let max_combined_tex_units = gl.get_parameter(GL::MAX_COMBINED_TEXTURE_IMAGE_UNITS);
        let max_vert_tex_units = gl.get_parameter(GL::MAX_VERTEX_TEXTURE_IMAGE_UNITS);
        let max_frag_tex_units = gl.get_parameter(GL::MAX_TEXTURE_IMAGE_UNITS);
        log::info!(
            "Max texture units: Vertex shader: {:?}, Fragment shader: {:?}, Combined: {:?}",
            max_vert_tex_units,
            max_frag_tex_units,
            max_combined_tex_units
        );

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

        let mut engine = Engine::new(gl.clone());

        let app_state: Arc<Mutex<AppState>> = AppState::new();
        {
            let mut app_state_mut = &mut *app_state.lock().unwrap();
            app_state_mut.gl = Some(gl);
            app_state_mut.phys_time_ms = 0.0;
            app_state_mut.real_time_ms = 0.0;
        }

        EngineInterface::setup_event_handlers(&canvas, app_state.clone());

        engine.res_man.initialize();

        return EngineInterface {
            engine,
            app_state,
            start_ms: js_sys::Date::now(),
            last_frame_ms: 0.0,
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
                    0 => {
                        // Don't revert back to "pressed" if it's already handled
                        if state.input.m0 == ButtonState::Depressed {
                            state.input.m0 = ButtonState::Pressed;
                        }
                    }

                    // 1 is the mouse wheel click
                    2 => {
                        state.input.m1 = ButtonState::Pressed;
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
                if state.input.m1 == ButtonState::Pressed {
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
                    0 => state.input.m0 = ButtonState::Depressed,

                    // 1 is the mouse wheel click
                    2 => {
                        state.input.m1 = ButtonState::Depressed;

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
                        state.input.forward = ButtonState::Pressed;
                    }
                    "KeyA" | "ArrowLeft" => {
                        state.input.left = ButtonState::Pressed;
                    }
                    "KeyS" | "ArrowDown" => {
                        state.input.back = ButtonState::Pressed;
                    }
                    "KeyD" | "ArrowRight" => {
                        state.input.right = ButtonState::Pressed;
                    }
                    "KeyE" => {
                        state.input.up = ButtonState::Pressed;
                    }
                    "KeyQ" => {
                        state.input.down = ButtonState::Pressed;
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
                        state.input.forward = ButtonState::Depressed;
                    }
                    "KeyA" | "ArrowLeft" => {
                        state.input.left = ButtonState::Depressed;
                    }
                    "KeyS" | "ArrowDown" => {
                        state.input.back = ButtonState::Depressed;
                    }
                    "KeyD" | "ArrowRight" => {
                        state.input.right = ButtonState::Depressed;
                    }
                    "KeyE" => {
                        state.input.up = ButtonState::Depressed;
                    }
                    "KeyQ" => {
                        state.input.down = ButtonState::Depressed;
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

    pub fn load_texture(&mut self, file_identifier: &str, data: &mut [u8]) {
        log::info!(
            "Loading texture from file '{}' ({} bytes)",
            file_identifier,
            data.len()
        );

        self.engine
            .res_man
            .create_texture(file_identifier, data, None);
    }

    pub fn load_gltf(&mut self, file_identifier: &str, data: &mut [u8]) {
        log::info!(
            "Loading GLTF from file '{}' ({} bytes)",
            file_identifier,
            data.len()
        );

        // TODO: Catch duplicate scenes

        if let Ok((gltf_doc, gltf_buffers, gltf_images)) = gltf::import_slice(data) {
            self.engine.res_man.load_textures_from_gltf(
                file_identifier,
                gltf_doc.textures(),
                &gltf_images,
            );

            let mat_index_to_parsed = self
                .engine
                .res_man
                .load_materials_from_gltf(file_identifier, gltf_doc.materials());

            self.engine.res_man.load_meshes_from_gltf(
                file_identifier,
                gltf_doc.meshes(),
                &gltf_buffers,
                &mat_index_to_parsed,
            );

            self.engine.scene_man.load_scenes_from_gltf(
                file_identifier,
                gltf_doc.scenes(),
                &self.engine.res_man,
            );
        }
    }

    fn temp_add_ellipse(&mut self, scene_name: &str, name: &str, elements: &OrbitalElements) {
        let scene = self.engine.scene_man.get_scene_mut(scene_name).unwrap();

        let orbit_transform = elements_to_circle_transform(&elements);
        log::warn!("orbit transform: {:#?}", orbit_transform);

        // Orbit
        let circle = scene.ent_man.new_entity(Some(&name));
        let trans_comp = scene
            .ent_man
            .add_component::<TransformComponent>(circle)
            .unwrap();
        *trans_comp.get_local_transform_mut() = orbit_transform;
        let mesh_comp = scene
            .ent_man
            .add_component::<MeshComponent>(circle)
            .unwrap();
        mesh_comp.set_mesh(self.engine.res_man.get_or_create_mesh("circle"));
    }

    fn load_ephemerides_internal(
        &mut self,
        file_name: &str,
        file_data: &str,
    ) -> Result<(), String> {
        log::info!(
            "load_ephemerides_internal, name: {}, data: {}",
            file_name,
            file_data
        );

        // let (elements, body) = parse_ephemerides(file_data)?;
        // log::info!(
        //     "Loaded ephemerides '{}'\n{:#?}\n{:#?}",
        //     file_name,
        //     body,
        //     elements
        // );

        // let orbit_transform = elements_to_circle_transform(&elements);

        // let scene = self
        //     .engine
        //     .scene_man
        //     .new_scene(file_name)
        //     .ok_or("Failed to create new scene!")?;

        // let planet_mat = self.engine.res_man.instantiate_material("gltf_metal_rough");
        // planet_mat.as_ref().unwrap().borrow_mut().name = String::from("planet_mat");
        // planet_mat.as_ref().unwrap().borrow_mut().set_uniform_value(
        //     UniformName::BaseColorFactor,
        //     UniformValue::Vec4([0.1, 0.8, 0.2, 1.0]),
        // );

        // // Lat-long sphere
        // let lat_long = scene.ent_man.new_entity(Some(&body.id));
        // let trans_comp = scene
        //     .ent_man
        //     .add_component::<TransformComponent>(lat_long)
        //     .unwrap();
        // trans_comp.get_local_transform_mut().trans = Vector3::new(10.0, 0.0, 0.0);
        // trans_comp.get_local_transform_mut().scale = Vector3::new(
        //     body.mean_radius as f32,
        //     body.mean_radius as f32,
        //     body.mean_radius as f32,
        // );
        // let mesh_comp = scene
        //     .ent_man
        //     .add_component::<MeshComponent>(lat_long)
        //     .unwrap();
        // mesh_comp.set_mesh(self.engine.res_man.get_or_create_mesh("lat_long_sphere"));
        // mesh_comp.set_material_override(planet_mat.clone(), 0);

        // self.temp_add_ellipse(
        //     file_name,
        //     "first",
        //     &OrbitalElements {
        //         semi_major_axis: 1000.0,
        //         eccentricity: 0.0,
        //         arg_periapsis: 0.0,
        //         inclination: 0.0,
        //         long_asc_node: 0.0,
        //         true_anomaly: 0.0,
        //     },
        // );

        // self.temp_add_ellipse(
        //     file_name,
        //     "second",
        //     &OrbitalElements {
        //         semi_major_axis: 1000.0,
        //         eccentricity: 0.9,
        //         arg_periapsis: 0.0,
        //         inclination: 0.0,
        //         long_asc_node: 0.0,
        //         true_anomaly: 0.0,
        //     },
        // );

        // self.temp_add_ellipse(
        //     file_name,
        //     "third",
        //     &OrbitalElements {
        //         semi_major_axis: 1000.0,
        //         eccentricity: 0.9,
        //         arg_periapsis: 0.0,
        //         inclination: 30.0,
        //         long_asc_node: 0.0,
        //         true_anomaly: 0.0,
        //     },
        // );

        // self.temp_add_ellipse(
        //     file_name,
        //     "third",
        //     &OrbitalElements {
        //         semi_major_axis: 1000.0,
        //         eccentricity: 0.9,
        //         arg_periapsis: 0.0,
        //         inclination: 30.0,
        //         long_asc_node: 45.0,
        //         true_anomaly: 0.0,
        //     },
        // );

        // self.temp_add_ellipse(
        //     file_name,
        //     "fourth",
        //     &OrbitalElements {
        //         semi_major_axis: 1000.0,
        //         eccentricity: 0.9,
        //         arg_periapsis: 30.0,
        //         inclination: 30.0,
        //         long_asc_node: 45.0,
        //         true_anomaly: 0.0,
        //     },
        // );

        return Ok(());
    }

    pub fn load_ephemerides(&mut self, file_name: &str, file_data: &str) {
        match self.load_ephemerides_internal(file_name, file_data) {
            Ok(_) => {
                log::info!("Loaded ephemerides from file '{}'", file_name);
            }
            Err(err) => {
                log::error!("Error when loading ephemerides:\n{}", err);
            }
        }
    }

    pub fn load_test_scene(&mut self) {
        self.engine
            .scene_man
            .load_test_scene("test", &mut self.engine.res_man);
        self.engine.scene_man.set_scene("test");
    }

    pub fn update(&mut self, width: u32, height: u32) {
        let mut app_state_mut = &mut *self.app_state.lock().unwrap();

        let now_ms = js_sys::Date::now() - self.start_ms;
        let real_delta_ms = now_ms - self.last_frame_ms;
        let phys_delta_ms = real_delta_ms * app_state_mut.simulation_speed;
        self.last_frame_ms = now_ms;

        app_state_mut.canvas_height = height;
        app_state_mut.canvas_width = width;
        app_state_mut.phys_time_ms += phys_delta_ms;
        app_state_mut.real_time_ms += real_delta_ms;
        app_state_mut.phys_delta_time_ms = phys_delta_ms;
        app_state_mut.real_delta_time_ms = real_delta_ms;

        self.engine.update(app_state_mut);
    }
}

#[wasm_bindgen]
pub async fn run() {
    log::info!("Beginning engine loop...");

    // log::info!("Before fetch");
    // crate::utils::web::test_fetch("./public/ephemerides/test.txt".to_owned()).await;
    // log::info!("After fetch");

    EI.with(|e| {
        let mut e = e.borrow_mut();

        e.load_test_scene();

        // e.engine
        //     .scene_man
        //     .inject_scene(
        //         "./public/ephemerides/3@sun.txt",
        //         Some(Transform {
        //             trans: Vector3::new(0.0, 0.0, 0.0),
        //             rot: UnitQuaternion::new_unchecked(Quaternion::identity()),
        //             scale: Vector3::new(1.0, 1.0, 1.0),
        //         }),
        //     )
        //     .expect("Failed to inject scene!");
    });

    let event_loop = EventLoop::new();

    let canvas = get_canvas();

    let window = WindowBuilder::new()
        .with_title("Title")
        .with_canvas(Some(canvas))
        .build(&event_loop)
        .expect("Failed to find window!");

    // Get a new one as we need to move it into the window builder for some reason
    let canvas = window.canvas();

    let style = canvas.style();
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
                        "Resized to engine: {}, h: {}",
                        canvas_width_on_screen,
                        canvas_height_on_screen
                    );
                }

                EI.with(|e_inner| {
                    let mut e_inner = e_inner.borrow_mut();

                    e_inner.update(canvas_width_on_screen, canvas_height_on_screen);
                });
            }

            Event::NewEvents(_) => {}
            Event::DeviceEvent {
                device_id: _,
                event: _,
            } => {}
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

#[wasm_bindgen]
pub fn load_ephemerides_external(file_name: &str, file_data: &str) {
    EI.with(|e_inner| {
        let mut e_inner = e_inner.borrow_mut();

        match e_inner.load_ephemerides_internal(file_name, file_data) {
            Ok(_) => {
                log::info!("Loaded ephemerides from file '{}'", file_name);
            }
            Err(err) => {
                log::error!("Error when loading ephemerides:\n{}", err);
            }
        }
    });
}
