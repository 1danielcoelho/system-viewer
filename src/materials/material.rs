use crate::{app_state::AppState, components::TransformType};

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
        let gl = state.gl.as_ref().unwrap();

        gl.use_program(Some(&self.program));

        gl.enable_vertex_attrib_array(self.a_position as u32);
        gl.enable_vertex_attrib_array(self.a_color as u32);

        // TODO: Actually use the transform

        // Get uniforms
        let w = cgmath::Matrix4::from_angle_x(cgmath::Deg(state.time_ms as f32 / 10.0))
            * cgmath::Matrix4::from_angle_y(cgmath::Deg(state.time_ms as f32 / 13.0))
            * cgmath::Matrix4::from_angle_z(cgmath::Deg(state.time_ms as f32 / 17.0));

        // TODO: Fetch framebuffer dimensions here instead of assuming canvas_dims are it
        let p = cgmath::perspective(
            state.camera.fov_v,
            state.canvas_width as f32 / state.canvas_height as f32,
            state.camera.near,
            state.camera.far,
        );

        let v = cgmath::Matrix4::look_at(
            state.camera.pos,
            state.camera.target,
            state.camera.up,
        );

        let proj = p * v * w;
        let proj_floats: &[f32; 16] = proj.as_ref();

        // Set uniforms
        gl.uniform_matrix4fv_with_f32_array(Some(&self.u_transform), false, proj_floats);
        gl.uniform1f(Some(&self.u_opacity), 1.0);
    }
}
