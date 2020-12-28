use crate::STATE;
use crate::{app_state::ButtonState, wasm_bindgen::JsCast};
use js_sys::encode_uri_component;
use wasm_bindgen::prelude::Closure;
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    HtmlCanvasElement, HtmlElement, Request, RequestInit, RequestMode, Response,
    WebGl2RenderingContext,
};

const OUR_CANVAS_ID: &str = "rustCanvas";

pub fn get_canvas() -> HtmlCanvasElement {
    let window = web_sys::window().expect("no global `window` exists");
    let document = window.document().expect("should have a document on window");
    let el = document.get_element_by_id(OUR_CANVAS_ID).unwrap();
    let canvas: HtmlCanvasElement = el.dyn_into().unwrap();
    return canvas;
}

pub fn get_gl_context(canvas: &HtmlCanvasElement) -> WebGl2RenderingContext {
    let gl: WebGl2RenderingContext = canvas
        .get_context("webgl2")
        .unwrap()
        .unwrap()
        .dyn_into()
        .unwrap();

    return gl;
}

pub fn force_full_canvas(canvas: &HtmlCanvasElement) {
    let style = canvas.style();
    style
        .set_property_with_priority("width", "100%", "")
        .expect("Failed to set width!");
    style
        .set_property_with_priority("height", "100%", "")
        .expect("Failed to set height!");
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
pub fn setup_event_handlers(canvas: &HtmlCanvasElement) {
    canvas.set_oncontextmenu(Some(&js_sys::Function::new_with_args(
        "ev",
        r"ev.preventDefault();return false;",
    )));

    // mousedown
    {
        let canvas_clone = canvas.clone();
        let handler = move |event: web_sys::MouseEvent| {
            STATE.with(|s| {
                let mut ref_mut = s.borrow_mut();
                let s = ref_mut.as_mut().unwrap();

                match event.button() as i16 {
                    0 => {
                        // Don't revert back to "pressed" if it's already handled
                        if s.input.m0 == ButtonState::Depressed {
                            s.input.m0 = ButtonState::Pressed;
                        }
                    }

                    // 1 is the mouse wheel click
                    2 => {
                        s.input.m1 = ButtonState::Pressed;
                        canvas_clone.request_pointer_lock();
                    }
                    _ => {}
                };
            });
        };

        let handler = Closure::wrap(Box::new(handler) as Box<dyn FnMut(_)>);
        canvas
            .add_event_listener_with_callback("mousedown", handler.as_ref().unchecked_ref())
            .expect("Failed to set mousedown event handler");
        handler.forget();
    }

    // mousemove
    {
        let handler = move |event: web_sys::MouseEvent| {
            STATE.with(|s| {
                let mut ref_mut = s.borrow_mut();
                let s = ref_mut.as_mut().unwrap();

                // With pointer lock client_x and client_y don't actually change, so we need movement_*
                if s.input.m1 == ButtonState::Pressed {
                    s.input.mouse_x += event.movement_x();
                    s.input.mouse_y += event.movement_y();
                } else {
                    s.input.mouse_x = event.client_x();
                    s.input.mouse_y = event.client_y();
                }
            });
        };

        let handler = Closure::wrap(Box::new(handler) as Box<dyn FnMut(_)>);
        canvas
            .add_event_listener_with_callback("mousemove", handler.as_ref().unchecked_ref())
            .expect("Failed to set mousemove event handler");
        handler.forget();
    }

    // mouseup
    {
        let handler = move |event: web_sys::MouseEvent| {
            STATE.with(|s| {
                let mut ref_mut = s.borrow_mut();
                let s = ref_mut.as_mut().unwrap();

                match event.button() as i16 {
                    0 => s.input.m0 = ButtonState::Depressed,

                    // 1 is the mouse wheel click
                    2 => {
                        s.input.m1 = ButtonState::Depressed;

                        // Release pointer lock
                        let window = web_sys::window().unwrap();
                        let doc = window.document().unwrap();
                        doc.exit_pointer_lock();
                    }
                    _ => {}
                };
            });
        };

        let handler = Closure::wrap(Box::new(handler) as Box<dyn FnMut(_)>);
        canvas
            .add_event_listener_with_callback("mouseup", handler.as_ref().unchecked_ref())
            .expect("Failed to set mouseup event handler");
        handler.forget();
    }

    // wheel
    {
        let handler = move |event: web_sys::WheelEvent| {
            STATE.with(|s| {
                let mut ref_mut = s.borrow_mut();
                let s = ref_mut.as_mut().unwrap();

                if event.delta_y() < 0.0 {
                    s.move_speed *= 1.1;
                } else {
                    s.move_speed *= 0.9;
                }
            });
        };

        let handler = Closure::wrap(Box::new(handler) as Box<dyn FnMut(_)>);
        canvas
            .add_event_listener_with_callback("wheel", handler.as_ref().unchecked_ref())
            .expect("Failed to set mouseup event handler");
        handler.forget();
    }

    // keydown
    {
        let handler = move |event: web_sys::KeyboardEvent| {
            STATE.with(|s| {
                let mut ref_mut = s.borrow_mut();
                let s = ref_mut.as_mut().unwrap();

                match (event.code() as String).as_str() {
                    "KeyW" | "ArrowUp" => {
                        s.input.forward = ButtonState::Pressed;
                    }
                    "KeyA" | "ArrowLeft" => {
                        s.input.left = ButtonState::Pressed;
                    }
                    "KeyS" | "ArrowDown" => {
                        s.input.back = ButtonState::Pressed;
                    }
                    "KeyD" | "ArrowRight" => {
                        s.input.right = ButtonState::Pressed;
                    }
                    "KeyE" => {
                        s.input.up = ButtonState::Pressed;
                    }
                    "KeyQ" => {
                        s.input.down = ButtonState::Pressed;
                    }
                    _ => {}
                };
            });
        };

        let handler = Closure::wrap(Box::new(handler) as Box<dyn FnMut(_)>);
        canvas
            .add_event_listener_with_callback("keydown", handler.as_ref().unchecked_ref())
            .expect("Failed to set keydown event handler");
        handler.forget();
    }

    // keyup
    {
        let handler = move |event: web_sys::KeyboardEvent| {
            STATE.with(|s| {
                let mut ref_mut = s.borrow_mut();
                let s = ref_mut.as_mut().unwrap();

                match (event.code() as String).as_str() {
                    "KeyW" | "ArrowUp" => {
                        s.input.forward = ButtonState::Depressed;
                    }
                    "KeyA" | "ArrowLeft" => {
                        s.input.left = ButtonState::Depressed;
                    }
                    "KeyS" | "ArrowDown" => {
                        s.input.back = ButtonState::Depressed;
                    }
                    "KeyD" | "ArrowRight" => {
                        s.input.right = ButtonState::Depressed;
                    }
                    "KeyE" => {
                        s.input.up = ButtonState::Depressed;
                    }
                    "KeyQ" => {
                        s.input.down = ButtonState::Depressed;
                    }
                    _ => {}
                };
            });
        };

        let handler = Closure::wrap(Box::new(handler) as Box<dyn FnMut(_)>);
        canvas
            .add_event_listener_with_callback("keyup", handler.as_ref().unchecked_ref())
            .expect("Failed to set keyup event handler");
        handler.forget();
    }
}
