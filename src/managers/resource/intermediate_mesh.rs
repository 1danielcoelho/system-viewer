use super::{
    collider::{Collider, CompoundCollider},
    material::Material,
    mesh::{Mesh, Primitive, PrimitiveAttribute},
};
use crate::managers::resource::mesh::DynamicPrimitive;
use crate::utils::gl::GL;
use crate::utils::memory::any_slice_to_u8_slice;
use crate::GLCTX;
use glow::*;
use na::{Vector2, Vector3, Vector4};
use std::{cell::RefCell, rc::Rc};

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

pub fn generate_dynamic_mesh() -> Rc<RefCell<Mesh>> {
    let result = Rc::new(RefCell::new(Mesh {
        name: String::from("points"),
        primitives: Vec::new(),
        dynamic_primitive: None,
        collider: None,
    }));

    GLCTX.with(|ctx| {
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

    let uvs: [f32; 4 * 2] = [
        0.0, 0.0, //
        0.0, 1.0, //
        1.0, 0.0, //
        1.0, 1.0, //
    ];

    GLCTX.with(|ctx| {
        // Create VAO
        unsafe {
            let vao = ctx.create_vertex_array().unwrap();
            ctx.bind_vertex_array(Some(vao));

            // Indices
            let index_buffer = ctx.create_buffer().unwrap();
            let indices_as_u8 = any_slice_to_u8_slice(&indices);
            ctx.bind_buffer(GL::ELEMENT_ARRAY_BUFFER, Some(index_buffer));
            ctx.buffer_data_u8_slice(GL::ELEMENT_ARRAY_BUFFER, indices_as_u8, GL::STATIC_DRAW);

            // Positions
            let position_buffer = ctx.create_buffer().unwrap();
            let positions_as_u8 = any_slice_to_u8_slice(&positions);
            ctx.bind_buffer(GL::ARRAY_BUFFER, Some(position_buffer));
            ctx.buffer_data_u8_slice(GL::ARRAY_BUFFER, positions_as_u8, GL::STATIC_DRAW);
            ctx.enable_vertex_attrib_array(PrimitiveAttribute::Position as u32);
            ctx.vertex_attrib_pointer_f32(
                PrimitiveAttribute::Position as u32,
                2,
                GL::FLOAT,
                false,
                0,
                0,
            );

            // UV0
            let uv0_buffer = ctx.create_buffer().unwrap();
            let uv0_as_u8 = any_slice_to_u8_slice(&uvs);
            ctx.bind_buffer(GL::ARRAY_BUFFER, Some(uv0_buffer));
            ctx.buffer_data_u8_slice(GL::ARRAY_BUFFER, uv0_as_u8, GL::STATIC_DRAW);
            ctx.enable_vertex_attrib_array(PrimitiveAttribute::UV0 as u32);
            ctx.vertex_attrib_pointer_f32(
                PrimitiveAttribute::UV0 as u32,
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
                vao,
                mode: GL::TRIANGLES,
                has_normals: false,
                has_tangents: false,
                has_colors: false,
                has_uv0: true,
                has_uv1: false,
                compatible_hash: 0,
                default_material,
                source_data: None,
            };
            primitive.update_hash();
            primitives.push(primitive);
        }
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
        unsafe {
            for prim in &inter.primitives {
                // Create VAO
                let vao = ctx.create_vertex_array().unwrap();
                ctx.bind_vertex_array(Some(vao));

                // Indices
                let index_buffer = ctx.create_buffer().unwrap();
                let indices_as_u8 = any_slice_to_u8_slice(&prim.indices);
                ctx.bind_buffer(GL::ELEMENT_ARRAY_BUFFER, Some(index_buffer));
                ctx.buffer_data_u8_slice(GL::ELEMENT_ARRAY_BUFFER, indices_as_u8, GL::STATIC_DRAW);

                // Positions
                let position_buffer = ctx.create_buffer().unwrap();
                let positions_as_u8 = any_slice_to_u8_slice(&prim.positions);
                ctx.bind_buffer(GL::ARRAY_BUFFER, Some(position_buffer));
                ctx.buffer_data_u8_slice(GL::ARRAY_BUFFER, positions_as_u8, GL::STATIC_DRAW);
                ctx.enable_vertex_attrib_array(PrimitiveAttribute::Position as u32);
                ctx.vertex_attrib_pointer_f32(
                    PrimitiveAttribute::Position as u32,
                    3,
                    GL::FLOAT,
                    false,
                    0,
                    0,
                );

                // Normals
                let has_normals = prim.normals.len() > 0;
                let normal_buffer = ctx.create_buffer().unwrap();
                let normals_as_u8 = any_slice_to_u8_slice(&prim.normals);
                ctx.bind_buffer(GL::ARRAY_BUFFER, Some(normal_buffer));
                ctx.buffer_data_u8_slice(GL::ARRAY_BUFFER, normals_as_u8, GL::STATIC_DRAW);
                ctx.enable_vertex_attrib_array(PrimitiveAttribute::Normal as u32);
                ctx.vertex_attrib_pointer_f32(
                    PrimitiveAttribute::Normal as u32,
                    3,
                    GL::FLOAT,
                    false,
                    0,
                    0,
                );

                // Tangents
                let has_tangents = prim.tangents.len() > 0;
                let tangent_buffer = ctx.create_buffer().unwrap();
                let tangents_as_u8 = any_slice_to_u8_slice(&prim.tangents);
                ctx.bind_buffer(GL::ARRAY_BUFFER, Some(tangent_buffer));
                ctx.buffer_data_u8_slice(GL::ARRAY_BUFFER, tangents_as_u8, GL::STATIC_DRAW);
                ctx.enable_vertex_attrib_array(PrimitiveAttribute::Tangent as u32);
                ctx.vertex_attrib_pointer_f32(
                    PrimitiveAttribute::Tangent as u32,
                    3,
                    GL::FLOAT,
                    false,
                    0,
                    0,
                );

                // Colors
                // TODO: Can I just not create these buffers if there's no data?
                let has_colors = prim.colors.len() > 0;
                let color_buffer = ctx.create_buffer().unwrap();
                let colors_as_u8 = any_slice_to_u8_slice(&prim.colors);
                ctx.bind_buffer(GL::ARRAY_BUFFER, Some(color_buffer));
                ctx.buffer_data_u8_slice(GL::ARRAY_BUFFER, colors_as_u8, GL::STATIC_DRAW);
                ctx.enable_vertex_attrib_array(PrimitiveAttribute::Color as u32);
                ctx.vertex_attrib_pointer_f32(
                    PrimitiveAttribute::Color as u32,
                    4,
                    GL::FLOAT,
                    false,
                    0,
                    0,
                );

                // UV0
                let has_uv0 = prim.uv0.len() > 0;
                let color_buffer = ctx.create_buffer().unwrap();
                let uv0_as_u8 = any_slice_to_u8_slice(&prim.uv0);
                ctx.bind_buffer(GL::ARRAY_BUFFER, Some(color_buffer));
                ctx.buffer_data_u8_slice(GL::ARRAY_BUFFER, uv0_as_u8, GL::STATIC_DRAW);
                ctx.enable_vertex_attrib_array(PrimitiveAttribute::UV0 as u32);
                ctx.vertex_attrib_pointer_f32(
                    PrimitiveAttribute::UV0 as u32,
                    2,
                    GL::FLOAT,
                    false,
                    0,
                    0,
                );

                // UV1
                let has_uv1 = prim.uv1.len() > 0;
                let color_buffer = ctx.create_buffer().unwrap();
                let uv1_as_u8 = any_slice_to_u8_slice(&prim.uv1);
                ctx.bind_buffer(GL::ARRAY_BUFFER, Some(color_buffer));
                ctx.buffer_data_u8_slice(GL::ARRAY_BUFFER, uv1_as_u8, GL::STATIC_DRAW);
                ctx.enable_vertex_attrib_array(PrimitiveAttribute::UV1 as u32);
                ctx.vertex_attrib_pointer_f32(
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
                    vao,
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
                    "Set prim compatible hash of prim '{}' of mesh '{}' as '{}'",
                    primitive.name,
                    inter.name,
                    primitive.compatible_hash
                );
                primitives.push(primitive);
            }
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
