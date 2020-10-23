use super::super::common_funcs as cf;
use js_sys::WebAssembly;
use wasm_bindgen::JsCast;
use web_sys::WebGlRenderingContext as GL;
use web_sys::*;

pub struct SimpleMaterial {
    pub program: WebGlProgram,

    pub u_opacity: WebGlUniformLocation,
    pub u_transform: WebGlUniformLocation,

    pub a_position: i32,
    pub a_color: i32,
}

impl SimpleMaterial {
    pub fn new(gl: &WebGlRenderingContext) -> Self {
        let program = cf::link_program(
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

    pub fn render(
        &self,
        gl: &WebGlRenderingContext,
        time: f32,
        canvas_width: f32,
        canvas_height: f32,
    ) {
        gl.use_program(Some(&self.program));

        // Get uniforms
        let w = cgmath::Matrix4::from_angle_x(cgmath::Deg(time / 10.0))
            * cgmath::Matrix4::from_angle_y(cgmath::Deg(time / 13.0))
            * cgmath::Matrix4::from_angle_z(cgmath::Deg(time / 17.0));
        // TODO: Fetch framebuffer dimensions here instead of assuming canvas_dims are it
        let p = cgmath::perspective(
            cgmath::Deg(65.0),
            canvas_width as f32 / canvas_height as f32,
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
