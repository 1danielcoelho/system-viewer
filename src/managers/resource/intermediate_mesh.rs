use std::{cell::RefCell, rc::Rc};

use js_sys::WebAssembly;
use wasm_bindgen::JsCast;
use web_sys::{WebGl2RenderingContext, WebGl2RenderingContext as GL, WebGlBuffer};

use super::{Material, Mesh, Primitive, PrimitiveAttribute};

pub struct IntermediateMesh {
    pub name: String,
    pub primitives: Vec<IntermediatePrimitive>,
}

pub struct IntermediatePrimitive {
    pub name: String,

    pub indices: Vec<u16>,
    pub positions: Vec<cgmath::Vector3<f32>>,
    pub normals: Vec<cgmath::Vector3<f32>>,
    pub tangents: Vec<cgmath::Vector3<f32>>,
    pub colors: Vec<cgmath::Vector4<f32>>,
    pub uv0: Vec<cgmath::Vector2<f32>>,
    pub uv1: Vec<cgmath::Vector2<f32>>,
    pub mode: u32,
    pub mat: Option<Rc<RefCell<Material>>>,
}

pub fn fill_float_attribute_buffer(
    ctx: &WebGl2RenderingContext,
    location: u32,
    num_elements: u32,
    out_buffer: &mut WebGlBuffer,
) {
    if num_elements == 0 {
        return;
    }

    let memory_buffer = wasm_bindgen::memory()
        .dyn_into::<WebAssembly::Memory>()
        .unwrap()
        .buffer();
    let buffer_array =
        js_sys::Float32Array::new(&memory_buffer).subarray(location, location + num_elements);
    ctx.bind_buffer(GL::ARRAY_BUFFER, Some(&out_buffer));
    ctx.buffer_data_with_array_buffer_view(GL::ARRAY_BUFFER, &buffer_array, GL::STATIC_DRAW);
}

pub fn fill_short_element_buffer(
    ctx: &WebGl2RenderingContext,
    location: u32,
    num_elements: u32,
    out_buffer: &mut WebGlBuffer,
) {
    if num_elements == 0 {
        return;
    }

    let memory_buffer = wasm_bindgen::memory()
        .dyn_into::<WebAssembly::Memory>()
        .unwrap()
        .buffer();
    let buffer_array =
        js_sys::Uint16Array::new(&memory_buffer).subarray(location, location + num_elements);
    ctx.bind_buffer(GL::ELEMENT_ARRAY_BUFFER, Some(&out_buffer));
    ctx.buffer_data_with_array_buffer_view(
        GL::ELEMENT_ARRAY_BUFFER,
        &buffer_array,
        GL::STATIC_DRAW,
    );
}

pub fn intermediate_to_mesh(inter: IntermediateMesh, ctx: &WebGl2RenderingContext) -> Rc<Mesh> {
    let mut primitives: Vec<Primitive> = Vec::new();
    primitives.reserve(inter.primitives.len());

    for prim in inter.primitives {
        // Create VAO
        let vao = ctx.create_vertex_array();
        ctx.bind_vertex_array(vao.as_ref());

        // Indices
        let mut index_buffer = ctx.create_buffer().unwrap();
        fill_short_element_buffer(
            &ctx,
            prim.indices.as_ptr() as u32 / 2, // Divided by 2 because the wasm_bindgen memory will be interpreted as a short array, so the position of indices needs to be divided by 2 bytes to get to the correct element
            prim.indices.len() as u32, // Not multiplying anything because we have exactly this many u16 indices
            &mut index_buffer,
        );
        ctx.bind_buffer(GL::ELEMENT_ARRAY_BUFFER, Some(&index_buffer));

        // Positions
        let mut position_buffer = ctx.create_buffer().unwrap();
        fill_float_attribute_buffer(
            &ctx,
            prim.positions.as_ptr() as u32 / 4, // Divided by 4 because the wasm_bindgen memory buffer will be interpreted as an array of floats, so the prim.positions' array pointer target address (u8* basically) needs to be divided by 4 to get the correct starting element
            prim.positions.len() as u32 * 3, // Multiplying by 3 because this will be moved into an f32 buffer, and we have len * 3 f32s
            &mut position_buffer,
        );
        ctx.enable_vertex_attrib_array(PrimitiveAttribute::Position as u32);
        ctx.bind_buffer(GL::ARRAY_BUFFER, Some(&position_buffer));
        ctx.vertex_attrib_pointer_with_i32(
            PrimitiveAttribute::Position as u32,
            3,
            GL::FLOAT,
            false,
            0,
            0,
        );

        // Normals
        let mut normal_buffer = ctx.create_buffer().unwrap();
        fill_float_attribute_buffer(
            &ctx,
            prim.normals.as_ptr() as u32 / 4,
            prim.normals.len() as u32 * 3,
            &mut normal_buffer,
        );
        ctx.enable_vertex_attrib_array(PrimitiveAttribute::Normal as u32);
        ctx.bind_buffer(GL::ARRAY_BUFFER, Some(&normal_buffer));
        ctx.vertex_attrib_pointer_with_i32(
            PrimitiveAttribute::Normal as u32,
            3,
            GL::FLOAT,
            false,
            0,
            0,
        );

        // Tangents
        let mut tangent_buffer = ctx.create_buffer().unwrap();
        fill_float_attribute_buffer(
            &ctx,
            prim.tangents.as_ptr() as u32 / 4,
            prim.tangents.len() as u32 * 3,
            &mut tangent_buffer,
        );
        ctx.enable_vertex_attrib_array(PrimitiveAttribute::Tangent as u32);
        ctx.bind_buffer(GL::ARRAY_BUFFER, Some(&tangent_buffer));
        ctx.vertex_attrib_pointer_with_i32(
            PrimitiveAttribute::Tangent as u32,
            3,
            GL::FLOAT,
            false,
            0,
            0,
        );

        // Colors
        let mut color_buffer = ctx.create_buffer().unwrap();
        fill_float_attribute_buffer(
            &ctx,
            prim.colors.as_ptr() as u32 / 4,
            prim.colors.len() as u32 * 4,
            &mut color_buffer,
        );
        ctx.enable_vertex_attrib_array(PrimitiveAttribute::Color as u32);
        ctx.bind_buffer(GL::ARRAY_BUFFER, Some(&color_buffer));
        ctx.vertex_attrib_pointer_with_i32(
            PrimitiveAttribute::Color as u32,
            4,
            GL::FLOAT,
            false,
            0,
            0,
        );

        // UV0
        let mut uv0_buffer = ctx.create_buffer().unwrap();
        fill_float_attribute_buffer(
            &ctx,
            prim.uv0.as_ptr() as u32 / 4,
            prim.uv0.len() as u32 * 2,
            &mut uv0_buffer,
        );
        ctx.enable_vertex_attrib_array(PrimitiveAttribute::UV0 as u32);
        ctx.bind_buffer(GL::ARRAY_BUFFER, Some(&uv0_buffer));
        ctx.vertex_attrib_pointer_with_i32(
            PrimitiveAttribute::UV0 as u32,
            2,
            GL::FLOAT,
            false,
            0,
            0,
        );

        // UV1
        let mut uv1_buffer = ctx.create_buffer().unwrap();
        fill_float_attribute_buffer(
            &ctx,
            prim.uv0.as_ptr() as u32 / 4,
            prim.uv0.len() as u32 * 2,
            &mut uv1_buffer,
        );
        ctx.enable_vertex_attrib_array(PrimitiveAttribute::UV1 as u32);
        ctx.bind_buffer(GL::ARRAY_BUFFER, Some(&uv0_buffer));
        ctx.vertex_attrib_pointer_with_i32(
            PrimitiveAttribute::UV1 as u32,
            2,
            GL::FLOAT,
            false,
            0,
            0,
        );

        ctx.bind_vertex_array(None);

        primitives.push(Primitive {
            name: String::from("0"),
            index_count: prim.indices.len() as i32,
            vao: vao.unwrap(),
            mode: prim.mode,
            default_material: prim.mat,
        });
    }

    let result = Rc::new(Mesh {
        id: 0,
        name: inter.name,
        primitives,
    });

    return result;
}
