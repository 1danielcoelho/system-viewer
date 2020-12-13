use std::{collections::HashMap, rc::Rc};

use web_sys::*;

use crate::{app_state::AppState, managers::resource::{Texture, TextureUnit}, systems::NUM_LIGHTS};

pub struct UniformData {
    pub w: [f32; 16],
    pub vp: [f32; 16],
    pub light_types: [i32; NUM_LIGHTS],
    pub light_pos_or_dir: [f32; NUM_LIGHTS * 3],
    pub light_colors: [f32; NUM_LIGHTS * 3],
    pub light_intensities: [f32; NUM_LIGHTS],
}

pub trait Material {
    fn set_name(&mut self, name: &str);
    fn get_name(&self) -> &str;

    fn set_program(&mut self, program: WebGlProgram);
    fn get_program(&self) -> &WebGlProgram;

    fn set_uniform_location(&mut self, id: &str, location: WebGlUniformLocation);

    fn set_texture(&mut self, unit: TextureUnit, texture: Rc<Texture>);
    fn get_texture(&mut self, unit: TextureUnit) -> Option<Rc<Texture>>;
    fn get_used_textures(&self) -> &HashMap<TextureUnit, Rc<Texture>>;

    fn bind_for_drawing(&self, state: &AppState, uniform_data: &UniformData);
    fn unbind_from_drawing(&self, state: &AppState);
}
