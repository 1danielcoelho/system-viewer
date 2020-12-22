use std::{cell::RefCell, rc::Rc};

use web_sys::WebGl2RenderingContext;
use web_sys::{WebGl2RenderingContext as GL, WebGlVertexArrayObject};

use super::Material;

#[repr(u32)]
pub enum PrimitiveAttribute {
    Position = 0,
    Normal = 1,
    Tangent = 2,
    Color = 3,
    UV0 = 4,
    UV1 = 5,
}

pub struct Mesh {
    pub id: u32,
    pub name: String,
    pub primitives: Vec<Primitive>,
}

pub struct Primitive {
    pub name: String,

    pub index_count: i32,
    pub mode: u32,
    pub vao: WebGlVertexArrayObject,

    pub default_material: Option<Rc<RefCell<Material>>>,
}

impl Primitive {
    pub fn draw(&self, ctx: &WebGl2RenderingContext) {
        ctx.bind_vertex_array(Some(&self.vao));
        ctx.draw_elements_with_i32(self.mode, self.index_count, GL::UNSIGNED_SHORT, 0);
        ctx.bind_vertex_array(None);
    }
}
