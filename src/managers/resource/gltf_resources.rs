use super::{
    intermediate_mesh::IntermediateMesh,
    intermediate_mesh::{intermediate_to_mesh, IntermediatePrimitive},
    material::UniformName,
    texture::TextureUnit,
};
use crate::GLCTX;
use crate::{
    managers::{
        resource::{
            collider::{AxisAlignedBoxCollider, MeshCollider},
            material::{Material, UniformValue},
            mesh::Mesh,
            texture::Texture,
        },
        ResourceManager,
    },
    utils::gl::GL,
};
use gltf::{
    image::Format,
    mesh::util::{ReadColors, ReadIndices, ReadTexCoords},
};
use na::{Point3, Vector2, Vector3, Vector4};
use std::{cell::RefCell, rc::Rc};
use web_sys::WebGl2RenderingContext;

pub trait GltfResource {
    fn get_identifier(&self, identifier: &str) -> String;
}

impl GltfResource for gltf::Mesh<'_> {
    fn get_identifier(&self, file_identifier: &str) -> String {
        let mut result = file_identifier.to_owned() + "_mesh_" + &self.index().to_string();
        if let Some(name) = self.name() {
            result += &("_".to_owned() + name);
        }

        return result;
    }
}

impl GltfResource for gltf::Material<'_> {
    fn get_identifier(&self, file_identifier: &str) -> String {
        let mut result =
            file_identifier.to_owned() + "_material_" + &self.index().unwrap().to_string();
        if let Some(name) = self.name() {
            result += &("_".to_owned() + name);
        }

        return result;
    }
}

impl GltfResource for gltf::Texture<'_> {
    fn get_identifier(&self, file_identifier: &str) -> String {
        let mut result = file_identifier.to_owned() + "_texture_" + &self.index().to_string();
        if let Some(name) = self.name() {
            result += &("_".to_owned() + name);
        }

        return result;
    }
}

impl GltfResource for gltf::Node<'_> {
    fn get_identifier(&self, scene_identifier: &str) -> String {
        let mut result = scene_identifier.to_owned() + "_node_" + &self.index().to_string();
        if let Some(name) = self.name() {
            result += &("_".to_owned() + name);
        }

        return result;
    }
}

impl GltfResource for gltf::Scene<'_> {
    fn get_identifier(&self, file_identifier: &str) -> String {
        let mut result = file_identifier.to_owned() + "_scene_" + &self.index().to_string();
        if let Some(name) = self.name() {
            result += &("_".to_owned() + name);
        }

        return result;
    }
}

impl ResourceManager {
    fn load_material_from_gltf(
        &mut self,
        file_identifier: &str,
        material: &gltf::Material,
    ) -> Result<Rc<RefCell<Material>>, String> {
        let identifier = material.get_identifier(file_identifier);
        log::info!("\tLoading gltf material '{}'", identifier);

        let mat = self.instantiate_material("gltf_metal_rough").unwrap();
        let mut mat_mut = mat.borrow_mut();

        let pbr = material.pbr_metallic_roughness();

        // Base color texture
        if let Some(gltf_tex) = pbr.base_color_texture() {
            let tex_identifier = gltf_tex.texture().get_identifier(file_identifier);
            if let Some(tex) = self.get_texture(&tex_identifier) {
                log::info!("\t\tBaseColor texture: '{}'", tex_identifier);
                mat_mut.set_texture(TextureUnit::BaseColor, Some(tex));
            } else {
                log::warn!(
                    "Failed to find texture '{}' referenced by material '{}'",
                    tex_identifier,
                    identifier
                );
            }
        }

        // Base color factor
        let factor = pbr.base_color_factor();
        mat_mut.set_uniform_value(UniformName::BaseColorFactor, UniformValue::Vec4(factor));
        log::info!("\t\tBaseColor factor: '{:?}'", factor);

        // Metallic-roughness texture
        if let Some(gltf_tex) = pbr.metallic_roughness_texture() {
            let tex_identifier = gltf_tex.texture().get_identifier(file_identifier);
            if let Some(tex) = self.get_texture(&tex_identifier) {
                log::info!("\t\tMetallicRoughness texture: '{}'", tex_identifier);
                mat_mut.set_texture(TextureUnit::MetallicRoughness, Some(tex));
            } else {
                log::warn!(
                    "Failed to find texture '{}' referenced by material '{}'",
                    tex_identifier,
                    identifier
                );
            }
        }

        // Metallic factor
        let factor = pbr.metallic_factor();
        mat_mut.set_uniform_value(UniformName::MetallicFactor, UniformValue::Float(factor));
        log::info!("\t\tMetallic factor: '{:?}'", factor);

        // Roughness factor
        let factor = pbr.roughness_factor();
        mat_mut.set_uniform_value(UniformName::RoughnessFactor, UniformValue::Float(factor));
        log::info!("\t\tRoughness factor: '{:?}'", factor);

        // Normal texture
        if let Some(gltf_tex) = material.normal_texture() {
            let tex_identifier = gltf_tex.texture().get_identifier(file_identifier);
            if let Some(tex) = self.get_texture(&tex_identifier) {
                log::info!("\t\tNormal texture: '{}'", tex_identifier);
                mat_mut.set_texture(TextureUnit::Normal, Some(tex));
            } else {
                log::warn!(
                    "Failed to find texture '{}' referenced by material '{}'",
                    tex_identifier,
                    identifier
                );
            }
        }

        // Occlusion texture
        if let Some(gltf_tex) = material.occlusion_texture() {
            let tex_identifier = gltf_tex.texture().get_identifier(file_identifier);
            if let Some(tex) = self.get_texture(&tex_identifier) {
                log::info!("\t\tOcclusion texture: '{}'", tex_identifier);
                mat_mut.set_texture(TextureUnit::Occlusion, Some(tex));
            } else {
                log::warn!(
                    "Failed to find texture '{}' referenced by material '{}'",
                    tex_identifier,
                    identifier
                );
            }
        }

        // Emissive texture
        if let Some(gltf_tex) = material.emissive_texture() {
            let tex_identifier = gltf_tex.texture().get_identifier(file_identifier);
            if let Some(tex) = self.get_texture(&tex_identifier) {
                log::info!("\t\tEmissive texture: '{}'", tex_identifier);
                mat_mut.set_texture(TextureUnit::Emissive, Some(tex));
            } else {
                log::warn!(
                    "Failed to find texture '{}' referenced by material '{}'",
                    tex_identifier,
                    identifier
                );
            }
        }

        // Emissive factor
        let factor = material.emissive_factor();
        mat_mut.set_uniform_value(UniformName::EmissiveFactor, UniformValue::Vec3(factor));
        log::info!("\t\tEmissive factor: '{:?}'", factor);

        return Ok(mat.clone());
    }

    /**
     * Also returns a vec of whatever we parsed for each material index, because we can't
     * find the exact material instance that we want via identifier alone, as it will have
     * a trailing suffix
     */
    pub fn load_materials_from_gltf(
        &mut self,
        file_identifier: &str,
        materials: gltf::iter::Materials,
    ) -> Vec<Option<Rc<RefCell<Material>>>> {
        log::info!(
            "Loading {} materials from gltf file {}",
            materials.len(),
            file_identifier
        );

        let mut result = Vec::new();
        result.resize(materials.len(), None);

        for (index, material) in materials.enumerate() {
            match self.load_material_from_gltf(file_identifier, &material) {
                Ok(new_mat) => {
                    self.materials
                        .insert(new_mat.borrow().name.clone(), new_mat.clone());
                    result[index] = Some(new_mat.clone());
                }
                Err(msg) => {
                    log::error!("Failed to load gltf texture: {}", msg);
                }
            }
        }

        return result;
    }

    fn load_mesh_from_gltf(
        &self,
        file_identifier: &str,
        mesh: &gltf::Mesh,
        buffers: &Vec<gltf::buffer::Data>,
        mat_index_to_parsed: &Vec<Option<Rc<RefCell<Material>>>>,
        ctx: &WebGl2RenderingContext,
    ) -> Result<Rc<RefCell<Mesh>>, String> {
        let identifier = mesh.get_identifier(file_identifier);

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
            let mut positions_vec: Vec<Vector3<f32>> = Vec::new();
            if let Some(positions) = reader.read_positions() {
                positions_vec = positions
                    .map(|arr| Vector3::new(arr[0], -arr[2], arr[1])) // Y-up right-handed to Z-up right-handed
                    .collect();
            }

            // Normals
            let mut normals_vec: Vec<Vector3<f32>> = Vec::new();
            if let Some(normals) = reader.read_normals() {
                normals_vec = normals
                    .map(|arr| Vector3::new(arr[0], -arr[2], arr[1])) // Y-up right-handed to Z-up right-handed
                    .collect();
            }

            // Normals
            let mut tangents_vec: Vec<Vector3<f32>> = Vec::new();
            if let Some(tangents) = reader.read_tangents() {
                tangents_vec = tangents
                    .map(|arr| Vector3::new(arr[0], -arr[2], arr[1])) // Y-up right-handed to Z-up right-handed
                    .collect();
            }

            // Colors
            let mut colors_vec: Vec<Vector4<f32>> = Vec::new();
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
            let mut uv0_vec: Vec<Vector2<f32>> = Vec::new();
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
            let mut uv1_vec: Vec<Vector2<f32>> = Vec::new();
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

            // Material
            let mut mat_instance: Rc<RefCell<Material>> =
                self.get_material("gltf_metal_rough").unwrap();
            if let Some(mat_index) = prim.material().index() {
                if let Some(mat) = &mat_index_to_parsed[mat_index] {
                    mat_instance = mat.clone();
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
                mat_instance.borrow().name,
            );

            inter_prims.push(IntermediatePrimitive {
                name: prim_name,
                indices: indices_vec,
                positions: positions_vec,
                normals: normals_vec,
                tangents: tangents_vec,
                colors: colors_vec,
                uv0: uv0_vec,
                uv1: uv1_vec,
                mode: prim.mode().as_gl_enum(),
                mat: Some(mat_instance),
                collider: None,
            });
        }

        // AABB collider as early out
        let mut mins: Point3<f32> =
            Point3::new(std::f32::INFINITY, std::f32::INFINITY, std::f32::INFINITY);
        let mut maxes: Point3<f32> = Point3::new(
            -std::f32::INFINITY,
            -std::f32::INFINITY,
            -std::f32::INFINITY,
        );
        for prim in &inter_prims {
            for pos in &prim.positions {
                mins.x = mins.x.min(pos.x);
                mins.y = mins.y.min(pos.y);
                mins.z = mins.z.min(pos.z);

                maxes.x = maxes.x.max(pos.x);
                maxes.y = maxes.y.max(pos.y);
                maxes.z = maxes.z.max(pos.z);
            }
        }

        // We'll need to keep some of the mesh data on the CPU for raycasting though
        let mut intermediate = IntermediateMesh {
            name: identifier,
            primitives: inter_prims,
        };

        let result = intermediate_to_mesh(&intermediate);

        let mesh_collider = Box::new(MeshCollider {
            mesh: Rc::downgrade(&result),
            additional_outer_collider: Some(Box::new(AxisAlignedBoxCollider { mins, maxes })),
        });

        {
            let mut mut_result = result.borrow_mut();

            mut_result.collider = Some(mesh_collider);

            // Keep indices and positions on the mesh primitive so we can raycast against it
            for (prim, inter_prim) in mut_result
                .primitives
                .iter_mut()
                .zip(intermediate.primitives.drain(..))
            {
                prim.source_data = Some(IntermediatePrimitive {
                    name: inter_prim.name,
                    indices: inter_prim.indices,
                    positions: inter_prim.positions,
                    normals: Vec::new(),
                    tangents: Vec::new(),
                    colors: Vec::new(),
                    uv0: Vec::new(),
                    uv1: Vec::new(),
                    mode: inter_prim.mode,
                    mat: None,
                    collider: None,
                });
            }
        }

        return Ok(result);
    }

    pub fn load_meshes_from_gltf(
        &mut self,
        file_identifier: &str,
        meshes: gltf::iter::Meshes,
        buffers: &Vec<gltf::buffer::Data>,
        mat_index_to_parsed: &Vec<Option<Rc<RefCell<Material>>>>,
    ) {
        log::info!(
            "Loading {} meshes from gltf file {}",
            meshes.len(),
            file_identifier
        );

        GLCTX.with(|ctx| {
            let ref_mut = ctx.borrow_mut();
            let ctx = ref_mut.as_ref().unwrap();

            for mesh in meshes {
                match self.load_mesh_from_gltf(
                    file_identifier,
                    &mesh,
                    &buffers,
                    mat_index_to_parsed,
                    &ctx,
                ) {
                    Ok(new_mesh) => {
                        let name = new_mesh.borrow().name.clone();
                        self.meshes.insert(name, new_mesh);
                    }
                    Err(msg) => {
                        log::error!("Failed to load gltf mesh: {}", msg);
                    }
                }
            }
        });
    }

    fn load_texture_from_gltf(
        file_identifier: &str,
        texture: &gltf::Texture,
        image_data: &gltf::image::Data,
        ctx: &WebGl2RenderingContext,
    ) -> Result<Rc<Texture>, String> {
        let identifier = texture.get_identifier(file_identifier);
        let width = image_data.width;
        let height = image_data.height;
        let (gl_format, num_channels) = match image_data.format {
            Format::R8 => (GL::ALPHA, 1),
            Format::R8G8 => (GL::LUMINANCE_ALPHA, 2),
            Format::R8G8B8 => (GL::RGB, 3),
            Format::R8G8B8A8 => (GL::RGBA, 4),
            Format::B8G8R8 => (GL::RGB, 3), // TODO: Switch to WebGL2
            Format::B8G8R8A8 => (GL::RGBA, 4),
            other => return Err(format!("Unsupported gltf texture format '{:?}'", other)),
        };

        log::info!(
            "\tLoading texture '{}': Width: {}, Height: {}, Format: {}, Num channels: {}",
            identifier,
            width,
            height,
            gl_format,
            num_channels
        );

        let gl_handle = ctx.create_texture().unwrap();
        ctx.active_texture(GL::TEXTURE0);
        ctx.bind_texture(GL::TEXTURE_2D, Some(&gl_handle));

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
            GL::UNSIGNED_BYTE,
            Some(&image_data.pixels),
        )
        .unwrap();

        ctx.bind_texture(GL::TEXTURE_2D, None);

        return Ok(Rc::new(Texture {
            name: identifier,
            width,
            height,
            gl_format,
            num_channels,
            gl_handle: Some(gl_handle),
        }));
    }

    pub fn load_textures_from_gltf(
        &mut self,
        file_identifier: &str,
        textures: gltf::iter::Textures,
        images: &Vec<gltf::image::Data>,
    ) {
        log::info!(
            "Loading {} textures from gltf file {}",
            textures.len(),
            file_identifier
        );

        GLCTX.with(|ctx| {
            let ref_mut = ctx.borrow_mut();
            let ctx = ref_mut.as_ref().unwrap();

            for texture in textures {
                match ResourceManager::load_texture_from_gltf(
                    file_identifier,
                    &texture,
                    &images[texture.source().index()],
                    ctx,
                ) {
                    Ok(new_tex) => {
                        self.textures.insert(new_tex.name.clone(), new_tex);
                    }
                    Err(msg) => {
                        log::error!("Failed to load gltf texture: {}", msg);
                    }
                }
            }
        });
    }
}
