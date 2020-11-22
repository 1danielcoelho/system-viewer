use std::{collections::HashMap, rc::Rc};

use web_sys::WebGlRenderingContext;
use web_sys::{WebGlProgram, WebGlRenderingContext as GL, WebGlShader};

use self::mesh_generation::{generate_axes, generate_cube, generate_grid, generate_plane};

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

    fn load_mesh_from_gltf(_mesh: &gltf::Mesh) -> Option<Rc<Mesh>> {
        let _vertex_buffer: Vec<f32> = Vec::new();
        let _indices_buffer: Vec<u16> = Vec::new();
        let _normals_buffer: Vec<f32> = Vec::new();
        let _color_buffer: Vec<f32> = Vec::new();
        let _uv0_buffer: Vec<f32> = Vec::new();
        let _uv1_buffer: Vec<f32> = Vec::new();

        return None;
    }

    pub fn load_meshes_from_gltf(&mut self, meshes: gltf::iter::Meshes) {
        let mut num_loaded = 0;
        let mut num_failed = 0;
        for mesh in meshes {
            match ResourceManager::load_mesh_from_gltf(&mesh) {
                Some(new_mesh) => {
                    self.meshes.insert(new_mesh.name.clone(), new_mesh);
                    num_loaded += 1;
                }
                None => {
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
