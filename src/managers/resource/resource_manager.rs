use crate::managers::resource::material::Material;
use crate::managers::resource::material::UniformName;
use crate::managers::resource::mesh::Mesh;
use crate::managers::resource::procedural_meshes::*;
use crate::managers::resource::texture::Texture;
use crate::utils::gl::GL;
use crate::utils::hashmap::InsertOrGet;
use crate::utils::string::{get_unique_name, remove_numbered_suffix};
use crate::utils::web::request_bytes;
use crate::{ENGINE, GLCTX};
use futures::future::join_all;
use glow::*;
use image::{io::Reader, DynamicImage};
use std::path::PathBuf;
use std::rc::Weak;
use std::{cell::RefCell, collections::HashMap, io::Cursor, rc::Rc};
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::spawn_local;

pub(super) fn load_texture_from_bytes(
    identifier: &str,
    width: u32,
    height: u32,
    num_channels: u8,
    format: u32,
    data: &[u8],
    mag_filter: Option<i32>,
    min_filter: Option<i32>,
    wrap_s: Option<i32>,
    wrap_t: Option<i32>,
) -> Result<Rc<RefCell<Texture>>, String> {
    return GLCTX.with(|ctx| {
        unsafe {
            let gl_tex = ctx.create_texture().unwrap();
            ctx.active_texture(GL::TEXTURE0);
            ctx.bind_texture(GL::TEXTURE_2D, Some(gl_tex));

            ctx.tex_parameter_i32(
                GL::TEXTURE_2D,
                GL::TEXTURE_WRAP_S,
                wrap_s.unwrap_or(GL::CLAMP_TO_EDGE as i32),
            );
            ctx.tex_parameter_i32(
                GL::TEXTURE_2D,
                GL::TEXTURE_WRAP_T,
                wrap_t.unwrap_or(GL::CLAMP_TO_EDGE as i32),
            );
            ctx.tex_parameter_i32(
                GL::TEXTURE_2D,
                GL::TEXTURE_MIN_FILTER,
                min_filter.unwrap_or(GL::LINEAR as i32),
            );
            ctx.tex_parameter_i32(
                GL::TEXTURE_2D,
                GL::TEXTURE_MAG_FILTER,
                mag_filter.unwrap_or(GL::LINEAR as i32),
            );

            ctx.tex_image_2d(
                GL::TEXTURE_2D,
                0,
                format as i32,
                width as i32,
                height as i32,
                0,
                format,
                GL::UNSIGNED_BYTE, // Just u8 for now
                Some(data),
            );

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
    });
}

fn load_texture_from_image_bytes(
    identifier: &str,
    bytes: &[u8],
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

    return load_texture_from_bytes(
        identifier,
        width,
        height,
        num_channels,
        format,
        buf,
        None,
        None,
        None,
        None,
    );
}

fn load_cubemap_face(
    face: usize,
    face_data: &Vec<u8>,
    ctx: &glow::Context,
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

    unsafe {
        ctx.tex_image_2d(
            TempCubemap::get_target_from_index(face).unwrap(),
            0,
            format as i32,
            width as i32,
            height as i32,
            0,
            format,
            GL::UNSIGNED_BYTE, // Just u8 for now
            Some(buf),
        );
    }

    return Ok((width, height, format, num_channels));
}

fn load_cubemap_texture_from_image_bytes(
    identifier: &str,
    cubemap: &mut TempCubemap,
    ctx: &glow::Context,
) -> Result<Rc<RefCell<Texture>>, String> {
    let gl_tex: Option<glow::Texture>;
    unsafe {
        gl_tex = Some(ctx.create_texture().unwrap());
        ctx.active_texture(GL::TEXTURE0);
        ctx.bind_texture(GL::TEXTURE_CUBE_MAP, gl_tex);
    }

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

    // Dump our input bytes now that the texture is created/failed
    for face in cubemap.faces.iter_mut() {
        face.clear();
    }
    cubemap.completed = false;

    unsafe {
        if success {
            ctx.generate_mipmap(GL::TEXTURE_CUBE_MAP);
            ctx.tex_parameter_i32(
                GL::TEXTURE_CUBE_MAP,
                GL::TEXTURE_MIN_FILTER,
                GL::LINEAR_MIPMAP_LINEAR as i32,
            );
        }

        ctx.bind_texture(GL::TEXTURE_CUBE_MAP, None);
    }

    return Ok(Rc::new(RefCell::new(Texture {
        name: identifier.to_owned(),
        width,
        height,
        gl_format: format,
        num_channels,
        gl_handle: gl_tex,
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
    pub(super) meshes: HashMap<String, Rc<RefCell<Mesh>>>,
    pub(super) textures: HashMap<String, Rc<RefCell<Texture>>>,
    pub(super) materials: HashMap<String, Rc<RefCell<Material>>>,

    // We have to request cubemap textures one face at a time. This is where we
    // temporarily store them until we have received all 6 to create
    pub(super) temp_cubemaps: HashMap<String, TempCubemap>,

    pub(super) default_texture: Option<Weak<RefCell<Texture>>>,
}
impl ResourceManager {
    pub fn new() -> Self {
        let new_res_man = Self {
            meshes: HashMap::new(),
            textures: HashMap::new(),
            materials: HashMap::new(),
            temp_cubemaps: HashMap::new(),
            default_texture: None,
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

        // TODO: Better handle material variants instead of just making a new instance every time
        let default_mat = self.instantiate_material("default", "default");
        let default_mat_quad = self.get_or_create_material("default_screenspace");

        let mesh: Option<Rc<RefCell<Mesh>>> = match identifier {
            "quad" => Some(generate_canvas_quad(default_mat_quad)),
            "cube" => Some(generate_cube(default_mat)),
            "plane" => Some(generate_plane(default_mat)),
            "grid" => Some(generate_grid(11, default_mat)),
            "axes" => Some(generate_axes(self.get_or_create_material("vertex_color"))),
            "circle" => Some(generate_circle(100, default_mat)),
            "disk" => Some(generate_disk(4, 128, 0.5, 1.0, true, default_mat)),
            "lat_long_sphere" => Some(generate_lat_long_sphere(
                32,
                32,
                1.0,
                true,
                true,
                default_mat,
            )),
            "ico_sphere" => Some(generate_ico_sphere(1.0, 2, false, default_mat)),
            "points" => Some(generate_points()),
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

        let internal_full_path = full_path.clone();
        spawn_local(async move {
            let mut vec = request_bytes(&internal_full_path).await.unwrap();

            ENGINE.with(|e| {
                let mut ref_mut = e.borrow_mut();
                let e = ref_mut.as_mut().unwrap();

                e.receive_bytes(&internal_full_path, "gltf", &mut vec);
            });
        });

        let temp_mesh = Some(generate_temp());
        self.meshes
            .insert(full_path, temp_mesh.as_ref().unwrap().clone());
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
                &[UniformName::WVPTrans, UniformName::Fcoef],
            )),
            "default_screenspace" => Some(Material::new(
                identifier,
                "screenspace.vert",
                "screenspace.frag",
                &[UniformName::BaseColor],
            )),
            "default_points" => Some(Material::new(
                identifier,
                "relay_points.vert",
                "color.frag",
                &[UniformName::Fcoef],
            )),
            "skybox" => Some(Material::new(
                identifier,
                "screenspace.vert",
                "skybox.frag",
                &[UniformName::VPInvTrans, UniformName::ExposureFactor],
            )),
            "vertex_color" => Some(Material::new(
                identifier,
                "relay_color.vert",
                "color.frag",
                &[UniformName::WVPTrans, UniformName::Fcoef],
            )),
            "local_normals" => Some(Material::new(
                identifier,
                "relay_locals.vert",
                "normals.frag",
                &[
                    UniformName::WVTrans,
                    UniformName::WVInvTranspTrans,
                    UniformName::WVPTrans,
                    UniformName::Fcoef,
                ],
            )),
            "world_normals" => Some(Material::new(
                identifier,
                "relay_all.vert",
                "normals.frag",
                &[
                    UniformName::WVTrans,
                    UniformName::WVInvTranspTrans,
                    UniformName::WVPTrans,
                    UniformName::Fcoef,
                ],
            )),
            "local_tangents" => Some(Material::new(
                identifier,
                "relay_locals.vert",
                "tangents.frag",
                &[
                    UniformName::WVTrans,
                    UniformName::WVInvTranspTrans,
                    UniformName::WVPTrans,
                    UniformName::Fcoef,
                ],
            )),
            "world_tangents" => Some(Material::new(
                identifier,
                "relay_all.vert",
                "tangents.frag",
                &[
                    UniformName::WVTrans,
                    UniformName::WVInvTranspTrans,
                    UniformName::WVPTrans,
                    UniformName::Fcoef,
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
                    UniformName::Fcoef,
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
                    UniformName::Fcoef,
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
                    UniformName::Fcoef,
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
                    UniformName::Fcoef,
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
                    UniformName::ExposureFactor,
                    UniformName::Fcoef,
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
        let tex = load_texture_from_image_bytes(identifier, bytes);
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
    }

    pub fn get_or_request_texture(
        &mut self,
        identifier: &str,
        is_cubemap: bool,
    ) -> Option<Rc<RefCell<Texture>>> {
        if let Some(tex) = self.textures.get(identifier) {
            return Some(tex.clone());
        }

        // log::info!(
        //     "Failed to find texture {} in texture map {:#?}",
        //     full_path,
        //     self.textures
        // );

        // We don't have this one, put out a request for this asset and just return
        // the default texture instead
        let internal_full_path = identifier.to_owned();
        if is_cubemap {
            spawn_local(async move {
                let urls = [
                    &(internal_full_path.to_owned() + "/Right.jpg"),
                    &(internal_full_path.to_owned() + "/Left.jpg"),
                    &(internal_full_path.to_owned() + "/Top.jpg"),
                    &(internal_full_path.to_owned() + "/Bottom.jpg"),
                    &(internal_full_path.to_owned() + "/Front.jpg"),
                    &(internal_full_path.to_owned() + "/Back.jpg"),
                ];

                let mut vecs: Vec<Vec<u8>> = join_all(urls.iter().map(|url| request_bytes(url)))
                    .await
                    .into_iter()
                    .collect::<Result<Vec<Vec<u8>>, JsValue>>()
                    .unwrap();

                ENGINE.with(|e| {
                    let mut ref_mut = e.borrow_mut();
                    let e = ref_mut.as_mut().unwrap();

                    for it in urls.iter().zip(vecs.iter_mut()) {
                        let (url, mut vec) = it;
                        e.receive_bytes(&url, "cubemap_face", &mut vec);
                    }
                });
            });
        } else {
            spawn_local(async move {
                let mut vec = request_bytes(&internal_full_path).await.unwrap();

                ENGINE.with(|e| {
                    let mut ref_mut = e.borrow_mut();
                    let e = ref_mut.as_mut().unwrap();

                    e.receive_bytes(&internal_full_path, "texture", &mut vec);
                });
            });
        }

        // Create the default texture on-demand.
        // Its much better to have this thing than to just end up with black, especially since we
        // have no IBL which means that 0 roughness instantly makes everything black
        if let None = self.default_texture {
            let buf: [u8; 16] = [
                128, 128, 255, 255, 128, 255, 128, 255, 255, 128, 128, 255, 255, 128, 255, 255,
            ];

            let default_texture = load_texture_from_bytes(
                "default_texture",
                2,
                2,
                4,
                GL::RGBA,
                &buf,
                None,
                None,
                None,
                None,
            )
            .unwrap();

            let default_full_path = "public/textures/".to_owned() + &default_texture.borrow().name;

            self.default_texture = Some(Rc::<RefCell<Texture>>::downgrade(&default_texture));

            self.textures.insert(default_full_path, default_texture);
        }

        let tex = Rc::new(RefCell::new(
            self.default_texture
                .as_ref()
                .unwrap()
                .upgrade()
                .unwrap()
                .borrow()
                .clone(),
        ));
        tex.borrow_mut().name = identifier.to_owned();

        self.textures.insert(identifier.to_owned(), tex.clone());

        return Some(tex);
    }
}
