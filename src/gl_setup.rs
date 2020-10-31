use std::sync::{Arc, Mutex};

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use web_sys::WebGlRenderingContext as GL;
use web_sys::*;

use crate::app_state::AppState;

pub fn initialize_webgl_context(
) -> Result<(WebGlRenderingContext, web_sys::HtmlCanvasElement), JsValue> {
    let window = window().unwrap();
    let document = window.document().unwrap();
    let canvas = document.get_element_by_id("rustCanvas").unwrap();
    let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into::<web_sys::HtmlCanvasElement>()?;
    let gl: WebGlRenderingContext = canvas.get_context("webgl")?.unwrap().dyn_into()?;

    gl.enable(GL::BLEND);
    gl.blend_func(GL::SRC_ALPHA, GL::ONE_MINUS_SRC_ALPHA);

    gl.enable(GL::CULL_FACE);
    gl.cull_face(GL::BACK);

    gl.clear_color(0.0, 0.0, 0.0, 1.0); //RGBA
    gl.clear_depth(1.);

    Ok((gl, canvas))
}

pub fn setup_event_handlers(canvas: &HtmlCanvasElement, app_state: Arc<Mutex<AppState>>) {
    // mousedown
    {
        let app_state_clone = app_state.clone();
        let canvas_clone = canvas.clone();
        let handler = move |event: web_sys::MouseEvent| {
            let app_state_mut = &mut *app_state_clone.lock().unwrap();
            match event.button() as i16 {
                0 => app_state_mut.input.m0_down = true,

                // 1 is the mouse wheel click
                2 => {
                    app_state_mut.input.m1_down = true;
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
            let app_state_mut = &mut *app_state_clone.lock().unwrap();

            // With pointer lock client_x and client_y don't actually change, so we need movement_*
            if app_state_mut.input.m1_down {
                app_state_mut.input.mouse_x += event.movement_x();
                app_state_mut.input.mouse_y += event.movement_y();
            } else {
                app_state_mut.input.mouse_x = event.client_x();
                app_state_mut.input.mouse_y = event.client_y();
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
            let app_state_mut = &mut *app_state_clone.lock().unwrap();
            match event.button() as i16 {
                0 => app_state_mut.input.m0_down = false,

                // 1 is the mouse wheel click
                2 => {
                    app_state_mut.input.m1_down = false;

                    // Release pointer lock
                    let window = window().unwrap();
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
            let app_state_mut = &mut *app_state_clone.lock().unwrap();

            if event.delta_y() < 0.0 {
                app_state_mut.move_speed *= 1.1;
            } else {
                app_state_mut.move_speed *= 0.9;
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
            let app_state_mut = &mut *app_state_clone.lock().unwrap();
            match (event.code() as String).as_str() {
                "KeyW" | "ArrowUp" => {
                    app_state_mut.input.forward_down = true;
                }
                "KeyA" | "ArrowLeft" => {
                    app_state_mut.input.left_down = true;
                }
                "KeyS" | "ArrowDown" => {
                    app_state_mut.input.back_down = true;
                }
                "KeyD" | "ArrowRight" => {
                    app_state_mut.input.right_down = true;
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
            let app_state_mut = &mut *app_state_clone.lock().unwrap();
            match (event.code() as String).as_str() {
                "KeyW" | "ArrowUp" => {
                    app_state_mut.input.forward_down = false;
                }
                "KeyA" | "ArrowLeft" => {
                    app_state_mut.input.left_down = false;
                }
                "KeyS" | "ArrowDown" => {
                    app_state_mut.input.back_down = false;
                }
                "KeyD" | "ArrowRight" => {
                    app_state_mut.input.right_down = false;
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
