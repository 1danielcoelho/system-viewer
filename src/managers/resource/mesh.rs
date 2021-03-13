use crate::managers::resource::collider::Collider;
use crate::managers::resource::intermediate_mesh::IntermediatePrimitive;
use crate::managers::resource::material::Material;
use crate::utils::gl::GL;
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

#[derive(Debug)]
pub struct Mesh {
    pub name: String,
    pub primitives: Vec<Primitive>,
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

    pub has_tangents: bool,
    pub has_normals: bool,
    pub has_colors: bool,
    pub has_uv0: bool,
    pub has_uv1: bool,

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
