use std::{collections::HashMap, rc::Rc};
use web_sys::WebGlRenderingContext as GL;

use crate::{
    app_state::AppState,
    managers::resource::{PrimitiveAttribute, Texture, TextureUnit},
};

use web_sys::*;

use super::{Material, UniformData};

pub struct TextureTestMaterial {
    pub name: String,
    pub program: WebGlProgram,
    pub uniform_locations: HashMap<String, WebGlUniformLocation>,
    pub textures: HashMap<TextureUnit, Rc<Texture>>,
}

impl Material for TextureTestMaterial {
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
            self.uniform_locations.get("u_world_trans"),
            false,
            &uniform_data.w,
        );
        gl.uniform_matrix4fv_with_f32_array(
            self.uniform_locations.get("u_view_proj_trans"),
            false,
            &uniform_data.vp,
        );
        gl.uniform1i(
            self.uniform_locations.get("us_albedo"),
            TextureUnit::Albedo as i32,
        );

        // Bind textures
        for (unit, tex) in &self.textures {
            gl.active_texture(GL::TEXTURE0 + (*unit as u32));
            gl.bind_texture(GL::TEXTURE_2D, tex.gl_handle.as_ref());
        }
    }

    fn unbind_from_drawing(&self, state: &AppState) {
        let gl = state.gl.as_ref().unwrap();

        for (unit, _) in &self.textures {
            gl.active_texture(GL::TEXTURE0 + (*unit as u32));
            gl.bind_texture(GL::TEXTURE_2D, None);
        }

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
