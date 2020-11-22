use js_sys::WebAssembly;
use std::{collections::HashMap, rc::Rc};
use wasm_bindgen::JsCast;
use web_sys::WebGlRenderingContext;
use web_sys::{WebGlProgram, WebGlRenderingContext as GL, WebGlShader};

use crate::systems::rendering::{Material, Mesh, Primitive, Texture};

fn generate_cube(ctx: &WebGlRenderingContext, default_material: Option<Rc<Material>>) -> Rc<Mesh> {
    let vertices_cube: [f32; 24] = [
        -1.0, -1.0, -1.0, -1.0, -1.0, 1.0, -1.0, 1.0, -1.0, -1.0, 1.0, 1.0, 1.0, -1.0, -1.0, 1.0,
        -1.0, 1.0, 1.0, 1.0, -1.0, 1.0, 1.0, 1.0,
    ];

    let colors_cube: [f32; 24] = [
        0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 1.0, 0.0, 0.0, 1.0, 1.0, 1.0, 0.0, 0.0, 1.0, 0.0, 1.0,
        1.0, 1.0, 0.0, 1.0, 1.0, 1.0,
    ];

    let indices_cube: [u16; 36] = [
        0, 1, 3, 0, 3, 2, 1, 5, 3, 5, 7, 3, 5, 4, 6, 6, 7, 5, 0, 2, 4, 2, 6, 4, 2, 3, 7, 2, 7, 6,
        0, 4, 5, 0, 5, 1,
    ];

    // Vertex positions
    let memory_buffer = wasm_bindgen::memory()
        .dyn_into::<WebAssembly::Memory>()
        .unwrap()
        .buffer();
    let vertices_location = vertices_cube.as_ptr() as u32 / 4;
    let vert_array = js_sys::Float32Array::new(&memory_buffer).subarray(
        vertices_location,
        vertices_location + vertices_cube.len() as u32,
    );
    let buffer_position = ctx
        .create_buffer()
        .ok_or("failed to create buffer")
        .unwrap();
    ctx.bind_buffer(GL::ARRAY_BUFFER, Some(&buffer_position));
    ctx.buffer_data_with_array_buffer_view(GL::ARRAY_BUFFER, &vert_array, GL::STATIC_DRAW);

    // Vertex colors
    let color_buffer = wasm_bindgen::memory()
        .dyn_into::<WebAssembly::Memory>()
        .unwrap()
        .buffer();
    let colors_location = colors_cube.as_ptr() as u32 / 4;
    let color_array = js_sys::Float32Array::new(&color_buffer)
        .subarray(colors_location, colors_location + colors_cube.len() as u32);
    let buffer_colors = ctx
        .create_buffer()
        .ok_or("failed to create buffer")
        .unwrap();
    ctx.bind_buffer(GL::ARRAY_BUFFER, Some(&buffer_colors));
    ctx.buffer_data_with_array_buffer_view(GL::ARRAY_BUFFER, &color_array, GL::STATIC_DRAW);

    // Vertex indices
    let indices_memory_buffer = wasm_bindgen::memory()
        .dyn_into::<WebAssembly::Memory>()
        .unwrap()
        .buffer();
    let indices_location = indices_cube.as_ptr() as u32 / 2;
    let indices_array = js_sys::Uint16Array::new(&indices_memory_buffer).subarray(
        indices_location,
        indices_location + indices_cube.len() as u32,
    );
    let buffer_indices = ctx.create_buffer().unwrap();
    ctx.bind_buffer(GL::ELEMENT_ARRAY_BUFFER, Some(&buffer_indices));
    ctx.buffer_data_with_array_buffer_view(
        GL::ELEMENT_ARRAY_BUFFER,
        &indices_array,
        GL::STATIC_DRAW,
    );

    return Rc::new(Mesh {
        id: 0,
        name: String::from("cube"),
        primitives: vec![Primitive {
            position_buffer: buffer_position,
            color_buffer: buffer_colors,
            indices_buffer: buffer_indices,
            index_count: indices_array.length() as i32,
            element_type: GL::TRIANGLES,
            name: String::from("0"),
            default_material: default_material,
        }],
    });
}

fn generate_plane(ctx: &WebGlRenderingContext, default_material: Option<Rc<Material>>) -> Rc<Mesh> {
    let vertices: [f32; 12] = [
        1.0, 1.0, 0.0, //
        1.0, -1.0, 0.0, //
        -1.0, 1.0, 0.0, //
        -1.0, -1.0, 0.0, //
    ];

    let colors: [f32; 12] = [
        0.0, 0.0, 0.0, //
        1.0, 0.0, 0.0, //
        0.0, 1.0, 0.0, //
        1.0, 1.0, 0.0, //
    ];

    let indices: [u16; 6] = [
        0, 1, 3, //
        0, 3, 2, //
    ];

    // Vertex positions
    let memory_buffer = wasm_bindgen::memory()
        .dyn_into::<WebAssembly::Memory>()
        .unwrap()
        .buffer();
    let vertices_location = vertices.as_ptr() as u32 / 4;
    let vert_array = js_sys::Float32Array::new(&memory_buffer)
        .subarray(vertices_location, vertices_location + vertices.len() as u32);
    let buffer_position = ctx
        .create_buffer()
        .ok_or("failed to create buffer")
        .unwrap();
    ctx.bind_buffer(GL::ARRAY_BUFFER, Some(&buffer_position));
    ctx.buffer_data_with_array_buffer_view(GL::ARRAY_BUFFER, &vert_array, GL::STATIC_DRAW);

    // Vertex colors
    let color_buffer = wasm_bindgen::memory()
        .dyn_into::<WebAssembly::Memory>()
        .unwrap()
        .buffer();
    let colors_location = colors.as_ptr() as u32 / 4;
    let color_array = js_sys::Float32Array::new(&color_buffer)
        .subarray(colors_location, colors_location + colors.len() as u32);
    let buffer_colors = ctx
        .create_buffer()
        .ok_or("failed to create buffer")
        .unwrap();
    ctx.bind_buffer(GL::ARRAY_BUFFER, Some(&buffer_colors));
    ctx.buffer_data_with_array_buffer_view(GL::ARRAY_BUFFER, &color_array, GL::STATIC_DRAW);

    // Vertex indices
    let indices_memory_buffer = wasm_bindgen::memory()
        .dyn_into::<WebAssembly::Memory>()
        .unwrap()
        .buffer();
    let indices_location = indices.as_ptr() as u32 / 2;
    let indices_array = js_sys::Uint16Array::new(&indices_memory_buffer)
        .subarray(indices_location, indices_location + indices.len() as u32);
    let buffer_indices = ctx.create_buffer().unwrap();
    ctx.bind_buffer(GL::ELEMENT_ARRAY_BUFFER, Some(&buffer_indices));
    ctx.buffer_data_with_array_buffer_view(
        GL::ELEMENT_ARRAY_BUFFER,
        &indices_array,
        GL::STATIC_DRAW,
    );

    return Rc::new(Mesh {
        id: 0,
        name: String::from("plane"),
        primitives: vec![Primitive {
            position_buffer: buffer_position,
            color_buffer: buffer_colors,
            indices_buffer: buffer_indices,
            index_count: indices_array.length() as i32,
            element_type: GL::TRIANGLES,
            name: String::from("0"),
            default_material: default_material,
        }],
    });
}

fn generate_grid(
    ctx: &WebGlRenderingContext,
    num_lines: u32,
    default_material: Option<Rc<Material>>,
) -> Rc<Mesh> {
    assert!(num_lines > 2);

    let incr = 1.0 / (num_lines - 1) as f32;
    let num_verts = num_lines * num_lines;

    let mut vertices: Vec<f32> = Vec::new();
    vertices.resize((num_verts * 3) as usize, 0.0);

    let mut colors: Vec<f32> = Vec::new();
    colors.resize((num_verts * 3) as usize, 0.0);

    for y_ind in 0..num_lines {
        for x_ind in 0..num_lines {
            let vert_ind = (x_ind + y_ind * num_lines) * 3;

            vertices[(vert_ind + 0) as usize] = x_ind as f32 * incr - 0.5;
            vertices[(vert_ind + 1) as usize] = y_ind as f32 * incr - 0.5;
            vertices[(vert_ind + 2) as usize] = 0.0;
            colors[(vert_ind + 0) as usize] = 1.0;
            colors[(vert_ind + 1) as usize] = 1.0;
            colors[(vert_ind + 2) as usize] = 1.0;
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

    // Vertex positions
    let memory_buffer = wasm_bindgen::memory()
        .dyn_into::<WebAssembly::Memory>()
        .unwrap()
        .buffer();
    let vertices_location = vertices.as_ptr() as u32 / 4;
    let vert_array = js_sys::Float32Array::new(&memory_buffer)
        .subarray(vertices_location, vertices_location + vertices.len() as u32);
    let buffer_position = ctx
        .create_buffer()
        .ok_or("failed to create buffer")
        .unwrap();
    ctx.bind_buffer(GL::ARRAY_BUFFER, Some(&buffer_position));
    ctx.buffer_data_with_array_buffer_view(GL::ARRAY_BUFFER, &vert_array, GL::STATIC_DRAW);

    // Vertex colors
    let color_buffer = wasm_bindgen::memory()
        .dyn_into::<WebAssembly::Memory>()
        .unwrap()
        .buffer();
    let colors_location = colors.as_ptr() as u32 / 4;
    let color_array = js_sys::Float32Array::new(&color_buffer)
        .subarray(colors_location, colors_location + colors.len() as u32);
    let buffer_colors = ctx
        .create_buffer()
        .ok_or("failed to create buffer")
        .unwrap();
    ctx.bind_buffer(GL::ARRAY_BUFFER, Some(&buffer_colors));
    ctx.buffer_data_with_array_buffer_view(GL::ARRAY_BUFFER, &color_array, GL::STATIC_DRAW);

    // Vertex indices
    let indices_memory_buffer = wasm_bindgen::memory()
        .dyn_into::<WebAssembly::Memory>()
        .unwrap()
        .buffer();
    let indices_location = indices.as_ptr() as u32 / 2;
    let indices_array = js_sys::Uint16Array::new(&indices_memory_buffer)
        .subarray(indices_location, indices_location + indices.len() as u32);
    let buffer_indices = ctx.create_buffer().unwrap();
    ctx.bind_buffer(GL::ELEMENT_ARRAY_BUFFER, Some(&buffer_indices));
    ctx.buffer_data_with_array_buffer_view(
        GL::ELEMENT_ARRAY_BUFFER,
        &indices_array,
        GL::STATIC_DRAW,
    );

    return Rc::new(Mesh {
        id: 0,
        name: String::from("grid"),
        primitives: vec![Primitive {
            position_buffer: buffer_position,
            color_buffer: buffer_colors,
            indices_buffer: buffer_indices,
            index_count: indices_array.length() as i32,
            element_type: GL::LINES,
            name: String::from("0"),
            default_material: default_material,
        }],
    });
}

fn generate_axes(ctx: &WebGlRenderingContext, default_material: Option<Rc<Material>>) -> Rc<Mesh> {
    let vertices: [f32; 12] = [
        0.0, 0.0, 0.0, //
        1.0, 0.0, 0.0, //
        0.0, 1.0, 0.0, //
        0.0, 0.0, 1.0, //
    ];

    let colors: [f32; 12] = [
        0.0, 0.0, 0.0, //
        1.0, 0.0, 0.0, //
        0.0, 1.0, 0.0, //
        0.0, 0.0, 1.0, //
    ];

    let indices: [u16; 6] = [0, 1, 0, 2, 0, 3];

    // Vertex positions
    let memory_buffer = wasm_bindgen::memory()
        .dyn_into::<WebAssembly::Memory>()
        .unwrap()
        .buffer();
    let vertices_location = vertices.as_ptr() as u32 / 4;
    let vert_array = js_sys::Float32Array::new(&memory_buffer)
        .subarray(vertices_location, vertices_location + vertices.len() as u32);
    let buffer_position = ctx
        .create_buffer()
        .ok_or("failed to create buffer")
        .unwrap();
    ctx.bind_buffer(GL::ARRAY_BUFFER, Some(&buffer_position));
    ctx.buffer_data_with_array_buffer_view(GL::ARRAY_BUFFER, &vert_array, GL::STATIC_DRAW);

    // Vertex colors
    let color_buffer = wasm_bindgen::memory()
        .dyn_into::<WebAssembly::Memory>()
        .unwrap()
        .buffer();
    let colors_location = colors.as_ptr() as u32 / 4;
    let color_array = js_sys::Float32Array::new(&color_buffer)
        .subarray(colors_location, colors_location + colors.len() as u32);
    let buffer_colors = ctx
        .create_buffer()
        .ok_or("failed to create buffer")
        .unwrap();
    ctx.bind_buffer(GL::ARRAY_BUFFER, Some(&buffer_colors));
    ctx.buffer_data_with_array_buffer_view(GL::ARRAY_BUFFER, &color_array, GL::STATIC_DRAW);

    // Vertex indices
    let indices_memory_buffer = wasm_bindgen::memory()
        .dyn_into::<WebAssembly::Memory>()
        .unwrap()
        .buffer();
    let indices_location = indices.as_ptr() as u32 / 2;
    let indices_array = js_sys::Uint16Array::new(&indices_memory_buffer)
        .subarray(indices_location, indices_location + indices.len() as u32);
    let buffer_indices = ctx.create_buffer().unwrap();
    ctx.bind_buffer(GL::ELEMENT_ARRAY_BUFFER, Some(&buffer_indices));
    ctx.buffer_data_with_array_buffer_view(
        GL::ELEMENT_ARRAY_BUFFER,
        &indices_array,
        GL::STATIC_DRAW,
    );

    return Rc::new(Mesh {
        id: 0,
        name: String::from("axes"),
        primitives: vec![Primitive {
            position_buffer: buffer_position,
            color_buffer: buffer_colors,
            indices_buffer: buffer_indices,
            index_count: indices_array.length() as i32,
            element_type: GL::LINES,
            name: String::from("0"),
            default_material: default_material,
        }],
    });
}

fn link_program(
    gl: &WebGlRenderingContext,
    vert_source: &str,
    frag_source: &str,
) -> Result<WebGlProgram, String> {
    let program = gl
        .create_program()
        .ok_or_else(|| String::from("Error creating program"))?;

    let vert_shader = compile_shader(&gl, GL::VERTEX_SHADER, vert_source).unwrap();

    let frag_shader = compile_shader(&gl, GL::FRAGMENT_SHADER, frag_source).unwrap();

    gl.attach_shader(&program, &vert_shader);
    gl.attach_shader(&program, &frag_shader);
    gl.link_program(&program);

    if gl
        .get_program_parameter(&program, WebGlRenderingContext::LINK_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(program)
    } else {
        Err(gl
            .get_program_info_log(&program)
            .unwrap_or_else(|| String::from("Unknown error creating program object")))
    }
}

fn compile_shader(
    gl: &WebGlRenderingContext,
    shader_type: u32,
    source: &str,
) -> Result<WebGlShader, String> {
    let shader = gl
        .create_shader(shader_type)
        .ok_or_else(|| String::from("Error creating shader"))?;
    gl.shader_source(&shader, source);
    gl.compile_shader(&shader);

    if gl
        .get_shader_parameter(&shader, WebGlRenderingContext::COMPILE_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(shader)
    } else {
        Err(gl
            .get_shader_info_log(&shader)
            .unwrap_or_else(|| String::from("Unable to get shader info log")))
    }
}

pub struct ResourceManager {
    meshes: HashMap<String, Rc<Mesh>>,
    textures: HashMap<String, Rc<Texture>>,
    materials: HashMap<String, Rc<Material>>,

    gl: WebGlRenderingContext,
}
impl ResourceManager {
    pub fn new(gl: WebGlRenderingContext) -> Self {
        return Self {
            meshes: HashMap::new(),
            textures: HashMap::new(),
            materials: HashMap::new(),
            gl,
        };
    }

    pub fn get_or_create_mesh(&mut self, name: &str) -> Option<Rc<Mesh>> {
        if let Some(mesh) = self.meshes.get(name) {
            return Some(mesh.clone());
        }

        let default_mat = self.get_or_create_material("default");

        if name == "cube" {
            let mesh = generate_cube(&self.gl, default_mat);
            log::info!("Generated mesh '{}'", name);
            self.meshes.insert(name.to_string(), mesh.clone());
            return Some(mesh);
        };

        if name == "plane" {
            let mesh = generate_plane(&self.gl, default_mat);
            log::info!("Generated mesh '{}'", name);
            self.meshes.insert(name.to_string(), mesh.clone());
            return Some(mesh);
        };

        if name == "grid" {
            let mesh = generate_grid(&self.gl, 200, default_mat);
            log::info!("Generated mesh '{}'", name);
            self.meshes.insert(name.to_string(), mesh.clone());
            return Some(mesh);
        };

        if name == "axes" {
            let mesh = generate_axes(&self.gl, default_mat);
            log::info!("Generated mesh '{}'", name);
            self.meshes.insert(name.to_string(), mesh.clone());
            return Some(mesh);
        };

        return None;
    }

    pub fn get_or_create_material(&mut self, name: &str) -> Option<Rc<Material>> {
        if let Some(mat) = self.materials.get(name) {
            return Some(mat.clone());
        }

        if name == "default" {
            let program = link_program(
                &self.gl,
                &crate::systems::rendering::vertex::pos_vertcolor::SHADER,
                &crate::systems::rendering::fragment::vertcolor::SHADER,
            )
            .expect(format!("Failed to compile material '{}'!", name).as_str());

            let default = Rc::new(Material {
                name: name.to_string(),
                u_opacity: self.gl.get_uniform_location(&program, "uOpacity").unwrap(),
                u_transform: self
                    .gl
                    .get_uniform_location(&program, "uTransform")
                    .unwrap(),
                a_position: self.gl.get_attrib_location(&program, "aPosition"),
                a_color: self.gl.get_attrib_location(&program, "aColor"),
                program: program,
            });

            log::info!("Generated material '{}'", name);
            self.materials.insert(name.to_string(), default.clone());
            return Some(default);
        };

        return None;
    }

    pub fn load_materials_from_gltf(&mut self, materials: gltf::iter::Materials) {}

    fn load_mesh_from_gltf(mesh: &gltf::Mesh) -> Option<Rc<Mesh>> {
        let vertex_buffer: Vec<f32> = Vec::new();
        let indices_buffer: Vec<u16> = Vec::new();
        let normals_buffer: Vec<f32> = Vec::new();
        let color_buffer: Vec<f32> = Vec::new();
        let uv0_buffer: Vec<f32> = Vec::new();
        let uv1_buffer: Vec<f32> = Vec::new();

        return None;
    }

    pub fn load_meshes_from_gltf(&mut self, meshes: gltf::iter::Meshes) {
        let mut num_loaded = 0;
        let mut num_failed = 0;
        for mesh in meshes {
            match ResourceManager::load_mesh_from_gltf(&mesh) {
                Some(new_mesh) => {
                    self.meshes.insert(new_mesh.name.clone(), new_mesh);
                    num_loaded += 1;
                }
                None => {
                    num_failed += 1;
                }
            }
        }

        log::info!(
            "Loaded {} meshes from gltf. {} failed",
            num_loaded,
            num_failed
        );
    }

    pub fn get_texture(&self, name: &str) -> Option<Rc<Texture>> {
        return None;
    }

    pub fn load_textures_from_gltf(&mut self, textures: gltf::iter::Textures) {}
}
