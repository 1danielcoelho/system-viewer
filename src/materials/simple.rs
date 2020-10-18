use super::super::common_funcs as cf;
use js_sys::WebAssembly;
use wasm_bindgen::JsCast;
use web_sys::WebGlRenderingContext as GL;
use web_sys::*;

pub struct SimpleMaterial {
    pub program: WebGlProgram,
    pub indices_buffer: WebGlBuffer,
    pub index_count: i32,
    pub position_buffer: WebGlBuffer,
    pub u_opacity: WebGlUniformLocation,
    pub u_transform: WebGlUniformLocation,
}

impl SimpleMaterial {
    pub fn new(gl: &WebGlRenderingContext) -> Self {
        let program = cf::link_program(
            &gl,
            &super::super::shaders::vertex::pos_vertcolor::SHADER,
            &super::super::shaders::fragment::vertcolor::SHADER,
        )
        .unwrap();

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
            u_opacity: gl.get_uniform_location(&program, "uOpacity").unwrap(),
            u_transform: gl.get_uniform_location(&program, "uTransform").unwrap(),
            program: program,
            indices_buffer: buffer_indices,
            index_count: indices_array.length() as i32,
            position_buffer: buffer_position,
        }
    }

    pub fn render(
        &self,
        gl: &WebGlRenderingContext,
        canvas_width: f32,
        canvas_height: f32,
    ) {
        gl.use_program(Some(&self.program));

        // Get uniforms
        // TODO: Fetch framebuffer dimensions here instead of assuming canvas_dims are it
        let p = cgmath::perspective(cgmath::Deg(65.0), canvas_width as f32 / canvas_height as f32, 1.0, 200.0);
        let v = cgmath::Matrix4::look_at(
            cgmath::Point3::new(1.5, -5.0, 3.0),
            cgmath::Point3::new(0.0, 0.0, 0.0),
            -cgmath::Vector3::unit_z(),
        );
        let proj = p * v;
        let proj_floats: &[f32; 16] = proj.as_ref();        

        // Set uniforms
        gl.uniform_matrix4fv_with_f32_array(
            Some(&self.u_transform),
            false,
            proj_floats,
        );
        gl.uniform1f(Some(&self.u_opacity), 1.0);

        // Bind vertex buffer
        gl.bind_buffer(GL::ARRAY_BUFFER, Some(&self.position_buffer));
        gl.vertex_attrib_pointer_with_i32(0, 3, GL::FLOAT, false, 0, 0);
        gl.enable_vertex_attrib_array(0);

        // Bind index buffer
        gl.bind_buffer(GL::ELEMENT_ARRAY_BUFFER, Some(&self.indices_buffer));

        // Draw
        gl.draw_elements_with_i32(GL::TRIANGLES, self.index_count, GL::UNSIGNED_SHORT, 0);
    }
}
