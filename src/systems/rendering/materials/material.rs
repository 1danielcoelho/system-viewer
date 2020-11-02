use crate::{app_state::AppState, components::transform::TransformType};

use cgmath::Matrix4;

use web_sys::*;

pub struct Material {
    pub name: String,

    pub program: WebGlProgram,

    pub u_opacity: WebGlUniformLocation,
    pub u_transform: WebGlUniformLocation,

    pub a_position: i32,
    pub a_color: i32,
}

impl Material {
    pub fn bind_for_drawing(&self, state: &AppState, transform: &TransformType) {        
        let w: Matrix4<f32> = transform.clone().into();
        
        // TODO: Fetch framebuffer dimensions here instead of assuming canvas_dims are it
        // TODO: This doesn't need to be computed for every single entity (neither does `v`)
        let p = cgmath::perspective(
            state.camera.fov_v,
            state.canvas_width as f32 / state.canvas_height as f32,
            state.camera.near,
            state.camera.far,
        );
        
        let v = cgmath::Matrix4::look_at(state.camera.pos, state.camera.target, state.camera.up);
        
        let proj = p * v * w;
        let proj_floats: &[f32; 16] = proj.as_ref();

        let gl = state.gl.as_ref().unwrap();

        // Set our shader program
        gl.use_program(Some(&self.program));

        // Enable attributes
        gl.enable_vertex_attrib_array(self.a_position as u32);
        gl.enable_vertex_attrib_array(self.a_color as u32);

        // Set uniforms
        gl.uniform_matrix4fv_with_f32_array(Some(&self.u_transform), false, proj_floats);
        gl.uniform1f(Some(&self.u_opacity), 1.0);
    }
}
