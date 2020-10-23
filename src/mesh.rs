use js_sys::WebAssembly;
use wasm_bindgen::JsCast;
use web_sys::{WebGlBuffer, WebGlRenderingContext};
use web_sys::WebGlRenderingContext as GL;

pub struct Mesh {
    pub name: String,
    pub position_buffer: WebGlBuffer,
    pub color_buffer: WebGlBuffer,
    pub indices_buffer: WebGlBuffer,
    pub index_count: i32,
}

impl Mesh {
    pub fn new(name: &str, gl: &WebGlRenderingContext) -> Self {
        #[rustfmt::skip]
        let vertices_cube: [f32; 24] = [
            -1.0, -1.0, -1.0, 
            -1.0, -1.0,  1.0, 
            -1.0,  1.0, -1.0, 
            -1.0,  1.0,  1.0, 

             1.0, -1.0, -1.0, 
             1.0, -1.0,  1.0, 
             1.0,  1.0, -1.0, 
             1.0,  1.0,  1.0, 
        ];

        let colors_cube: [f32; 24] = [
            0.0, 0.0, 0.0,
            0.0, 0.0, 1.0,
            0.0, 1.0, 0.0,
            0.0, 1.0, 1.0,

            1.0, 0.0, 0.0,
            1.0, 0.0, 1.0,
            1.0, 1.0, 0.0,
            1.0, 1.0, 1.0,
        ];

        let indices_cube: [u16; 36] = [
            0, 1, 3,
            0, 3, 2,
            
            1, 5, 3,
            5, 7, 3,

            5, 4, 6,
            6, 7, 5,

            0, 2, 4,
            2, 6, 4,

            2, 3, 7,
            2, 7, 6,

            0, 4, 5,
            0, 5, 1
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
        let buffer_position = gl.create_buffer().ok_or("failed to create buffer").unwrap();
        gl.bind_buffer(GL::ARRAY_BUFFER, Some(&buffer_position));
        gl.buffer_data_with_array_buffer_view(GL::ARRAY_BUFFER, &vert_array, GL::STATIC_DRAW);

        // Vertex colors
        let color_buffer = wasm_bindgen::memory()
            .dyn_into::<WebAssembly::Memory>()
            .unwrap()
            .buffer();
        let colors_location = colors_cube.as_ptr() as u32 / 4;
        let color_array = js_sys::Float32Array::new(&color_buffer).subarray(
            colors_location,
            colors_location + colors_cube.len() as u32,
        );
        let buffer_colors = gl.create_buffer().ok_or("failed to create buffer").unwrap();
        gl.bind_buffer(GL::ARRAY_BUFFER, Some(&buffer_colors));
        gl.buffer_data_with_array_buffer_view(GL::ARRAY_BUFFER, &color_array, GL::STATIC_DRAW);

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
        let buffer_indices = gl.create_buffer().unwrap();
        gl.bind_buffer(GL::ELEMENT_ARRAY_BUFFER, Some(&buffer_indices));
        gl.buffer_data_with_array_buffer_view(
            GL::ELEMENT_ARRAY_BUFFER,
            &indices_array,
            GL::STATIC_DRAW,
        );

        Self {
            name: String::from(name),
            position_buffer: buffer_position,
            color_buffer: buffer_colors,
            indices_buffer: buffer_indices,
            index_count: indices_array.length() as i32,
        }
    }

    pub fn draw(&self, gl: &WebGlRenderingContext) {
        // Bind vertex buffer
        gl.bind_buffer(GL::ARRAY_BUFFER, Some(&self.position_buffer));
        gl.enable_vertex_attrib_array(self.a_position as u32);
        gl.vertex_attrib_pointer_with_i32(0, 3, GL::FLOAT, false, 0, 0);

        // Bind color buffer
        gl.bind_buffer(GL::ARRAY_BUFFER, Some(&self.color_buffer));
        gl.enable_vertex_attrib_array(self.a_color as u32);
        gl.vertex_attrib_pointer_with_i32(1, 3, GL::FLOAT, false, 0, 0);

        // Bind index buffer
        gl.bind_buffer(GL::ELEMENT_ARRAY_BUFFER, Some(&self.indices_buffer));

        // Draw
        gl.draw_elements_with_i32(GL::TRIANGLES, self.index_count, GL::UNSIGNED_SHORT, 0);
    }
}