use crate::wasm_bindgen::JsCast;
use js_sys::encode_uri_component;
use wasm_bindgen::prelude::Closure;
use web_sys::{FileReader, HtmlElement, HtmlInputElement};

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

pub fn read_string_from_file_prompt() -> String {
    let window = web_sys::window().expect("no global `window` exists");
    let document = window.document().expect("should have a document on window");

    let el = document.create_element("input").unwrap();
    el.set_id("temp_input");

    let html_el = el.dyn_ref::<HtmlInputElement>().unwrap();
    html_el.set_type("file");
    html_el.set_accept(".json");

    let on_load = move |event: web_sys::Event| {
        log::info!(
            "on_load called! {:?}",
            event
                .target()
                .unwrap()
                .dyn_ref::<FileReader>()
                .unwrap()
                .result()
                .unwrap()
                .as_string()
                .unwrap()
        );
    };
    let on_load_closure = Closure::wrap(Box::new(on_load) as Box<dyn FnMut(_)>);

    let handler = move |event: web_sys::Event| {
        let window = web_sys::window().expect("no global `window` exists");
        let document = window.document().expect("should have a document on window");

        let el = document
            .get_element_by_id("temp_input")
            .expect("should have temp_input element on the page");

        let input = el.dyn_ref::<HtmlInputElement>().unwrap();
        if let Some(files) = input.files() {
            for i in 0..files.length() {
                let file = files.get(i).unwrap();

                let reader = web_sys::FileReader::new().unwrap();
                reader.set_onload(Some(on_load_closure.as_ref().unchecked_ref()));

                reader.read_as_text(&file);
            }
        }
    };

    let body = document.body().expect("document should have a body");
    body.append_child(&html_el);

    let handler = Closure::wrap(Box::new(handler) as Box<dyn FnMut(_)>);
    html_el
        .add_event_listener_with_callback("change", handler.as_ref().unchecked_ref())
        .expect("Failed to set mousedown event handler");
    handler.forget();

    html_el.click();

    return String::new();
}
