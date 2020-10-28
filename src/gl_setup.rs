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
        let handler = move |event: web_sys::MouseEvent| {
            let app_state_mut = &mut *app_state_clone.lock().unwrap();
            app_state_mut.mouse_down = true;
        };
    
        let handler = Closure::wrap(Box::new(handler) as Box<dyn FnMut(_)>);
        canvas.add_event_listener_with_callback("mousedown", handler.as_ref().unchecked_ref()).expect("Failed to set mousedown event handler");
        handler.forget();
    }

    // mousemove
    {
        let app_state_clone = app_state.clone();
        let handler = move |event: web_sys::MouseEvent| {
            let app_state_mut = &mut *app_state_clone.lock().unwrap();
            app_state_mut.mouse_x = event.client_x();
            app_state_mut.mouse_y = event.client_y();
        };
    
        let handler = Closure::wrap(Box::new(handler) as Box<dyn FnMut(_)>);
        canvas.add_event_listener_with_callback("mousemove", handler.as_ref().unchecked_ref()).expect("Failed to set mousemove event handler");
        handler.forget();
    }

    // mouseup
    {
        let app_state_clone = app_state.clone();
        let handler = move |event: web_sys::MouseEvent| {
            let app_state_mut = &mut *app_state_clone.lock().unwrap();
            app_state_mut.mouse_down = false;
        };
    
        let handler = Closure::wrap(Box::new(handler) as Box<dyn FnMut(_)>);
        canvas.add_event_listener_with_callback("mouseup", handler.as_ref().unchecked_ref()).expect("Failed to set mouseup event handler");
        handler.forget();
    }
}
