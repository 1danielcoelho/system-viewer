use crate::components::light::LightType;
use crate::components::{
    LightComponent, MeshComponent, MetadataComponent, PhysicsComponent, TransformComponent,
};
use crate::managers::resource::body_description::{BodyDescription, BodyType};
use crate::managers::resource::material::{Material, UniformName, UniformValue};
use crate::managers::resource::mesh::Mesh;
use crate::managers::resource::texture::TextureUnit;
use crate::managers::scene::description::BodyInstanceDescription;
use crate::managers::scene::Scene;
use crate::managers::ResourceManager;
use crate::utils::string::decode_hex;
use crate::utils::units::Jdn;
use na::*;
use nalgebra::Vector3;
use std::cell::RefCell;
use std::rc::Rc;

pub fn add_free_body(
    scene: &mut Scene,
    _epoch: Jdn,
    body: &BodyDescription,
    body_instance: &BodyInstanceDescription,
    res_man: &mut ResourceManager,
) {
    if body.body_type == BodyType::Barycenter {
        return;
    }

    if body.mass.is_none() {
        log::warn!(
            "Skipping body {} ('{}') for having no mass",
            body.id.as_ref().unwrap(),
            body.name
        );
        return;
    }

    if body_instance.pos.is_none() || body_instance.linvel.is_none() {
        log::warn!(
            "Skipping body '{}' for having no state vector",
            body.id.as_ref().unwrap()
        );
        return;
    }

    log::info!(
        "Adding body '{}' to scene '{}'",
        body.name,
        scene.identifier
    );

    let mut name = &body.name;
    if name.is_empty() {
        name = body.id.as_ref().unwrap();
    }

    // Entity
    let body_ent = scene.new_entity(Some(&name));
    let trans_comp = scene.add_component::<TransformComponent>(body_ent);
    let trans = trans_comp.get_local_transform_mut();
    trans.trans = body_instance.pos.unwrap();
    if let Some(rot) = body_instance.rot {
        trans.rot = UnitQuaternion::from_euler_angles(
            rot.x.to_radians(),
            rot.y.to_radians(),
            rot.z.to_radians(),
        );
    }
    if let Some(scale) = body_instance.scale {
        trans.scale = scale;
    }

    // Mesh
    if let Some(radius) = body.radius {
        if radius > 0.0 {
            trans_comp
                .get_local_transform_mut()
                .scale
                .scale_mut(radius as f64);

            let mesh_comp = scene.add_component::<MeshComponent>(body_ent);
            mesh_comp.set_mesh(get_body_mesh(body, res_man));

            if let Some(mat_over) = get_body_material(body, res_man) {
                log::info!(
                    "Overriding slot 0 with material '{:?}'",
                    mat_over.borrow().get_name()
                );
                mesh_comp.set_material_override(Some(mat_over.clone()), 0);
            }
        }
    }

    // Physics
    let phys_comp = scene.add_component::<PhysicsComponent>(body_ent);
    phys_comp.mass = body.mass.unwrap() as f64;
    phys_comp.lin_mom = body_instance
        .linvel
        .unwrap()
        .scale(body.mass.unwrap() as f64);
    if let Some(ang_vel) = body_instance.angvel {
        phys_comp.ang_mom += phys_comp.mass * ang_vel; // TODO: VERY WRONG! Needs to be moment of inertia instead of mass here
    }

    // Light
    if body.body_type == BodyType::Star || body.brightness.is_some() {
        let light_comp = scene.add_component::<LightComponent>(body_ent);
        light_comp.color = Vector3::new(1.0, 1.0, 1.0);
        light_comp.intensity = body.brightness.unwrap();
        log::info!("Banana sun brightness {}", light_comp.intensity);
        light_comp.light_type = LightType::Point;
    }

    // Metadata
    let meta_comp = scene.add_component::<MetadataComponent>(body_ent);
    meta_comp.set_metadata(
        "body_id",
        &body.id.as_ref().unwrap_or(&String::from("None")),
    );
    meta_comp.set_metadata("body_type", &format!("{:?}", body.body_type));
    for (key, value) in body.meta.iter() {
        meta_comp.set_metadata(key, value);
    }
    if let Some(mat_params) = &body.material_params {
        for (key, value) in mat_params.iter() {
            meta_comp.set_metadata(key, value);
        }
    }
    if let Some(mass) = body.mass {
        meta_comp.set_metadata("body_mass", &mass.to_string());
    }
    if let Some(radius) = body.radius {
        meta_comp.set_metadata("body_radius", &radius.to_string());
    }
    if let Some(albedo) = body.albedo {
        meta_comp.set_metadata("body_albedo", &albedo.to_string());
    }
    if let Some(magnitude) = body.magnitude {
        meta_comp.set_metadata("body_magnitude", &magnitude.to_string());
    }
    if let Some(rotation_period) = body.rotation_period {
        meta_comp.set_metadata("body_rotation_period", &rotation_period.to_string());
    }
    if let Some(rotation_axis) = body.rotation_axis {
        meta_comp.set_metadata(
            "body_rotation_period",
            &format!(
                "[{}, {}, {}]",
                rotation_axis[0], rotation_axis[1], rotation_axis[2]
            ),
        );
    }
    if let Some(spec) = &body.spec_smassii {
        meta_comp.set_metadata("body_spec_smassii", &spec);
    }
    if let Some(spec) = &body.spec_tholen {
        meta_comp.set_metadata("body_spec_tholen", &spec);
    }
}

pub fn get_body_mesh(
    body: &BodyDescription,
    res_man: &mut ResourceManager,
) -> Option<Rc<RefCell<Mesh>>> {
    let mesh = match &body.mesh {
        Some(identifier) => res_man.get_or_create_mesh(&identifier),
        None => res_man.get_or_create_mesh("lat_long_sphere"),
    };

    return mesh;
}

/// TODO: Support multiple materials per body. Then this would return a Vec, and they would
/// all become the material overrides
pub fn get_body_material(
    body: &BodyDescription,
    res_man: &mut ResourceManager,
) -> Option<Rc<RefCell<Material>>> {
    let mut mat = match body.material.clone().unwrap_or(String::from("")).as_str() {
        // TODO: Actual different procedural materials with different uniforms and so on
        "rocky" => Some(
            res_man
                .instantiate_material(
                    "gltf_metal_rough",
                    &("rocky_".to_owned() + &body.id.as_ref().unwrap()),
                )
                .unwrap(),
        ),
        "earth" => Some(
            res_man
                .instantiate_material(
                    "gltf_metal_rough",
                    &("earth_".to_owned() + &body.id.as_ref().unwrap()),
                )
                .unwrap(),
        ),
        "atmo" => Some(
            res_man
                .instantiate_material(
                    "gltf_metal_rough",
                    &("atmo_".to_owned() + &body.id.as_ref().unwrap()),
                )
                .unwrap(),
        ),
        "gas" => Some(
            res_man
                .instantiate_material(
                    "gltf_metal_rough",
                    &("gas_".to_owned() + &body.id.as_ref().unwrap()),
                )
                .unwrap(),
        ),
        _ => None,
    };

    // Default to just fetching a material, as we may have "phong" in there or something like that
    if mat.is_none() {
        if let Some(mat_name) = &body.material {
            mat = res_man.instantiate_material(&mat_name, &mat_name);
        }
    }

    // If we still have nothing fetch a material for a specific body type. For now this just means
    // using the gltf, but later on we will pick between these other materials like rocky, atmo, etc.
    // Don't do it for artificial bodies though as they usually are gltf files with their own materials
    if mat.is_none() && body.body_type != BodyType::Artificial {
        mat = res_man.instantiate_material("gltf_metal_rough", "gltf_metal_rough");
    }

    if mat.is_some() && body.material_params.is_some() {
        let mut mat_mut = mat.as_ref().unwrap().borrow_mut();
        let params = body.material_params.as_ref().unwrap();

        if let Some(color) = params.get("base_color") {
            let mut bytes: Vec<f32> = decode_hex(color)
                .unwrap()
                .iter()
                .map(|u| *u as f32 / 255.0)
                .collect();
            bytes.resize(4, 1.0);

            log::info!("Parsed base_color {:?} for body {:?}", bytes, body.id);

            mat_mut.set_uniform_value(
                UniformName::BaseColorFactor,
                UniformValue::Vec4([bytes[0], bytes[1], bytes[2], bytes[3]]),
            );
        }

        if let Some(path) = params.get("base_color_texture") {
            log::info!(
                "Parsed base_color_texture {:?} for body {:?}",
                path,
                body.id
            );

            mat_mut.set_texture(
                TextureUnit::BaseColor,
                res_man.get_or_request_texture(path, false),
            );
        }

        if let Some(path) = params.get("normal_texture") {
            log::info!("Parsed normal_texture {:?} for body {:?}", path, body.id);

            mat_mut.set_texture(
                TextureUnit::Normal,
                res_man.get_or_request_texture(path, false),
            );
        }

        if let Some(path) = params.get("metal_rough_texture") {
            log::info!(
                "Parsed metal_rough_texture {:?} for body {:?}",
                path,
                body.id
            );

            mat_mut.set_texture(
                TextureUnit::MetallicRoughness,
                res_man.get_or_request_texture(path, false),
            );
        }

        if let Some(path) = params.get("emissive_texture") {
            log::info!("Parsed emissive_texture {:?} for body {:?}", path, body.id);

            mat_mut.set_texture(
                TextureUnit::Emissive,
                res_man.get_or_request_texture(path, false),
            );
        }

        if let Some(float_vec_str) = params.get("emissive_factor") {
            log::info!(
                "Parsed emissive_factor {:?} for body {:?}",
                float_vec_str,
                body.id
            );

            let mut floats = float_vec_str
                .strip_prefix("[")
                .and_then(|s| s.strip_suffix("]"))
                .unwrap()
                .split(",")
                .filter_map(|s| s.trim().parse::<f32>().ok())
                .collect::<Vec<_>>();
            floats.resize(3, 1.0);

            mat_mut.set_uniform_value(
                UniformName::EmissiveFactor,
                UniformValue::Vec3([floats[0], floats[1], floats[2]]),
            );
        }
    };

    return mat;
}

pub fn fetch_default_motion_if_needed(
    body_id: &str,
    body_instance: &mut BodyInstanceDescription,
    res_man: &mut ResourceManager,
    default_time: Jdn,
) {
    let vectors = res_man.get_state_vectors().get(body_id);
    if vectors.is_none() {
        return;
    };

    let vectors = vectors.unwrap();
    if vectors.len() < 1 {
        return;
    }

    // Search for vector closest to default_time
    // Have to do min_by_key manually because Rust
    // Technically we could early out here when delta starts increasing but what the hell, this is one-off code
    let mut lowest_index: usize = 0;
    let mut lowest_delta: f64 = std::f64::INFINITY;
    for (index, vec) in vectors.iter().enumerate() {
        let delta = (vec.jdn_date.0 - default_time.0).abs();
        if delta < lowest_delta {
            lowest_index = index;
            lowest_delta = delta;
        }
    }

    if lowest_delta > 0.1 {
        log::warn!("Using state vector '{:?}' with time delta of '{}' days to scene time '{}', for used for body '{}'", vectors[lowest_index], lowest_delta, default_time.0, body_id);
    }

    let vector = vectors[lowest_index];
    
    if let None = body_instance.pos {
        body_instance.pos = Some(vector.pos.coords);
    }

    if let None = body_instance.linvel {
        body_instance.linvel = Some(vector.vel);
    }
}
