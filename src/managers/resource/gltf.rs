use crate::managers::resource::collider::AxisAlignedBoxCollider;
use crate::managers::resource::intermediate_mesh::{
    intermediate_to_mesh, IntermediateMesh, IntermediatePrimitive,
};
use crate::managers::resource::material::{Material, UniformName, UniformValue, ShaderDefine};
use crate::managers::resource::mesh::Mesh;
use crate::managers::resource::texture::{Texture, TextureUnit};
use crate::managers::ResourceManager;
use crate::utils::gl::GL;
use crate::utils::transform::Transform;
use crate::GLCTX;
use gltf::image::Format;
use gltf::mesh::util::{ReadColors, ReadIndices, ReadTexCoords};
use na::*;
use std::{cell::RefCell, f32::INFINITY, rc::Rc};
use web_sys::WebGl2RenderingContext;

pub trait GltfResource {
    fn get_identifier(&self, identifier: &str) -> String;
}

impl GltfResource for gltf::Mesh<'_> {
    fn get_identifier(&self, file_identifier: &str) -> String {
        let mut result = file_identifier.to_owned() + "/mesh_" + &self.index().to_string();
        if let Some(name) = self.name() {
            result += &("/".to_owned() + name);
        }

        return result;
    }
}

impl GltfResource for gltf::Material<'_> {
    fn get_identifier(&self, file_identifier: &str) -> String {
        let mut result =
            file_identifier.to_owned() + "/material_" + &self.index().unwrap().to_string();
        if let Some(name) = self.name() {
            result += &("/".to_owned() + name);
        }

        return result;
    }
}

impl GltfResource for gltf::Texture<'_> {
    fn get_identifier(&self, file_identifier: &str) -> String {
        let mut result = file_identifier.to_owned() + "/texture_" + &self.index().to_string();
        if let Some(name) = self.name() {
            result += &("/".to_owned() + name);
        }

        return result;
    }
}

impl GltfResource for gltf::Node<'_> {
    fn get_identifier(&self, scene_identifier: &str) -> String {
        let mut result = scene_identifier.to_owned() + "/node_" + &self.index().to_string();
        if let Some(name) = self.name() {
            result += &("/".to_owned() + name);
        }

        return result;
    }
}

impl GltfResource for gltf::Scene<'_> {
    fn get_identifier(&self, file_identifier: &str) -> String {
        let mut result = file_identifier.to_owned() + "/scene_" + &self.index().to_string();
        if let Some(name) = self.name() {
            result += &("/".to_owned() + name);
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

        let mat = self
            .instantiate_material("gltf_metal_rough", &identifier)
            .unwrap();
        let mut mat_mut = mat.borrow_mut();

        let pbr = material.pbr_metallic_roughness();

        // Base color texture
        if let Some(gltf_tex) = pbr.base_color_texture() {
            let tex_identifier = gltf_tex.texture().get_identifier(file_identifier);
            if let Some(tex) = self.get_or_request_texture(&tex_identifier, false) {
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
            if let Some(tex) = self.get_or_request_texture(&tex_identifier, false) {
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
            if let Some(tex) = self.get_or_request_texture(&tex_identifier, false) {
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
            if let Some(tex) = self.get_or_request_texture(&tex_identifier, false) {
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
            if let Some(tex) = self.get_or_request_texture(&tex_identifier, false) {
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

    /// Also returns a vec of whatever we parsed for each material index, because we can't
    /// find the exact material instance that we want via identifier alone, as it will have
    /// a trailing suffix
    fn load_materials_from_gltf(
        &mut self,
        file_identifier: &str,
        materials: gltf::iter::Materials,
    ) -> Vec<Option<Rc<RefCell<Material>>>> {
        log::info!(
            "Loading {} materials from gltf file '{}'",
            materials.len(),
            file_identifier
        );

        let mut result = Vec::new();
        result.resize(materials.len(), None);

        for (index, material) in materials.enumerate() {
            match self.load_material_from_gltf(file_identifier, &material) {
                Ok(new_mat) => {
                    result[index] = Some(new_mat);
                }
                Err(msg) => {
                    log::error!("Failed to load gltf material: {}", msg);
                }
            }
        }

        return result;
    }

    fn load_mesh_from_gltf(
        &mut self,
        file_identifier: &str,
        mesh: &gltf::Mesh,
        buffers: &Vec<gltf::buffer::Data>,
        parsed_mats: &Vec<Option<Rc<RefCell<Material>>>>,
    ) -> Result<IntermediateMesh, String> {
        let identifier = mesh.get_identifier(file_identifier);

        log::info!(
            "\tMesh '{}', num_prims: {}",
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

            // Tangents
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
            let mut mat_instance: Option<Rc<RefCell<Material>>> = None;
            if let Some(mat_index) = prim.material().index() {
                if let Some(mat) = &parsed_mats[mat_index] {
                    mat_instance = Some(mat.clone());
                }
            }

            log::info!(
                "\t\tPrim {}, Ind: {}, Pos: {}, Nor: {}, Tan: {}, Col: {}, UV0: {}, UV1: {}, mode: {}, mat: {}",
                prim_name,
                indices_vec.len(),
                positions_vec.len(),
                normals_vec.len(),
                tangents_vec.len(),
                colors_vec.len(),
                uv0_vec.len(),
                uv1_vec.len(),
                prim.mode().as_gl_enum(),
                mat_instance.as_ref().and_then(|m| Some(m.borrow().name.clone())).unwrap_or(String::from("none")),
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
                mat: mat_instance,
                collider: None,
            });
        }

        // let mesh_collider = Box::new(MeshCollider {
        //     mesh: Rc::downgrade(&result),
        //     additional_outer_collider: Some(Box::new(AxisAlignedBoxCollider { mins, maxes })),
        // });

        // {
        //     let mut mut_result = result.borrow_mut();

        //     mut_result.collider = Some(mesh_collider);

        //     // Keep indices and positions on the mesh primitive so we can raycast against it
        //     for (prim, inter_prim) in mut_result
        //         .primitives
        //         .iter_mut()
        //         .zip(intermediate.primitives.drain(..))
        //     {
        //         prim.source_data = Some(IntermediatePrimitive {
        //             name: inter_prim.name,
        //             indices: inter_prim.indices,
        //             positions: inter_prim.positions,
        //             normals: Vec::new(),
        //             tangents: Vec::new(),
        //             colors: Vec::new(),
        //             uv0: Vec::new(),
        //             uv1: Vec::new(),
        //             mode: inter_prim.mode,
        //             mat: None,
        //             collider: None,
        //         });
        //     }
        // }

        return Ok(IntermediateMesh {
            name: identifier,
            primitives: inter_prims,
        });
    }

    fn load_meshes_from_gltf(
        &mut self,
        file_identifier: &str,
        meshes: gltf::iter::Meshes,
        buffers: &Vec<gltf::buffer::Data>,
        parsed_mats: &Vec<Option<Rc<RefCell<Material>>>>,
    ) -> Vec<Option<IntermediateMesh>> {
        log::info!(
            "Loading {} meshes from gltf file '{}'",
            meshes.len(),
            file_identifier
        );

        return meshes
            .map(|m| {
                self.load_mesh_from_gltf(file_identifier, &m, &buffers, parsed_mats)
                    .ok()
            })
            .collect::<Vec<Option<IntermediateMesh>>>();
    }

    fn load_texture_from_gltf(
        file_identifier: &str,
        texture: &gltf::Texture,
        image_data: &gltf::image::Data,
        ctx: &WebGl2RenderingContext,
    ) -> Result<Rc<RefCell<Texture>>, String> {
        let identifier = texture.get_identifier(file_identifier);
        let width = image_data.width;
        let height = image_data.height;
        let (gl_format, num_channels) = match image_data.format {
            Format::R8 => (GL::ALPHA, 1),
            Format::R8G8 => (GL::LUMINANCE_ALPHA, 2),
            Format::R8G8B8 => (GL::RGB, 3),
            Format::R8G8B8A8 => (GL::RGBA, 4),
            Format::B8G8R8 => (GL::RGB, 3),
            Format::B8G8R8A8 => (GL::RGBA, 4),
            other => return Err(format!("Unsupported gltf texture format '{:?}'", other)),
        };

        let gl_handle = ctx.create_texture().unwrap();
        ctx.active_texture(GL::TEXTURE0);
        ctx.bind_texture(GL::TEXTURE_2D, Some(&gl_handle));

        let sampler = texture.sampler();
        let mag_filter = match sampler.mag_filter() {
            Some(gltf::texture::MagFilter::Nearest) => GL::NEAREST,
            Some(gltf::texture::MagFilter::Linear) => GL::LINEAR,
            None => GL::LINEAR,
        };
        // TODO: Enable mipmap texture filtering. Needs special handling because not all texture formats
        // support automatic generation of mipmaps, so I'd need to check for a proper extension in some cases
        let min_filter = match sampler.min_filter() {
            Some(gltf::texture::MinFilter::Nearest) => GL::NEAREST,
            Some(_) => GL::LINEAR,
            // Some(gltf::texture::MinFilter::Linear) => GL::LINEAR,
            // Some(gltf::texture::MinFilter::NearestMipmapNearest) => GL::NEAREST_MIPMAP_NEAREST,
            // Some(gltf::texture::MinFilter::LinearMipmapNearest) => GL::LINEAR_MIPMAP_NEAREST,
            // Some(gltf::texture::MinFilter::NearestMipmapLinear) => GL::NEAREST_MIPMAP_LINEAR,
            // Some(gltf::texture::MinFilter::LinearMipmapLinear) => GL::LINEAR_MIPMAP_LINEAR,
            None => GL::LINEAR,
        };
        let wrap_s = match sampler.wrap_s() {
            gltf::texture::WrappingMode::ClampToEdge => GL::CLAMP_TO_EDGE,
            gltf::texture::WrappingMode::MirroredRepeat => GL::MIRRORED_REPEAT,
            gltf::texture::WrappingMode::Repeat => GL::REPEAT,
        };
        let wrap_t = match sampler.wrap_t() {
            gltf::texture::WrappingMode::ClampToEdge => GL::CLAMP_TO_EDGE,
            gltf::texture::WrappingMode::MirroredRepeat => GL::MIRRORED_REPEAT,
            gltf::texture::WrappingMode::Repeat => GL::REPEAT,
        };

        log::info!(
            "\tLoading texture '{}': Width: {}, Height: {}, Format: {}, Num channels: {}, wrap_s: {:?}, wrap_t: {:?}, mag_filter: {:?}, min_filter: {:?}",
            identifier,
            width,
            height,
            gl_format,
            num_channels,
            sampler.wrap_s(),
            sampler.wrap_t(),
            sampler.mag_filter(),
            sampler.min_filter()
        );

        // ctx.generate_mipmap(GL::TEXTURE_2D);

        ctx.tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_MAG_FILTER, mag_filter as i32);
        ctx.tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_MIN_FILTER, min_filter as i32);
        ctx.tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_WRAP_S, wrap_s as i32);
        ctx.tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_WRAP_T, wrap_t as i32);

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

        return Ok(Rc::new(RefCell::new(Texture {
            name: identifier,
            width,
            height,
            gl_format,
            num_channels,
            gl_handle: Some(gl_handle),
            is_cubemap: false,
        })));
    }

    fn load_textures_from_gltf(
        &mut self,
        file_identifier: &str,
        textures: gltf::iter::Textures,
        images: &Vec<gltf::image::Data>,
    ) {
        log::info!(
            "Loading {} textures from gltf file '{}'",
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
                        let name = &new_tex.borrow().name;

                        if let Some(existing_tex) = self.textures.get(name) {
                            existing_tex.swap(&new_tex);

                            log::info!(
                                "Mutating existing texture resource '{}' with new data from '{}'",
                                name,
                                file_identifier
                            );
                        } else if let Some(_) =
                            self.textures.insert(name.to_owned(), new_tex.clone())
                        {
                            log::info!("Changing tracked texture resource for name '{}'", name);
                        }
                    }
                    Err(msg) => {
                        log::error!("Failed to load gltf texture: {}", msg);
                    }
                }
            }
        });
    }

    fn load_gltf_node(
        node: &gltf::Node,
        parent_trans: &Transform<f32>,
        combined_mesh: &mut IntermediateMesh,
        meshes: &Vec<Option<IntermediateMesh>>,
    ) {
        // TODO: I think it may be a problem that I'm still using my own concatenation functions to
        // concat the GLTF transform? Maybe I should let it concatenate itself and only convert the final one

        // Propagate parent transform down to all leaves
        let (pos, quat, scale) = node.transform().decomposed();
        let trans: Transform<f32> = Transform {
            trans: Vector3::new(pos[0], pos[1], pos[2]),
            rot: UnitQuaternion::new_normalize(Quaternion::new(quat[3], quat[0], quat[1], quat[2])),
            scale: Vector3::new(scale[0], scale[1], scale[2]),
        };
        let global_trans = parent_trans.concat_clone(&trans);

        if let Some(template_mesh) = node
            .mesh()
            .and_then(|m| Some(m.index()))
            .and_then(|i| meshes.get(i).unwrap().as_ref())
        {
            // Convert transform to our coordinate system only once, after fully concatenated
            let trans_to_apply: Transform<f32> = Transform {
                trans: Vector3::new(
                    global_trans.trans.x,
                    -global_trans.trans.z,
                    global_trans.trans.y,
                ),
                rot: UnitQuaternion::new_normalize(Quaternion::new(
                    global_trans.rot.w,
                    global_trans.rot.i,
                    -global_trans.rot.k,
                    global_trans.rot.j,
                )),
                scale: Vector3::new(
                    global_trans.scale[0],
                    global_trans.scale[2],
                    global_trans.scale[1],
                ),
            };

            let mat = trans_to_apply.to_matrix4();

            let mut mat_no_trans = mat.clone();
            mat_no_trans[(0, 3)] = 0.0;
            mat_no_trans[(1, 3)] = 0.0;
            mat_no_trans[(2, 3)] = 0.0;
            let inv_trans = mat_no_trans.try_inverse().unwrap().transpose();

            // Duplicate the template mesh
            let mut instance = template_mesh.clone();

            // Bake the transform into the mesh vertices
            for primitive in &mut instance.primitives {
                primitive
                    .positions
                    .iter_mut()
                    .for_each(|p| *p = mat.transform_point(&Point3::from(*p)).coords);

                primitive
                    .normals
                    .iter_mut()
                    .for_each(|v| *v = inv_trans.transform_vector(v).normalize());

                // Note that tangents should not use the inverse transpose
                primitive
                    .tangents
                    .iter_mut()
                    .for_each(|t| *t = mat.transform_vector(t).normalize());
            }

            // Flatten the primitives into the combined_mesh
            combined_mesh.primitives.append(&mut instance.primitives);
        }

        for child in node.children() {
            ResourceManager::load_gltf_node(&child, &global_trans, combined_mesh, meshes);
        }
    }

    /// We'll traverse the gltf scene, baking instances of `meshes` with the global transform
    /// of each node. We'll combine all primitives in a flat list and generate a single mesh for the entire file
    fn load_gltf_scenes(
        &mut self,
        file_identifier: &str,
        scenes: gltf::iter::Scenes,
        meshes: &Vec<Option<IntermediateMesh>>,
    ) -> Option<Rc<RefCell<Mesh>>> {
        log::info!(
            "Loading {} scenes from gltf file '{}':",
            scenes.len(),
            file_identifier
        );

        let mut combined_mesh = IntermediateMesh {
            name: file_identifier.to_string(),
            primitives: Vec::new(),
        };

        // Combine all meshes into one
        let identity = Transform::<f32>::identity();
        for gltf_scene in scenes {
            for child_node in gltf_scene.nodes() {
                ResourceManager::load_gltf_node(
                    &child_node,
                    &identity,
                    &mut combined_mesh,
                    &meshes,
                );
            }
        }

        // Create a single combined AABB. No point in doing this sooner because we
        // have no idea what the transforms are going to be
        // AABB collider as early out
        let mut mins = Point3::new(INFINITY, INFINITY, INFINITY);
        let mut maxes = Point3::new(-INFINITY, -INFINITY, -INFINITY);
        for prim in &combined_mesh.primitives {
            for pos in &prim.positions {
                mins.x = mins.x.min(pos.x);
                mins.y = mins.y.min(pos.y);
                mins.z = mins.z.min(pos.z);

                maxes.x = maxes.x.max(pos.x);
                maxes.y = maxes.y.max(pos.y);
                maxes.z = maxes.z.max(pos.z);
            }
        }
        maxes.x = mins.x.max(maxes.x + 0.00001);
        maxes.y = mins.y.max(maxes.y + 0.00001);
        maxes.z = mins.z.max(maxes.z + 0.00001);

        let mesh = intermediate_to_mesh(&combined_mesh);
        mesh.borrow_mut().collider = Some(Box::new(AxisAlignedBoxCollider { mins, maxes }));

        // Set the defines onto the prim's default material (it should be an instance)
        for prim in &mut mesh.borrow_mut().primitives {
            if let Some(mat) = &prim.default_material {
                mat.borrow_mut().set_prim_defines(prim);
            }
        }

        return Some(mesh);
    }

    /// Will parse and bake the entire gltf file into a single mesh with the same name as the file identifier.
    /// WIll also load all materials and textures contained within the file.
    pub fn load_gltf_data(
        &mut self,
        file_identifier: &str,
        gltf_doc: &gltf::Document,
        gltf_buffers: &Vec<gltf::buffer::Data>,
        gltf_images: &Vec<gltf::image::Data>,
    ) {
        self.load_textures_from_gltf(file_identifier, gltf_doc.textures(), gltf_images);

        let parsed_mats = self.load_materials_from_gltf(file_identifier, gltf_doc.materials());

        let parsed_meshes = self.load_meshes_from_gltf(
            file_identifier,
            gltf_doc.meshes(),
            gltf_buffers,
            &parsed_mats,
        );

        if let Some(combined_mesh) =
            self.load_gltf_scenes(file_identifier, gltf_doc.scenes(), &parsed_meshes)
        {
            if let Some(existing_mesh) = self.meshes.get(file_identifier) {
                existing_mesh.swap(&combined_mesh);

                log::info!(
                    "Mutating existing mesh resource '{}' with new data ",
                    file_identifier
                );
            } else {
                log::info!("Inserting new mesh resource '{}'", file_identifier);
                self.meshes
                    .insert(file_identifier.to_owned(), combined_mesh);
            }
        }
    }
}
