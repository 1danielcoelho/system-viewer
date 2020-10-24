use js_sys::WebAssembly;
use std::{collections::HashMap, rc::Rc};
use wasm_bindgen::JsCast;
use web_sys::WebGlRenderingContext as GL;
use web_sys::WebGlRenderingContext;

use crate::{mesh::Mesh, texture::Texture};

fn generate_cube(ctx: &WebGlRenderingContext) -> Mesh {
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

    return Mesh {
        id: 0,
        name: String::from("cube"),
        position_buffer: buffer_position,
        color_buffer: buffer_colors,
        indices_buffer: buffer_indices,
        index_count: indices_array.length() as i32,
    };
}

pub struct ResourceManager {
    meshes: HashMap<String, Rc<Mesh>>,
    textures: HashMap<String, Rc<Texture>>,
}
impl ResourceManager {
    pub fn new() -> Self {
        return Self {
            meshes: HashMap::new(),
            textures: HashMap::new(),
        };
    }

    // TODO: Add options, like num_segments, sizes, etc.
    pub fn generate_mesh(&mut self, name: &str, ctx: &WebGlRenderingContext) -> Option<Rc<Mesh>> {
        if let Some(mesh) = self.meshes.get(name) {
            return Some(mesh.clone());
        }

        if name == "Cube" {
            let mesh = Rc::new(generate_cube(ctx));
            self.meshes.insert(name.to_string(), mesh.clone());
            return Some(mesh);
        };

        return None;
    }
    
    pub fn get_mesh(&self, name: &str) -> Option<Rc<Mesh>> {
        return Some(self.meshes.get(&name.to_string()).unwrap().clone());
    }
}
