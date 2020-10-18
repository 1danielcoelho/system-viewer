extern crate wasm_bindgen;
use std::time::Duration;

use imgui_winit_support::{HiDpiMode, WinitPlatform};
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
mod programs;
mod shaders;

#[macro_export]
macro_rules! glc {
    ($ctx:expr, $any:expr) => {
        unsafe {
            #[cfg(debug_assertions)]
            while $ctx.get_error() != 0 {}
            let out = $any;
            #[cfg(debug_assertions)]
            while match $ctx.get_error() {
                0 => false,
                err => {
                    log::error!("[OpenGL Error] {}", err);
                    true
                }
            } {}
            out
        }
    };
}

#[wasm_bindgen(start)]
pub fn main() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));

    console_log::init_with_level(log::Level::Debug).expect("Unable to initialize console logging");
}

#[wasm_bindgen]
pub struct Viewer {
    context: web_sys::WebGlRenderingContext,
    window: winit::window::Window,
    canvas: web_sys::HtmlCanvasElement,
    platform: WinitPlatform,
    imgui: imgui::Context,
    event_loop: EventLoop<()>,
    _program_color_2d: programs::Color2D,
    _program_color_2d_gradient: programs::Color2DGradient,
    program_graph_3d: programs::Graph3D,
}

#[wasm_bindgen]
impl Viewer {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        console_error_panic_hook::set_once();
        let (context, canvas) = gl_setup::initialize_webgl_context().unwrap();

        let event_loop = EventLoop::new();

        let window = WindowBuilder::new()
            .with_title("Title")
            .with_canvas(Some(canvas))
            .build(&event_loop)
            .expect("Failed to find window!");

        let canvas = window.canvas();
        canvas.set_oncontextmenu(Some(&js_sys::Function::new_with_args(
            "ev",
            r"
                ev.preventDefault();
                return false;
            ",
        )));

        let mut imgui = imgui::Context::create();
        let mut platform = WinitPlatform::init(&mut imgui);
        platform.attach_window(imgui.io_mut(), &window, HiDpiMode::Default);

        Self {
            _program_color_2d: programs::Color2D::new(&context),
            _program_color_2d_gradient: programs::Color2DGradient::new(&context),
            program_graph_3d: programs::Graph3D::new(&context),
            context,
            canvas,
            window,
            event_loop,
            platform,
            imgui,
        }
    }

    #[wasm_bindgen]
    pub fn start(&mut self) -> ! {
        self.event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Poll; // Can change this to Wait to pause when no input is given

            match event {
                Event::NewEvents(_) => self.imgui.io_mut().update_delta_time(Duration::from_millis(1)),

                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    window_id,
                } if window_id == self.window.id() => *control_flow = ControlFlow::Exit,

                Event::MainEventsCleared => {
                    self.window.request_redraw();
                }

                Event::RedrawRequested(window_id) if window_id == self.window.id() => {
                    if let Err(err) = self.platform.prepare_frame(self.imgui.io_mut(), &self.window) {
                        log::error!("{}", err);
                        // TODO: check if error is recoverable
                        *control_flow = ControlFlow::Exit;
                    }

                    let ctx = &self.context; 

                    glc!(ctx, ctx.clear_color(0.5, 0.2, 0.2, 1.0));
                    glc!(ctx, ctx.clear(GL::COLOR_BUFFER_BIT | GL::DEPTH_BUFFER_BIT));

                    let curr_state = app_state::get_curr_state();

                    // self.program_color_2d.render(
                    //     &self.context,
                    //     curr_state.control_bottom,
                    //     curr_state.control_top,
                    //     curr_state.control_left,
                    //     curr_state.control_right,
                    //     curr_state.canvas_height,
                    //     curr_state.canvas_width,
                    // );

                    // self.program_color_2d_gradient.render(
                    //     &self.context,
                    //     curr_state.control_bottom + 20.,
                    //     curr_state.control_top - 20.,
                    //     curr_state.control_left + 20.,
                    //     curr_state.control_right - 20.,
                    //     curr_state.canvas_height,
                    //     curr_state.canvas_width,
                    // );

                    let ui = self.imgui.frame();

                    // if let Err(err) = renderer.update(&meta, &handle) {
                    //     log::error!("{}", err);
                    //     // TODO: check if error is recoverable
                    //     *control_flow = ControlFlow::Exit;
                    // }

                    self.platform.prepare_render(&ui, &self.window);

                    self.program_graph_3d.render(
                        &self.context,
                        curr_state.control_bottom,
                        curr_state.control_top,
                        curr_state.control_left,
                        curr_state.control_right,
                        curr_state.canvas_height,
                        curr_state.canvas_width,
                        curr_state.rotation_x_axis,
                        curr_state.rotation_y_axis,
                        &common_funcs::get_updated_3d_y_values(curr_state.time),
                    );

                    let dd = ui.render();
                    let fbw = (dd.display_size[0] * dd.framebuffer_scale[0]) as i32;
                    let fbh = (dd.display_size[1] * dd.framebuffer_scale[1]) as i32;
                    if fbw <= 0 || fbh <= 0 {
                        log::error!("Weird result out of imgui");
                    }
                }

                event => { 
                    self.platform.handle_event(self.imgui.io_mut(), &self.window, &event);
                    // TODO: Handle input events
                }
            }
        });
    }
}
