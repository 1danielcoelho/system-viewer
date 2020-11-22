use std::{mem::size_of, rc::Rc};

use cgmath::{Vector2, Vector3, Vector4};
use js_sys::WebAssembly;
use wasm_bindgen::JsCast;
use web_sys::WebGlRenderingContext as GL;
use web_sys::{WebGlBuffer, WebGlRenderingContext};

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
    //pub bb:
}

pub fn fill_float_buffer(
    ctx: &WebGlRenderingContext,
    location: u32,
    num_elements: u32,
    out_buffer: &mut WebGlBuffer,
) {
    if num_elements == 0 || location == 0 {
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

pub fn fill_short_buffer(
    ctx: &WebGlRenderingContext,
    location: u32,
    num_elements: u32,
    out_buffer: &mut WebGlBuffer,
) {
    if num_elements == 0 || location == 0 {
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
        fill_short_buffer(
            &ctx,
            prim.indices.as_ptr() as u32 / 2,
            prim.indices.len() as u32,
            &mut index_buffer,
        );

        // Positions
        let mut position_buffer = ctx.create_buffer().unwrap();
        fill_float_buffer(
            &ctx,
            prim.positions.as_ptr() as u32 / 4,
            prim.positions.len() as u32 * 3,
            &mut position_buffer,
        );

        // Normals
        let mut normal_buffer = ctx.create_buffer().unwrap();
        fill_float_buffer(
            &ctx,
            prim.normals.as_ptr() as u32 / 4,
            prim.normals.len() as u32 * 3,
            &mut normal_buffer,
        );

        // Colors
        let mut color_buffer = ctx.create_buffer().unwrap();
        fill_float_buffer(
            &ctx,
            prim.colors.as_ptr() as u32 / 4,
            prim.colors.len() as u32 * 4,
            &mut color_buffer,
        );

        // UV0
        let mut uv0_buffer = ctx.create_buffer().unwrap();
        fill_float_buffer(
            &ctx,
            prim.uv0.as_ptr() as u32 / 4,
            prim.uv0.len() as u32 * 2,
            &mut uv0_buffer,
        );

        // UV1
        let mut uv1_buffer = ctx.create_buffer().unwrap();
        fill_float_buffer(
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

pub fn generate_cube(
    ctx: &WebGlRenderingContext,
    default_material: Option<Rc<Material>>,
) -> Rc<Mesh> {
    intermediate_to_mesh(
        IntermediateMesh {
            name: String::from("cube"),
            primitives: vec![IntermediatePrimitive {
                name: String::from("0"),
                indices: vec![
                    0, 1, 3, 0, 3, 2, 1, 5, 3, 5, 7, 3, 5, 4, 6, 6, 7, 5, 0, 2, 4, 2, 6, 4, 2, 3,
                    7, 2, 7, 6, 0, 4, 5, 0, 5, 1,
                ],
                positions: vec![
                    Vector3::new(-1.0, -1.0, -1.0),
                    Vector3::new(-1.0, -1.0, 1.0),
                    Vector3::new(-1.0, 1.0, -1.0),
                    Vector3::new(-1.0, 1.0, 1.0),
                    Vector3::new(1.0, -1.0, -1.0),
                    Vector3::new(1.0, -1.0, 1.0),
                    Vector3::new(1.0, 1.0, -1.0),
                    Vector3::new(1.0, 1.0, 1.0),
                ],
                normals: vec![],
                colors: vec![
                    Vector4::new(0.0, 0.0, 0.0, 1.0),
                    Vector4::new(0.0, 0.0, 1.0, 1.0),
                    Vector4::new(0.0, 1.0, 0.0, 1.0),
                    Vector4::new(0.0, 1.0, 1.0, 1.0),
                    Vector4::new(1.0, 0.0, 0.0, 1.0),
                    Vector4::new(1.0, 0.0, 1.0, 1.0),
                    Vector4::new(1.0, 1.0, 0.0, 1.0),
                    Vector4::new(1.0, 1.0, 1.0, 1.0),
                ],
                uv0: vec![],
                uv1: vec![],
                mode: GL::TRIANGLES,
                mat: default_material,
            }],
        },
        ctx,
    )
}

pub fn generate_plane(
    ctx: &WebGlRenderingContext,
    default_material: Option<Rc<Material>>,
) -> Rc<Mesh> {
    intermediate_to_mesh(
        IntermediateMesh {
            name: String::from("plane"),
            primitives: vec![IntermediatePrimitive {
                name: String::from("0"),
                indices: vec![0, 1, 3, 0, 3, 2],
                positions: vec![
                    Vector3::new(1.0, 1.0, 0.0),
                    Vector3::new(1.0, -1.0, 0.0),
                    Vector3::new(-1.0, 1.0, 0.0),
                    Vector3::new(-1.0, -1.0, 0.0),
                ],
                normals: vec![],
                colors: vec![
                    Vector4::new(0.0, 0.0, 0.0, 1.0),
                    Vector4::new(0.0, 0.0, 1.0, 1.0),
                    Vector4::new(0.0, 1.0, 0.0, 1.0),
                    Vector4::new(0.0, 1.0, 1.0, 1.0),
                ],
                uv0: vec![],
                uv1: vec![],
                mode: GL::TRIANGLES,
                mat: default_material,
            }],
        },
        ctx,
    )
}

pub fn generate_grid(
    ctx: &WebGlRenderingContext,
    num_lines: u32,
    default_material: Option<Rc<Material>>,
) -> Rc<Mesh> {
    assert!(num_lines > 2);

    let incr = 1.0 / (num_lines - 1) as f32;
    let num_verts = num_lines * num_lines;

    let mut positions: Vec<Vector3<f32>> = Vec::new();
    positions.resize((num_verts * 3) as usize, Vector3::new(0.0, 0.0, 0.0));

    let mut colors: Vec<Vector4<f32>> = Vec::new();
    colors.resize((num_verts * 3) as usize, Vector4::new(1.0, 1.0, 1.0, 1.0));

    for y_ind in 0..num_lines {
        for x_ind in 0..num_lines {
            let vert_ind = x_ind + y_ind * num_lines;

            positions[vert_ind as usize].x = x_ind as f32 * incr - 0.5;
            positions[vert_ind as usize].y = y_ind as f32 * incr - 0.5;
        }
    }

    let mut indices: Vec<u16> = Vec::new();
    indices.resize((num_lines * 4) as usize, 0);
    for col_ind in 0..num_lines {
        let ind = col_ind * 2;

        indices[(ind + 0) as usize] = col_ind as u16;
        indices[(ind + 1) as usize] = (num_lines * num_lines - (num_lines - col_ind)) as u16;
    }

    for row_ind in 0..num_lines {
        let ind = (row_ind * 2) + num_lines * 2;

        indices[(ind + 0) as usize] = (row_ind * num_lines) as u16;
        indices[(ind + 1) as usize] = ((row_ind + 1) * num_lines - 1) as u16;
    }

    return intermediate_to_mesh(
        IntermediateMesh {
            name: String::from("grid"),
            primitives: vec![IntermediatePrimitive {
                name: String::from("0"),
                indices,
                positions,
                normals: vec![],
                colors,
                uv0: vec![],
                uv1: vec![],
                mat: default_material,
                mode: GL::LINES,
            }],
        },
        ctx,
    );
}

pub fn generate_axes(
    ctx: &WebGlRenderingContext,
    default_material: Option<Rc<Material>>,
) -> Rc<Mesh> {
    intermediate_to_mesh(
        IntermediateMesh {
            name: String::from("axes"),
            primitives: vec![IntermediatePrimitive {
                name: String::from("0"),
                indices: vec![0, 1, 0, 2, 0, 3],
                positions: vec![
                    Vector3::new(0.0, 0.0, 0.0),
                    Vector3::new(1.0, 0.0, 0.0),
                    Vector3::new(0.0, 1.0, 0.0),
                    Vector3::new(0.0, 0.0, 1.0),
                ],
                normals: vec![],
                colors: vec![
                    Vector4::new(0.0, 0.0, 0.0, 1.0),
                    Vector4::new(1.0, 0.0, 0.0, 1.0),
                    Vector4::new(0.0, 1.0, 0.0, 1.0),
                    Vector4::new(0.0, 0.0, 1.0, 1.0),
                ],
                uv0: vec![],
                uv1: vec![],
                mode: GL::LINES,
                mat: default_material,
            }],
        },
        ctx,
    )
}
