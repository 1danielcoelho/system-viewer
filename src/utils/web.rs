use crate::wasm_bindgen::JsCast;
use futures::{
    channel::oneshot::{self, channel},
    executor::block_on,
};
use instant::Duration;
use js_sys::{encode_uri_component, Promise};
use wasm_bindgen::prelude::*;
use wasm_bindgen::{prelude::Closure, JsValue};
use wasm_bindgen_futures::{spawn_local, JsFuture};
use web_sys::{
    Event, File, FileReader, HtmlElement, HtmlInputElement, Request, RequestInit, RequestMode,
    Response,
};

#[wasm_bindgen(module = "/garbage.js")]
extern "C" {
    fn test_garbage();
}

pub fn load(event: &Event) -> Result<JsValue, JsValue> {
    let target = match event.target() {
        None => return Err(JsValue::NULL),
        Some(t) => t,
    };
    let file_reader: FileReader = target.dyn_into()?;
    let result = file_reader.result();
    return result;
}

pub fn read_file(file: &File) -> Promise {
    return Promise::new(&mut |resolve, reject| {
        let file_reader = match FileReader::new() {
            Ok(r) => r,
            Err(_) => return,
        };
        let onload = Closure::wrap(Box::new(move |event: Event| {
            match load(&event) {
                Ok(r) => {
                    log::info!("Finished loading!");
                    resolve.call1(&JsValue::NULL, &r)
                }
                Err(e) => reject.call1(&JsValue::NULL, &e),
            }
            .unwrap_or(JsValue::NULL);
        }) as Box<dyn FnMut(_)>);
        file_reader.set_onload(Some(onload.as_ref().unchecked_ref()));
        onload.forget();
        file_reader.read_as_text(&file).unwrap_or_default();
    });
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

async fn test(i: i32) {
    log::info!("test ran {}", i);
}

pub async fn test_fetch(url: String) {
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
}

fn start_http_call(url: String, tx: oneshot::Sender<String>) {
    log::info!("start_http_call");
    spawn_local(async move {
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
        match tx.send(actual_txt) {
            Ok(_) => log::info!("Sent text"),
            Err(e) => log::error!("Error when sending: {}", e),
        };
    });
}

pub fn read_string_from_file_prompt() -> String {
    // let (sender, mut receiver) = channel();

    test_garbage();

    // log::info!("Before http call");
    // start_http_call("./public/ephemerides/test.txt".to_owned(), sender);
    // log::info!("After http call");

    // let fut = async move {
    //     loop {
    //         log::info!("Loop");
    //         match receiver.try_recv() {
    //             Ok(data) => {
    //                 log::info!("Received data: {:?}", data);
    //             }
    //             Err(e) => {
    //                 log::error!("Error receiving: {}", e);
    //             }
    //         };
    //     }
    // };
    // spawn_local(fut);
    // log::info!("After recev");

    //wasm_bindgen_futures::spawn_local(async {});

    // let window = web_sys::window().expect("no global `window` exists");
    // let document = window.document().expect("should have a document on window");

    // let el = document.create_element("input").unwrap();
    // el.set_id("temp_input");

    // let html_el = el.dyn_ref::<HtmlInputElement>().unwrap();
    // html_el.set_type("file");
    // html_el.set_accept(".json");

    // let (sender, receiver) = channel();

    // let on_load = move |event: web_sys::Event| {
    //     log::info!(
    //         "on_load called! {:?}",
    //         event
    //             .target()
    //             .unwrap()
    //             .dyn_ref::<FileReader>()
    //             .unwrap()
    //             .result()
    //             .unwrap()
    //             .as_string()
    //             .unwrap()
    //     );

    //     match sender.send(2) {
    //         Ok(_) => log::info!("Sent signal to channel"),
    //         Err(e) => log::error!("Error when sending: {}", e),
    //     }
    // };
    // let on_load_closure = Closure::wrap(Box::new(on_load) as Box<dyn FnMut(_)>);

    // let handler = move |event: web_sys::Event| {
    //     let window = web_sys::window().expect("no global `window` exists");
    //     let document = window.document().expect("should have a document on window");

    //     let el = document
    //         .get_element_by_id("temp_input")
    //         .expect("should have temp_input element on the page");

    //     let input = el.dyn_ref::<HtmlInputElement>().unwrap();
    //     if let Some(files) = input.files() {
    //         for i in 0..files.length() {
    //             let file = files.get(i).unwrap();

    //             let reader = web_sys::FileReader::new().unwrap();
    //             reader.set_onload(Some(on_load_closure.as_ref().unchecked_ref()));

    //             reader.read_as_text(&file);
    //         }
    //     }
    // };

    // let body = document.body().expect("document should have a body");
    // body.append_child(&html_el);

    // let handler = Closure::wrap(Box::new(handler) as Box<dyn FnMut(_)>);
    // html_el
    //     .add_event_listener_with_callback("change", handler.as_ref().unchecked_ref())
    //     .expect("Failed to set mousedown event handler");
    // handler.forget();

    // html_el.click();

    // log::info!("Before waiting");
    // match receiver.recv() {
    //     Ok(data) => log::info!("Received data {}", data),
    //     Err(e) => log::error!("Error receiving: {}", e),
    // }
    // log::info!("After waiting");

    // log::info!("After spawn local");

    return String::new();
}
