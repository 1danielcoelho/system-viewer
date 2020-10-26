use crate::{app_state::AppState, components::TransformType};

use web_sys::WebGlRenderingContext as GL;

use web_sys::*;

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

pub struct Material {
    pub program: WebGlProgram,

    pub u_opacity: WebGlUniformLocation,
    pub u_transform: WebGlUniformLocation,

    pub a_position: i32,
    pub a_color: i32,
}

impl Material {
    pub fn new(gl: &WebGlRenderingContext) -> Rc<Self> {
        let program = link_program(
            &gl,
            &super::super::shaders::vertex::pos_vertcolor::SHADER,
            &super::super::shaders::fragment::vertcolor::SHADER,
        )
        .unwrap();

        Self {
            u_opacity: gl.get_uniform_location(&program, "uOpacity").unwrap(),
            u_transform: gl.get_uniform_location(&program, "uTransform").unwrap(),
            a_position: gl.get_attrib_location(&program, "aPosition"),
            a_color: gl.get_attrib_location(&program, "aColor"),
            program: program,
        }
    }

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
            cgmath::Deg(65.0),
            state.canvas_width as f32 / state.canvas_height as f32,
            1.0,
            200.0,
        );

        let v = cgmath::Matrix4::look_at(
            cgmath::Point3::new(1.5, -5.0, 3.0),
            cgmath::Point3::new(0.0, 0.0, 0.0),
            -cgmath::Vector3::unit_z(),
        );

        let proj = p * v * w;
        let proj_floats: &[f32; 16] = proj.as_ref();

        // Set uniforms
        gl.uniform_matrix4fv_with_f32_array(Some(&self.u_transform), false, proj_floats);
        gl.uniform1f(Some(&self.u_opacity), 1.0);
    }
}
