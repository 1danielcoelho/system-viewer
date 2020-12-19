#![allow(dead_code)]

extern crate wasm_bindgen;

use wasm_bindgen::prelude::*;

mod app_state;
mod components;
mod engine;
mod engine_interface;
mod managers;
mod systems;
mod utils;

#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();

    console_log::init_with_level(log::Level::Debug).expect("Unable to initialize console logging");
}
