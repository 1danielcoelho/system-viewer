use std::{collections::HashMap, rc::Rc};

use cgmath::{Vector2, Vector3, Vector4};
use gltf::mesh::util::{ReadColors, ReadIndices, ReadTexCoords};
use web_sys::WebGlRenderingContext;
use web_sys::{WebGlProgram, WebGlRenderingContext as GL, WebGlShader};

use self::mesh_generation::{
    generate_axes, generate_cube, generate_grid, generate_plane, intermediate_to_mesh,
    IntermediateMesh, IntermediatePrimitive,
};

pub use materials::*;
pub use mesh::*;
pub use shaders::*;
pub use texture::*;

mod materials;
mod mesh;
mod shaders;
mod texture;

mod mesh_generation;

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

    gl.bind_attrib_location(&program, PrimitiveAttribute::Position as u32, "aPosition");
    gl.bind_attrib_location(&program, PrimitiveAttribute::Normal as u32, "aNormal");
    gl.bind_attrib_location(&program, PrimitiveAttribute::Color as u32, "aColor");
    gl.bind_attrib_location(&program, PrimitiveAttribute::UV0 as u32, "aUV0");
    gl.bind_attrib_location(&program, PrimitiveAttribute::UV1 as u32, "aUV1");

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

pub struct ResourceManager {
    meshes: HashMap<String, Rc<Mesh>>,
    textures: HashMap<String, Rc<Texture>>,
    materials: HashMap<String, Rc<Material>>,

    gl: WebGlRenderingContext,
}
impl ResourceManager {
    pub fn new(gl: WebGlRenderingContext) -> Self {
        return Self {
            meshes: HashMap::new(),
            textures: HashMap::new(),
            materials: HashMap::new(),
            gl,
        };
    }

    pub fn get_or_create_mesh(&mut self, name: &str) -> Option<Rc<Mesh>> {
        if let Some(mesh) = self.meshes.get(name) {
            return Some(mesh.clone());
        }

        let default_mat = self.get_or_create_material("default");

        if name == "cube" {
            let mesh = generate_cube(&self.gl, default_mat);
            log::info!("Generated mesh '{}'", name);
            self.meshes.insert(name.to_string(), mesh.clone());
            return Some(mesh);
        };

        if name == "plane" {
            let mesh = generate_plane(&self.gl, default_mat);
            log::info!("Generated mesh '{}'", name);
            self.meshes.insert(name.to_string(), mesh.clone());
            return Some(mesh);
        };

        if name == "grid" {
            let mesh = generate_grid(&self.gl, 200, default_mat);
            log::info!("Generated mesh '{}'", name);
            self.meshes.insert(name.to_string(), mesh.clone());
            return Some(mesh);
        };

        if name == "axes" {
            let mesh = generate_axes(&self.gl, default_mat);
            log::info!("Generated mesh '{}'", name);
            self.meshes.insert(name.to_string(), mesh.clone());
            return Some(mesh);
        };

        log::warn!("Failed to find mesh with name '{}'", name);
        return None;
    }

    pub fn get_or_create_material(&mut self, name: &str) -> Option<Rc<Material>> {
        if let Some(mat) = self.materials.get(name) {
            return Some(mat.clone());
        }

        if name == "default" {
            let program = link_program(
                &self.gl,
                &crate::managers::resource::shaders::vertex::default::SHADER,
                &crate::managers::resource::shaders::fragment::default::SHADER,
            )
            .expect(format!("Failed to compile material '{}'!", name).as_str());

            let default = Rc::new(Material {
                name: name.to_string(),
                u_transform: self
                    .gl
                    .get_uniform_location(&program, "uTransform")
                    .unwrap(),
                program: program,
            });

            log::info!("Generated material '{}'", name);
            self.materials.insert(name.to_string(), default.clone());
            return Some(default);
        };

        return None;
    }

    pub fn load_materials_from_gltf(&mut self, _materials: gltf::iter::Materials) {}

    fn load_mesh_from_gltf(
        mesh: &gltf::Mesh,
        buffers: &Vec<gltf::buffer::Data>,
        default_material: &Option<Rc<Material>>,
        ctx: &WebGlRenderingContext,
    ) -> Result<Rc<Mesh>, String> {
        let mut name = mesh.index().to_string();
        if let Some(mesh_name) = mesh.name() {
            name = mesh_name.to_owned() + &name;
        }

        let mut inter_prims: Vec<IntermediatePrimitive> = Vec::new();
        inter_prims.reserve(mesh.primitives().len());
        for prim in mesh.primitives() {
            let reader = prim.reader(|buffer| Some(&buffers[buffer.index()]));

            // Name
            let prim_name = prim.index().to_string();

            // Indices
            let mut indices_vec: Vec<u16> = Vec::new();
            if let Some(indices) = reader.read_indices() {
                match indices {
                    ReadIndices::U8(iter) => {
                        indices_vec = iter.map(|byte| byte as u16).collect();
                    }
                    ReadIndices::U16(iter) => {
                        indices_vec = iter.collect();
                    }
                    ReadIndices::U32(_) => {
                        log::warn!(
                            "Skipping prim {} of mesh {} because it uses u32 vertex indices",
                            prim.index(),
                            name
                        );
                        continue;
                    }
                }
            }

            // Positions
            let mut positions_vec: Vec<cgmath::Vector3<f32>> = Vec::new();
            if let Some(positions) = reader.read_positions() {
                positions_vec = positions
                    .map(|arr| Vector3::new(arr[0], -arr[2], arr[1])) // Y-up right-handed to Z-up right-handed
                    .collect();
            }

            // Normals
            let mut normals_vec: Vec<cgmath::Vector3<f32>> = Vec::new();
            if let Some(normals) = reader.read_normals() {
                normals_vec = normals
                    .map(|arr| Vector3::new(arr[0], -arr[2], arr[1])) // Y-up right-handed to Z-up right-handed
                    .collect();
            }

            // Colors
            let mut colors_vec: Vec<cgmath::Vector4<f32>> = Vec::new();
            if let Some(colors) = reader.read_colors(0) {
                // TODO: Set the proper webgl buffer values and don't do this conversion?
                match colors {
                    ReadColors::RgbU8(arr) => {
                        colors_vec = arr
                            .map(|c| {
                                Vector4::new(
                                    c[0] as f32 / std::u8::MAX as f32,
                                    c[1] as f32 / std::u8::MAX as f32,
                                    c[2] as f32 / std::u8::MAX as f32,
                                    1.0,
                                )
                            })
                            .collect()
                    }
                    ReadColors::RgbU16(arr) => {
                        colors_vec = arr
                            .map(|c| {
                                Vector4::new(
                                    c[0] as f32 / std::u16::MAX as f32,
                                    c[1] as f32 / std::u16::MAX as f32,
                                    c[2] as f32 / std::u16::MAX as f32,
                                    1.0,
                                )
                            })
                            .collect()
                    }
                    ReadColors::RgbF32(arr) => {
                        colors_vec = arr.map(|c| Vector4::new(c[0], c[1], c[2], 1.0)).collect()
                    }
                    ReadColors::RgbaU8(arr) => {
                        colors_vec = arr
                            .map(|c| {
                                Vector4::new(
                                    c[0] as f32 / std::u8::MAX as f32,
                                    c[1] as f32 / std::u8::MAX as f32,
                                    c[2] as f32 / std::u8::MAX as f32,
                                    c[3] as f32 / std::u8::MAX as f32,
                                )
                            })
                            .collect()
                    }
                    ReadColors::RgbaU16(arr) => {
                        colors_vec = arr
                            .map(|c| {
                                Vector4::new(
                                    c[0] as f32 / std::u16::MAX as f32,
                                    c[1] as f32 / std::u16::MAX as f32,
                                    c[2] as f32 / std::u16::MAX as f32,
                                    c[3] as f32 / std::u16::MAX as f32,
                                )
                            })
                            .collect()
                    }
                    ReadColors::RgbaF32(arr) => {
                        colors_vec = arr.map(|c| Vector4::new(c[0], c[1], c[2], c[3])).collect()
                    }
                }
            }

            // UV0
            let mut uv0_vec: Vec<cgmath::Vector2<f32>> = Vec::new();
            if let Some(uv0) = reader.read_tex_coords(0) {
                match uv0 {
                    ReadTexCoords::U8(arr) => {
                        uv0_vec = arr
                            .map(|c| {
                                Vector2::new(
                                    c[0] as f32 / std::u8::MAX as f32,
                                    c[1] as f32 / std::u8::MAX as f32,
                                )
                            })
                            .collect()
                    }
                    ReadTexCoords::U16(arr) => {
                        uv0_vec = arr
                            .map(|c| {
                                Vector2::new(
                                    c[0] as f32 / std::u16::MAX as f32,
                                    c[1] as f32 / std::u16::MAX as f32,
                                )
                            })
                            .collect()
                    }
                    ReadTexCoords::F32(arr) => {
                        uv0_vec = arr.map(|c| Vector2::new(c[0], c[1])).collect()
                    }
                }
            }

            // UV1
            let mut uv1_vec: Vec<cgmath::Vector2<f32>> = Vec::new();
            if let Some(uv1) = reader.read_tex_coords(1) {
                match uv1 {
                    ReadTexCoords::U8(arr) => {
                        uv1_vec = arr
                            .map(|c| {
                                Vector2::new(
                                    c[0] as f32 / std::u8::MAX as f32,
                                    c[1] as f32 / std::u8::MAX as f32,
                                )
                            })
                            .collect()
                    }
                    ReadTexCoords::U16(arr) => {
                        uv1_vec = arr
                            .map(|c| {
                                Vector2::new(
                                    c[0] as f32 / std::u16::MAX as f32,
                                    c[1] as f32 / std::u16::MAX as f32,
                                )
                            })
                            .collect()
                    }
                    ReadTexCoords::F32(arr) => {
                        uv1_vec = arr.map(|c| Vector2::new(c[0], c[1])).collect()
                    }
                }
            }

            log::info!(
                "Loaded gltf prim {}, num_indices: {}, num_positions: {}, num_colors: {}, mat: {}",
                prim_name,
                indices_vec.len(),
                positions_vec.len(),
                colors_vec.len(),
                default_material.as_ref().unwrap().name,
            );

            inter_prims.push(IntermediatePrimitive {
                name: prim_name,
                indices: indices_vec,
                positions: positions_vec,
                normals: normals_vec,
                colors: colors_vec,
                uv0: uv0_vec,
                uv1: uv1_vec,
                mode: prim.mode().as_gl_enum(),
                mat: Some(default_material.as_ref().unwrap().clone()),
            });
        }

        log::info!(
            "Loaded gltf mesh {}, num_prims: {}",
            name,
            inter_prims.len()
        );

        return Ok(intermediate_to_mesh(
            IntermediateMesh {
                name,
                primitives: inter_prims,
            },
            ctx,
        ));
    }

    pub fn load_meshes_from_gltf(
        &mut self,
        meshes: gltf::iter::Meshes,
        buffers: &Vec<gltf::buffer::Data>,
    ) {
        let default_mat = self.get_or_create_material("default");

        let mut num_loaded = 0;
        let mut num_failed = 0;
        for mesh in meshes {
            match ResourceManager::load_mesh_from_gltf(&mesh, &buffers, &default_mat, &self.gl) {
                Ok(new_mesh) => {
                    log::info!("Loaded gltf mesh: {}", new_mesh.name);
                    self.meshes.insert(new_mesh.name.clone(), new_mesh);
                    num_loaded += 1;
                }
                Err(msg) => {
                    log::error!("Failed to load gltf mesh: {}", msg);
                    num_failed += 1;
                }
            }
        }

        log::info!(
            "Loaded {} meshes from gltf. {} failed",
            num_loaded,
            num_failed
        );
    }

    pub fn get_texture(&self, _name: &str) -> Option<Rc<Texture>> {
        return None;
    }

    pub fn load_textures_from_gltf(&mut self, _textures: gltf::iter::Textures) {}
}
