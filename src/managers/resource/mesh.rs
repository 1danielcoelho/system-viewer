use crate::managers::resource::collider::Collider;
use crate::managers::resource::intermediate_mesh::IntermediatePrimitive;
use crate::managers::resource::material::Material;
use crate::utils::gl::GL;
use js_sys::WebAssembly;
use std::borrow::BorrowMut;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::JsCast;
use web_sys::WebGlVertexArrayObject;
use web_sys::{WebGl2RenderingContext, WebGlBuffer};

pub enum PrimitiveAttribute {
    Position = 0,
    Normal = 1,
    Tangent = 2,
    Color = 3,
    UV0 = 4,
    UV1 = 5,
}

#[derive(Debug, Default)]
pub struct Mesh {
    pub name: String,
    pub primitives: Vec<Primitive>,
    pub dynamic_primitive: Option<DynamicPrimitive>,
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
    pub compatible_hash: u64,

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

    pub fn update_hash(&mut self) {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        self.compatible_hash = hasher.finish();
    }
}
impl Hash for Primitive {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.has_tangents.hash(state);
        self.has_normals.hash(state);
        self.has_colors.hash(state);
        self.has_uv0.hash(state);
        self.has_uv1.hash(state);
    }
}

#[derive(Debug)]
pub struct DynamicPrimitive {
    pub mode: u32,
    pub vao: WebGlVertexArrayObject,
    pub buffer: WebGlBuffer,
    buffer_data: Vec<f32>,
    last_uploaded_buffer_size: usize,
}
impl DynamicPrimitive {
    pub fn new(ctx: &WebGl2RenderingContext) -> Self {
        let vao = ctx.create_vertex_array().unwrap();
        ctx.bind_vertex_array(Some(&vao));

        let buffer = ctx.create_buffer().unwrap();
        ctx.bind_buffer(GL::ARRAY_BUFFER, Some(buffer.as_ref()));
        ctx.buffer_data_with_opt_array_buffer(GL::ARRAY_BUFFER, None, GL::DYNAMIC_DRAW);

        ctx.enable_vertex_attrib_array(PrimitiveAttribute::Position as u32);
        ctx.vertex_attrib_pointer_with_i32(
            PrimitiveAttribute::Position as u32,
            3,
            GL::FLOAT,
            false,
            0,
            0,
        );

        ctx.bind_vertex_array(None);

        Self {
            mode: GL::POINTS,
            vao,
            buffer,
            buffer_data: Vec::new(),
            last_uploaded_buffer_size: 0,
        }
    }

    pub fn draw(&self, ctx: &WebGl2RenderingContext) {
        ctx.bind_vertex_array(Some(&self.vao));
        ctx.draw_arrays(self.mode, 0, (self.last_uploaded_buffer_size / 3) as i32);
        ctx.bind_vertex_array(None);
    }

    pub fn set_buffer_size(&mut self, num_elements: usize) {
        self.buffer_data.resize(num_elements, 0.0);
    }

    pub fn get_buffer(&self) -> &[f32] {
        return self.buffer_data.as_slice();
    }

    pub fn get_buffer_mut(&mut self) -> &mut [f32] {
        return self.buffer_data.as_mut_slice();
    }

    pub fn upload_buffer_data(&mut self, ctx: &WebGl2RenderingContext) {
        let buffer_data_location = self.buffer_data.as_ptr() as u32 / 4;
        let buffer_data_len = self.buffer_data.len() as u32;

        let memory_buffer = wasm_bindgen::memory()
            .dyn_into::<WebAssembly::Memory>()
            .unwrap()
            .buffer();

        let arr = js_sys::Float32Array::new(&memory_buffer)
            .subarray(buffer_data_location, buffer_data_location + buffer_data_len);

        ctx.bind_vertex_array(Some(&self.vao));
        ctx.bind_buffer(GL::ARRAY_BUFFER, Some(self.buffer.as_ref()));

        if self.buffer_data.len() != self.last_uploaded_buffer_size {
            ctx.buffer_data_with_array_buffer_view(GL::ARRAY_BUFFER, &arr, GL::DYNAMIC_DRAW);
            self.last_uploaded_buffer_size = self.buffer_data.len();
        } else {
            ctx.buffer_sub_data_with_i32_and_array_buffer_view(GL::ARRAY_BUFFER, 0, &arr);
        }

        ctx.bind_buffer(GL::ARRAY_BUFFER, None);
        ctx.bind_vertex_array(None);
    }
}
