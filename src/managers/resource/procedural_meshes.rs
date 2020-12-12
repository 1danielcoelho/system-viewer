use std::{f32::consts::PI, rc::Rc};

use cgmath::{InnerSpace, Vector2, Vector3, Vector4};
use web_sys::WebGlRenderingContext as GL;
use web_sys::WebGlRenderingContext;

use super::{
    intermediate_mesh::intermediate_to_mesh, intermediate_mesh::IntermediateMesh,
    intermediate_mesh::IntermediatePrimitive, Material, Mesh,
};

// Praise songho: http://www.songho.ca/opengl/gl_sphere.html
pub fn generate_lat_long_sphere(
    ctx: &WebGlRenderingContext,
    num_lat_segs: u32,
    num_long_segs: u32,
    radius: f32,
    default_material: Option<Rc<Material>>,
) -> Rc<Mesh> {
    let long_step = 2.0 * PI / (num_long_segs as f32);
    let lat_step = PI / (num_lat_segs as f32);
    let inv_radius = 1.0 / radius;

    let num_verts = ((num_lat_segs + 1) * (num_long_segs + 1)) as usize; // Poles have a ton of vertices for different uvs

    let mut indices: Vec<u16> = Vec::new();
    let mut positions: Vec<Vector3<f32>> = Vec::new();
    let mut normals: Vec<Vector3<f32>> = Vec::new();
    let mut uv0: Vec<Vector2<f32>> = Vec::new();

    indices.reserve(num_verts);
    positions.reserve(num_verts);
    normals.reserve(num_verts);
    uv0.reserve(num_verts);

    for lat_index in 0..=num_lat_segs {
        let lat_angle = PI / 2.0 - (lat_index as f32) * lat_step;
        let xy = radius * lat_angle.cos();
        let z = radius * lat_angle.sin();

        for long_index in 0..=num_long_segs {
            let long_angle = (long_index as f32) * long_step;

            let x = xy * long_angle.cos();
            let y = xy * long_angle.sin();

            positions.push(Vector3::new(x, y, z));
            normals.push(Vector3::new(x * inv_radius, y * inv_radius, z * inv_radius));
            uv0.push(Vector2::new(
                (long_index as f32) / (num_long_segs as f32),
                1.0 - (lat_index as f32) / (num_lat_segs as f32), // Flip here because we iterate top to bottom but I want UV y 0 at the bottom
            ));
        }
    }

    for i in 0..num_lat_segs {
        let mut k1 = i * (num_long_segs + 1);
        let mut k2 = k1 + num_long_segs + 1;

        for _ in 0..num_long_segs {
            if i != 0 {
                indices.push(k1 as u16);
                indices.push(k2 as u16);
                indices.push((k1 + 1) as u16);
            }

            if i != (num_lat_segs - 1) {
                indices.push((k1 + 1) as u16);
                indices.push(k2 as u16);
                indices.push((k2 + 1) as u16);
            }

            k1 += 1;
            k2 += 1;
        }
    }

    return intermediate_to_mesh(
        IntermediateMesh {
            name: String::from("lat_long_sphere"),
            primitives: vec![IntermediatePrimitive {
                name: String::from("0"),
                indices,
                positions,
                normals,
                colors: vec![],
                uv0,
                uv1: vec![],
                mat: default_material,
                mode: GL::TRIANGLES,
            }],
        },
        ctx,
    );
}

// Praise songho: http://www.songho.ca/opengl/gl_sphere.html
// https://schneide.blog/2016/07/15/generating-an-icosphere-in-c/
pub fn generate_ico_sphere(
    ctx: &WebGlRenderingContext,
    radius: f32,
    num_subdiv: u32,
    default_material: Option<Rc<Material>>,
) -> Rc<Mesh> {
    let final_num_verts = (20 * 3 * 4u32.pow(num_subdiv)) as usize;

    let mut indices: Vec<u16> = Vec::new();
    let mut positions: Vec<Vector3<f32>> = Vec::new();
    let mut normals: Vec<Vector3<f32>> = Vec::new();
    let mut temp_indices: Vec<u16> = Vec::new();
    let mut temp_positions: Vec<Vector3<f32>> = Vec::new();

    indices.reserve(final_num_verts);
    positions.reserve(final_num_verts);
    normals.resize(final_num_verts, Vector3::new(0.0, 0.0, 1.0));
    temp_indices.reserve(final_num_verts);
    temp_positions.reserve(final_num_verts);

    let X = 0.525731112119133606 * radius;
    let Z = 0.850650808352039932 * radius;
    let N = 0.0;

    positions.push(Vector3::new(-X, N, Z));
    positions.push(Vector3::new(X, N, Z));
    positions.push(Vector3::new(-X, N, -Z));
    positions.push(Vector3::new(X, N, -Z));
    positions.push(Vector3::new(N, Z, X));
    positions.push(Vector3::new(N, Z, -X));
    positions.push(Vector3::new(N, -Z, X));
    positions.push(Vector3::new(N, -Z, -X));
    positions.push(Vector3::new(Z, X, N));
    positions.push(Vector3::new(-Z, X, N));
    positions.push(Vector3::new(Z, -X, N));
    positions.push(Vector3::new(-Z, -X, N));

    // 60 indices
    indices = vec![
        0, 4, 1, 0, 9, 4, 9, 5, 4, 4, 5, 8, 4, 8, 1, 8, 10, 1, 8, 3, 10, 5, 3, 8, 5, 2, 3, 2, 7, 3,
        7, 10, 3, 7, 6, 10, 7, 11, 6, 11, 0, 6, 0, 1, 6, 6, 1, 10, 9, 0, 11, 9, 11, 2, 9, 2, 5, 7,
        2, 11,
    ];

    // We start off with 12 positions, but for each subdiv we'll only share vertices within a face
    // Doesn't matter much as we'll end up with 3 verts per triangle in the end anyway, and this way
    // the algorithm is simple
    let mut num_verts_after_subdiv = 30;
    let mut num_indices_after_subdiv = 60;
    for _ in 0..num_subdiv {
        num_verts_after_subdiv *= 4;
        num_indices_after_subdiv *= 4;

        temp_positions.resize(num_verts_after_subdiv, Vector3::new(0.0, 0.0, 0.0));
        temp_indices.resize(num_indices_after_subdiv, 0);

        let mut new_vert_index = 0;
        let mut new_indices_index = 0;

        // Step over each triangle from the previous subdiv, and create 4 others in the temp vecs
        for triangle_index in 0..(indices.len() / 3) {
            let p0 = &positions[indices[triangle_index * 3 + 0] as usize];
            let p1 = &positions[indices[triangle_index * 3 + 1] as usize];
            let p2 = &positions[indices[triangle_index * 3 + 2] as usize];

            //         p0
            //        / \
            //       / 1 \
            //      /     \
            //     p3------p4
            //    / \  3  / \
            //   /   \   /   \
            //  /  2  \ /  4  \
            // p1------p5------p2

            temp_positions[new_vert_index + 0] = *p0;
            temp_positions[new_vert_index + 1] = *p1;
            temp_positions[new_vert_index + 2] = *p2;
            temp_positions[new_vert_index + 3] = radius * (p0 + p1).normalize();
            temp_positions[new_vert_index + 4] = radius * (p0 + p2).normalize();
            temp_positions[new_vert_index + 5] = radius * (p1 + p2).normalize();

            temp_indices[new_indices_index + 0] = (new_vert_index + 0) as u16;
            temp_indices[new_indices_index + 1] = (new_vert_index + 3) as u16;
            temp_indices[new_indices_index + 2] = (new_vert_index + 4) as u16;

            temp_indices[new_indices_index + 3] = (new_vert_index + 3) as u16;
            temp_indices[new_indices_index + 4] = (new_vert_index + 1) as u16;
            temp_indices[new_indices_index + 5] = (new_vert_index + 5) as u16;

            temp_indices[new_indices_index + 6] = (new_vert_index + 3) as u16;
            temp_indices[new_indices_index + 7] = (new_vert_index + 5) as u16;
            temp_indices[new_indices_index + 8] = (new_vert_index + 4) as u16;

            temp_indices[new_indices_index + 9] = (new_vert_index + 4) as u16;
            temp_indices[new_indices_index + 10] = (new_vert_index + 5) as u16;
            temp_indices[new_indices_index + 11] = (new_vert_index + 2) as u16;

            new_vert_index += 6;
            new_indices_index += 12;
        }

        std::mem::swap(&mut positions, &mut temp_positions);
        std::mem::swap(&mut indices, &mut temp_indices);
    }

    // Final pass to generate 1 vertex per triangle and normals.. no uv1 yet because that's non-trivial 
    temp_positions.resize(final_num_verts, Vector3::new(0.0, 0.0, 0.0));
    temp_indices.resize(final_num_verts, 0);
    let mut new_index = 0;
    for triangle_index in 0..(indices.len() / 3) {
        let p0 = &positions[indices[triangle_index * 3 + 0] as usize];
        let p1 = &positions[indices[triangle_index * 3 + 1] as usize];
        let p2 = &positions[indices[triangle_index * 3 + 2] as usize];

        temp_positions[new_index + 0] = *p0;
        temp_positions[new_index + 1] = *p1;
        temp_positions[new_index + 2] = *p2;

        // TODO: Why did I need to flip winding order here? I think my original icosahedron was clockwise?
        temp_indices[new_index + 0] = (new_index + 0) as u16;
        temp_indices[new_index + 2] = (new_index + 1) as u16;
        temp_indices[new_index + 1] = (new_index + 2) as u16;

        normals[new_index + 0] = p0.normalize();
        normals[new_index + 1] = p1.normalize();
        normals[new_index + 2] = p2.normalize();

        new_index += 3;
    }

    std::mem::swap(&mut positions, &mut temp_positions);
    std::mem::swap(&mut indices, &mut temp_indices);

    return intermediate_to_mesh(
        IntermediateMesh {
            name: String::from("ico_sphere"),
            primitives: vec![IntermediatePrimitive {
                name: String::from("0"),
                indices,
                positions,
                normals,
                colors: vec![],
                uv0: vec![],
                uv1: vec![],
                mat: default_material,
                mode: GL::TRIANGLES,
            }],
        },
        ctx,
    );
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
                    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21,
                    22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35,
                ],
                positions: vec![
                    // Face 0
                    Vector3::new(-1.0, -1.0, -1.0), //0
                    Vector3::new(-1.0, -1.0, 1.0),  //1
                    Vector3::new(-1.0, 1.0, 1.0),   //3
                    Vector3::new(-1.0, -1.0, -1.0), //0
                    Vector3::new(-1.0, 1.0, 1.0),   //3
                    Vector3::new(-1.0, 1.0, -1.0),  //2
                    // Face 1
                    Vector3::new(-1.0, -1.0, 1.0), //1
                    Vector3::new(1.0, -1.0, 1.0),  //5
                    Vector3::new(-1.0, 1.0, 1.0),  //3
                    Vector3::new(1.0, -1.0, 1.0),  //5
                    Vector3::new(1.0, 1.0, 1.0),   //7
                    Vector3::new(-1.0, 1.0, 1.0),  //3
                    // Face 2
                    Vector3::new(1.0, -1.0, 1.0),  //5
                    Vector3::new(1.0, -1.0, -1.0), //4
                    Vector3::new(1.0, 1.0, -1.0),  //6
                    Vector3::new(1.0, 1.0, -1.0),  //6
                    Vector3::new(1.0, 1.0, 1.0),   //7
                    Vector3::new(1.0, -1.0, 1.0),  //5
                    // Face 3
                    Vector3::new(-1.0, -1.0, -1.0), //0
                    Vector3::new(-1.0, 1.0, -1.0),  //2
                    Vector3::new(1.0, -1.0, -1.0),  //4
                    Vector3::new(-1.0, 1.0, -1.0),  //2
                    Vector3::new(1.0, 1.0, -1.0),   //6
                    Vector3::new(1.0, -1.0, -1.0),  //4
                    // Face 4
                    Vector3::new(-1.0, 1.0, -1.0), //2
                    Vector3::new(-1.0, 1.0, 1.0),  //3
                    Vector3::new(1.0, 1.0, 1.0),   //7
                    Vector3::new(-1.0, 1.0, -1.0), //2
                    Vector3::new(1.0, 1.0, 1.0),   //7
                    Vector3::new(1.0, 1.0, -1.0),  //6
                    // Face 5
                    Vector3::new(-1.0, -1.0, -1.0), //0
                    Vector3::new(1.0, -1.0, -1.0),  //4
                    Vector3::new(1.0, -1.0, 1.0),   //5
                    Vector3::new(-1.0, -1.0, -1.0), //0
                    Vector3::new(1.0, -1.0, 1.0),   //5
                    Vector3::new(-1.0, -1.0, 1.0),  //1
                ],
                normals: vec![
                    Vector3::new(-1.0, 0.0, 0.0),
                    Vector3::new(-1.0, 0.0, 0.0),
                    Vector3::new(-1.0, 0.0, 0.0),
                    Vector3::new(-1.0, 0.0, 0.0),
                    Vector3::new(-1.0, 0.0, 0.0),
                    Vector3::new(-1.0, 0.0, 0.0),
                    Vector3::new(0.0, 0.0, 1.0),
                    Vector3::new(0.0, 0.0, 1.0),
                    Vector3::new(0.0, 0.0, 1.0),
                    Vector3::new(0.0, 0.0, 1.0),
                    Vector3::new(0.0, 0.0, 1.0),
                    Vector3::new(0.0, 0.0, 1.0),
                    Vector3::new(1.0, 0.0, 0.0),
                    Vector3::new(1.0, 0.0, 0.0),
                    Vector3::new(1.0, 0.0, 0.0),
                    Vector3::new(1.0, 0.0, 0.0),
                    Vector3::new(1.0, 0.0, 0.0),
                    Vector3::new(1.0, 0.0, 0.0),
                    Vector3::new(0.0, 0.0, -1.0),
                    Vector3::new(0.0, 0.0, -1.0),
                    Vector3::new(0.0, 0.0, -1.0),
                    Vector3::new(0.0, 0.0, -1.0),
                    Vector3::new(0.0, 0.0, -1.0),
                    Vector3::new(0.0, 0.0, -1.0),
                    Vector3::new(0.0, 1.0, 0.0),
                    Vector3::new(0.0, 1.0, 0.0),
                    Vector3::new(0.0, 1.0, 0.0),
                    Vector3::new(0.0, 1.0, 0.0),
                    Vector3::new(0.0, 1.0, 0.0),
                    Vector3::new(0.0, 1.0, 0.0),
                    Vector3::new(0.0, -1.0, 0.0),
                    Vector3::new(0.0, -1.0, 0.0),
                    Vector3::new(0.0, -1.0, 0.0),
                    Vector3::new(0.0, -1.0, 0.0),
                    Vector3::new(0.0, -1.0, 0.0),
                    Vector3::new(0.0, -1.0, 0.0),
                ],
                colors: vec![
                    Vector4::new(0.0, 0.0, 0.0, 1.0), //0
                    Vector4::new(0.0, 0.0, 1.0, 1.0), //1
                    Vector4::new(0.0, 1.0, 1.0, 1.0), //3
                    Vector4::new(0.0, 0.0, 0.0, 1.0), //0
                    Vector4::new(0.0, 1.0, 1.0, 1.0), //3
                    Vector4::new(0.0, 1.0, 0.0, 1.0), //2
                    Vector4::new(0.0, 0.0, 1.0, 1.0), //1
                    Vector4::new(1.0, 0.0, 1.0, 1.0), //5
                    Vector4::new(0.0, 1.0, 1.0, 1.0), //3
                    Vector4::new(1.0, 0.0, 1.0, 1.0), //5
                    Vector4::new(1.0, 1.0, 1.0, 1.0), //7
                    Vector4::new(0.0, 1.0, 1.0, 1.0), //3
                    Vector4::new(1.0, 0.0, 1.0, 1.0), //5
                    Vector4::new(1.0, 0.0, 0.0, 1.0), //4
                    Vector4::new(1.0, 1.0, 0.0, 1.0), //6
                    Vector4::new(1.0, 1.0, 0.0, 1.0), //6
                    Vector4::new(1.0, 1.0, 1.0, 1.0), //7
                    Vector4::new(1.0, 0.0, 1.0, 1.0), //5
                    Vector4::new(0.0, 0.0, 0.0, 1.0), //0
                    Vector4::new(0.0, 1.0, 0.0, 1.0), //2
                    Vector4::new(1.0, 0.0, 0.0, 1.0), //4
                    Vector4::new(0.0, 1.0, 0.0, 1.0), //2
                    Vector4::new(1.0, 1.0, 0.0, 1.0), //6
                    Vector4::new(1.0, 0.0, 0.0, 1.0), //4
                    Vector4::new(0.0, 1.0, 0.0, 1.0), //2
                    Vector4::new(0.0, 1.0, 1.0, 1.0), //3
                    Vector4::new(1.0, 1.0, 1.0, 1.0), //7
                    Vector4::new(0.0, 1.0, 0.0, 1.0), //2
                    Vector4::new(1.0, 1.0, 1.0, 1.0), //7
                    Vector4::new(1.0, 1.0, 0.0, 1.0), //6
                    Vector4::new(0.0, 0.0, 0.0, 1.0), //0
                    Vector4::new(1.0, 0.0, 0.0, 1.0), //4
                    Vector4::new(1.0, 0.0, 1.0, 1.0), //5
                    Vector4::new(0.0, 0.0, 0.0, 1.0), //0
                    Vector4::new(1.0, 0.0, 1.0, 1.0), //5
                    Vector4::new(0.0, 0.0, 1.0, 1.0), //1
                ],
                uv0: vec![
                    // Face 0
                    Vector2::new(1.0, 0.0),
                    Vector2::new(1.0, 1.0),
                    Vector2::new(0.0, 1.0),
                    Vector2::new(1.0, 0.0),
                    Vector2::new(0.0, 1.0),
                    Vector2::new(0.0, 0.0),
                    // Face 1
                    Vector2::new(0.0, 1.0),
                    Vector2::new(0.0, 0.0),
                    Vector2::new(1.0, 1.0),
                    Vector2::new(0.0, 0.0),
                    Vector2::new(1.0, 0.0),
                    Vector2::new(1.0, 1.0),
                    // Face 2
                    Vector2::new(0.0, 1.0),
                    Vector2::new(0.0, 0.0),
                    Vector2::new(1.0, 0.0),
                    Vector2::new(1.0, 0.0),
                    Vector2::new(1.0, 1.0),
                    Vector2::new(0.0, 1.0),
                    // Face 3
                    Vector2::new(0.0, 0.0),
                    Vector2::new(1.0, 0.0),
                    Vector2::new(0.0, 1.0),
                    Vector2::new(1.0, 0.0),
                    Vector2::new(1.0, 1.0),
                    Vector2::new(0.0, 1.0),
                    // Face 4
                    Vector2::new(1.0, 0.0),
                    Vector2::new(1.0, 1.0),
                    Vector2::new(0.0, 1.0),
                    Vector2::new(1.0, 0.0),
                    Vector2::new(0.0, 1.0),
                    Vector2::new(0.0, 0.0),
                    // Face 5
                    Vector2::new(0.0, 0.0),
                    Vector2::new(1.0, 0.0),
                    Vector2::new(1.0, 1.0),
                    Vector2::new(0.0, 0.0),
                    Vector2::new(1.0, 1.0),
                    Vector2::new(0.0, 1.0),
                ],
                uv1: vec![
                    // Face 0
                    Vector2::new(1.0, 0.0),
                    Vector2::new(1.0, 1.0),
                    Vector2::new(0.0, 1.0),
                    Vector2::new(1.0, 0.0),
                    Vector2::new(0.0, 1.0),
                    Vector2::new(0.0, 0.0),
                    // Face 1
                    Vector2::new(0.0, 1.0),
                    Vector2::new(0.0, 0.0),
                    Vector2::new(1.0, 1.0),
                    Vector2::new(0.0, 0.0),
                    Vector2::new(1.0, 0.0),
                    Vector2::new(1.0, 1.0),
                    // Face 2
                    Vector2::new(0.0, 1.0),
                    Vector2::new(0.0, 0.0),
                    Vector2::new(1.0, 0.0),
                    Vector2::new(1.0, 0.0),
                    Vector2::new(1.0, 1.0),
                    Vector2::new(0.0, 1.0),
                    // Face 3
                    Vector2::new(0.0, 0.0),
                    Vector2::new(1.0, 0.0),
                    Vector2::new(0.0, 1.0),
                    Vector2::new(1.0, 0.0),
                    Vector2::new(1.0, 1.0),
                    Vector2::new(0.0, 1.0),
                    // Face 4
                    Vector2::new(1.0, 0.0),
                    Vector2::new(1.0, 1.0),
                    Vector2::new(0.0, 1.0),
                    Vector2::new(1.0, 0.0),
                    Vector2::new(0.0, 1.0),
                    Vector2::new(0.0, 0.0),
                    // Face 5
                    Vector2::new(0.0, 0.0),
                    Vector2::new(1.0, 0.0),
                    Vector2::new(1.0, 1.0),
                    Vector2::new(0.0, 0.0),
                    Vector2::new(1.0, 1.0),
                    Vector2::new(0.0, 1.0),
                ],
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
                indices: vec![0, 1, 2, 3, 4, 5],
                positions: vec![
                    Vector3::new(1.0, -1.0, 0.0),
                    Vector3::new(1.0, 1.0, 0.0),
                    Vector3::new(-1.0, -1.0, 0.0),
                    Vector3::new(-1.0, 1.0, 0.0),
                    Vector3::new(-1.0, -1.0, 0.0),
                    Vector3::new(1.0, 1.0, 0.0),
                ],
                normals: vec![
                    Vector3::new(0.0, 0.0, 1.0),
                    Vector3::new(0.0, 0.0, 1.0),
                    Vector3::new(0.0, 0.0, 1.0),
                    Vector3::new(0.0, 0.0, 1.0),
                    Vector3::new(0.0, 0.0, 1.0),
                    Vector3::new(0.0, 0.0, 1.0),
                ],
                colors: vec![
                    Vector4::new(0.0, 0.0, 0.0, 1.0),
                    Vector4::new(1.0, 0.0, 0.0, 1.0),
                    Vector4::new(0.0, 1.0, 0.0, 1.0),
                    Vector4::new(1.0, 1.0, 0.0, 1.0),
                    Vector4::new(0.0, 1.0, 0.0, 1.0),
                    Vector4::new(1.0, 0.0, 0.0, 1.0),
                ],
                uv0: vec![
                    Vector2::new(0.0, 0.0),
                    Vector2::new(1.0, 0.0),
                    Vector2::new(0.0, 1.0),
                    Vector2::new(1.0, 1.0),
                    Vector2::new(0.0, 1.0),
                    Vector2::new(1.0, 0.0),
                ],
                uv1: vec![
                    Vector2::new(0.0, 0.0),
                    Vector2::new(1.0, 0.0),
                    Vector2::new(0.0, 1.0),
                    Vector2::new(1.0, 1.0),
                    Vector2::new(0.0, 1.0),
                    Vector2::new(1.0, 0.0),
                ],
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
