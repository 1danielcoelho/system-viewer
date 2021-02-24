use self::{material::Material, mesh::Mesh, procedural_meshes::*, texture::Texture};
use crate::fetch_bytes;
use crate::managers::resource::body_description::{BodyDescription, OrbitalElements, StateVector};
use crate::managers::resource::material::UniformName;
use crate::managers::resource::texture::TextureUnit;
use crate::utils::gl::GL;
use crate::utils::string::{get_unique_name, remove_numbered_suffix};
use crate::{managers::scene::Scene, GLCTX};
use image::{io::Reader, DynamicImage, ImageFormat};
use std::{cell::RefCell, collections::HashMap, io::Cursor, rc::Rc};
use web_sys::WebGl2RenderingContext;

pub mod body_description;
pub mod collider;
pub mod gltf;
pub mod intermediate_mesh;
pub mod material;
pub mod mesh;
pub mod procedural_meshes;
pub mod shaders;
pub mod texture;

fn load_texture_from_bytes(
    width: u32,
    height: u32,
    gl_format: u32,
    bytes: &[u8],
    ctx: &WebGl2RenderingContext,
) -> web_sys::WebGlTexture {
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
        gl_format as i32,
        width as i32,
        height as i32,
        0,
        gl_format,
        GL::UNSIGNED_BYTE, // Just u8 for now
        Some(bytes),
    )
    .unwrap();

    ctx.bind_texture(GL::TEXTURE_2D, None);

    return gl_tex;
}

fn load_texture_from_image_bytes(
    identifier: &str,
    bytes: &[u8],
    image_format: Option<ImageFormat>,
    ctx: &WebGl2RenderingContext,
) -> Result<Rc<RefCell<Texture>>, String> {
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

    let gl_tex = load_texture_from_bytes(width, height, format, buf.as_ref().unwrap(), ctx);

    return Ok(Rc::new(RefCell::new(Texture {
        name: identifier.to_owned(),
        width,
        height,
        gl_format: format,
        num_channels,
        gl_handle: Some(gl_tex),
    })));
}

pub struct ResourceManager {
    meshes: HashMap<String, Rc<RefCell<Mesh>>>,
    textures: HashMap<String, Rc<RefCell<Texture>>>,
    materials: HashMap<String, Rc<RefCell<Material>>>,
    bodies: HashMap<String, HashMap<String, BodyDescription>>,
    state_vectors: HashMap<String, Vec<StateVector>>,
    osc_elements: HashMap<String, Vec<OrbitalElements>>,
}
impl ResourceManager {
    pub fn new() -> Self {
        let mut new_res_man = Self {
            meshes: HashMap::new(),
            textures: HashMap::new(),
            materials: HashMap::new(),
            bodies: HashMap::new(),
            state_vectors: HashMap::new(),
            osc_elements: HashMap::new(),
        };

        new_res_man.create_default_resources();

        return new_res_man;
    }

    /// Create resources that are used to signal that a resource is not yet available (e.g. default
    /// texture)
    fn create_default_resources(&mut self) {
        GLCTX.with(|ctx| {
            let ref_mut = ctx.borrow_mut();
            let ctx = ref_mut.as_ref().unwrap();

            let magenta: [u8; 4] = [255, 0, 255, 255];
            let gl_tex = load_texture_from_bytes(1, 1, GL::RGBA, &magenta, &ctx);

            let tex = Rc::new(RefCell::new(Texture {
                name: String::from("default_texture"),
                width: 1,
                height: 1,
                gl_format: GL::RGBA,
                num_channels: 4,
                gl_handle: Some(gl_tex),
            }));

            self.textures.insert("default_texture".to_string(), tex);
        });
    }

    fn provision_texture(&mut self, tex: &Rc<RefCell<Texture>>) -> Option<Rc<RefCell<Texture>>> {
        let tex_borrow = tex.borrow();
        if let Some(existing_tex) = self.get_texture(&tex_borrow.name) {
            log::info!("Reusing existing texture '{}'", existing_tex.borrow().name);
            return Some(existing_tex);
        }

        // TODO: Fetch for this texture data
        if tex_borrow.gl_handle.is_none() {
            let scene_name = gltf::get_scene_name(&tex_borrow.name);
            log::info!(
                "Creating fetch request for texture '{}', from file '{}'",
                tex_borrow.name,
                scene_name
            );

            // TODO: Restore asset manifest to keep track of the path instead of just assuming it's at public
            fetch_bytes(&("public/".to_owned() + scene_name), "glb_resouce");
        }

        log::info!("Keeping temporary texture '{}'", tex_borrow.name);
        self.textures.insert(tex_borrow.name.clone(), tex.clone());
        return None;
    }

    /** If we have a material with this name already, return that material. Otherwise start tracking this material and provision its textures */
    fn provision_material(
        &mut self,
        mat: &mut Rc<RefCell<Material>>,
    ) -> Option<Rc<RefCell<Material>>> {
        let mut mat_mut = mat.borrow_mut();
        if let Some(existing_mat) = self.get_or_create_material(&mat_mut.get_name()) {
            log::info!("Reusing existing material '{}'", mat_mut.get_name());
            return Some(existing_mat);
        }

        // We're creating a new material. Let's request all the textures that it needs
        let mut requested_textures: HashMap<TextureUnit, Rc<RefCell<Texture>>> = HashMap::new();
        for (unit, tex) in mat_mut.get_textures().iter() {
            if let Some(new_tex) = self.provision_texture(&tex) {
                requested_textures.insert(*unit, new_tex);
            }
        }
        for (unit, tex) in requested_textures.iter() {
            mat_mut.set_texture(*unit, Some(tex.clone()));
        }

        log::info!("Keeping material '{}'", mat_mut.get_name());
        self.materials
            .insert(mat_mut.get_name().to_owned(), mat.clone());
        return None;
    }

    /** Try getting/creating a mesh with the same name. If we have nothing, start tracking this mesh and provision its materials */
    fn provision_mesh(&mut self, mesh: &mut Rc<RefCell<Mesh>>) -> Option<Rc<RefCell<Mesh>>> {
        let mesh_mut = mesh.borrow_mut();
        if let Some(existing_mesh) = self.get_or_create_mesh(&mesh_mut.name) {
            log::info!("Reusing existing mesh '{}'", mesh_mut.name);
            return Some(existing_mesh);
        }

        if !mesh_mut.loaded {
            let scene_name = gltf::get_scene_name(&mesh_mut.name);
            log::info!(
                "Creating fetch request for mesh '{}', from file '{}'",
                mesh_mut.name,
                scene_name
            );

            // TODO: Restore asset manifest to keep track of the path instead of just assuming it's at public
            fetch_bytes(&("public/".to_owned() + scene_name), "glb_resource");
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
            "grid" => Some(generate_grid(11, default_mat)),
            "axes" => Some(generate_axes(self.get_or_create_material("vertex_color"))),
            "circle" => Some(generate_circle(100, default_mat)),
            "lat_long_sphere" => Some(generate_lat_long_sphere(
                32,
                32,
                1.0,
                true,
                true,
                default_mat,
            )),
            "ico_sphere" => Some(generate_ico_sphere(1.0, 2, false, default_mat)),
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

        log::error!(
            "Failed to find a material for identifier '{}'. Current valid identifiers:\n{:#?}",
            identifier,
            self.materials.keys()
        );

        return None;
    }

    /** This is used so that the GLTF import code path can instantiate GLTF materials and swap those with existing temp materials we may have */
    fn instantiate_material_without_inserting(
        &mut self,
        master: &str,
        name: &str,
    ) -> Option<Rc<RefCell<Material>>> {
        let master_mat = self.get_or_create_material(master);
        if master_mat.is_none() {
            return None;
        };

        let instance = Rc::new(RefCell::new(master_mat.as_ref().unwrap().borrow().clone()));

        let new_name = get_unique_name(remove_numbered_suffix(name), &self.materials);
        instance.borrow_mut().name = new_name.to_owned();

        log::info!(
            "Generated material instance '{}' from master '{}'",
            instance.borrow().name,
            master_mat.unwrap().borrow().name
        );

        return Some(instance);
    }

    pub fn instantiate_material(
        &mut self,
        master: &str,
        name: &str,
    ) -> Option<Rc<RefCell<Material>>> {
        let instance = self.instantiate_material_without_inserting(master, name);
        if instance.is_none() {
            return None;
        }
        let instance = instance.unwrap();

        let new_name = &instance.borrow().name;
        assert!(!self.materials.contains_key(new_name));

        self.materials.insert(new_name.to_owned(), instance.clone());
        return Some(instance.clone());
    }

    pub fn get_or_create_material(&mut self, identifier: &str) -> Option<Rc<RefCell<Material>>> {
        if let Some(mat) = self.materials.get(identifier) {
            return Some(mat.clone());
        }

        let mat = match identifier {
            "default" => Some(Material::new(
                identifier,
                "relay_color.vert",
                "white.frag",
                &[UniformName::WVPTrans],
            )),
            "vertex_color" => Some(Material::new(
                identifier,
                "relay_color.vert",
                "color.frag",
                &[UniformName::WVPTrans],
            )),
            "world_normals" => Some(Material::new(
                identifier,
                "relay_all.vert",
                "world_normals.frag",
                &[
                    UniformName::WVTrans,
                    UniformName::WVInvTranspTrans,
                    UniformName::WVPTrans,
                ],
            )),
            "world_tangents" => Some(Material::new(
                identifier,
                "relay_all.vert",
                "world_tangents.frag",
                &[
                    UniformName::WVTrans,
                    UniformName::WVInvTranspTrans,
                    UniformName::WVPTrans,
                ],
            )),
            "uv0" => Some(Material::new(
                identifier,
                "relay_all.vert",
                "uv0.frag",
                &[
                    UniformName::WVTrans,
                    UniformName::WVInvTranspTrans,
                    UniformName::WVPTrans,
                ],
            )),
            "uv1" => Some(Material::new(
                identifier,
                "relay_all.vert",
                "uv1.frag",
                &[
                    UniformName::WVTrans,
                    UniformName::WVInvTranspTrans,
                    UniformName::WVPTrans,
                ],
            )),
            "basecolor" => Some(Material::new(
                identifier,
                "relay_all.vert",
                "basecolor.frag",
                &[
                    UniformName::WVTrans,
                    UniformName::WVInvTranspTrans,
                    UniformName::WVPTrans,
                    UniformName::BaseColor,
                ],
            )),
            "phong" => Some(Material::new(
                identifier,
                "relay_all.vert",
                "phong.frag",
                &[
                    UniformName::WVTrans,
                    UniformName::WVInvTranspTrans,
                    UniformName::WVPTrans,
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
                    UniformName::WVTrans,
                    UniformName::WVInvTranspTrans,
                    UniformName::WVPTrans,
                    UniformName::LightTypes,
                    UniformName::LightPosDir,
                    UniformName::LightColors,
                    UniformName::LightIntensities,
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
            log::error!(
                "Invalid material identifier '{}'. Current valid identifiers:\n{:#?}",
                identifier,
                self.materials.keys()
            );
            return None;
        }

        let ref_mat = Rc::new(RefCell::new(mat.unwrap()));

        assert!(!self.materials.contains_key(identifier));

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
        GLCTX.with(|ctx| {
            let ref_mut = ctx.borrow_mut();
            let ctx = ref_mut.as_ref().unwrap();

            match load_texture_from_image_bytes(identifier, bytes, image_format, &ctx) {
                Ok(tex) => {
                    log::info!("Generated texture '{}'", identifier);

                    if let Some(existing_tex) = self.textures.get(identifier) {
                        existing_tex.swap(&tex);

                        log::info!(
                            "Mutating existing texture resource '{}' with new data",
                            identifier
                        );
                    } else if let Some(_) = self.textures.insert(identifier.to_owned(), tex) {
                        log::info!(
                            "Changing tracked material resource for name '{}'",
                            identifier
                        );
                    }
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

    pub fn get_texture(&self, identifier: &str) -> Option<Rc<RefCell<Texture>>> {
        if let Some(tex) = self.textures.get(identifier) {
            return Some(tex.clone());
        }

        // We don't have this one, put out a request for this asset and just return
        // the default pink texture instead
        fetch_bytes(&("public/textures/".to_owned() + identifier), "texture");

        return self.textures.get("default_texture").cloned();
    }

    pub fn load_database_file(&mut self, url: &str, content_type: &str, text: &str) {
        match content_type {
            "body_database" => {
                let mut parsed_data: HashMap<String, BodyDescription> =
                    serde_json::de::from_str(text)
                        .map_err(|e| format!("Database deserialization error:\n{}", e).to_owned())
                        .unwrap();

                // TODO: Do I need the ids in the bodies as well?
                for (key, val) in parsed_data.iter_mut() {
                    val.id = Some(key.clone());
                }

                let database_name: String = std::path::Path::new(url)
                    .file_stem()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_owned();

                let num_parsed = parsed_data.len();
                self.bodies.insert(database_name, parsed_data);

                log::info!("Parsed {} bodies from database '{}'", num_parsed, url);
            }
            "vectors_database" => {
                let parsed_data: HashMap<String, Vec<StateVector>> = serde_json::de::from_str(text)
                    .map_err(|e| format!("Database deserialization error:\n{}", e).to_owned())
                    .unwrap();

                let num_parsed = parsed_data.len();
                self.state_vectors = parsed_data;

                log::info!(
                    "Parsed {} state vectors from database '{}'",
                    num_parsed,
                    url
                );
            }
            "elements_database" => {
                let parsed_data: HashMap<String, Vec<OrbitalElements>> =
                    serde_json::de::from_str(text)
                        .map_err(|e| format!("Database deserialization error:\n{}", e).to_owned())
                        .unwrap();

                let num_parsed = parsed_data.len();
                self.osc_elements = parsed_data;

                log::info!(
                    "Parsed {} orbital elements from database '{}'",
                    num_parsed,
                    url
                );
            }
            _ => {
                log::error!(
                    "Unexpected database content type '{}' with url '{}'",
                    content_type,
                    url
                );
                return;
            }
        }
    }

    pub fn take_body_database(
        &mut self,
        db_name: &str,
    ) -> Result<HashMap<String, BodyDescription>, String> {
        let db = self.bodies.remove(db_name).ok_or(String::from(format!(
            "Resource manager has no database with name '{}'",
            db_name
        )))?;

        return Ok(db);
    }

    pub fn set_body_database(&mut self, db_name: &str, db: HashMap<String, BodyDescription>) {
        self.bodies.insert(db_name.to_owned(), db);
    }

    pub fn get_state_vectors(&mut self) -> &HashMap<String, Vec<StateVector>> {
        return &self.state_vectors;
    }

    pub fn get_osc_elements(&mut self) -> &HashMap<String, Vec<OrbitalElements>> {
        return &self.osc_elements;
    }

    pub fn get_body(&self, db_name: &str, body_id: &str) -> Result<&BodyDescription, String> {
        let db = self.bodies.get(db_name).ok_or(String::from(format!(
            "Resource manager has no database with name '{}'",
            db_name
        )))?;

        let body = db.get(body_id).ok_or(String::from(format!(
            "Resource manager's database '{}' has no body with id '{}'",
            db_name, body_id
        )))?;

        return Ok(body);
    }
}
