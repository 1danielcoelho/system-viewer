use std::{cell::RefCell, collections::HashMap, io::Cursor, rc::Rc};

use image::{io::Reader, DynamicImage, ImageFormat};
use web_sys::WebGlRenderingContext as GL;
use web_sys::WebGlRenderingContext;

use crate::utils::{get_unique_name, remove_numbered_suffix};

use self::procedural_meshes::*;

pub use gltf_resources::*;
pub use material::*;
pub use mesh::*;
pub use shaders::*;
pub use texture::*;

pub mod gltf_resources;
mod intermediate_mesh;
mod material;
mod mesh;
mod procedural_meshes;
mod shaders;
mod texture;

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
    materials: HashMap<String, Rc<RefCell<Material>>>,

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

    pub fn get_material(&self, identifier: &str) -> Option<Rc<RefCell<Material>>> {
        if let Some(mat) = self.materials.get(identifier) {
            return Some(mat.clone());
        }

        return None;
    }

    pub fn instantiate_material(&mut self, identifier: &str) -> Option<Rc<RefCell<Material>>> {
        let master_mat = self.get_or_create_material(identifier);
        if master_mat.is_none() {
            return None;
        };

        let instance = Rc::new(RefCell::new(master_mat.unwrap().borrow().clone()));

        let new_name = get_unique_name(remove_numbered_suffix(identifier), &self.materials);
        instance.borrow_mut().name = new_name.clone();

        log::info!("Generated material instance '{}'", new_name);
        self.materials
            .insert(new_name.to_string(), instance.clone());
        return Some(instance);
    }

    pub fn get_or_create_material(&mut self, identifier: &str) -> Option<Rc<RefCell<Material>>> {
        if let Some(mat) = self.get_material(identifier) {
            return Some(mat);
        }

        let mat = match identifier {
            "default" => Some(Material::new(
                identifier,
                "relay_color.vert",
                "color.frag",
                &[UniformName::WorldTrans, UniformName::ViewProjTrans],
            )),
            "world_normal" => Some(Material::new(
                identifier,
                "relay_all.vert",
                "world_normal.frag",
                &[UniformName::WorldTrans, UniformName::ViewProjTrans],
            )),
            "uv0" => Some(Material::new(
                identifier,
                "relay_all.vert",
                "uv0.frag",
                &[UniformName::WorldTrans, UniformName::ViewProjTrans],
            )),
            "uv1" => Some(Material::new(
                identifier,
                "relay_all.vert",
                "uv1.frag",
                &[UniformName::WorldTrans, UniformName::ViewProjTrans],
            )),
            "basecolor" => Some(Material::new(
                identifier,
                "relay_all.vert",
                "basecolor.frag",
                &[
                    UniformName::WorldTrans,
                    UniformName::ViewProjTrans,
                    UniformName::BaseColor,
                ],
            )),
            "phong" => Some(Material::new(
                identifier,
                "relay_all.vert",
                "phong.frag",
                &[
                    UniformName::WorldTrans,
                    UniformName::ViewProjTrans,
                    UniformName::LightTypes,
                    UniformName::LightPosDir,
                    UniformName::LightColors,
                    UniformName::LightIntensities,
                ],
            )),
            "gltf_metal_rough" => Some(Material::new(
                identifier,
                "relay_all.vert",
                "gltf_metal_rough.frag",
                &[
                    UniformName::WorldTrans,
                    UniformName::ViewProjTrans,
                    UniformName::LightTypes,
                    UniformName::LightPosDir,
                    UniformName::LightColors,
                    UniformName::LightIntensities,
                    UniformName::CameraPos,
                    UniformName::BaseColor,
                    UniformName::BaseColorFactor,
                    UniformName::MetallicRoughness,
                    UniformName::MetallicFactor,
                    UniformName::RoughnessFactor,
                    UniformName::Normal,
                    UniformName::Emissive,
                    UniformName::EmissiveFactor,
                    UniformName::Occlusion,
                ],
            )),
            _ => None,
        };
        if mat.is_none() {
            log::error!("Invalid material identifier '{}'", identifier);
            return None;
        }

        let ref_mat = Rc::new(RefCell::new(mat.unwrap()));

        log::info!("Generated material '{}'", identifier);
        self.materials
            .insert(identifier.to_string(), ref_mat.clone());
        return Some(ref_mat);
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
