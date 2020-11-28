use std::rc::Rc;

use js_sys::WebAssembly;
use wasm_bindgen::JsCast;
use web_sys::{WebGlBuffer, WebGlRenderingContext, WebGlRenderingContext as GL};

use super::{Material, Mesh, Primitive};

pub struct IntermediateMesh {
    pub name: String,
    pub primitives: Vec<IntermediatePrimitive>,
}

pub struct IntermediatePrimitive {
    pub name: String,

    pub indices: Vec<u16>,
    pub positions: Vec<cgmath::Vector3<f32>>,
    pub normals: Vec<cgmath::Vector3<f32>>,
    pub colors: Vec<cgmath::Vector4<f32>>,
    pub uv0: Vec<cgmath::Vector2<f32>>,
    pub uv1: Vec<cgmath::Vector2<f32>>,
    pub mode: u32,
    pub mat: Option<Rc<Material>>,
}

pub fn fill_float_attribute_buffer(
    ctx: &WebGlRenderingContext,
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
    ctx: &WebGlRenderingContext,
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

pub fn intermediate_to_mesh(inter: IntermediateMesh, ctx: &WebGlRenderingContext) -> Rc<Mesh> {
    let mut primitives: Vec<Primitive> = Vec::new();
    primitives.reserve(inter.primitives.len());

    for prim in inter.primitives {
        // Indices
        let mut index_buffer = ctx.create_buffer().unwrap();
        fill_short_element_buffer(
            &ctx,
            prim.indices.as_ptr() as u32 / 2, // Divided by 2 because the wasm_bindgen memory will be interpreted as a short array, so the position of indices needs to be divided by 2 bytes to get to the correct element
            prim.indices.len() as u32, // Not multiplying anything because we have exactly this many u16 indices
            &mut index_buffer,
        );

        // Positions
        let mut position_buffer = ctx.create_buffer().unwrap();
        fill_float_attribute_buffer(
            &ctx,
            prim.positions.as_ptr() as u32 / 4, // Divided by 4 because the wasm_bindgen memory buffer will be interpreted as an array of floats, so the prim.positions' array pointer target address (u8* basically) needs to be divided by 4 to get the correct starting element
            prim.positions.len() as u32 * 3, // Multiplying by 3 because this will be moved into an f32 buffer, and we have len * 3 f32s
            &mut position_buffer,
        );

        // Normals
        let mut normal_buffer = ctx.create_buffer().unwrap();
        fill_float_attribute_buffer(
            &ctx,
            prim.normals.as_ptr() as u32 / 4,
            prim.normals.len() as u32 * 3,
            &mut normal_buffer,
        );

        // Colors
        let mut color_buffer = ctx.create_buffer().unwrap();
        fill_float_attribute_buffer(
            &ctx,
            prim.colors.as_ptr() as u32 / 4,
            prim.colors.len() as u32 * 4,
            &mut color_buffer,
        );

        // UV0
        let mut uv0_buffer = ctx.create_buffer().unwrap();
        fill_float_attribute_buffer(
            &ctx,
            prim.uv0.as_ptr() as u32 / 4,
            prim.uv0.len() as u32 * 2,
            &mut uv0_buffer,
        );

        // UV1
        let mut uv1_buffer = ctx.create_buffer().unwrap();
        fill_float_attribute_buffer(
            &ctx,
            prim.uv0.as_ptr() as u32 / 4,
            prim.uv0.len() as u32 * 2,
            &mut uv1_buffer,
        );

        primitives.push(Primitive {
            name: String::from("0"),
            index_count: prim.indices.len() as i32,
            index_buffer,
            position_buffer,
            normal_buffer,
            color_buffer,
            uv0_buffer,
            uv1_buffer,
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
