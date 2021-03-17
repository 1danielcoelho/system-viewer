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

    pub pos_buffer: WebGlBuffer,
    pos_buffer_data: Vec<f32>,

    pub color_buffer: WebGlBuffer,
    color_buffer_data: Vec<f32>,

    last_uploaded_point_count: usize,
}
impl DynamicPrimitive {
    pub fn new(ctx: &WebGl2RenderingContext) -> Self {
        let vao = ctx.create_vertex_array().unwrap();
        ctx.bind_vertex_array(Some(&vao));

        // Positions
        let pos_buffer = ctx.create_buffer().unwrap();
        ctx.bind_buffer(GL::ARRAY_BUFFER, Some(pos_buffer.as_ref()));
        ctx.enable_vertex_attrib_array(PrimitiveAttribute::Position as u32);
        ctx.vertex_attrib_pointer_with_i32(
            PrimitiveAttribute::Position as u32,
            3,
            GL::FLOAT,
            false,
            0,
            0,
        );

        // Colors
        let color_buffer = ctx.create_buffer().unwrap();
        ctx.bind_buffer(GL::ARRAY_BUFFER, Some(color_buffer.as_ref()));
        ctx.enable_vertex_attrib_array(PrimitiveAttribute::Color as u32);
        ctx.vertex_attrib_pointer_with_i32(
            PrimitiveAttribute::Color as u32,
            4,
            GL::FLOAT,
            false,
            0,
            0,
        );

        ctx.bind_vertex_array(None);

        Self {
            mode: GL::POINTS,
            vao,
            pos_buffer,
            color_buffer,
            pos_buffer_data: Vec::new(),
            color_buffer_data: Vec::new(),
            last_uploaded_point_count: 0,
        }
    }

    pub fn draw(&self, ctx: &WebGl2RenderingContext) {
        ctx.bind_vertex_array(Some(&self.vao));
        ctx.draw_arrays(self.mode, 0, self.last_uploaded_point_count as i32);
        ctx.bind_vertex_array(None);
    }

    pub fn set_num_elements(&mut self, num_points: usize) {
        self.pos_buffer_data.resize(num_points * 3, 0.0);
        self.color_buffer_data.resize(num_points * 4, 1.0);
    }

    pub fn get_num_elements(&self) -> usize {
        return self.pos_buffer_data.len() / 3;
    }

    pub fn get_pos_buffer(&self) -> &[f32] {
        return self.pos_buffer_data.as_slice();
    }

    pub fn get_pos_buffer_mut(&mut self) -> &mut [f32] {
        return self.pos_buffer_data.as_mut_slice();
    }

    pub fn get_color_buffer(&self) -> &[f32] {
        return self.color_buffer_data.as_slice();
    }

    pub fn get_color_buffer_mut(&mut self) -> &mut [f32] {
        return self.color_buffer_data.as_mut_slice();
    }

    pub fn upload_buffers(&mut self, ctx: &WebGl2RenderingContext) {
        let pos_buffer_data_location = self.pos_buffer_data.as_ptr() as u32 / 4;
        let pos_buffer_data_len = self.pos_buffer_data.len() as u32;

        let color_buffer_data_location = self.color_buffer_data.as_ptr() as u32 / 4;
        let color_buffer_data_len = self.color_buffer_data.len() as u32;

        let memory_buffer = wasm_bindgen::memory()
            .dyn_into::<WebAssembly::Memory>()
            .unwrap()
            .buffer();

        let pos_arr = js_sys::Float32Array::new(&memory_buffer).subarray(
            pos_buffer_data_location,
            pos_buffer_data_location + pos_buffer_data_len,
        );
        let color_arr = js_sys::Float32Array::new(&memory_buffer).subarray(
            color_buffer_data_location,
            color_buffer_data_location + color_buffer_data_len,
        );

        ctx.bind_buffer(GL::ARRAY_BUFFER, Some(self.pos_buffer.as_ref()));

        if self.pos_buffer_data.len() != self.last_uploaded_point_count {
            ctx.buffer_data_with_array_buffer_view(GL::ARRAY_BUFFER, &pos_arr, GL::DYNAMIC_DRAW);

            // If we're resizing also upload our color buffer
            ctx.bind_buffer(GL::ARRAY_BUFFER, Some(self.color_buffer.as_ref()));
            ctx.buffer_data_with_array_buffer_view(GL::ARRAY_BUFFER, &color_arr, GL::DYNAMIC_DRAW);

            self.last_uploaded_point_count = self.pos_buffer_data.len() / 3;
        } else {
            ctx.buffer_sub_data_with_i32_and_array_buffer_view(GL::ARRAY_BUFFER, 0, &pos_arr);
        }
    }
}
