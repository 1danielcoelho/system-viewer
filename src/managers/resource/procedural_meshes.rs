use std::rc::Rc;

use cgmath::{Vector2, Vector3, Vector4};
use web_sys::WebGlRenderingContext as GL;
use web_sys::WebGlRenderingContext;

use super::{
    intermediate_mesh::intermediate_to_mesh, intermediate_mesh::IntermediateMesh,
    intermediate_mesh::IntermediatePrimitive, Material, Mesh,
};

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
