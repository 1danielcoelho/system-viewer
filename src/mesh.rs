use std::rc::Rc;

use js_sys::WebAssembly;
use wasm_bindgen::JsCast;
use web_sys::WebGlRenderingContext as GL;
use web_sys::{WebGlBuffer, WebGlRenderingContext};

use crate::resources::ResourceManager;

pub struct Mesh {
    pub id: u32,
    pub name: String,
    pub position_buffer: WebGlBuffer,
    pub color_buffer: WebGlBuffer,
    pub indices_buffer: WebGlBuffer,
    pub index_count: i32,
    pub element_type: u32,
}

impl Mesh {
    pub fn draw(&self, ctx: &WebGlRenderingContext) {
        // Bind vertex buffer
        ctx.bind_buffer(GL::ARRAY_BUFFER, Some(&self.position_buffer));
        ctx.vertex_attrib_pointer_with_i32(0, 3, GL::FLOAT, false, 0, 0);

        // Bind color buffer
        ctx.bind_buffer(GL::ARRAY_BUFFER, Some(&self.color_buffer));
        ctx.vertex_attrib_pointer_with_i32(1, 3, GL::FLOAT, false, 0, 0);

        // Bind index buffer
        ctx.bind_buffer(GL::ELEMENT_ARRAY_BUFFER, Some(&self.indices_buffer));

        // Draw
        ctx.draw_elements_with_i32(self.element_type, self.index_count, GL::UNSIGNED_SHORT, 0);
    }
}
