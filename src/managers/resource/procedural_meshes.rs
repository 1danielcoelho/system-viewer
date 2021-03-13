use crate::managers::resource::collider::{AxisAlignedBoxCollider, SphereCollider};
use crate::managers::resource::intermediate_mesh::{
    generate_dynamic_mesh, generate_screen_space_quad,
};
use crate::managers::resource::intermediate_mesh::{
    intermediate_to_mesh, IntermediateMesh, IntermediatePrimitive,
};
use crate::managers::resource::material::Material;
use crate::managers::resource::mesh::Mesh;
use crate::utils::gl::GL;
use na::{Point3, Vector2, Vector3, Vector4};
use std::{cell::RefCell, f32::consts::PI, rc::Rc};

// Praise songho: http://www.songho.ca/opengl/gl_sphere.html
pub fn generate_lat_long_sphere(
    mut num_lat_segs: u32,
    mut num_long_segs: u32,
    radius: f32,
    smooth_normals: bool,
    mut shared_vertices: bool,
    default_material: Option<Rc<RefCell<Material>>>,
) -> Rc<RefCell<Mesh>> {
    if !smooth_normals {
        shared_vertices = false;
    }
    num_lat_segs = num_lat_segs.max(3);
    num_long_segs = num_long_segs.max(3);

    let long_step = 2.0 * PI / (num_long_segs as f32);
    let lat_step = PI / (num_lat_segs as f32);

    // First and last rows only have 1 triangle per segment, while others have 2
    // Pole vertices are never shared (for different uvs), even with shared_vertices == true
    // Always (num_long_segs + 1) because we have an extra latitudinal seam of vertices for different uvs, even when shared
    let num_verts: usize;
    let num_inds: usize;
    if shared_vertices {
        // 2 "rows" of polar vertices + each remaining vertex row
        num_verts = ((2 * (num_long_segs + 1)) + (num_lat_segs - 1) * (num_long_segs + 1)) as usize; // == (num_lat_segs + 1) * (num_long_segs + 1) btw
        num_inds = (3 * 2 * num_long_segs + 6 * (num_lat_segs - 2) * num_long_segs) as usize;
    } else {
        // 2 "rows" of polar vertex triangles + remaining lat segs * 6 verts per seg
        num_verts = (3 * 2 * num_long_segs + 6 * (num_lat_segs - 2) * num_long_segs) as usize;
        num_inds = num_verts;
    }

    log::info!(
        "Generating uv sphere: Radius: {}, Vertices: {}, Indices: {}, Smooth: {}, Shared verts: {}",
        radius,
        num_verts,
        num_inds,
        smooth_normals,
        shared_vertices
    );

    // Generate unique vertices (extra row for poles and the latitude seam)
    let num_shared_verts = ((num_lat_segs + 1) * (num_long_segs + 1)) as usize;
    let mut temp_positions: Vec<Vector3<f32>> = Vec::new();
    let mut temp_normals: Vec<Vector3<f32>> = Vec::new();
    let mut temp_uv0: Vec<Vector2<f32>> = Vec::new();
    temp_positions.reserve(num_shared_verts);
    temp_uv0.reserve(num_shared_verts);
    temp_normals.reserve(num_shared_verts);

    for lat_index in 0..=num_lat_segs {
        let lat_angle = PI / 2.0 - (lat_index as f32) * lat_step;
        let xy = radius * lat_angle.cos();
        let z = radius * lat_angle.sin();

        for long_index in 0..=num_long_segs {
            let long_angle = (long_index as f32) * long_step;

            let x = xy * long_angle.cos();
            let y = xy * long_angle.sin();

            temp_positions.push(Vector3::new(x, y, z));
            temp_uv0.push(Vector2::new(
                (long_index as f32) / (num_long_segs as f32),
                (lat_index as f32) / (num_lat_segs as f32),
            ));
        }
    }

    // Generate final vertex data
    let mut indices: Vec<u16> = Vec::new();
    let mut positions: Vec<Vector3<f32>> = Vec::new();
    let mut uv0: Vec<Vector2<f32>> = Vec::new();
    indices.reserve(num_inds);
    if shared_vertices {
        for i in 0..num_lat_segs {
            let mut k1 = i * (num_long_segs + 1);
            let mut k2 = k1 + (num_long_segs + 1);

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
        std::mem::swap(&mut temp_positions, &mut positions);
        std::mem::swap(&mut temp_uv0, &mut uv0);
    } else {
        positions.reserve(num_verts);
        uv0.reserve(num_verts);

        let mut index = 0;
        for i in 0..num_lat_segs {
            let mut k1 = i * (num_long_segs + 1);
            let mut k2 = k1 + (num_long_segs + 1);

            for _ in 0..num_long_segs {
                let p_k1 = &temp_positions[k1 as usize];
                let p_k2 = &temp_positions[k2 as usize];
                let p_k1p1 = &temp_positions[(k1 + 1) as usize];
                let p_k2p1 = &temp_positions[(k2 + 1) as usize];

                let uv_k1 = &temp_uv0[k1 as usize];
                let uv_k2 = &temp_uv0[k2 as usize];
                let uv_k1p1 = &temp_uv0[(k1 + 1) as usize];
                let uv_k2p1 = &temp_uv0[(k2 + 1) as usize];

                if i != 0 {
                    positions.push(*p_k1);
                    positions.push(*p_k2);
                    positions.push(*p_k1p1);

                    uv0.push(*uv_k1);
                    uv0.push(*uv_k2);
                    uv0.push(*uv_k1p1);

                    indices.push(index + 0);
                    indices.push(index + 1);
                    indices.push(index + 2);
                    index += 3;
                }

                if i != (num_lat_segs - 1) {
                    positions.push(*p_k1p1);
                    positions.push(*p_k2);
                    positions.push(*p_k2p1);

                    uv0.push(*uv_k1p1);
                    uv0.push(*uv_k2);
                    uv0.push(*uv_k2p1);

                    indices.push(index + 0);
                    indices.push(index + 1);
                    indices.push(index + 2);
                    index += 3;
                }

                k1 += 1;
                k2 += 1;
            }
        }
    }

    // Do normals in a separate pass as they don't need lat/long index
    let mut normals: Vec<Vector3<f32>> = Vec::new();
    let mut tangents: Vec<Vector3<f32>> = Vec::new();
    normals.resize(num_verts, Vector3::new(0.0, 0.0, 1.0));
    tangents.resize(num_verts, Vector3::new(0.0, 0.0, 1.0));
    for triangle_index in 0..(indices.len() / 3) {
        let i0 = indices[triangle_index * 3 + 0] as usize;
        let i1 = indices[triangle_index * 3 + 1] as usize;
        let i2 = indices[triangle_index * 3 + 2] as usize;

        let p0 = &positions[i0];
        let p1 = &positions[i1];
        let p2 = &positions[i2];

        if smooth_normals {
            normals[i0] = p0.normalize();
            normals[i1] = p1.normalize();
            normals[i2] = p2.normalize();
            tangents[i0] = normals[i0].cross(&Vector3::new(0.0, 0.0, 1.0)).normalize();
            tangents[i1] = normals[i1].cross(&Vector3::new(0.0, 0.0, 1.0)).normalize();
            tangents[i2] = normals[i2].cross(&Vector3::new(0.0, 0.0, 1.0)).normalize();
        } else {
            let normal = (p1 - p0).cross(&(p2 - p0)).normalize();
            let tangent = normal.cross(&Vector3::new(0.0, 0.0, 1.0)).normalize();
            normals[i0] = normal;
            normals[i1] = normal;
            normals[i2] = normal;
            tangents[i0] = tangent;
            tangents[i1] = tangent;
            tangents[i2] = tangent;
        }
    }

    log::info!(
        "\tUV sphere final verts: {}/{}, ind: {}/{}, normal: {}/{}, tangent: {}/{}, uv: {}/{}",
        positions.len(),
        positions.capacity(),
        indices.len(),
        indices.capacity(),
        normals.len(),
        normals.capacity(),
        tangents.len(),
        tangents.capacity(),
        uv0.len(),
        uv0.capacity(),
    );

    return intermediate_to_mesh(&IntermediateMesh {
        name: String::from("lat_long_sphere"),
        primitives: vec![IntermediatePrimitive {
            name: String::from("0"),
            indices,
            positions,
            normals,
            tangents,
            colors: vec![],
            uv0,
            uv1: vec![],
            mat: default_material,
            mode: GL::TRIANGLES,
            collider: Some(Box::new(SphereCollider {
                center: Point3::new(0.0, 0.0, 0.0),
                radius2: radius * radius,
            })),
        }],
    });
}

// Praise songho: http://www.songho.ca/opengl/gl_sphere.html
// https://schneide.blog/2016/07/15/generating-an-icosphere-in-c/
// TODO: Fix weirdness when radius < 0
// TODO: Option for shared vertices
// TODO: Texture coordinates
pub fn generate_ico_sphere(
    radius: f32,
    num_subdiv: u32,
    smooth_normals: bool,
    default_material: Option<Rc<RefCell<Material>>>,
) -> Rc<RefCell<Mesh>> {
    let final_num_verts = (20 * 3 * 4u32.pow(num_subdiv)) as usize;

    log::info!(
        "Generating ico sphere: Radius: {}, Num subdiv: {}, Vertices: {}, Smooth: {}",
        radius,
        num_subdiv,
        final_num_verts,
        smooth_normals,
    );

    let mut indices: Vec<u16> = Vec::new();
    let mut positions: Vec<Vector3<f32>> = Vec::new();
    let mut normals: Vec<Vector3<f32>> = Vec::new();
    let mut tangents: Vec<Vector3<f32>> = Vec::new();
    let mut temp_indices: Vec<u16> = Vec::new();
    let mut temp_positions: Vec<Vector3<f32>> = Vec::new();

    indices.reserve(final_num_verts);
    positions.reserve(final_num_verts);
    normals.resize(final_num_verts, Vector3::new(0.0, 0.0, 1.0));
    tangents.resize(final_num_verts, Vector3::new(1.0, 0.0, 0.0));
    temp_indices.reserve(final_num_verts);
    temp_positions.reserve(final_num_verts);

    let x = 0.525731112119133606 * radius;
    let z = 0.850650808352039932 * radius;
    let n = 0.0;

    positions.push(Vector3::new(-x, n, z));
    positions.push(Vector3::new(x, n, z));
    positions.push(Vector3::new(-x, n, -z));
    positions.push(Vector3::new(x, n, -z));
    positions.push(Vector3::new(n, z, x));
    positions.push(Vector3::new(n, z, -x));
    positions.push(Vector3::new(n, -z, x));
    positions.push(Vector3::new(n, -z, -x));
    positions.push(Vector3::new(z, x, n));
    positions.push(Vector3::new(-z, x, n));
    positions.push(Vector3::new(z, -x, n));
    positions.push(Vector3::new(-z, -x, n));

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

        if smooth_normals {
            normals[new_index + 0] = p0.normalize();
            normals[new_index + 1] = p1.normalize();
            normals[new_index + 2] = p2.normalize();
            tangents[new_index + 0] = normals[new_index + 0]
                .cross(&Vector3::new(0.0, 0.0, 1.0))
                .normalize();
            tangents[new_index + 1] = normals[new_index + 1]
                .cross(&Vector3::new(0.0, 0.0, 1.0))
                .normalize();
            tangents[new_index + 2] = normals[new_index + 2]
                .cross(&Vector3::new(0.0, 0.0, 1.0))
                .normalize();
        } else {
            let normal = (p2 - p0).cross(&(p1 - p0)).normalize();
            let tangent = normal.cross(&Vector3::new(0.0, 0.0, 1.0)).normalize();
            normals[new_index + 0] = normal;
            normals[new_index + 1] = normal;
            normals[new_index + 2] = normal;
            tangents[new_index + 0] = tangent;
            tangents[new_index + 1] = tangent;
            tangents[new_index + 2] = tangent;
        }

        new_index += 3;
    }

    std::mem::swap(&mut positions, &mut temp_positions);
    std::mem::swap(&mut indices, &mut temp_indices);

    log::info!(
        "\tIco sphere final verts: {}/{}, ind: {}/{}, normal: {}/{}, tangent: {}/{}, uv: {}/{}",
        positions.len(),
        positions.capacity(),
        indices.len(),
        indices.capacity(),
        normals.len(),
        normals.capacity(),
        tangents.len(),
        tangents.capacity(),
        0,
        0,
    );

    return intermediate_to_mesh(&IntermediateMesh {
        name: String::from("ico_sphere"),
        primitives: vec![IntermediatePrimitive {
            name: String::from("0"),
            indices,
            positions,
            normals,
            tangents,
            colors: vec![],
            uv0: vec![],
            uv1: vec![],
            mat: default_material,
            mode: GL::TRIANGLES,
            collider: Some(Box::new(SphereCollider {
                center: Point3::new(0.0, 0.0, 0.0),
                radius2: radius * radius,
            })),
        }],
    });
}

pub fn generate_canvas_quad(default_material: Option<Rc<RefCell<Material>>>) -> Rc<RefCell<Mesh>> {
    return generate_screen_space_quad(default_material);
}

pub fn generate_temp() -> Rc<RefCell<Mesh>> {
    Rc::new(RefCell::new(Mesh {
        name: String::from("temp"),
        primitives: Vec::new(),
        collider: None,
        ..Default::default()
    }))
}

pub fn generate_cube(default_material: Option<Rc<RefCell<Material>>>) -> Rc<RefCell<Mesh>> {
    intermediate_to_mesh(&IntermediateMesh {
        name: String::from("cube"),
        primitives: vec![IntermediatePrimitive {
            name: String::from("0"),
            indices: vec![
                0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22,
                23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35,
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
                // Face 0
                Vector3::new(-1.0, 0.0, 0.0),
                Vector3::new(-1.0, 0.0, 0.0),
                Vector3::new(-1.0, 0.0, 0.0),
                Vector3::new(-1.0, 0.0, 0.0),
                Vector3::new(-1.0, 0.0, 0.0),
                Vector3::new(-1.0, 0.0, 0.0),
                // Face 1
                Vector3::new(0.0, 0.0, 1.0),
                Vector3::new(0.0, 0.0, 1.0),
                Vector3::new(0.0, 0.0, 1.0),
                Vector3::new(0.0, 0.0, 1.0),
                Vector3::new(0.0, 0.0, 1.0),
                Vector3::new(0.0, 0.0, 1.0),
                // Face 2
                Vector3::new(1.0, 0.0, 0.0),
                Vector3::new(1.0, 0.0, 0.0),
                Vector3::new(1.0, 0.0, 0.0),
                Vector3::new(1.0, 0.0, 0.0),
                Vector3::new(1.0, 0.0, 0.0),
                Vector3::new(1.0, 0.0, 0.0),
                // Face 3
                Vector3::new(0.0, 0.0, -1.0),
                Vector3::new(0.0, 0.0, -1.0),
                Vector3::new(0.0, 0.0, -1.0),
                Vector3::new(0.0, 0.0, -1.0),
                Vector3::new(0.0, 0.0, -1.0),
                Vector3::new(0.0, 0.0, -1.0),
                // Face 4
                Vector3::new(0.0, 1.0, 0.0),
                Vector3::new(0.0, 1.0, 0.0),
                Vector3::new(0.0, 1.0, 0.0),
                Vector3::new(0.0, 1.0, 0.0),
                Vector3::new(0.0, 1.0, 0.0),
                Vector3::new(0.0, 1.0, 0.0),
                // Face 5
                Vector3::new(0.0, -1.0, 0.0),
                Vector3::new(0.0, -1.0, 0.0),
                Vector3::new(0.0, -1.0, 0.0),
                Vector3::new(0.0, -1.0, 0.0),
                Vector3::new(0.0, -1.0, 0.0),
                Vector3::new(0.0, -1.0, 0.0),
            ],
            tangents: vec![
                // Face 0
                Vector3::new(0.0, -1.0, 0.0),
                Vector3::new(0.0, -1.0, 0.0),
                Vector3::new(0.0, -1.0, 0.0),
                Vector3::new(0.0, -1.0, 0.0),
                Vector3::new(0.0, -1.0, 0.0),
                Vector3::new(0.0, -1.0, 0.0),
                // Face 1
                Vector3::new(1.0, 0.0, 0.0),
                Vector3::new(1.0, 0.0, 0.0),
                Vector3::new(1.0, 0.0, 0.0),
                Vector3::new(1.0, 0.0, 0.0),
                Vector3::new(1.0, 0.0, 0.0),
                Vector3::new(1.0, 0.0, 0.0),
                // Face 2
                Vector3::new(0.0, 1.0, 0.0),
                Vector3::new(0.0, 1.0, 0.0),
                Vector3::new(0.0, 1.0, 0.0),
                Vector3::new(0.0, 1.0, 0.0),
                Vector3::new(0.0, 1.0, 0.0),
                Vector3::new(0.0, 1.0, 0.0),
                // Face 3
                Vector3::new(-1.0, 0.0, 0.0),
                Vector3::new(-1.0, 0.0, 0.0),
                Vector3::new(-1.0, 0.0, 0.0),
                Vector3::new(-1.0, 0.0, 0.0),
                Vector3::new(-1.0, 0.0, 0.0),
                Vector3::new(-1.0, 0.0, 0.0),
                // Face 4
                Vector3::new(-1.0, 0.0, 0.0),
                Vector3::new(-1.0, 0.0, 0.0),
                Vector3::new(-1.0, 0.0, 0.0),
                Vector3::new(-1.0, 0.0, 0.0),
                Vector3::new(-1.0, 0.0, 0.0),
                Vector3::new(-1.0, 0.0, 0.0),
                // Face 5
                Vector3::new(1.0, 0.0, 0.0),
                Vector3::new(1.0, 0.0, 0.0),
                Vector3::new(1.0, 0.0, 0.0),
                Vector3::new(1.0, 0.0, 0.0),
                Vector3::new(1.0, 0.0, 0.0),
                Vector3::new(1.0, 0.0, 0.0),
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
                Vector2::new(1.0, 1.0),
                Vector2::new(1.0, 0.0),
                Vector2::new(0.0, 0.0),
                Vector2::new(1.0, 1.0),
                Vector2::new(0.0, 0.0),
                Vector2::new(0.0, 1.0),
                // Face 1
                Vector2::new(0.0, 1.0),
                Vector2::new(1.0, 1.0),
                Vector2::new(0.0, 0.0),
                Vector2::new(1.0, 1.0),
                Vector2::new(1.0, 0.0),
                Vector2::new(0.0, 0.0),
                // Face 2
                Vector2::new(0.0, 0.0),
                Vector2::new(0.0, 1.0),
                Vector2::new(1.0, 1.0),
                Vector2::new(1.0, 1.0),
                Vector2::new(1.0, 0.0),
                Vector2::new(0.0, 0.0),
                // Face 3
                Vector2::new(0.0, 1.0),
                Vector2::new(1.0, 1.0),
                Vector2::new(0.0, 0.0),
                Vector2::new(1.0, 1.0),
                Vector2::new(1.0, 0.0),
                Vector2::new(0.0, 0.0),
                // Face 4
                Vector2::new(1.0, 1.0),
                Vector2::new(1.0, 0.0),
                Vector2::new(0.0, 0.0),
                Vector2::new(1.0, 1.0),
                Vector2::new(0.0, 0.0),
                Vector2::new(0.0, 1.0),
                // Face 5
                Vector2::new(0.0, 1.0),
                Vector2::new(1.0, 1.0),
                Vector2::new(1.0, 0.0),
                Vector2::new(0.0, 1.0),
                Vector2::new(1.0, 0.0),
                Vector2::new(0.0, 0.0),
            ],
            uv1: vec![
                // Face 0
                Vector2::new(1.0, 1.0),
                Vector2::new(1.0, 0.0),
                Vector2::new(0.0, 0.0),
                Vector2::new(1.0, 1.0),
                Vector2::new(0.0, 0.0),
                Vector2::new(0.0, 1.0),
                // Face 1
                Vector2::new(0.0, 1.0),
                Vector2::new(1.0, 1.0),
                Vector2::new(0.0, 0.0),
                Vector2::new(1.0, 1.0),
                Vector2::new(1.0, 0.0),
                Vector2::new(0.0, 0.0),
                // Face 2
                Vector2::new(0.0, 0.0),
                Vector2::new(0.0, 1.0),
                Vector2::new(1.0, 1.0),
                Vector2::new(1.0, 1.0),
                Vector2::new(1.0, 0.0),
                Vector2::new(0.0, 0.0),
                // Face 3
                Vector2::new(0.0, 1.0),
                Vector2::new(1.0, 1.0),
                Vector2::new(0.0, 0.0),
                Vector2::new(1.0, 1.0),
                Vector2::new(1.0, 0.0),
                Vector2::new(0.0, 0.0),
                // Face 4
                Vector2::new(1.0, 1.0),
                Vector2::new(1.0, 0.0),
                Vector2::new(0.0, 0.0),
                Vector2::new(1.0, 1.0),
                Vector2::new(0.0, 0.0),
                Vector2::new(0.0, 1.0),
                // Face 5
                Vector2::new(0.0, 1.0),
                Vector2::new(1.0, 1.0),
                Vector2::new(1.0, 0.0),
                Vector2::new(0.0, 1.0),
                Vector2::new(1.0, 0.0),
                Vector2::new(0.0, 0.0),
            ],
            mode: GL::TRIANGLES,
            mat: default_material,
            collider: Some(Box::new(AxisAlignedBoxCollider {
                maxes: Point3::new(1.0, 1.0, 1.0),
                mins: Point3::new(-1.0, -1.0, -1.0),
            })),
        }],
    })
}

pub fn generate_plane(default_material: Option<Rc<RefCell<Material>>>) -> Rc<RefCell<Mesh>> {
    intermediate_to_mesh(&IntermediateMesh {
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
            tangents: vec![
                Vector3::new(1.0, 0.0, 0.0),
                Vector3::new(1.0, 0.0, 0.0),
                Vector3::new(1.0, 0.0, 0.0),
                Vector3::new(1.0, 0.0, 0.0),
                Vector3::new(1.0, 0.0, 0.0),
                Vector3::new(1.0, 0.0, 0.0),
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
                Vector2::new(1.0, 1.0),
                Vector2::new(1.0, 0.0),
                Vector2::new(0.0, 1.0),
                Vector2::new(0.0, 0.0),
                Vector2::new(0.0, 1.0),
                Vector2::new(1.0, 0.0),
            ],
            uv1: vec![
                Vector2::new(1.0, 1.0),
                Vector2::new(1.0, 0.0),
                Vector2::new(0.0, 1.0),
                Vector2::new(0.0, 0.0),
                Vector2::new(0.0, 1.0),
                Vector2::new(1.0, 0.0),
            ],
            mode: GL::TRIANGLES,
            mat: default_material,
            collider: Some(Box::new(AxisAlignedBoxCollider {
                maxes: Point3::new(1.0, 1.0, 0.00001), // @Hack. Maybe use a mesh collider for the plane?
                mins: Point3::new(-1.0, -1.0, -0.00001),
            })),
        }],
    })
}

pub fn generate_grid(
    num_lines: u32,
    default_material: Option<Rc<RefCell<Material>>>,
) -> Rc<RefCell<Mesh>> {
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

    return intermediate_to_mesh(&IntermediateMesh {
        name: String::from("grid"),
        primitives: vec![IntermediatePrimitive {
            name: String::from("0"),
            indices,
            positions,
            normals: vec![],
            tangents: vec![],
            colors,
            uv0: vec![],
            uv1: vec![],
            mat: default_material,
            mode: GL::LINES,
            collider: None,
        }],
    });
}

pub fn generate_circle(
    num_pts: u32,
    default_material: Option<Rc<RefCell<Material>>>,
) -> Rc<RefCell<Mesh>> {
    assert!(num_pts > 2);

    let mut positions: Vec<Vector3<f32>> = Vec::new();
    positions.reserve((num_pts) as usize);

    // TODO: Remove this and use a different shader
    let mut colors: Vec<Vector4<f32>> = Vec::new();
    colors.resize((num_pts) as usize, Vector4::new(1.0, 1.0, 1.0, 1.0));

    // uv0.x gives angle to true anomaly 0 degrees
    let mut uv0: Vec<Vector2<f32>> = Vec::new();
    uv0.reserve((num_pts) as usize);

    let mut indices: Vec<u16> = Vec::new();
    indices.reserve((num_pts * 2) as usize);

    let incr: f32 = 1.0 / num_pts as f32;
    for i in 0..num_pts {
        let sum_angle = (2.0 * std::f32::consts::PI) * incr * i as f32;

        positions.push(Vector3::new(sum_angle.cos(), sum_angle.sin(), 0.0));
        uv0.push(Vector2::new(incr, 0.0));

        indices.push(i as u16);
        indices.push((i + 1) as u16);
    }

    let last_index = indices.len() - 1;
    indices[last_index] = 0;

    return intermediate_to_mesh(&IntermediateMesh {
        name: String::from("circle"),
        primitives: vec![IntermediatePrimitive {
            name: String::from("0"),
            indices,
            positions,
            normals: vec![],
            tangents: vec![],
            colors,
            uv0,
            uv1: vec![],
            mat: default_material,
            mode: GL::LINES,
            collider: None,
        }],
    });
}

pub fn generate_axes(default_material: Option<Rc<RefCell<Material>>>) -> Rc<RefCell<Mesh>> {
    intermediate_to_mesh(&IntermediateMesh {
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
            tangents: vec![],
            colors: vec![
                Vector4::new(1.0, 1.0, 1.0, 1.0),
                Vector4::new(1.0, 0.0, 0.0, 1.0),
                Vector4::new(0.0, 1.0, 0.0, 1.0),
                Vector4::new(0.0, 0.0, 1.0, 1.0),
            ],
            uv0: vec![],
            uv1: vec![],
            mode: GL::LINES,
            mat: default_material,
            collider: None,
        }],
    })
}

pub fn generate_points() -> Rc<RefCell<Mesh>> {
    return generate_dynamic_mesh();
}
