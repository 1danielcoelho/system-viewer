use std::{cell::RefCell, collections::HashMap, io::Cursor, rc::Rc};

use image::{io::Reader, DynamicImage, ImageFormat};
use web_sys::{WebGlProgram, WebGlRenderingContext as GL, WebGlShader};
use web_sys::{WebGlRenderingContext, WebGlUniformLocation};

use crate::utils::{get_unique_name, remove_numbered_suffix};

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

fn load_texture_from_bytes(
    identifier: &str,
    bytes: &[u8],
    image_format: Option<ImageFormat>,
    ctx: &WebGlRenderingContext,
) -> Result<Rc<Texture>, String> {
    let mut reader = Reader::new(Cursor::new(bytes));
    match image_format {
        Some(format) => {
            reader.set_format(format);
        }
        None => match reader.with_guessed_format() {
            Ok(new_reader) => {
                reader = new_reader;
            }
            Err(err) => {
                return Err(err.to_string());
            }
        },
    };

    let decoded = reader.decode();

    if let Err(error) = decoded {
        return Err(std::format!("Error loading texture: {}", error));
    }
    let decoded = decoded.unwrap();

    let mut width: u32 = 0;
    let mut height: u32 = 0;
    let mut format: u32 = 0;
    let mut num_channels: u8 = 0;
    let mut buf: Option<&[u8]> = None;
    let converted_bgr;
    let converted_bgra;

    match decoded {
        // R
        DynamicImage::ImageLuma8(ref img) => {
            width = img.width();
            height = img.height();
            format = GL::ALPHA;
            num_channels = 1;
            buf = Some(img.as_raw());
        }
        // RG
        DynamicImage::ImageLumaA8(ref img) => {
            width = img.width();
            height = img.height();
            format = GL::LUMINANCE_ALPHA;
            num_channels = 2;
            buf = Some(img.as_raw());
        }
        // RGB
        DynamicImage::ImageRgb8(ref img) => {
            width = img.width();
            height = img.height();
            format = GL::RGB;
            num_channels = 3;
            buf = Some(img.as_raw());
        }
        DynamicImage::ImageBgr8(_) => {
            converted_bgr = decoded.to_rgb8();
            width = converted_bgr.width();
            height = converted_bgr.height();
            format = GL::RGB;
            num_channels = 3;
            buf = Some(converted_bgr.as_raw());
        }
        // RGBA
        DynamicImage::ImageRgba8(ref img) => {
            width = img.width();
            height = img.height();
            format = GL::RGBA;
            num_channels = 4;
            buf = Some(img.as_raw());
        }
        DynamicImage::ImageBgra8(_) => {
            converted_bgra = decoded.to_rgba8();
            width = converted_bgra.width();
            height = converted_bgra.height();
            format = GL::RGBA;
            num_channels = 4;
            buf = Some(converted_bgra.as_raw());
        }
        _ => {}
    };

    if buf.is_none() {
        return Err(format!("Failed to decode {:?}", image_format));
    }

    let gl_tex = ctx.create_texture().unwrap();
    ctx.active_texture(GL::TEXTURE0);
    ctx.bind_texture(GL::TEXTURE_2D, Some(&gl_tex));

    ctx.tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_WRAP_S, GL::CLAMP_TO_EDGE as i32);
    ctx.tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_WRAP_T, GL::CLAMP_TO_EDGE as i32);
    ctx.tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_MIN_FILTER, GL::NEAREST as i32);
    ctx.tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_MAG_FILTER, GL::NEAREST as i32);

    ctx.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
        GL::TEXTURE_2D,
        0,
        format as i32,
        width as i32,
        height as i32,
        0,
        format,
        GL::UNSIGNED_BYTE, // Just u8 for now
        buf,
    )
    .unwrap();

    ctx.bind_texture(GL::TEXTURE_2D, None);

    return Ok(Rc::new(Texture {
        name: identifier.to_owned(),
        width,
        height,
        gl_format: format,
        num_channels,
        gl_handle: Some(gl_tex),
    }));
}

pub struct ResourceManager {
    meshes: HashMap<String, Rc<Mesh>>,
    textures: HashMap<String, Rc<Texture>>,
    materials: HashMap<String, Rc<RefCell<dyn Material>>>,

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
            "lat_long_sphere" => Some(generate_lat_long_sphere(
                &self.gl,
                16,
                16,
                0.8,
                true,
                true,
                default_mat,
            )),
            "ico_sphere" => Some(generate_ico_sphere(&self.gl, 0.8, 2, false, default_mat)),
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

    pub fn get_material(&self, identifier: &str) -> Option<Rc<RefCell<dyn Material>>> {
        if let Some(mat) = self.materials.get(identifier) {
            return Some(mat.clone());
        }

        return None;
    }

    pub fn instantiate_material(&mut self, identifier: &str) -> Option<Rc<RefCell<dyn Material>>> {
        let master_mat = self.get_or_create_material(identifier);
        if master_mat.is_none() {
            return None;
        };

        let instance = master_mat.clone();

        let new_name = get_unique_name(remove_numbered_suffix(identifier), &self.materials);
        instance.as_ref().unwrap().borrow_mut().set_name(&new_name);

        log::info!("Generated material instance '{}'", new_name);
        self.materials
            .insert(new_name.to_string(), instance.as_ref().unwrap().clone());
        return instance;
    }

    pub fn get_or_create_material(
        &mut self,
        identifier: &str,
    ) -> Option<Rc<RefCell<dyn Material>>> {
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
            "albedo" => {
                material_type = "texture";
                link_program(
                    &self.gl,
                    &shaders::vertex::RELAY_ALL,
                    &shaders::fragment::ALBEDO,
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
            "gltf_metal_rough" => {
                material_type = "gltf_metal_rough";
                link_program(
                    &self.gl,
                    &shaders::vertex::RELAY_ALL,
                    &shaders::fragment::GLTF_METAL_ROUGH,
                )
            }
            _ => Err("Invalid material identifier".to_owned()),
        };

        if program.is_err() {
            log::error!(
                "Failed to generate material '{}'. Error: {}",
                identifier,
                program.err().unwrap()
            );
            return None;
        };
        let program = program.unwrap();

        let mat: Rc<RefCell<dyn Material>> = match material_type {
            "unlit" => Rc::new(RefCell::new(UnlitMaterial {
                name: identifier.to_string(),
                uniform_locations: get_uniform_location_map(
                    &self.gl,
                    &program,
                    &[UniformName::WorldTrans, UniformName::ViewProjTrans],
                ),
                program: program,
                textures: HashMap::new(),
            })),
            "lit" => Rc::new(RefCell::new(PhongMaterial {
                name: identifier.to_string(),
                uniform_locations: get_uniform_location_map(
                    &self.gl,
                    &program,
                    &[
                        UniformName::WorldTrans,
                        UniformName::ViewProjTrans,
                        UniformName::LightTypes,
                        UniformName::LightPosDir,
                        UniformName::LightColors,
                        UniformName::LightIntensities,
                    ],
                ),
                program: program,
                textures: HashMap::new(),
            })),
            "texture" => Rc::new(RefCell::new(TextureTestMaterial {
                name: identifier.to_string(),
                uniform_locations: get_uniform_location_map(
                    &self.gl,
                    &program,
                    &[
                        UniformName::WorldTrans,
                        UniformName::ViewProjTrans,
                        UniformName::Albedo,
                    ],
                ),
                program: program,
                textures: HashMap::new(),
            })),
            "gltf_metal_rough" => Rc::new(RefCell::new(GltfMetalRough {
                name: identifier.to_string(),
                uniform_locations: get_uniform_location_map(
                    &self.gl,
                    &program,
                    &[
                        UniformName::WorldTrans,
                        UniformName::ViewProjTrans,
                        UniformName::LightTypes,
                        UniformName::LightPosDir,
                        UniformName::LightColors,
                        UniformName::LightIntensities,
                        UniformName::Albedo,
                        UniformName::MetallicRoughness,
                        UniformName::Normal,
                        UniformName::Emissive,
                        UniformName::Opacity,
                        UniformName::Occlusion,
                    ],
                ),
                program: program,
                textures: HashMap::new(),
            })),
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

    pub fn create_texture(
        &mut self,
        identifier: &str,
        bytes: &[u8],
        image_format: Option<ImageFormat>,
    ) {
        if let Some(_) = self.get_texture(identifier) {
            log::warn!(
                "Tried to overwrite a texture with identifier '{}'",
                identifier
            );
            return;
        }

        match load_texture_from_bytes(identifier, bytes, image_format, &self.gl) {
            Ok(tex) => {
                log::info!("Generated texture '{}'", identifier);
                self.textures.insert(identifier.to_string(), tex.clone());
            }
            Err(msg) => {
                log::error!(
                    "Error when trying to load texture '{}': {}",
                    identifier,
                    msg
                );
            }
        };
    }

    pub fn get_texture(&self, identifier: &str) -> Option<Rc<Texture>> {
        if let Some(tex) = self.textures.get(identifier) {
            return Some(tex.clone());
        }

        return None;
    }
}
