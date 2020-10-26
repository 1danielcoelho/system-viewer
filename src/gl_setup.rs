use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use web_sys::WebGlRenderingContext as GL;
use web_sys::*;

pub fn initialize_webgl_context(
) -> Result<(WebGlRenderingContext, web_sys::HtmlCanvasElement), JsValue> {
    let window = window().unwrap();
    let document = window.document().unwrap();
    let canvas = document.get_element_by_id("rustCanvas").unwrap();
    let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into::<web_sys::HtmlCanvasElement>()?;
    let gl: WebGlRenderingContext = canvas.get_context("webgl")?.unwrap().dyn_into()?;

    // attach_mouse_down_handler(&canvas)?;
    // attach_mouse_up_handler(&canvas)?;
    // attach_mouse_move_handler(&canvas)?;

    gl.enable(GL::BLEND);
    gl.blend_func(GL::SRC_ALPHA, GL::ONE_MINUS_SRC_ALPHA);

    gl.enable(GL::CULL_FACE);
    gl.cull_face(GL::BACK);

    gl.clear_color(0.0, 0.0, 0.0, 1.0); //RGBA
    gl.clear_depth(1.);

    Ok((gl, canvas))
}

// fn attach_mouse_down_handler(canvas: &HtmlCanvasElement) -> Result<(), JsValue> {
//     let handler = move |event: web_sys::MouseEvent| {
//         super::app_state::update_mouse_down(event.client_x() as f32, event.client_y() as f32, true);
//     };

//     let handler = Closure::wrap(Box::new(handler) as Box<dyn FnMut(_)>);
//     canvas.add_event_listener_with_callback("mousedown", handler.as_ref().unchecked_ref())?;
//     handler.forget();

//     Ok(())
// }

// fn attach_mouse_up_handler(canvas: &HtmlCanvasElement) -> Result<(), JsValue> {
//     let handler = move |event: web_sys::MouseEvent| {
//         super::app_state::update_mouse_down(event.client_x() as f32, event.client_y() as f32, false);
//     };

//     let handler = Closure::wrap(Box::new(handler) as Box<dyn FnMut(_)>);
//     canvas.add_event_listener_with_callback("mouseup", handler.as_ref().unchecked_ref())?;
//     handler.forget();

//     Ok(())
// }

// fn attach_mouse_move_handler(canvas: &HtmlCanvasElement) -> Result<(), JsValue> {
//     let handler = move |event: web_sys::MouseEvent| {
//         super::app_state::update_mouse_position(event.client_x() as f32, event.client_y() as f32);
//     };

//     let handler = Closure::wrap(Box::new(handler) as Box<dyn FnMut(_)>);
//     canvas.add_event_listener_with_callback("mousemove", handler.as_ref().unchecked_ref())?;
//     handler.forget();

//     Ok(())
// }
