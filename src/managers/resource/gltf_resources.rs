use std::{io::Cursor, rc::Rc};

use cgmath::{Vector2, Vector3, Vector4};
use gltf::mesh::util::{ReadColors, ReadIndices, ReadTexCoords};
use image::{io::Reader, DynamicImage, ImageError, ImageFormat};
use web_sys::WebGlRenderingContext;
use web_sys::WebGlRenderingContext as GL;

use super::{
    intermediate_mesh::IntermediateMesh,
    intermediate_mesh::{intermediate_to_mesh, IntermediatePrimitive},
    Material, Mesh, ResourceManager, Texture,
};

pub trait GltfResource {
    fn get_identifier(&self, scene_identifier: &str) -> String;
}

impl GltfResource for gltf::Mesh<'_> {
    fn get_identifier(&self, scene_identifier: &str) -> String {
        return scene_identifier.to_owned() + "_mesh_" + &self.index().to_string();
    }
}

impl GltfResource for gltf::Material<'_> {
    fn get_identifier(&self, scene_identifier: &str) -> String {
        return scene_identifier.to_owned() + "_mesh_" + &self.index().unwrap().to_string();
    }
}

impl GltfResource for gltf::Texture<'_> {
    fn get_identifier(&self, scene_identifier: &str) -> String {
        return scene_identifier.to_owned() + "_mesh_" + &self.index().to_string();
    }
}

impl GltfResource for gltf::Node<'_> {
    fn get_identifier(&self, scene_identifier: &str) -> String {
        return scene_identifier.to_owned() + "_node_" + &self.index().to_string();
    }
}

impl GltfResource for gltf::Scene<'_> {
    fn get_identifier(&self, scene_identifier: &str) -> String {
        return scene_identifier.to_owned() + "_scene_" + &self.index().to_string();
    }
}

impl ResourceManager {
    pub fn load_materials_from_gltf(
        &mut self,
        file_identifier: &str,
        materials: gltf::iter::Materials,
    ) {
        log::info!(
            "Loading {} materials from gltf file {}",
            materials.len(),
            file_identifier
        );
    }

    fn load_mesh_from_gltf(
        scene_identifier: &str,
        mesh: &gltf::Mesh,
        buffers: &Vec<gltf::buffer::Data>,
        default_material: &Option<Rc<Material>>,
        ctx: &WebGlRenderingContext,
    ) -> Result<Rc<Mesh>, String> {
        let identifier = mesh.get_identifier(scene_identifier);

        log::info!(
            "\tMesh {}, num_prims: {}",
            identifier,
            mesh.primitives().len()
        );

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
                            identifier
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
                "\t\tPrim {}, Ind: {}, Pos: {}, Nor: {}, Col: {}, UV0: {}, UV1: {}, mode: {}, mat: {}",
                prim_name,
                indices_vec.len(),
                positions_vec.len(),
                normals_vec.len(),
                colors_vec.len(),
                uv0_vec.len(),
                uv1_vec.len(),
                prim.mode().as_gl_enum(),
                default_material.as_ref().unwrap().get_name(),
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

        return Ok(intermediate_to_mesh(
            IntermediateMesh {
                name: identifier,
                primitives: inter_prims,
            },
            ctx,
        ));
    }

    pub fn load_meshes_from_gltf(
        &mut self,
        file_identifier: &str,
        meshes: gltf::iter::Meshes,
        buffers: &Vec<gltf::buffer::Data>,
    ) {
        let default_mat = self.get_or_create_material("phong");

        log::info!(
            "Loading {} meshes from gltf file {}",
            meshes.len(),
            file_identifier
        );

        for mesh in meshes {
            match ResourceManager::load_mesh_from_gltf(
                file_identifier,
                &mesh,
                &buffers,
                &default_mat,
                &self.gl,
            ) {
                Ok(new_mesh) => {
                    self.meshes.insert(new_mesh.name.clone(), new_mesh);
                }
                Err(msg) => {
                    log::error!("Failed to load gltf mesh: {}", msg);
                }
            }
        }
    }

    fn load_texture_from_bytes(
        bytes: &[u8],
        image_format: ImageFormat,
        ctx: &WebGlRenderingContext,
    ) -> Result<Texture, String> {
        let mut reader = Reader::new(Cursor::new(bytes));
        reader.set_format(image_format);
        let decoded = reader.decode();
        
        if let Err(error) = decoded {
            return Err(std::format!("Error loading texture: {}", error));
        }
        let decoded = decoded.unwrap();

        let mut width: u32 = 0;
        let mut height: u32 = 0;
        let mut format: u32 = 0;
        let mut buf: Option<&[u8]> = None;
        let converted_bgr;
        let converted_bgra;

        match decoded {
            // R
            DynamicImage::ImageLuma8(ref img) => {
                width = img.width();
                height = img.height();
                format = GL::ALPHA;
                buf = Some(img.as_raw());
            }
            // RG
            DynamicImage::ImageLumaA8(ref img) => {
                width = img.width();
                height = img.height();
                format = GL::LUMINANCE_ALPHA;
                buf = Some(img.as_raw());
            }
            // RGB
            DynamicImage::ImageRgb8(ref img) => {
                width = img.width();
                height = img.height();
                format = GL::RGB;
                buf = Some(img.as_raw());
            }
            DynamicImage::ImageBgr8(_) => {
                converted_bgr = decoded.to_rgb8();
                width = converted_bgr.width();
                height = converted_bgr.height();
                format = GL::RGB;
                buf = Some(converted_bgr.as_raw());
            }
            // RGBA
            DynamicImage::ImageRgba8(ref img) => {
                width = img.width();
                height = img.height();
                format = GL::RGBA;
                buf = Some(img.as_raw());
            }
            DynamicImage::ImageBgra8(_) => {
                converted_bgra = decoded.to_rgba8();
                width = converted_bgra.width();
                height = converted_bgra.height();
                format = GL::RGBA;
                buf = Some(converted_bgra.as_raw());
            }
            _ => {}
        };

        if buf.is_none() {
            return Err(format!("Failed to decode {:?}", image_format));
        }

        let mut tex = Texture::new();
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
        tex.gl_handle = Some(gl_tex);

        return Ok(tex);
    }

    pub fn load_textures_from_gltf(
        &mut self,
        file_identifier: &str,
        textures: gltf::iter::Textures,
    ) {
        log::info!(
            "Loading {} textures from gltf file {}",
            textures.len(),
            file_identifier
        );

        for texture in textures {
            log::info!(
                "Texture {}: Name: {}",
                texture.index(),
                texture.name().unwrap_or("")
            );
            let img = texture.source();
            log::info!(
                "\tImage {}: Name: {}",
                img.index(),
                img.name().unwrap_or("")
            );
            match img.source() {
                gltf::image::Source::View { view, mime_type } => {
                    log::info!("\t\tView: {:#?}, Mime type: {}", view, mime_type);
                }
                gltf::image::Source::Uri { uri, mime_type } => {
                    log::info!(
                        "\t\tUri: {:#?}, Mime type: {}",
                        uri,
                        mime_type.unwrap_or("")
                    );
                }
            }
        }
    }
}
