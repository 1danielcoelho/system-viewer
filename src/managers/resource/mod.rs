use std::{collections::HashMap, rc::Rc};

use web_sys::{WebGlProgram, WebGlRenderingContext as GL, WebGlShader};
use web_sys::{WebGlRenderingContext, WebGlUniformLocation};

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

// https://stackoverflow.com/a/28392068/2434460
macro_rules! hashmap {
    ($( $key: expr => $val: expr ),*) => {{
         let mut map = ::std::collections::HashMap::new();
         $( map.insert($key, $val); )*
         map
    }}
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
    uniform_names: &[&str],
) -> HashMap<String, WebGlUniformLocation> {
    let mut result: HashMap<String, WebGlUniformLocation> = HashMap::new();

    for uniform_name in uniform_names {
        if let Some(loc) = gl.get_uniform_location(&program, uniform_name) {
            result.insert((*uniform_name).to_owned(), loc);
        }
    }

    return result;
}

pub struct ResourceManager {
    meshes: HashMap<String, Rc<Mesh>>,
    textures: HashMap<String, Rc<Texture>>,
    materials: HashMap<String, Rc<dyn Material>>,

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
        self.get_or_create_material("world_normal");
        self.get_or_create_material("phong");

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

        let mesh: Option<Rc<Mesh>> = match identifier {
            "cube" => Some(generate_cube(&self.gl, default_mat)),
            "plane" => Some(generate_plane(&self.gl, default_mat)),
            "grid" => Some(generate_grid(&self.gl, 200, default_mat)),
            "axes" => Some(generate_axes(&self.gl, default_mat)),
            _ => None,
        };

        match mesh {
            Some(ref mesh) => {
                log::info!("Generated mesh '{}'", identifier);
                self.meshes.insert(identifier.to_string(), mesh.clone());
            }
            None => log::warn!("Failed to find mesh with name '{}'", identifier),
        }

        return mesh;
    }

    pub fn get_material(&self, identifier: &str) -> Option<Rc<dyn Material>> {
        if let Some(mat) = self.materials.get(identifier) {
            return Some(mat.clone());
        }

        return None;
    }

    pub fn get_or_create_material(&mut self, identifier: &str) -> Option<Rc<dyn Material>> {
        if let Some(mat) = self.get_material(identifier) {
            return Some(mat);
        }

        let mut material_type = "";
        let program = match identifier {
            "default" => {
                material_type = "unlit";
                link_program(
                    &self.gl,
                    &shaders::vertex::RELAY_COLOR,
                    &shaders::fragment::COLOR,
                )
            }
            "world_normal" => {
                material_type = "unlit";
                link_program(
                    &self.gl,
                    &shaders::vertex::RELAY_ALL,
                    &shaders::fragment::WORLD_NORMAL,
                )
            }
            "uv0" => {
                material_type = "unlit";
                link_program(
                    &self.gl,
                    &shaders::vertex::RELAY_ALL,
                    &shaders::fragment::UV0,
                )
            }
            "uv1" => {
                material_type = "unlit";
                link_program(
                    &self.gl,
                    &shaders::vertex::RELAY_ALL,
                    &shaders::fragment::UV1,
                )
            }
            "phong" => {
                material_type = "lit";
                link_program(
                    &self.gl,
                    &shaders::vertex::RELAY_ALL,
                    &shaders::fragment::PHONG,
                )
            }
            _ => Err("Invalid material identifier".to_owned()),
        };

        if program.is_err() {
            log::error!("Failed to generate material '{}'", identifier);
            return None;
        };
        let program = program.unwrap();

        let mat: Rc<dyn Material> = match material_type {
            "unlit" => Rc::new(UnlitMaterial {
                name: identifier.to_string(),
                uniform_locations: get_uniform_location_map(
                    &self.gl,
                    &program,
                    &["u_world_trans", "u_view_proj_trans"],
                ),
                program: program,
            }),
            "lit" => Rc::new(PhongMaterial {
                name: identifier.to_string(),
                uniform_locations: get_uniform_location_map(
                    &self.gl,
                    &program,
                    &[
                        "u_world_trans",
                        "u_view_proj_trans",
                        "u_light_types",
                        "u_light_pos_or_dir",
                        "u_light_colors",
                        "u_light_intensities",
                    ],
                ),
                program: program,
            }),
            _ => {
                log::error!(
                    "Unexpected material type '{}' requested for identifier {}",
                    material_type,
                    identifier
                );
                return None;
            }
        };

        log::info!("Generated material '{}'", identifier);
        self.materials.insert(identifier.to_string(), mat.clone());
        return Some(mat);
    }

    pub fn get_texture(&self, _name: &str) -> Option<Rc<Texture>> {
        return None;
    }
}
