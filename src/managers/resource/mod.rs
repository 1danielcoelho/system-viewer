use self::{material::Material, mesh::Mesh, procedural_meshes::*, texture::Texture};
use crate::{
    managers::resource::{material::UniformName, texture::TextureUnit},
    utils::{
        gl::GL,
        string::{get_unique_name, remove_numbered_suffix},
    },
};
use crate::{managers::scene::Scene, GLCTX};
use image::{io::Reader, DynamicImage, ImageFormat};
use std::{
    cell::{RefCell, RefMut},
    collections::HashMap,
    io::Cursor,
    rc::Rc,
};
use web_sys::WebGl2RenderingContext;

pub mod collider;
pub mod gltf_resources;
pub mod intermediate_mesh;
pub mod material;
pub mod mesh;
pub mod procedural_meshes;
pub mod shaders;
pub mod texture;

fn load_texture_from_bytes(
    identifier: &str,
    bytes: &[u8],
    image_format: Option<ImageFormat>,
    ctx: &WebGl2RenderingContext,
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
    meshes: HashMap<String, Rc<RefCell<Mesh>>>,
    textures: HashMap<String, Rc<Texture>>,
    materials: HashMap<String, Rc<RefCell<Material>>>,
}
impl ResourceManager {
    pub fn new() -> Self {
        let new_res_man = Self {
            meshes: HashMap::new(),
            textures: HashMap::new(),
            materials: HashMap::new(),
        };

        return new_res_man;
    }

    fn provision_texture(&mut self, tex: &Rc<Texture>) -> Option<Rc<Texture>> {
        if let Some(existing_tex) = self.get_texture(&tex.name) {
            log::info!("Reusing existing texture '{}'", existing_tex.name);
            return Some(existing_tex);
        }

        // TODO: Fetch for this texture data
        if tex.gl_handle.is_none() {
            log::info!("Creating fetch request for texture '{}'", tex.name);
            todo!();
        }

        log::info!("Keeping temporary texture '{}'", tex.name);
        self.textures.insert(tex.name.clone(), tex.clone());
        return None;
    }

    /** If we have a material with this name already, return that material. Otherwise start tracking this material and provision its textures */
    fn provision_material(
        &mut self,
        mat: &mut Rc<RefCell<Material>>,
    ) -> Option<Rc<RefCell<Material>>> {
        let mut mat_mut = mat.borrow_mut();
        if let Some(existing_mat) = self.get_or_create_material(&mat_mut.name) {
            log::info!("Reusing existing material '{}'", existing_mat.borrow().name);
            return Some(existing_mat);
        }

        // We're creating a new material. Let's request all the textures that it needs
        let mut requested_textures: HashMap<TextureUnit, Rc<Texture>> = HashMap::new();
        for (unit, tex) in mat_mut.get_textures().iter() {
            if let Some(new_tex) = self.provision_texture(&tex) {
                requested_textures.insert(*unit, new_tex);
            }
        }
        for (unit, tex) in requested_textures.iter() {
            mat_mut.set_texture(*unit, Some(tex.clone()));
        }

        log::info!("Keeping material '{}'", mat_mut.name);
        self.materials.insert(mat_mut.name.clone(), mat.clone());
        return None;
    }

    /** Try getting/creating a mesh with the same name. If we have nothing, start tracking this mesh and provision its materials */
    fn provision_mesh(&mut self, mesh: &mut Rc<RefCell<Mesh>>) -> Option<Rc<RefCell<Mesh>>> {
        let mesh_mut = mesh.borrow_mut();
        if let Some(existing_mesh) = self.get_or_create_mesh(&mesh_mut.name) {
            log::info!("Reusing existing mesh '{}'", existing_mesh.borrow().name);
            return Some(existing_mesh);
        }

        // TODO: fetch request for this mesh from its identifier to e.g. try loading it from GLB
        // Likely will need to either cleave the glb path from the mesh name or start storing
        // a mesh "source" path, that can be like "public/Duck.glb/0"
        if !mesh_mut.loaded {
            log::info!("Creating fetch request for mesh '{}'", mesh_mut.name);
            todo!();
        }

        // Note: We don't have to care about provisioning the default materials: They will be loaded with
        // the mesh, whether the mesh is already existing, or whether we'll load it from the fetch request.
        // There is no way to specify different "default materials" for a mesh otherwise

        log::info!("Keeping temporary mesh '{}'", mesh_mut.name);
        self.meshes.insert(mesh_mut.name.clone(), mesh.clone());
        return None;
    }

    /** Makes sure we have all the assets that `scene` requires, and that its reference to those assets are deduplicated */
    pub fn provision_scene_assets(&mut self, scene: &mut Scene) {
        log::info!("Provisioning assets for scene '{}'...", scene.identifier);
        for mesh_comp in scene.mesh.iter_mut() {
            if let Some(mut mesh) = mesh_comp.get_mesh() {
                // Mesh
                if let Some(other_mesh) = self.provision_mesh(&mut mesh) {
                    mesh_comp.set_mesh(Some(other_mesh));
                }

                // Material overrides
                let num_mats = mesh.borrow_mut().primitives.len();
                for mat_index in 0..num_mats {
                    if let Some(mut over) = mesh_comp.get_material_override(mat_index) {
                        if let Some(other_mat) = self.provision_material(&mut over) {
                            mesh_comp.set_material_override(Some(other_mat), mat_index);
                        }
                    }
                }
            }
        }
    }

    pub fn get_mesh(&self, identifier: &str) -> Option<Rc<RefCell<Mesh>>> {
        if let Some(mesh) = self.meshes.get(identifier) {
            return Some(mesh.clone());
        }

        return None;
    }

    pub fn get_or_create_mesh(&mut self, identifier: &str) -> Option<Rc<RefCell<Mesh>>> {
        if let Some(mesh) = self.get_mesh(identifier) {
            return Some(mesh);
        }

        let default_mat = self.get_or_create_material("default");

        let mesh: Option<Rc<RefCell<Mesh>>> = match identifier {
            "cube" => Some(generate_cube(default_mat)),
            "plane" => Some(generate_plane(default_mat)),
            "grid" => Some(generate_grid(200, default_mat)),
            "axes" => Some(generate_axes(default_mat)),
            "circle" => Some(generate_circle(100, default_mat)),
            "lat_long_sphere" => Some(generate_lat_long_sphere(
                32,
                32,
                0.8,
                true,
                true,
                default_mat,
            )),
            "ico_sphere" => Some(generate_ico_sphere(0.8, 2, false, default_mat)),
            _ => None,
        };

        match mesh {
            Some(ref mesh) => {
                log::info!("Generated mesh '{}'", identifier);
                assert!(!self.meshes.contains_key(identifier));

                mesh.borrow_mut().name = identifier.to_owned();
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
            "world_normals" => Some(Material::new(
                identifier,
                "relay_all.vert",
                "world_normals.frag",
                &[
                    UniformName::WorldTrans,
                    UniformName::WorldTransInvTranspose,
                    UniformName::ViewProjTrans,
                ],
            )),
            "world_tangents" => Some(Material::new(
                identifier,
                "relay_all.vert",
                "world_tangents.frag",
                &[
                    UniformName::WorldTrans,
                    UniformName::WorldTransInvTranspose,
                    UniformName::ViewProjTrans,
                ],
            )),
            "uv0" => Some(Material::new(
                identifier,
                "relay_all.vert",
                "uv0.frag",
                &[
                    UniformName::WorldTrans,
                    UniformName::WorldTransInvTranspose,
                    UniformName::ViewProjTrans,
                ],
            )),
            "uv1" => Some(Material::new(
                identifier,
                "relay_all.vert",
                "uv1.frag",
                &[
                    UniformName::WorldTrans,
                    UniformName::WorldTransInvTranspose,
                    UniformName::ViewProjTrans,
                ],
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
                    UniformName::WorldTransInvTranspose,
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
                    UniformName::WorldTransInvTranspose,
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

        GLCTX.with(|ctx| {
            let ref_mut = ctx.borrow_mut();
            let ctx = ref_mut.as_ref().unwrap();

            match load_texture_from_bytes(identifier, bytes, image_format, &ctx) {
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
        });
    }

    pub fn get_texture(&self, identifier: &str) -> Option<Rc<Texture>> {
        if let Some(tex) = self.textures.get(identifier) {
            return Some(tex.clone());
        }

        return None;
    }
}
