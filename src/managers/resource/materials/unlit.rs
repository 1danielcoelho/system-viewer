use std::collections::HashMap;

use crate::{
    app_state::AppState, components::LightComponent, managers::resource::PrimitiveAttribute,
    systems::NUM_LIGHTS,
};

use web_sys::*;

use super::{Material, UniformData};

pub struct UnlitMaterial {
    pub name: String,
    pub program: WebGlProgram,
    pub uniform_locations: HashMap<String, WebGlUniformLocation>,
}

impl Material for UnlitMaterial {
    fn set_name(&mut self, name: &str) {
        self.name = name.to_owned();
    }

    fn get_name(&self) -> &str {
        return self.name.as_ref();
    }

    fn set_program(&mut self, program: WebGlProgram) {
        self.program = program;
    }

    fn get_program(&self) -> &WebGlProgram {
        return &self.program;
    }

    fn set_uniform_location(&mut self, id: &str, location: WebGlUniformLocation) {
        self.uniform_locations.insert(id.to_owned(), location);
    }

    fn bind_for_drawing(&self, state: &AppState, uniform_data: &UniformData) {
        let gl = state.gl.as_ref().unwrap();

        // Set our shader program
        gl.use_program(Some(&self.program));

        // Enable attributes
        gl.enable_vertex_attrib_array(PrimitiveAttribute::Position as u32);
        gl.enable_vertex_attrib_array(PrimitiveAttribute::Normal as u32);
        gl.enable_vertex_attrib_array(PrimitiveAttribute::Color as u32);
        gl.enable_vertex_attrib_array(PrimitiveAttribute::UV0 as u32);
        gl.enable_vertex_attrib_array(PrimitiveAttribute::UV1 as u32);

        // Set uniforms
        gl.uniform_matrix4fv_with_f32_array(
            Some(&self.uniform_locations["u_transform"]),
            false,
            &uniform_data.wvp,
        );
    }

    fn unbind_from_drawing(&self, state: &AppState) {
        let gl = state.gl.as_ref().unwrap();

        gl.disable_vertex_attrib_array(PrimitiveAttribute::Position as u32);
        gl.disable_vertex_attrib_array(PrimitiveAttribute::Normal as u32);
        gl.disable_vertex_attrib_array(PrimitiveAttribute::Color as u32);
        gl.disable_vertex_attrib_array(PrimitiveAttribute::UV0 as u32);
        gl.disable_vertex_attrib_array(PrimitiveAttribute::UV1 as u32);
        gl.use_program(None);
    }
}
