use std::{cell::RefCell, rc::Rc};

use web_sys::WebGlRenderingContext as GL;
use web_sys::{WebGlBuffer, WebGlRenderingContext};

use super::Material;

#[repr(u32)]
pub enum PrimitiveAttribute {
    Position = 0,    
    Normal = 1,    
    Color = 2,    
    UV0 = 3,    
    UV1 = 4,    
}

pub struct Mesh {
    pub id: u32,
    pub name: String,
    pub primitives: Vec<Primitive>,
}

pub struct Primitive {
    pub name: String,

    pub index_count: i32,
    pub index_buffer: WebGlBuffer,
    pub position_buffer: WebGlBuffer,
    pub normal_buffer: WebGlBuffer,
    pub color_buffer: WebGlBuffer,
    pub uv0_buffer: WebGlBuffer,
    pub uv1_buffer: WebGlBuffer,

    pub mode: u32,

    pub default_material: Option<Rc<RefCell<Material>>>,
}

impl Primitive {
    pub fn draw(&self, ctx: &WebGlRenderingContext) {
        // Bind index buffer
        ctx.bind_buffer(GL::ELEMENT_ARRAY_BUFFER, Some(&self.index_buffer));

        // Bind vertex buffer
        ctx.bind_buffer(GL::ARRAY_BUFFER, Some(&self.position_buffer));
        ctx.vertex_attrib_pointer_with_i32(0, 3, GL::FLOAT, false, 0, 0);

        // Bind normal buffer
        ctx.bind_buffer(GL::ARRAY_BUFFER, Some(&self.normal_buffer));
        ctx.vertex_attrib_pointer_with_i32(1, 3, GL::FLOAT, false, 0, 0);

        // Bind color buffer
        ctx.bind_buffer(GL::ARRAY_BUFFER, Some(&self.color_buffer));
        ctx.vertex_attrib_pointer_with_i32(2, 4, GL::FLOAT, false, 0, 0);

        // Bind uv0 buffer
        ctx.bind_buffer(GL::ARRAY_BUFFER, Some(&self.uv0_buffer));
        ctx.vertex_attrib_pointer_with_i32(3, 2, GL::FLOAT, false, 0, 0);

        // Bind uv1 buffer
        ctx.bind_buffer(GL::ARRAY_BUFFER, Some(&self.uv0_buffer));
        ctx.vertex_attrib_pointer_with_i32(4, 2, GL::FLOAT, false, 0, 0);

        // Draw
        ctx.draw_elements_with_i32(self.mode, self.index_count, GL::UNSIGNED_SHORT, 0);
    }
}
