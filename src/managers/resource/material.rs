use super::{
    mesh::PrimitiveAttribute,
    texture::{Texture, TextureUnit},
};
use crate::{
    components::light::LightType, managers::details_ui::DetailsUI, managers::resource::shaders::*,
    utils::gl::GL,
};
use egui::Ui;
use na::Matrix4;
use serde::{Deserialize, Serialize};
use std::{cell::RefCell, collections::HashMap, rc::Rc};
use web_sys::*;

pub struct FrameUniformValues {
    pub v: Matrix4<f64>,  // World space to camera space
    pub pv: Matrix4<f64>, // World space to NDC
    pub light_types: Vec<i32>,
    pub light_pos_or_dir_c: Vec<f32>, // For point lights, position; For directional lights, direction; Always in camera space
    pub light_colors: Vec<f32>,
    pub light_intensities: Vec<f32>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum UniformName {
    WVTrans,
    WVInvTranspTrans,
    WVPTrans,
    VPInvTrans,
    LightTypes,
    LightPosDir,
    LightColors,
    LightIntensities,
    BaseColor,
    BaseColorFactor,
    MetallicRoughness,
    MetallicFactor,
    RoughnessFactor,
    Normal,
    Emissive,
    EmissiveFactor,
    Occlusion,
}
impl UniformName {
    pub fn default_value(&self) -> UniformValue {
        match *self {
            UniformName::WVTrans
            | UniformName::WVInvTranspTrans
            | UniformName::WVPTrans
            | UniformName::VPInvTrans => {
                UniformValue::Matrix([
                    1.0, 0.0, 0.0, 0.0, //
                    0.0, 1.0, 0.0, 0.0, //
                    0.0, 0.0, 1.0, 0.0, //
                    0.0, 0.0, 0.0, 0.0, //
                ])
            }
            UniformName::LightTypes => UniformValue::IntArr(vec![LightType::Directional as i32]),
            UniformName::LightPosDir => UniformValue::Vec3Arr(vec![0.0, 0.0, -1.0]),
            UniformName::LightColors => UniformValue::Vec3Arr(vec![1.0, 1.0, 1.0]),
            UniformName::LightIntensities => UniformValue::FloatArr(vec![1.0]),
            UniformName::BaseColor => UniformValue::Int(TextureUnit::BaseColor as i32),
            UniformName::BaseColorFactor => UniformValue::Vec4([1.0, 1.0, 1.0, 1.0]),
            UniformName::MetallicRoughness => {
                UniformValue::Int(TextureUnit::MetallicRoughness as i32)
            }
            UniformName::MetallicFactor => UniformValue::Float(1.0),
            UniformName::RoughnessFactor => UniformValue::Float(1.0),
            UniformName::Normal => UniformValue::Int(TextureUnit::Normal as i32),
            UniformName::Emissive => UniformValue::Int(TextureUnit::Emissive as i32),
            UniformName::EmissiveFactor => UniformValue::Vec3([0.0, 0.0, 0.0]),
            UniformName::Occlusion => UniformValue::Int(TextureUnit::Occlusion as i32),
        }
    }

    pub fn as_str(&self) -> &'static str {
        match *self {
            UniformName::WVTrans => "u_wv_trans",
            UniformName::WVInvTranspTrans => "u_wv_inv_transp_trans",
            UniformName::WVPTrans => "u_wvp_trans",
            UniformName::VPInvTrans => "u_vp_inv_trans",
            UniformName::LightTypes => "u_light_types",
            UniformName::LightPosDir => "u_light_pos_or_dir_c",
            UniformName::LightColors => "u_light_colors",
            UniformName::LightIntensities => "u_light_intensities",
            UniformName::BaseColor => "us_basecolor",
            UniformName::BaseColorFactor => "u_basecolor_factor",
            UniformName::MetallicRoughness => "us_metal_rough",
            UniformName::MetallicFactor => "u_metallic_factor",
            UniformName::RoughnessFactor => "u_roughness_factor",
            UniformName::Normal => "us_normal",
            UniformName::Emissive => "us_emissive",
            UniformName::EmissiveFactor => "u_emissive_factor",
            UniformName::Occlusion => "us_occlusion",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Uniform {
    pub value: UniformValue,

    #[serde(skip)]
    pub location: Option<WebGlUniformLocation>,
}

fn link_program(
    gl: &WebGl2RenderingContext,
    prefix_lines: &[String],
    vert_source: &str,
    frag_source: &str,
) -> Result<WebGlProgram, String> {
    let program = gl
        .create_program()
        .ok_or_else(|| String::from("Error creating program"))?;

    let vert_shader = compile_shader(
        &gl,
        GL::VERTEX_SHADER,
        prefix_lines,
        &SHADER_STORAGE[vert_source],
    )?;
    let frag_shader = compile_shader(
        &gl,
        GL::FRAGMENT_SHADER,
        prefix_lines,
        &SHADER_STORAGE[frag_source],
    )?;

    gl.attach_shader(&program, &vert_shader);
    gl.attach_shader(&program, &frag_shader);

    gl.bind_attrib_location(&program, PrimitiveAttribute::Position as u32, "a_position");
    gl.bind_attrib_location(&program, PrimitiveAttribute::Normal as u32, "a_normal");
    gl.bind_attrib_location(&program, PrimitiveAttribute::Tangent as u32, "a_tangent");
    gl.bind_attrib_location(&program, PrimitiveAttribute::Color as u32, "a_color");
    gl.bind_attrib_location(&program, PrimitiveAttribute::UV0 as u32, "a_uv0");
    gl.bind_attrib_location(&program, PrimitiveAttribute::UV1 as u32, "a_uv1");

    gl.link_program(&program);

    if gl
        .get_program_parameter(&program, WebGl2RenderingContext::LINK_STATUS)
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
    gl: &WebGl2RenderingContext,
    shader_type: u32,
    prefix_lines: &[String],
    source: &str,
) -> Result<WebGlShader, String> {
    let final_source = "#version 300 es\n".to_owned() + &prefix_lines.join("\n") + "\n" + source;

    let shader = gl
        .create_shader(shader_type)
        .ok_or_else(|| String::from("Error creating shader"))?;

    gl.shader_source(&shader, &final_source);
    gl.compile_shader(&shader);

    if gl
        .get_shader_parameter(&shader, WebGl2RenderingContext::COMPILE_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(shader)
    } else {
        Err(format!(
            "Error compiling shader source below: \n\n{}\n\n=======================================================================\nError:\n{}",
            final_source,
            gl.get_shader_info_log(&shader)
                .unwrap_or_else(|| String::from("Unable to get shader info log"))
        ))
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Material {
    pub(super) name: String,

    // These and defines could technically be &'static str, but being owned simplifies serialization
    master: String,
    vert: String,
    frag: String,

    #[serde(skip)]
    program: Option<WebGlProgram>,

    textures: HashMap<TextureUnit, Rc<RefCell<Texture>>>,
    uniforms: HashMap<UniformName, Uniform>,
    defines: Vec<String>,

    failed_to_compile: bool,
}
impl Material {
    pub(super) fn new(
        master: &str,
        vert: &'static str,
        frag: &'static str,
        uniform_names: &[UniformName],
    ) -> Self {
        let mut uniforms: HashMap<UniformName, Uniform> = HashMap::new();
        for uniform_name in uniform_names {
            uniforms.insert(
                *uniform_name,
                Uniform {
                    value: uniform_name.default_value(),
                    location: None,
                },
            );
        }

        Self {
            name: master.to_owned(),
            master: master.to_owned(),
            vert: vert.to_owned(),
            frag: frag.to_owned(),
            program: None,
            textures: HashMap::new(),
            uniforms,
            defines: Vec::new(),
            failed_to_compile: false,
        }
    }

    pub fn get_name(&self) -> &str {
        return &self.name;
    }

    pub fn recompile_program(&mut self, gl: &WebGl2RenderingContext) {
        if self.failed_to_compile {
            return;
        }

        let program = link_program(gl, &self.defines, &self.vert, &self.frag);
        if program.is_err() {
            log::error!(
                "Error compiling material '{}': '{}'",
                self.name,
                program.err().unwrap_or_default(),
            );
            self.failed_to_compile = true;
            return;
        }
        let program = program.unwrap();

        for (uniform_name, uniform) in self.uniforms.iter_mut() {
            uniform.location = gl.get_uniform_location(&program, uniform_name.as_str());
            if uniform.location.is_none() {
                log::warn!(
                    "Failed to find uniform '{}' on shaders used by material '{}': Vert: '{}', frag: '{}'",
                    uniform_name.as_str(),
                    &self.name,
                    &self.vert,
                    &self.frag
                );
            }
        }

        self.program = Some(program);
    }

    pub fn set_define(&mut self, define: &'static str) {
        self.defines.push(define.to_owned());
        self.program = None;
        self.failed_to_compile = false;
    }

    pub fn clear_define(&mut self, define: &'static str) {
        if let Some(pos) = self.defines.iter().position(|x| *x == define) {
            self.defines.remove(pos);
            self.program = None;
            self.failed_to_compile = false;
        }
    }

    pub fn set_texture(&mut self, unit: TextureUnit, tex: Option<Rc<RefCell<Texture>>>) {
        if let Some(tex) = tex {
            self.set_define(unit.get_define());

            log::info!(
                "\t\t\tSet texture '{}' on unit '{:?}' of material '{}'. Defines: '{:?}'",
                tex.borrow().name,
                unit,
                self.name,
                self.defines
            );
            self.textures.insert(unit, tex);
        } else {
            self.textures.remove(&unit);
            self.clear_define(unit.get_define());

            log::info!(
                "\t\t\tRemoved texture on unit '{:?}' of material '{}'. Defines: '{:?}'",
                unit,
                self.name,
                self.defines
            );
        }
    }

    pub fn get_textures(&self) -> &HashMap<TextureUnit, Rc<RefCell<Texture>>> {
        return &self.textures;
    }

    pub fn set_uniform_value(&mut self, name: UniformName, value: UniformValue) {
        if let Some(uniform) = self.uniforms.get_mut(&name) {
            if std::mem::discriminant(&uniform.value) != std::mem::discriminant(&value) {
                log::warn!("Tried to set uniform '{:?}' with value '{:?}' which is of a different variant than it's current value of '{:?}'!", name, value, uniform.value);
            }
            uniform.value = value;
        }
    }

    pub fn bind_for_drawing(&mut self, gl: &WebGl2RenderingContext) {
        if self.program.is_none() {
            // Prevent repeatedly trying to recompile something that doesn't work
            if self.failed_to_compile {
                return;
            }

            self.recompile_program(gl);
        }

        // Set our shader program
        gl.use_program(self.program.as_ref());

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
            // log::info!("\tBinding texture {} to unit {:?}", tex.name, unit);

            gl.active_texture(GL::TEXTURE0 + (*unit as u32));

            let tex_borrow = tex.borrow();
            let target = match tex_borrow.is_cubemap {
                false => GL::TEXTURE_2D,
                true => GL::TEXTURE_CUBE_MAP,
            };
            gl.bind_texture(target, tex_borrow.gl_handle.as_ref());
        }
    }

    pub fn unbind_from_drawing(&self, gl: &WebGl2RenderingContext) {
        for (unit, tex) in &self.textures {
            gl.active_texture(GL::TEXTURE0 + (*unit as u32));

            let tex_borrow = tex.borrow();
            let target = match tex_borrow.is_cubemap {
                false => GL::TEXTURE_2D,
                true => GL::TEXTURE_CUBE_MAP,
            };
            gl.bind_texture(target, None);
        }

        gl.use_program(None);
    }
}

impl DetailsUI for Material {
    fn draw_details_ui(&mut self, ui: &mut Ui) {
        ui.collapsing(format!("Material: {}", &self.name), |ui| {
            ui.collapsing("Uniforms:", |ui| {
                for (name, val) in &mut self.uniforms {
                    ui.columns(2, |cols| {
                        cols[0].label(name.as_str());

                        match &mut val.value {
                            UniformValue::Float(ref mut f) => {
                                cols[1].add(egui::DragValue::f32(f).speed(0.1));
                            }
                            UniformValue::Int(ref mut i) => {
                                cols[1].add(egui::DragValue::i32(i));
                            }
                            UniformValue::Vec2(ref mut v2) => {
                                cols[1].horizontal(|ui| {
                                    ui.add(
                                        egui::DragValue::f32(&mut v2[0]).prefix("x: ").speed(0.1),
                                    );
                                    ui.add(
                                        egui::DragValue::f32(&mut v2[1]).prefix("y: ").speed(0.1),
                                    );
                                });
                            }
                            UniformValue::Vec3(ref mut v3) => {
                                cols[1].horizontal(|ui| {
                                    ui.add(
                                        egui::DragValue::f32(&mut v3[0]).prefix("x: ").speed(0.1),
                                    );
                                    ui.add(
                                        egui::DragValue::f32(&mut v3[1]).prefix("y: ").speed(0.1),
                                    );
                                    ui.add(
                                        egui::DragValue::f32(&mut v3[2]).prefix("z: ").speed(0.1),
                                    );
                                });
                            }
                            UniformValue::Vec4(ref mut v4) => {
                                cols[1].horizontal(|ui| {
                                    ui.add(
                                        egui::DragValue::f32(&mut v4[0]).prefix("x: ").speed(0.1),
                                    );
                                    ui.add(
                                        egui::DragValue::f32(&mut v4[1]).prefix("y: ").speed(0.1),
                                    );
                                    ui.add(
                                        egui::DragValue::f32(&mut v4[2]).prefix("z: ").speed(0.1),
                                    );
                                    ui.add(
                                        egui::DragValue::f32(&mut v4[3]).prefix("w: ").speed(0.1),
                                    );
                                });
                            }
                            // TODO
                            UniformValue::Matrix(_) => {}
                            UniformValue::FloatArr(_) => {}
                            UniformValue::IntArr(_) => {}
                            UniformValue::Vec2Arr(_) => {}
                            UniformValue::Vec3Arr(_) => {}
                            UniformValue::Vec4Arr(_) => {}
                        };
                    });
                }
            });

            ui.collapsing("Textures:", |ui| {
                for (unit, tex) in &mut self.textures {
                    ui.columns(2, |cols| {
                        cols[0].label(format!("{:?}", unit));
                        cols[1].label(&tex.borrow().name);
                    });
                }
            });
        });
    }
}
