use std::{collections::HashMap, rc::Rc};

use web_sys::WebGlRenderingContext as GL;
use web_sys::*;

use crate::{
    app_state::AppState,
    managers::resource::{PrimitiveAttribute, Texture, TextureUnit},
};

use super::shaders;

pub struct FrameUniformValues {
    pub vp: [f32; 16],
    pub light_types: Vec<i32>,
    pub light_pos_or_dir: Vec<f32>,
    pub light_colors: Vec<f32>,
    pub light_intensities: Vec<f32>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum UniformName {
    WorldTrans,
    ViewProjTrans,
    LightTypes,
    LightPosDir,
    LightColors,
    LightIntensities,
    Albedo,
    MetallicRoughness,
    Normal,
    Emissive,
    Opacity,
    Occlusion,
}
impl UniformName {
    pub fn as_str(&self) -> &'static str {
        match *self {
            UniformName::WorldTrans => "u_world_trans",
            UniformName::ViewProjTrans => "u_view_proj_trans",
            UniformName::LightTypes => "u_light_types",
            UniformName::LightPosDir => "u_light_pos_or_dir",
            UniformName::LightColors => "u_light_colors",
            UniformName::LightIntensities => "u_light_intensities",
            UniformName::Albedo => "us_albedo",
            UniformName::MetallicRoughness => "us_metal_rough",
            UniformName::Normal => "us_normal",
            UniformName::Emissive => "us_emissive",
            UniformName::Opacity => "us_opacity",
            UniformName::Occlusion => "us_occlusion",
        }
    }
}

#[derive(Debug)]
pub enum UniformValue {
    Float(f32),
    Int(i32),
    Vec2([f32; 2]),
    Vec3([f32; 3]),
    Vec4([f32; 4]),
    Matrix([f32; 16]),
    FloatArr(Vec<f32>),
    IntArr(Vec<i32>),
    Vec2Arr(Vec<f32>),
    Vec3Arr(Vec<f32>),
    Vec4Arr(Vec<f32>),
}

pub struct Uniform {
    pub value: UniformValue,
    pub location: Option<WebGlUniformLocation>,
}

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

    gl.bind_attrib_location(&program, PrimitiveAttribute::Position as u32, "a_position");
    gl.bind_attrib_location(&program, PrimitiveAttribute::Normal as u32, "a_normal");
    gl.bind_attrib_location(&program, PrimitiveAttribute::Color as u32, "a_color");
    gl.bind_attrib_location(&program, PrimitiveAttribute::UV0 as u32, "a_uv0");
    gl.bind_attrib_location(&program, PrimitiveAttribute::UV1 as u32, "a_uv1");

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

fn get_uniform_location_map(
    gl: &WebGlRenderingContext,
    program: &WebGlProgram,
    uniform_names: &[UniformName],
) -> HashMap<UniformName, WebGlUniformLocation> {
    let mut result: HashMap<UniformName, WebGlUniformLocation> = HashMap::new();

    for uniform_name in uniform_names {
        if let Some(loc) = gl.get_uniform_location(&program, uniform_name.as_str()) {
            result.insert(*uniform_name, loc);
        }
    }

    return result;
}

pub struct Material {
    pub name: String,
    pub program: WebGlProgram,
    pub textures: HashMap<TextureUnit, Rc<Texture>>,

    uniforms: HashMap<UniformName, Uniform>,
}

impl Material {
    pub fn new(
        gl: &WebGlRenderingContext,
        name: &str,
        vert: &str,
        frag: &str,
        uniforms: &[UniformName],
    ) -> Result<Self, String> {
        let program = link_program(gl, vert, frag)?;

        let mut new_mat = Self {
            name: name.to_owned(),
            program,
            textures: HashMap::new(),
            uniforms: HashMap::new(),
        };

        for uniform_name in uniforms {
            new_mat.uniforms.insert(
                *uniform_name,
                Uniform {
                    value: UniformValue::Int(0),
                    location: gl.get_uniform_location(&new_mat.program, uniform_name.as_str()),
                },
            );
        }

        return Ok(new_mat);
    }

    pub fn set_uniform_value(&mut self, name: UniformName, value: UniformValue) {
        if let Some(uniform) = self.uniforms.get_mut(&name) {
            uniform.value = value;
        }
    }

    pub fn bind_for_drawing(&self, state: &AppState) {
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
        for (_, uniform) in self.uniforms.iter() {
            match &uniform.value {
                UniformValue::Float(value) => gl.uniform1f(uniform.location.as_ref(), *value),
                UniformValue::Int(value) => gl.uniform1i(uniform.location.as_ref(), *value),
                UniformValue::Vec2(value) => {
                    gl.uniform2f(uniform.location.as_ref(), value[0], value[1])
                }
                UniformValue::Vec3(value) => {
                    gl.uniform3f(uniform.location.as_ref(), value[0], value[1], value[2])
                }
                UniformValue::Vec4(value) => gl.uniform4f(
                    uniform.location.as_ref(),
                    value[0],
                    value[1],
                    value[2],
                    value[3],
                ),
                UniformValue::Matrix(value) => {
                    gl.uniform_matrix4fv_with_f32_array(uniform.location.as_ref(), false, value)
                }
                UniformValue::FloatArr(value) => {
                    gl.uniform1fv_with_f32_array(uniform.location.as_ref(), &value)
                }
                UniformValue::IntArr(value) => {
                    gl.uniform1iv_with_i32_array(uniform.location.as_ref(), &value)
                }
                UniformValue::Vec2Arr(value) => {
                    gl.uniform2fv_with_f32_array(uniform.location.as_ref(), &value)
                }
                UniformValue::Vec3Arr(value) => {
                    gl.uniform3fv_with_f32_array(uniform.location.as_ref(), &value)
                }
                UniformValue::Vec4Arr(value) => {
                    gl.uniform4fv_with_f32_array(uniform.location.as_ref(), &value)
                }
            }
        }

        // Bind textures
        for (unit, tex) in &self.textures {
            gl.active_texture(GL::TEXTURE0 + (*unit as u32));
            gl.bind_texture(GL::TEXTURE_2D, tex.gl_handle.as_ref());
        }
    }

    pub fn unbind_from_drawing(&self, state: &AppState) {
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
}
