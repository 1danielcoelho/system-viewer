use crate::{app_state::AppState, STATE};
use crate::{app_state::ButtonState, wasm_bindgen::JsCast};
use js_sys::{ArrayBuffer, Uint8Array};
use wasm_bindgen::prelude::Closure;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    HtmlCanvasElement, Request, RequestInit, RequestMode, Response, WebGl2RenderingContext,
};

const OUR_CANVAS_ID: &str = "rustCanvas";

pub fn get_canvas() -> HtmlCanvasElement {
    let window = web_sys::window().expect("no global `window` exists");
    let document = window.document().expect("should have a document on window");
    let el = document.get_element_by_id(OUR_CANVAS_ID).unwrap();
    let canvas: HtmlCanvasElement = el.dyn_into().unwrap();
    return canvas;
}

pub fn get_window() -> web_sys::Window {
    web_sys::window().expect("no global `window` exists")
}

pub fn request_animation_frame(f: &Closure<dyn FnMut()>) {
    get_window()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("failed to request animation frame");
}

pub fn get_gl_context() -> glow::Context {
    let canvas = get_canvas();

    let gl: WebGl2RenderingContext = canvas
        .get_context("webgl2")
        .unwrap()
        .unwrap()
        .dyn_into()
        .unwrap();

    return glow::Context::from_webgl2_context(gl);
}

pub async fn request_text(url: &str) -> Result<String, JsValue> {
    let mut opts = RequestInit::new();
    opts.method("GET");
    opts.mode(RequestMode::Cors);

    let request = Request::new_with_str_and_init(url, &opts)?;

    let resp_value = JsFuture::from(get_window().fetch_with_request(&request)).await?;

    // `resp_value` is a `Response` object.
    assert!(resp_value.is_instance_of::<Response>());
    let resp: Response = resp_value.dyn_into().unwrap();

    // Convert this other `Promise` into a rust `Future`.
    let text = JsFuture::from(resp.text()?).await?.as_string().unwrap();
    return Ok(text);
}

pub async fn request_bytes(url: &str) -> Result<Vec<u8>, JsValue> {
    let mut opts = RequestInit::new();
    opts.method("GET");
    opts.mode(RequestMode::Cors);

    let request = Request::new_with_str_and_init(url, &opts)?;

    let resp_value = JsFuture::from(get_window().fetch_with_request(&request)).await?;

    // `resp_value` is a `Response` object.
    assert!(resp_value.is_instance_of::<Response>());
    let resp: Response = resp_value.dyn_into().unwrap();

    let array_buffer_value = JsFuture::from(resp.array_buffer()?).await?;
    assert!(array_buffer_value.is_instance_of::<ArrayBuffer>());

    // TODO: This is probably copying more than needed
    let array_buffer: ArrayBuffer = array_buffer_value.dyn_into().unwrap();
    let u8array: Uint8Array = Uint8Array::new(&array_buffer);
    let vec: Vec<u8> = u8array.to_vec();
    return Ok(vec);
}

/// From https://github.com/emilk/egui/blob/650450bc3a01f8fe44ba89781597c3c8f60c2777/egui_web/src/lib.rs#L516
fn modifiers_from_event(event: &web_sys::KeyboardEvent) -> egui::Modifiers {
    egui::Modifiers {
        alt: event.alt_key(),
        ctrl: event.ctrl_key(),
        shift: event.shift_key(),

        // Ideally we should know if we are running or mac or not,
        // but this works good enough for now.
        mac_cmd: event.meta_key(),

        // Ideally we should know if we are running or mac or not,
        // but this works good enough for now.
        command: event.ctrl_key() || event.meta_key(),
    }
}

pub fn pos_from_mouse_event(canvas: &HtmlCanvasElement, event: &web_sys::MouseEvent) -> egui::Pos2 {
    let rect = canvas.get_bounding_client_rect();
    egui::Pos2 {
        x: event.client_x() as f32 - rect.left() as f32,
        y: event.client_y() as f32 - rect.top() as f32,
    }
}

pub fn button_from_mouse_event(event: &web_sys::MouseEvent) -> Option<egui::PointerButton> {
    match event.button() {
        0 => Some(egui::PointerButton::Primary),
        1 => Some(egui::PointerButton::Middle),
        2 => Some(egui::PointerButton::Secondary),
        _ => None,
    }
}

/// From https://github.com/emilk/egui/blob/650450bc3a01f8fe44ba89781597c3c8f60c2777/egui_web/src/lib.rs#L272
/// Web sends all keys as strings, so it is up to us to figure out if it is
/// a real text input or the name of a key.
fn should_ignore_key(key: &str) -> bool {
    let is_function_key = key.starts_with('F') && key.len() > 1;
    is_function_key
        || matches!(
            key,
            "Alt"
                | "ArrowDown"
                | "ArrowLeft"
                | "ArrowRight"
                | "ArrowUp"
                | "Backspace"
                | "CapsLock"
                | "ContextMenu"
                | "Control"
                | "Delete"
                | "End"
                | "Enter"
                | "Esc"
                | "Escape"
                | "Help"
                | "Home"
                | "Insert"
                | "Meta"
                | "NumLock"
                | "PageDown"
                | "PageUp"
                | "Pause"
                | "ScrollLock"
                | "Shift"
                | "Tab"
        )
}

// TODO: I feel like some of this should maybe be inside the input manager. I mean, there's no
// web stuff in this function at all
fn handle_key_press(key: &str, modifiers: &egui::Modifiers, s: &mut AppState, pressed: bool) {
    let button_state = if pressed {
        ButtonState::Pressed
    } else {
        ButtonState::Depressed
    };

    let mut egui_key: Option<egui::Key> = None;
    match key {
        "ArrowUp" => {
            s.input.forward = button_state;
            egui_key = Some(egui::Key::ArrowUp);
        }
        "ArrowLeft" => {
            s.input.left = button_state;
            egui_key = Some(egui::Key::ArrowLeft);
        }
        "ArrowDown" => {
            s.input.back = button_state;
            egui_key = Some(egui::Key::ArrowDown);
        }
        "ArrowRight" => {
            s.input.right = button_state;
            egui_key = Some(egui::Key::ArrowRight);
        }
        "w" | "W" => {
            s.input.forward = button_state;
            egui_key = Some(egui::Key::W);
        }
        "Backspace" => egui_key = Some(egui::Key::Backspace),
        "Delete" => egui_key = Some(egui::Key::Delete),
        "End" => egui_key = Some(egui::Key::End),
        "Enter" => egui_key = Some(egui::Key::Enter),
        "Space" | " " => {
            if button_state == ButtonState::Pressed && s.input.spacebar == ButtonState::Depressed {
                s.input.spacebar = ButtonState::Pressed;
            } else if button_state == ButtonState::Depressed {
                s.input.spacebar = ButtonState::Depressed;
            }

            egui_key = Some(egui::Key::Space);
        }
        "Esc" | "Escape" => {
            s.input.esc = button_state;
            egui_key = Some(egui::Key::Escape);
        }
        "Help" | "Insert" => egui_key = Some(egui::Key::Insert),
        "Home" => egui_key = Some(egui::Key::Home),
        "PageDown" => egui_key = Some(egui::Key::PageDown),
        "PageUp" => egui_key = Some(egui::Key::PageUp),
        "Tab" => egui_key = Some(egui::Key::Tab),
        "a" | "A" => {
            s.input.left = button_state;
            egui_key = Some(egui::Key::A);
        }
        "s" | "S" => {
            s.input.back = button_state;
        }
        "d" | "D" => {
            s.input.right = button_state;
        }
        "e" | "E" => {
            s.input.up = button_state;
        }
        "q" | "Q" => {
            s.input.down = button_state;
        }
        "k" | "K" => {
            egui_key = Some(egui::Key::K);
        }
        "u" | "U" => {
            egui_key = Some(egui::Key::U);
        }
        "z" | "Z" => {
            egui_key = Some(egui::Key::Z);
        }
        "f" | "F" => {
            s.input.f = button_state;
        }
        "g" | "G" => {
            s.input.g = button_state;
        }
        _ => {}
    };

    if let Some(key) = egui_key {
        s.input.egui_events.push(egui::Event::Key {
            key,
            pressed: pressed,
            modifiers: *modifiers,
        });
    }
}

/// Sets up the canvas event handlers to change the app_state blackboard
pub fn setup_event_handlers() {
    let canvas = get_canvas();
    let window = get_window();

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

                if let Some(button) = button_from_mouse_event(&event) {
                    let pos = pos_from_mouse_event(&canvas_clone, &event);

                    match button {
                        egui::PointerButton::Primary => {
                            // Don't revert back to "pressed" if it's already handled
                            if s.input.m0 == ButtonState::Depressed {
                                s.input.m0 = ButtonState::Pressed;
                            }

                            if s.input.modifiers.alt {
                                canvas_clone.request_pointer_lock();
                                s.input.mouse_x = event.client_x();
                                s.input.mouse_y = event.client_y();
                            }
                        }
                        egui::PointerButton::Secondary => {
                            s.input.m1 = ButtonState::Pressed;
                            canvas_clone.request_pointer_lock();
                            s.input.mouse_x = event.client_x();
                            s.input.mouse_y = event.client_y();
                        }
                        egui::PointerButton::Middle => {}
                    }

                    s.input.egui_events.push(egui::Event::PointerButton {
                        pos,
                        button,
                        pressed: true,
                        modifiers: s.input.modifiers,
                    });
                }
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
        let canvas_clone = canvas.clone();
        let handler = move |event: web_sys::MouseEvent| {
            STATE.with(|s| {
                let mut ref_mut = s.borrow_mut();
                let s = ref_mut.as_mut().unwrap();

                // Capture these during mouse move as other behavior depends on it (hiding labels, orbit, etc.)
                let modifiers = egui::Modifiers {
                    alt: event.alt_key(),
                    ctrl: event.ctrl_key(),
                    shift: event.shift_key(),
                    mac_cmd: event.meta_key(),
                    command: event.ctrl_key() || event.meta_key(),
                };
                s.input.modifiers = modifiers;

                let window = web_sys::window().unwrap();
                let doc = window.document().unwrap();

                // With pointer lock client_x and client_y don't actually change, so we need movement_*
                if let Some(_) = doc.pointer_lock_element() {
                    s.input.mouse_x += event.movement_x();
                    s.input.mouse_y += event.movement_y();
                } else {
                    s.input.mouse_x = event.client_x();
                    s.input.mouse_y = event.client_y();
                }

                let pos = pos_from_mouse_event(&canvas_clone, &event);
                s.input.egui_events.push(egui::Event::PointerMoved(pos));
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
        let canvas_clone = canvas.clone();
        let handler = move |event: web_sys::MouseEvent| {
            STATE.with(|s| {
                let mut ref_mut = s.borrow_mut();
                let s = ref_mut.as_mut().unwrap();

                if let Some(button) = button_from_mouse_event(&event) {
                    let pos = pos_from_mouse_event(&canvas_clone, &event);

                    match button {
                        egui::PointerButton::Primary => s.input.m0 = ButtonState::Depressed,
                        egui::PointerButton::Secondary => s.input.m1 = ButtonState::Depressed,
                        egui::PointerButton::Middle => {}
                    };

                    s.input.egui_events.push(egui::Event::PointerButton {
                        pos,
                        button,
                        pressed: false,
                        modifiers: s.input.modifiers,
                    });

                    // Release pointer lock
                    let window = web_sys::window().unwrap();
                    let doc = window.document().unwrap();
                    doc.exit_pointer_lock();
                }
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

                s.input.scroll_delta_x += event.delta_x() as i32;
                s.input.scroll_delta_y += event.delta_y() as i32;

                s.input.egui_events.push(egui::Event::Scroll {
                    0: egui::Vec2::new(event.delta_x() as f32 * 0.5, -event.delta_y() as f32 * 0.5),
                });

                event.stop_propagation();
                event.prevent_default();
            });
        };

        let handler = Closure::wrap(Box::new(handler) as Box<dyn FnMut(_)>);
        canvas
            .add_event_listener_with_callback("wheel", handler.as_ref().unchecked_ref())
            .expect("Failed to set mouseup event handler");
        handler.forget();
    }

    // keydown (some of this is copied from egui's web demo: https://github.com/emilk/egui/blob/650450bc3a01f8fe44ba89781597c3c8f60c2777/egui_web/src/lib.rs )
    {
        let handler = move |event: web_sys::KeyboardEvent| {
            STATE.with(|s| {
                let mut ref_mut = s.borrow_mut();
                let s = ref_mut.as_mut().unwrap();

                if event.is_composing() || event.key_code() == 229 {
                    // https://www.fxsitecompat.dev/en-CA/docs/2018/keydown-and-keyup-events-are-now-fired-during-ime-composition/
                    return;
                }

                let modifiers = modifiers_from_event(&event);
                s.input.modifiers = modifiers;

                let key = event.key();
                handle_key_press(&key, &modifiers, s, true);

                if !modifiers.ctrl && !modifiers.command && !should_ignore_key(&key) {
                    s.input.egui_events.push(egui::Event::Text(key.to_owned()));
                }

                if modifiers.alt
                    || matches!(
                        event.key().as_str(),
                        "Backspace"  // so we don't go back to previous page when deleting text
                    | "Tab" // so that e.g. tab doesn't move focus to url bar
                    )
                {
                    event.prevent_default();
                }
            });
        };

        let handler = Closure::wrap(Box::new(handler) as Box<dyn FnMut(_)>);
        window
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

                let modifiers = modifiers_from_event(&event);
                s.input.modifiers = modifiers;

                let key = event.key();
                handle_key_press(&key, &modifiers, s, false);

                if modifiers.alt {
                    event.prevent_default();
                }
            });
        };

        let handler = Closure::wrap(Box::new(handler) as Box<dyn FnMut(_)>);
        window
            .add_event_listener_with_callback("keyup", handler.as_ref().unchecked_ref())
            .expect("Failed to set keyup event handler");
        handler.forget();
    }
}

// From egui web backend
pub fn local_storage() -> Option<web_sys::Storage> {
    web_sys::window()?.local_storage().ok()?
}

pub fn local_storage_get(key: &str) -> Option<String> {
    local_storage().map(|storage| storage.get_item(key).ok())??
}

pub fn is_local_storage_enabled() -> bool {
    return local_storage_get("storage_ok").is_some();
}

pub fn local_storage_enable() {
    local_storage().map(|storage| storage.set_item("storage_ok", "true"));
}

pub fn local_storage_set(key: &str, value: &str) {
    if let Some(_) = local_storage_get("storage_ok") {
        local_storage().map(|storage| storage.set_item(key, value));
    }
}

pub fn local_storage_remove(key: &str) {
    local_storage().map(|storage| storage.remove_item(key));
}

pub fn local_storage_clear() {
    local_storage().map(|storage| storage.clear());
}
