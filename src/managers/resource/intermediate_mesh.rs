use super::{
    collider::{Collider, CompoundCollider},
    material::Material,
    mesh::{Mesh, Primitive, PrimitiveAttribute},
};
use crate::managers::resource::mesh::DynamicPrimitive;
use crate::utils::gl::GL;
use crate::GLCTX;
use js_sys::WebAssembly;
use na::{Vector2, Vector3, Vector4};
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::JsCast;
use web_sys::{WebGl2RenderingContext, WebGlBuffer};

#[derive(Clone)]
pub struct IntermediateMesh {
    pub name: String,
    pub primitives: Vec<IntermediatePrimitive>,
}

#[derive(Debug, Clone)]
pub struct IntermediatePrimitive {
    pub name: String,

    pub indices: Vec<u16>,
    pub positions: Vec<Vector3<f32>>,
    pub normals: Vec<Vector3<f32>>,
    pub tangents: Vec<Vector3<f32>>,
    pub colors: Vec<Vector4<f32>>,
    pub uv0: Vec<Vector2<f32>>,
    pub uv1: Vec<Vector2<f32>>,
    pub mode: u32,
    pub mat: Option<Rc<RefCell<Material>>>,
    pub collider: Option<Box<dyn Collider>>,
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

pub fn generate_dynamic_mesh() -> Rc<RefCell<Mesh>> {
    let result = Rc::new(RefCell::new(Mesh {
        name: String::from("points"),
        primitives: Vec::new(),
        dynamic_primitive: None,
        collider: None,
    }));

    GLCTX.with(|ctx| {
        let ref_mut = ctx.borrow_mut();
        let ctx = ref_mut.as_ref().unwrap();

        let mut result_mut = RefCell::borrow_mut(&result);
        result_mut.dynamic_primitive = Some(DynamicPrimitive::new(ctx));
    });

    return result;
}

pub fn generate_screen_space_quad(
    default_material: Option<Rc<RefCell<Material>>>,
) -> Rc<RefCell<Mesh>> {
    let mut primitives: Vec<Primitive> = Vec::new();
    primitives.reserve(1);

    let positions: [f32; 4 * 2] = [
        -1.0, -1.0, //
        -1.0, 1.0, //
        1.0, -1.0, //
        1.0, 1.0, //
    ];

    let indices: [u16; 6] = [
        0, 2, 1, //
        1, 2, 3, //
    ];

    GLCTX.with(|ctx| {
        let ref_mut = ctx.borrow_mut();
        let ctx = ref_mut.as_ref().unwrap();

        // Create VAO
        let vao = ctx.create_vertex_array();
        ctx.bind_vertex_array(vao.as_ref());

        // Indices
        let mut index_buffer = ctx.create_buffer().unwrap();
        fill_short_element_buffer(
            &ctx,
            indices.as_ptr() as u32 / 2, // Divided by 2 because the wasm_bindgen memory will be interpreted as a short array, so the position of indices needs to be divided by 2 bytes to get to the correct element
            indices.len() as u32, // Not multiplying anything because we have exactly this many u16 indices
            &mut index_buffer,
        );
        ctx.bind_buffer(GL::ELEMENT_ARRAY_BUFFER, Some(&index_buffer));

        // Positions
        let mut position_buffer = ctx.create_buffer().unwrap();
        fill_float_attribute_buffer(
            &ctx,
            positions.as_ptr() as u32 / 4, // Divided by 4 because the wasm_bindgen memory buffer will be interpreted as an array of floats, so the prim.positions' array pointer target address (u8* basically) needs to be divided by 4 to get the correct starting element
            positions.len() as u32, // Not multiplying anything because we have exactly this many u16 indices
            &mut position_buffer,
        );
        ctx.enable_vertex_attrib_array(PrimitiveAttribute::Position as u32);
        ctx.bind_buffer(GL::ARRAY_BUFFER, Some(&position_buffer));
        ctx.vertex_attrib_pointer_with_i32(
            PrimitiveAttribute::Position as u32,
            2,
            GL::FLOAT,
            false,
            0,
            0,
        );

        ctx.bind_vertex_array(None);

        let mut primitive = Primitive {
            name: String::from("0"),
            index_count: indices.len() as i32,
            vao: vao.unwrap(),
            mode: GL::TRIANGLES,
            has_normals: false,
            has_tangents: false,
            has_colors: false,
            has_uv0: false,
            has_uv1: false,
            compatible_hash: 0,
            default_material,
            source_data: None,
        };
        primitive.update_hash();
        primitives.push(primitive);
    });

    let result = Rc::new(RefCell::new(Mesh {
        name: String::from("quad"),
        primitives,
        collider: None,
        ..Default::default()
    }));

    return result;
}

pub fn intermediate_to_mesh(inter: &IntermediateMesh) -> Rc<RefCell<Mesh>> {
    let mut primitives: Vec<Primitive> = Vec::new();
    primitives.reserve(inter.primitives.len());

    GLCTX.with(|ctx| {
        let ref_mut = ctx.borrow_mut();
        let ctx = ref_mut.as_ref().unwrap();

        for prim in &inter.primitives {
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
            let has_normals = prim.normals.len() > 0;
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
            let has_tangents = prim.tangents.len() > 0;
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
            let has_colors = prim.colors.len() > 0;
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
            let has_uv0 = prim.uv0.len() > 0;
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
            let has_uv1 = prim.uv1.len() > 0;
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

            let mut primitive = Primitive {
                name: String::from("0"),
                index_count: prim.indices.len() as i32,
                vao: vao.unwrap(),
                mode: prim.mode,
                has_normals,
                has_tangents,
                has_colors,
                has_uv0,
                has_uv1,
                compatible_hash: 0,
                default_material: prim.mat.clone(),
                source_data: None,
            };
            primitive.update_hash();
            log::info!(
                "Set prim compatible hash of prim '{}' of mat '{}' as '{}'",
                primitive.name,
                inter.name,
                primitive.compatible_hash
            );
            primitives.push(primitive);
        }
    });

    let collider = {
        if inter.primitives.len() == 1 {
            inter.primitives[0].collider.clone()
        } else {
            let mut compound = Box::new(CompoundCollider {
                colliders: Vec::new(),
            });

            for prim in &inter.primitives {
                if let Some(prim_collider) = &prim.collider {
                    compound.colliders.push(prim_collider.clone());
                }
            }

            Some(compound as Box<dyn Collider>)
        }
    };

    let result = Rc::new(RefCell::new(Mesh {
        name: inter.name.clone(),
        primitives,
        collider,
        ..Default::default()
    }));

    return result;
}
