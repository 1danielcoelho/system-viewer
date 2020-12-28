use crate::{
    app_state::AppState,
    components::MeshComponent,
    engine::Engine,
    utils::{
        orbital_elements::{elements_to_circle_transform, OrbitalElements},
        web::{get_canvas, setup_event_handlers},
    },
};
use crate::{app_state::ButtonState, components::TransformComponent};
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
    static EI: RefCell<EngineInterface> = RefCell::new(EngineInterface::new());
}

pub struct EngineInterface {
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

        let mut engine = Engine::new(gl.clone());

        let app_state: Arc<Mutex<AppState>> = AppState::new();
        {
            let mut app_state_mut = &mut *app_state.lock().unwrap();
            app_state_mut.gl = Some(gl);
            app_state_mut.phys_time_ms = 0.0;
            app_state_mut.real_time_ms = 0.0;
            app_state_mut.start_ms = js_sys::Date::now();
            app_state_mut.last_frame_ms = 0.0;
        }

        setup_event_handlers(&canvas, app_state.clone());

        engine.res_man.initialize();

        return EngineInterface { engine, app_state };
    }

    pub fn load_test_scene(&mut self) {
        self.engine
            .scene_man
            .load_test_scene("test", &mut self.engine.res_man);
        self.engine.scene_man.set_scene("test");
    }

    pub fn update(&mut self, width: u32, height: u32) {
        let mut app_state_mut = &mut *self.app_state.lock().unwrap();

        let now_ms = js_sys::Date::now() - app_state_mut.start_ms;
        let real_delta_ms = now_ms - app_state_mut.last_frame_ms;
        let phys_delta_ms = real_delta_ms * app_state_mut.simulation_speed;
        app_state_mut.last_frame_ms = now_ms;

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
    });

    let event_loop = EventLoop::new();

    // Give winit our canvas so that it can break its styling, then fix the styling
    let window = WindowBuilder::new()
        .with_title("Title")
        .with_canvas(Some(get_canvas()))
        .build(&event_loop)
        .expect("Failed to find window!");
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

                EI.with(|e| {
                    let mut e = e.borrow_mut();

                    e.update(canvas_width_on_screen, canvas_height_on_screen);
                });
            }
            _ => {}
        }
    });
}

fn receive_ephemerides_text(ei: &mut EngineInterface, file_name: &str, file_data: &str) {
    log::info!(
        "receive_ephemerides_text, name: {}, data: {}",
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
}

fn temp_add_ellipse(
    ei: &mut EngineInterface,
    scene_name: &str,
    name: &str,
    elements: &OrbitalElements,
) {
    let scene = ei.engine.scene_man.get_scene_mut(scene_name).unwrap();

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
    mesh_comp.set_mesh(ei.engine.res_man.get_or_create_mesh("circle"));
}

pub fn receive_texture_bytes(ei: &mut EngineInterface, file_identifier: &str, data: &mut [u8]) {
    log::info!(
        "Loading texture from file '{}' ({} bytes)",
        file_identifier,
        data.len()
    );

    ei.engine
        .res_man
        .create_texture(file_identifier, data, None);
}

pub fn receive_gltf_bytes(ei: &mut EngineInterface, file_identifier: &str, data: &mut [u8]) {
    log::info!(
        "Loading GLTF from file '{}' ({} bytes)",
        file_identifier,
        data.len()
    );

    // TODO: Catch duplicate scenes

    if let Ok((gltf_doc, gltf_buffers, gltf_images)) = gltf::import_slice(data) {
        ei.engine.res_man.load_textures_from_gltf(
            file_identifier,
            gltf_doc.textures(),
            &gltf_images,
        );

        let mat_index_to_parsed = ei
            .engine
            .res_man
            .load_materials_from_gltf(file_identifier, gltf_doc.materials());

        ei.engine.res_man.load_meshes_from_gltf(
            file_identifier,
            gltf_doc.meshes(),
            &gltf_buffers,
            &mat_index_to_parsed,
        );

        ei.engine.scene_man.load_scenes_from_gltf(
            file_identifier,
            gltf_doc.scenes(),
            &ei.engine.res_man,
        );
    }
}

/** Synchronous function that JS calls to inject bytes data into the engine because we can't await for a JS promise from within the winit engine loop */
#[wasm_bindgen]
pub fn receive_text(url: &str, content_type: &str, text: &str) {
    EI.with(|e| {
        let mut e = e.borrow_mut();

        match content_type {
            "ephemerides" => receive_ephemerides_text(&mut e, url, text),
            _ => log::error!(
                "Unexpected content_type for receive_text: '{}'. url: '{}'",
                content_type,
                url
            ),
        }
    });
}

/** Synchronous function that JS calls to inject text data into the engine because we can't await for a JS promise from within the winit engine loop */
#[wasm_bindgen]
pub fn receive_bytes(url: &str, content_type: &str, data: &mut [u8]) {
    EI.with(|e| {
        let mut e = e.borrow_mut();

        match content_type {
            "texture" => receive_texture_bytes(&mut e, url, data),
            "gltf" => receive_gltf_bytes(&mut e, url, data),
            _ => log::error!(
                "Unexpected content_type for receive bytes: '{}'. url: '{}'",
                content_type,
                url
            ),
        }
    });
}

#[wasm_bindgen(module = "/io.js")]
extern "C" {
    pub fn prompt_for_text_file(content_type: &str);
}
