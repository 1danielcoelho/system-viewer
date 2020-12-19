use std::{collections::HashMap, rc::Rc};

use crate::{
    app_state::AppState,
    managers::resource::{PrimitiveAttribute, Texture, TextureUnit},
};

use web_sys::*;

use super::{Material, UniformData, UniformName};

pub struct PhongMaterial {
    pub name: String,
    pub program: WebGlProgram,
    pub uniform_locations: HashMap<UniformName, WebGlUniformLocation>,
    pub textures: HashMap<TextureUnit, Rc<Texture>>,
}

impl Material for PhongMaterial {
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

    fn set_uniform_location(&mut self, id: UniformName, location: WebGlUniformLocation) {
        self.uniform_locations.insert(id, location);
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
            self.uniform_locations.get(&UniformName::WorldTrans),
            false,
            &uniform_data.w,
        );
        gl.uniform_matrix4fv_with_f32_array(
            self.uniform_locations.get(&UniformName::ViewProjTrans),
            false,
            &uniform_data.vp,
        );
        gl.uniform1iv_with_i32_array(
            self.uniform_locations.get(&UniformName::LightTypes),
            &uniform_data.light_types,
        );
        gl.uniform3fv_with_f32_array(
            self.uniform_locations.get(&UniformName::LightPosDir),
            &uniform_data.light_pos_or_dir,
        );
        gl.uniform3fv_with_f32_array(
            self.uniform_locations.get(&UniformName::LightColors),
            &uniform_data.light_colors,
        );
        gl.uniform1fv_with_f32_array(
            self.uniform_locations.get(&UniformName::LightIntensities),
            &uniform_data.light_intensities,
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

    fn set_texture(&mut self, unit: TextureUnit, texture: Rc<Texture>) {
        self.textures.insert(unit, texture);
    }

    fn get_texture(&mut self, unit: TextureUnit) -> Option<Rc<Texture>> {
        return match self.textures.get(&unit) {
            Some(tex) => Some(tex.clone()),
            None => None,
        };
    }

    fn get_used_textures(&self) -> &HashMap<TextureUnit, Rc<Texture>> {
        return &self.textures;
    }
}
