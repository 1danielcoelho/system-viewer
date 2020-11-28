use std::{collections::HashMap, rc::Rc};

use web_sys::WebGlRenderingContext;
use web_sys::{WebGlProgram, WebGlRenderingContext as GL, WebGlShader};

use self::procedural_meshes::*;

pub use gltf_resources::*;
pub use materials::*;
pub use mesh::*;
pub use shaders::*;
pub use texture::*;

pub mod gltf_resources;
mod intermediate_mesh;
mod materials;
mod mesh;
mod procedural_meshes;
mod shaders;
mod texture;

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

    /** Don't call this to generate engine meshes/materials on-demand. Call this to make sure they're all loaded in at some point and you can fetch what you need through non-mut refs. */
    pub fn initialize(&mut self) {
        self.get_or_create_material("default");
        self.get_or_create_material("local_normal");

        self.get_or_create_mesh("cube");
        self.get_or_create_mesh("plane");
        self.get_or_create_mesh("grid");
        self.get_or_create_mesh("axes");
    }

    pub fn get_mesh(&self, identifier: &str) -> Option<Rc<Mesh>> {
        if let Some(mesh) = self.meshes.get(identifier) {
            return Some(mesh.clone());
        }

        return None;
    }

    pub fn get_or_create_mesh(&mut self, identifier: &str) -> Option<Rc<Mesh>> {
        if let Some(mesh) = self.get_mesh(identifier) {
            return Some(mesh);
        }

        let default_mat = self.get_or_create_material("default");

        if identifier == "cube" {
            let mesh = generate_cube(&self.gl, default_mat);
            log::info!("Generated mesh '{}'", identifier);
            self.meshes.insert(identifier.to_string(), mesh.clone());
            return Some(mesh);
        };

        if identifier == "plane" {
            let mesh = generate_plane(&self.gl, default_mat);
            log::info!("Generated mesh '{}'", identifier);
            self.meshes.insert(identifier.to_string(), mesh.clone());
            return Some(mesh);
        };

        if identifier == "grid" {
            let mesh = generate_grid(&self.gl, 200, default_mat);
            log::info!("Generated mesh '{}'", identifier);
            self.meshes.insert(identifier.to_string(), mesh.clone());
            return Some(mesh);
        };

        if identifier == "axes" {
            let mesh = generate_axes(&self.gl, default_mat);
            log::info!("Generated mesh '{}'", identifier);
            self.meshes.insert(identifier.to_string(), mesh.clone());
            return Some(mesh);
        };

        log::warn!("Failed to find mesh with name '{}'", identifier);
        return None;
    }

    pub fn get_material(&self, identifier: &str) -> Option<Rc<Material>> {
        if let Some(mat) = self.materials.get(identifier) {
            return Some(mat.clone());
        }

        return None;
    }

    pub fn get_or_create_material(&mut self, identifier: &str) -> Option<Rc<Material>> {
        if let Some(mat) = self.get_material(identifier) {
            return Some(mat);
        }

        if identifier == "default" {
            log::info!("Generating material '{}'", identifier);
            let program = link_program(
                &self.gl,
                &crate::managers::resource::shaders::vertex::relay_color::SHADER,
                &crate::managers::resource::shaders::fragment::color::SHADER,
            )
            .expect(format!("Failed to compile material '{}'!", identifier).as_str());

            let default = Rc::new(Material {
                name: identifier.to_string(),
                u_transform: self
                    .gl
                    .get_uniform_location(&program, "uTransform")
                    .unwrap(),
                program: program,
            });

            self.materials
                .insert(identifier.to_string(), default.clone());
            return Some(default);
        } else if identifier == "local_normal" {
            log::info!("Generating material '{}'", identifier);
            let program = link_program(
                &self.gl,
                &crate::managers::resource::shaders::vertex::relay_all::SHADER,
                &crate::managers::resource::shaders::fragment::local_normal::SHADER,
            )
            .expect(format!("Failed to compile material '{}'!", identifier).as_str());

            let local_normal = Rc::new(Material {
                name: identifier.to_string(),
                u_transform: self
                    .gl
                    .get_uniform_location(&program, "uTransform")
                    .unwrap(),
                program: program,
            });

            self.materials
                .insert(identifier.to_string(), local_normal.clone());
            return Some(local_normal);
        }

        return None;
    }

    pub fn get_texture(&self, _name: &str) -> Option<Rc<Texture>> {
        return None;
    }
}
