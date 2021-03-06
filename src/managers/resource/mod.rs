use self::{material::Material, mesh::Mesh, procedural_meshes::*, texture::Texture};
use crate::fetch_bytes;
use crate::managers::resource::body_description::{BodyDescription, OrbitalElements, StateVector};
use crate::managers::resource::material::UniformName;
use crate::managers::resource::texture::TextureUnit;
use crate::utils::gl::GL;
use crate::utils::hashmap::InsertOrGet;
use crate::utils::string::{get_unique_name, remove_numbered_suffix};
use crate::{managers::scene::Scene, GLCTX};
use image::{io::Reader, DynamicImage};
use std::path::PathBuf;
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

fn load_texture_from_image_bytes(
    identifier: &str,
    bytes: &[u8],
    ctx: &WebGl2RenderingContext,
) -> Result<Rc<RefCell<Texture>>, String> {
    let reader = Reader::new(Cursor::new(bytes))
        .with_guessed_format()
        .map_err(|e| e.to_string())?;

    let decoded = reader.decode().map_err(|e| e.to_string())?;

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

    let buf = buf.ok_or(format!(
        "Failed to retrieve a buffer for texture with identifier '{}'",
        identifier
    ))?;

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
        Some(buf),
    )
    .unwrap();

    ctx.bind_texture(GL::TEXTURE_2D, None);

    return Ok(Rc::new(RefCell::new(Texture {
        name: identifier.to_owned(),
        width,
        height,
        gl_format: format,
        num_channels,
        gl_handle: Some(gl_tex),
        is_cubemap: false,
    })));
}

fn load_cubemap_face(
    face: usize,
    face_data: &Vec<u8>,
    ctx: &WebGl2RenderingContext,
) -> Result<(u32, u32, u32, u8), String> {
    let reader = Reader::new(Cursor::new(face_data))
        .with_guessed_format()
        .map_err(|e| e.to_string())?;

    let decoded = reader.decode().map_err(|e| e.to_string())?;

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

    let buf = buf.ok_or(String::from(
        "Failed to retrieve a buffer for cubemap face texture",
    ))?;

    ctx.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
        TempCubemap::get_target_from_index(face).unwrap(),
        0,
        format as i32,
        width as i32,
        height as i32,
        0,
        format,
        GL::UNSIGNED_BYTE, // Just u8 for now
        Some(buf),
    )
    .unwrap();

    return Ok((width, height, format, num_channels));
}

fn load_cubemap_texture_from_image_bytes(
    identifier: &str,
    cubemap: &mut TempCubemap,
    ctx: &WebGl2RenderingContext,
) -> Result<Rc<RefCell<Texture>>, String> {
    let gl_tex = ctx.create_texture().unwrap();
    ctx.active_texture(GL::TEXTURE0);
    ctx.bind_texture(GL::TEXTURE_CUBE_MAP, Some(&gl_tex));

    let mut success: bool = true;
    let mut width: u32 = 0;
    let mut height: u32 = 0;
    let mut format: u32 = 0;
    let mut num_channels: u8 = 0;

    for (index, face) in cubemap.faces.iter_mut().enumerate() {
        if let Ok((face_width, face_height, face_format, face_channels)) =
            load_cubemap_face(index, face, ctx)
        {
            // For now we just assume they'll all be the same
            width = face_width;
            height = face_height;
            format = face_format;
            num_channels = face_channels;
        } else {
            log::error!(
                "Error loading cubemap face {} for identifier '{}'",
                index,
                identifier
            );
            success = false;
            break;
        }
    }

    if success {
        ctx.generate_mipmap(GL::TEXTURE_CUBE_MAP);
        ctx.tex_parameteri(
            GL::TEXTURE_CUBE_MAP,
            GL::TEXTURE_MIN_FILTER,
            GL::LINEAR_MIPMAP_LINEAR as i32,
        );
    }

    // Dump our input bytes now that the texture is created/failed
    for face in cubemap.faces.iter_mut() {
        face.clear();
    }
    cubemap.completed = false;

    ctx.bind_texture(GL::TEXTURE_CUBE_MAP, None);

    return Ok(Rc::new(RefCell::new(Texture {
        name: identifier.to_owned(),
        width,
        height,
        gl_format: format,
        num_channels,
        gl_handle: Some(gl_tex),
        is_cubemap: true,
    })));
}

#[derive(Default)]
pub struct TempCubemap {
    faces: [Vec<u8>; 6],
    completed: bool,
}
impl TempCubemap {
    const LEFT: usize = 0;
    const RIGHT: usize = 1;
    const TOP: usize = 2;
    const BOTTOM: usize = 3;
    const FRONT: usize = 4;
    const BACK: usize = 5;

    pub fn get_face_from_filename(&mut self, filename: &str) -> Option<&mut Vec<u8>> {
        match filename {
            "Left" => Some(&mut self.faces[TempCubemap::LEFT]),
            "Right" => Some(&mut self.faces[TempCubemap::RIGHT]),
            "Top" => Some(&mut self.faces[TempCubemap::TOP]),
            "Bottom" => Some(&mut self.faces[TempCubemap::BOTTOM]),
            "Front" => Some(&mut self.faces[TempCubemap::FRONT]),
            "Back" => Some(&mut self.faces[TempCubemap::BACK]),
            _ => {
                log::error!("Unexpected cubemap face '{}'", filename);
                None
            }
        }
    }

    pub fn get_target_from_index(face_index: usize) -> Option<u32> {
        match face_index {
            TempCubemap::LEFT => Some(GL::TEXTURE_CUBE_MAP_NEGATIVE_Y),
            TempCubemap::RIGHT => Some(GL::TEXTURE_CUBE_MAP_POSITIVE_Y),

            TempCubemap::TOP => Some(GL::TEXTURE_CUBE_MAP_POSITIVE_Z),
            TempCubemap::BOTTOM => Some(GL::TEXTURE_CUBE_MAP_NEGATIVE_X),

            TempCubemap::FRONT => Some(GL::TEXTURE_CUBE_MAP_NEGATIVE_Z),
            TempCubemap::BACK => Some(GL::TEXTURE_CUBE_MAP_POSITIVE_X),
            _ => None,
        }
    }

    pub fn is_ready_to_convert(&self) -> bool {
        return !self.completed && self.faces.iter().all(|f| f.len() > 0);
    }
}

pub struct ResourceManager {
    meshes: HashMap<String, Rc<RefCell<Mesh>>>,
    textures: HashMap<String, Rc<RefCell<Texture>>>,
    materials: HashMap<String, Rc<RefCell<Material>>>,
    bodies: HashMap<String, HashMap<String, BodyDescription>>,
    state_vectors: HashMap<String, Vec<StateVector>>,
    osc_elements: HashMap<String, Vec<OrbitalElements>>,

    // We have to request cubemap textures one face at a time. This is where we
    // temporarily store them until we have received all 6 to create
    temp_cubemaps: HashMap<String, TempCubemap>,
}
impl ResourceManager {
    pub fn new() -> Self {
        let new_res_man = Self {
            meshes: HashMap::new(),
            textures: HashMap::new(),
            materials: HashMap::new(),
            bodies: HashMap::new(),
            state_vectors: HashMap::new(),
            osc_elements: HashMap::new(),
            temp_cubemaps: HashMap::new(),
        };

        return new_res_man;
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
        let default_mat_quad = self.get_or_create_material("default_screenspace");

        let mesh: Option<Rc<RefCell<Mesh>>> = match identifier {
            "quad" => Some(generate_canvas_quad(default_mat_quad)),
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

        if let Some(mesh) = mesh {
            log::info!("Generated mesh '{}'", identifier);
            assert!(!self.meshes.contains_key(identifier));

            mesh.borrow_mut().name = identifier.to_owned();
            self.meshes.insert(identifier.to_string(), mesh.clone());
            return Some(mesh);
        }
        
        let full_path: String = "public/gltf/".to_owned() + identifier;
        fetch_bytes(&full_path, "gltf");
        let temp_mesh = Some(generate_temp());
        self.meshes.insert(full_path, temp_mesh.as_ref().unwrap().clone());
        return temp_mesh;
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

    pub fn instantiate_material(
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
            "default_screenspace" => Some(Material::new(
                identifier,
                "screenspace.vert",
                "screenspace.frag",
                &[],
            )),
            "skybox" => Some(Material::new(
                identifier,
                "screenspace.vert",
                "skybox.frag",
                &[UniformName::VPInvTrans],
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

    pub fn receive_cubemap_face_file_bytes(&mut self, identifier: &str, bytes: &[u8]) {
        let buf = PathBuf::from(identifier);
        let path = buf.as_path();
        let file = path.file_stem().unwrap().to_str().unwrap_or_default();
        let cubemap_identifier: String = path
            .parent()
            .and_then(|p| p.to_str())
            .unwrap_or_default()
            .to_string();

        let cubemap: &mut TempCubemap =
            self.temp_cubemaps.insert_or_get(cubemap_identifier.clone());

        if cubemap.completed {
            log::error!(
                "Received face '{}' for completed cubemap '{}",
                file,
                identifier
            );
            return;
        }

        if let Some(face_vec) = cubemap.get_face_from_filename(file) {
            face_vec.resize(bytes.len(), 0);
            face_vec.copy_from_slice(bytes);
        }

        log::info!(
            "Received cubemap face bytes: '{}'. Skybox id: '{}', file: '{}'",
            identifier,
            cubemap_identifier,
            file
        );

        if !cubemap.is_ready_to_convert() {
            return;
        }

        cubemap.completed = true;

        log::info!(
            "Received all faces for cubemap: '{}'. Creating texture...",
            cubemap_identifier,
        );

        let mut tex: Option<Rc<RefCell<Texture>>> = None;
        GLCTX.with(|ctx| {
            let ref_mut = ctx.borrow_mut();
            let ctx = ref_mut.as_ref().unwrap();

            let converted_tex =
                load_cubemap_texture_from_image_bytes(&cubemap_identifier, cubemap, &ctx);
            if let Err(err) = converted_tex {
                log::error!(
                    "Error when trying to load cubemap texture '{}': {}",
                    cubemap_identifier,
                    err
                );
                return;
            }
            tex = Some(converted_tex.unwrap());
        });
        let tex = tex.unwrap();

        log::info!("Generated cubemap texture '{}'", cubemap_identifier);
        if let Some(existing_tex) = self.textures.get(&cubemap_identifier) {
            existing_tex.swap(&tex);

            log::info!(
                "Mutating existing cubemap texture resource '{}' with new data",
                cubemap_identifier
            );
        } else {
            self.textures.insert(cubemap_identifier.to_owned(), tex);
        };
    }

    pub fn receive_texture_file_bytes(&mut self, identifier: &str, bytes: &[u8]) {
        GLCTX.with(|ctx| {
            let ref_mut = ctx.borrow_mut();
            let ctx = ref_mut.as_ref().unwrap();

            let tex = load_texture_from_image_bytes(identifier, bytes, &ctx);
            if let Err(err) = tex {
                log::error!(
                    "Error when trying to load texture '{}': {}",
                    identifier,
                    err
                );
                return;
            }
            let tex = tex.unwrap();

            log::info!("Generated texture '{}'", identifier);
            if let Some(existing_tex) = self.textures.get(identifier) {
                existing_tex.swap(&tex);

                log::info!(
                    "Mutating existing texture resource '{}' with new data",
                    identifier
                );
            } else {
                self.textures.insert(identifier.to_owned(), tex);
            }
        });
    }

    pub fn get_or_request_texture(
        &mut self,
        identifier: &str,
        is_cubemap: bool,
    ) -> Option<Rc<RefCell<Texture>>> {
        if let Some(tex) = self.textures.get(identifier) {
            return Some(tex.clone());
        }

        // We don't have this one, put out a request for this asset and just return
        // the default pink texture instead
        let full_path: String = "public/textures/".to_owned() + identifier;
        if is_cubemap {
            fetch_bytes(&(full_path.clone() + "/Right.jpg"), "cubemap_face");
            fetch_bytes(&(full_path.clone() + "/Left.jpg"), "cubemap_face");
            fetch_bytes(&(full_path.clone() + "/Top.jpg"), "cubemap_face");
            fetch_bytes(&(full_path.clone() + "/Bottom.jpg"), "cubemap_face");
            fetch_bytes(&(full_path.clone() + "/Front.jpg"), "cubemap_face");
            fetch_bytes(&(full_path.clone() + "/Back.jpg"), "cubemap_face");
        } else {
            let full_path: String = "public/textures/".to_owned() + identifier;
            fetch_bytes(&full_path, "texture");
        }

        let tex = Rc::new(RefCell::new(Texture {
            name: full_path.to_owned(),
            width: 1,
            height: 1,
            gl_format: GL::RGBA,
            num_channels: 4,
            gl_handle: None,
            is_cubemap,
        }));
        self.textures.insert(full_path, tex.clone());

        return Some(tex);
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
