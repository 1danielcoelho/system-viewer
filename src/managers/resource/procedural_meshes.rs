use std::{f32::consts::PI, rc::Rc};

use cgmath::{Vector2, Vector3, Vector4};
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
