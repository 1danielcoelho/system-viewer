use std::sync::{Arc, Mutex};

use crate::{
    app_state::{AppState, ButtonState},
    wasm_bindgen::JsCast,
};
use js_sys::{encode_uri_component, Promise};
use wasm_bindgen::{prelude::Closure, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    Event, File, FileReader, HtmlCanvasElement, HtmlElement, Request, RequestInit, RequestMode,
    Response, WebGl2RenderingContext,
};

const OUR_CANVAS_ID: &str = "rustCanvas";

pub fn get_canvas() -> HtmlCanvasElement {
    let window = web_sys::window().expect("no global `window` exists");
    let document = window.document().expect("should have a document on window");
    let el = document.get_element_by_id(OUR_CANVAS_ID).unwrap();
    let canvas: HtmlCanvasElement = el.dyn_into().unwrap();
    return canvas;
}

pub fn get_gl_context() -> WebGl2RenderingContext {
    let canvas = get_canvas();
    let gl: WebGl2RenderingContext = canvas
        .get_context("webgl2")
        .unwrap()
        .unwrap()
        .dyn_into()
        .unwrap();

    return gl;
}

pub fn write_string_to_file_prompt(file_name: &str, data: &str) {
    let window = web_sys::window().expect("no global `window` exists");
    let document = window.document().expect("should have a document on window");

    let data_str =
        "data:text/json;charset=utf-8,".to_owned() + &String::from(encode_uri_component(data));

    let el = document.create_element("a").unwrap();
    let html_el = el.dyn_ref::<HtmlElement>().unwrap();
    html_el
        .set_attribute("href", &data_str)
        .expect("Failed to set href");
    html_el
        .set_attribute("download", file_name)
        .expect("Failed to set download");
    html_el.click();
}

pub async fn fetch_text(url: String) -> String {
    let mut opts = RequestInit::new();
    opts.method("GET");
    opts.mode(RequestMode::Cors);

    let request = Request::new_with_str_and_init(&url, &opts).unwrap();

    request.headers().set("Accept", "text/plain").unwrap();

    let window = web_sys::window().unwrap();
    let resp_value = JsFuture::from(window.fetch_with_request(&request))
        .await
        .unwrap();

    // `resp_value` is a `Response` object.
    assert!(resp_value.is_instance_of::<Response>());
    let resp: Response = resp_value.dyn_into().unwrap();

    // Convert this other `Promise` into a rust `Future`.
    let text = JsFuture::from(resp.text().unwrap()).await.unwrap();
    let actual_txt = format!("{}", text.as_string().unwrap());
    log::info!("Response text: {}", actual_txt);

    return actual_txt;
}

/** Sets up the canvas event handlers to change the app_state blackboard */
pub fn setup_event_handlers(canvas: &HtmlCanvasElement, app_state: Arc<Mutex<AppState>>) {
    canvas.set_oncontextmenu(Some(&js_sys::Function::new_with_args(
        "ev",
        r"ev.preventDefault();return false;",
    )));

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
