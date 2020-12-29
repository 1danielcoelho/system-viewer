use crate::{
    managers::resource::{
        collider::Collider, intermediate_mesh::IntermediatePrimitive, material::Material,
    },
    utils::gl::GL,
};
use serde::{Deserialize, Serialize};
use std::{cell::RefCell, rc::Rc};
use web_sys::WebGl2RenderingContext;
use web_sys::WebGlVertexArrayObject;

pub enum PrimitiveAttribute {
    Position = 0,
    Normal = 1,
    Tangent = 2,
    Color = 3,
    UV0 = 4,
    UV1 = 5,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Mesh {
    pub name: String,

    // Set to true when we fill in our primitives. Always initially false when deserializing,
    // so that somebody tries loading a mesh with or name
    #[serde(skip)]
    pub loaded: bool,

    #[serde(skip)]
    pub primitives: Vec<Primitive>,

    #[serde(skip)]
    pub collider: Option<Box<dyn Collider>>,
}
impl PartialEq for Mesh {
    fn eq(&self, other: &Self) -> bool {
        return self.name == other.name;
    }
}

#[derive(Debug)]
pub struct Primitive {
    pub name: String,

    pub index_count: i32,
    pub mode: u32,
    pub vao: WebGlVertexArrayObject,

    // We keep these around sometimes in case this mesh is used as a collider
    pub source_data: Option<IntermediatePrimitive>,

    pub default_material: Option<Rc<RefCell<Material>>>,
}

impl Primitive {
    pub fn draw(&self, ctx: &WebGl2RenderingContext) {
        ctx.bind_vertex_array(Some(&self.vao));
        ctx.draw_elements_with_i32(self.mode, self.index_count, GL::UNSIGNED_SHORT, 0);
        ctx.bind_vertex_array(None);
    }
}
