use crate::components::light::LightType;
use crate::components::{
    LightComponent, MeshComponent, MetadataComponent, PhysicsComponent, TransformComponent,
};
use crate::managers::orbit::{BodyDescription, BodyInstanceDescription, BodyType, StateVector};
use crate::managers::resource::material::{Material, UniformName, UniformValue};
use crate::managers::resource::mesh::Mesh;
use crate::managers::resource::texture::TextureUnit;
use crate::managers::scene::Scene;
use crate::managers::{OrbitManager, ResourceManager};
use crate::utils::string::decode_hex;
use crate::utils::units::Jdn;
use na::*;
use nalgebra::Vector3;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub fn add_body_instance_entities(
    scene: &mut Scene,
    _epoch: Jdn,
    body: Option<&BodyDescription>,
    body_instance: &BodyInstanceDescription,
    default_state_vector: Option<StateVector>,
    res_man: &mut ResourceManager,
) {
    // Get overridable properties
    let mut name = None;
    let mut mass = None;
    let mut brightness = None;
    let mut mesh_params = None;
    let mut material_params = None;
    let mut pos = default_state_vector.as_ref().and_then(|v| Some(v.pos));
    let mut linvel = default_state_vector.as_ref().and_then(|v| Some(v.vel));
    if let Some(body) = body {
        name = Some(&body.name);
        mass = body.mass;
        brightness = body.brightness;
        mesh_params = body.mesh_params.as_ref();
        material_params = body.material_params.as_ref();
    }

    // Override them if we have any
    if let Some(instance_name) = body_instance.name.as_ref() {
        name = Some(instance_name);
    }
    if let Some(instance_mass) = body_instance.mass {
        mass = Some(instance_mass);
    }
    if let Some(instance_brightness) = body_instance.brightness {
        brightness = Some(instance_brightness);
    }
    if let Some(instance_params) = body_instance.mesh_params.as_ref() {
        mesh_params = Some(instance_params);
    }
    if let Some(instance_params) = body_instance.material_params.as_ref() {
        material_params = Some(instance_params);
    }
    if let Some(instance_pos) = body_instance.pos.as_ref() {
        pos = Some(*instance_pos);
    }
    if let Some(instance_vel) = body_instance.linvel.as_ref() {
        linvel = Some(*instance_vel);
    }

    // Skip barycenters for now as they also end up generating points and just waste time in general
    if let Some(t) = body.and_then(|b| Some(b.body_type)) {
        if t == BodyType::Barycenter {
            return;
        }
    }

    log::info!(
        "Adding body '{}' to scene '{}'",
        name.unwrap_or(&String::new()),
        scene.identifier,
    );

    // Main entity
    let body_ent = scene.new_entity(name.and_then(|s| Some(s.as_str())));
    let trans_comp = scene.add_component::<TransformComponent>(body_ent);
    let trans = trans_comp.get_local_transform_mut();
    trans.trans = pos.unwrap_or(Point3::new(0.0, 0.0, 0.0)).coords;

    // Light
    if (body.is_some() && body.unwrap().body_type == BodyType::Star) || brightness.is_some() {
        let light_comp = scene.add_component::<LightComponent>(body_ent);
        light_comp.color = Vector3::new(1.0, 1.0, 1.0);
        light_comp.intensity = brightness.unwrap();
        light_comp.light_type = LightType::Point;
    }

    // Metadata
    let meta_comp = scene.add_component::<MetadataComponent>(body_ent);
    if let Some(body) = body {
        meta_comp.set_metadata(
            "body_id",
            &body.id.as_ref().unwrap_or(&String::from("None")),
        );
        meta_comp.set_metadata("body_type", &format!("{:?}", body.body_type));
        for (key, value) in body.meta.iter() {
            meta_comp.set_metadata(key, value);
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
    if let Some(params) = &material_params {
        for (key, value) in params.iter() {
            meta_comp.set_metadata(key, value);
        }
    }
    if let Some(params) = &mesh_params {
        for (key, value) in params.iter() {
            meta_comp.set_metadata(key, value);
        }
    }
    if let Some(mass) = mass {
        meta_comp.set_metadata("body_mass", &mass.to_string());
    }

    // Child mesh holder entity
    // Separate entity so that our main entity can have children, and yet a separate child mesh
    // that rotates independently of it
    let sphere_ent = scene.new_entity(None);
    scene.set_entity_parent(body_ent, sphere_ent);
    let sphere_trans_comp = scene.add_component::<TransformComponent>(sphere_ent);
    let sphere_trans = sphere_trans_comp.get_local_transform_mut();
    if let Some(rot) = body_instance.rot {
        sphere_trans.rot = UnitQuaternion::from_euler_angles(
            rot.x.to_radians(),
            rot.y.to_radians(),
            rot.z.to_radians(),
        );
    }
    if let Some(scale) = body_instance.scale {
        sphere_trans.scale = scale;
    }
    if let Some(radius) = body.and_then(|b| b.radius) {
        if radius > 0.0 {
            sphere_trans.scale.scale_mut(radius as f64);
        }
    }

    // Child mesh
    let mesh_comp = scene.add_component::<MeshComponent>(sphere_ent);
    mesh_comp.set_mesh(get_body_mesh(body, body_instance, res_man));
    if let Some(mat_over) = get_body_material(body, body_instance, res_man) {
        log::info!(
            "Overriding slot 0 with material '{:?}'",
            mat_over.borrow().get_name()
        );
        mesh_comp.set_material_override(Some(mat_over.clone()), 0);
    }

    // Physics
    if body_instance.parent.is_none() && mass.is_some() && mass.unwrap() > 0.0 {
        let phys_comp = scene.add_component::<PhysicsComponent>(body_ent);
        phys_comp.mass = mass.unwrap() as f64;

        if let Some(linvel) = linvel {
            phys_comp.lin_mom = linvel.scale(phys_comp.mass);
        }

        if let Some(ang_vel) = body_instance.angvel {
            phys_comp.ang_mom += phys_comp.mass * ang_vel; // TODO: VERY WRONG! Needs to be moment of inertia instead of mass here
        }
    }
}

pub fn get_body_mesh(
    body: Option<&BodyDescription>,
    body_instance: &BodyInstanceDescription,
    res_man: &mut ResourceManager,
) -> Option<Rc<RefCell<Mesh>>> {
    let mut mesh = body_instance
        .mesh
        .as_ref()
        .or(body.and_then(|b| b.mesh.as_ref()))
        .cloned();

    // Fetch mesh according to body type if we have nothing else yet
    if mesh.is_none() && body.is_some() {
        mesh = match body.unwrap().body_type {
            BodyType::Star => Some(String::from("lat_long_sphere")),
            BodyType::Planet => Some(String::from("lat_long_sphere")),
            BodyType::Satellite => Some(String::from("lat_long_sphere")),
            BodyType::Asteroid => Some(String::from("lat_long_sphere")),
            BodyType::Comet => Some(String::from("lat_long_sphere")),
            BodyType::Artificial => None,
            BodyType::Barycenter => None,
            BodyType::Other => None,
        }
    }

    return mesh.and_then(|m| res_man.get_or_create_mesh(m.as_ref()));
}

pub fn get_body_material(
    body: Option<&BodyDescription>,
    body_instance: &BodyInstanceDescription,
    res_man: &mut ResourceManager,
) -> Option<Rc<RefCell<Material>>> {
    let mut mat_name = body_instance
        .material
        .as_ref()
        .or(body.and_then(|b| b.material.as_ref()))
        .cloned();

    // If we have no specific mesh or mesh override we will use a generic mesh for this body (i.e. a planet mesh),
    // so fetch a generic material for it too.
    // Only doing this in these circumstances also allows us to use an artificial gltf body or whatever (like a test
    // scene), and still have its own material show through
    if body.and_then(|b| b.mesh.as_ref()).is_none() && body_instance.mesh.is_none() {
        mat_name = Some(String::from("gltf_metal_rough"));
    }

    if mat_name.is_none() {
        return None;
    }
    let mat_name = mat_name.unwrap();

    let mat = res_man.instantiate_material(&mat_name, &mat_name).unwrap();

    let mut params: HashMap<String, String> = HashMap::new();
    if let Some(body_params) = body.and_then(|b| b.material_params.as_ref()) {
        params.extend(body_params.iter().map(|(k, v)| (k.clone(), v.clone())));
    }
    if let Some(over_params) = body_instance.material_params.as_ref() {
        params.extend(over_params.iter().map(|(k, v)| (k.clone(), v.clone())));
    }

    let body_name = body_instance
        .name
        .as_ref()
        .cloned()
        .or(body.and_then(|b| Some(b.name.clone())))
        .unwrap_or(String::new());

    // Set material parameters
    {
        let mut mat_mut = mat.borrow_mut();

        if let Some(color) = params.get("base_color") {
            let mut bytes: Vec<f32> = decode_hex(color)
                .unwrap()
                .iter()
                .map(|u| *u as f32 / 255.0)
                .collect();
            bytes.resize(4, 1.0);

            log::info!("Parsed base_color {:?} for body {:?}", bytes, body_name);

            mat_mut.set_uniform_value(
                UniformName::BaseColorFactor,
                UniformValue::Vec4([bytes[0], bytes[1], bytes[2], bytes[3]]),
            );
        }

        if let Some(path) = params.get("base_color_texture") {
            log::info!(
                "Parsed base_color_texture {:?} for body {:?}",
                path,
                body_name
            );

            mat_mut.set_texture(
                TextureUnit::BaseColor,
                res_man.get_or_request_texture(path, false),
            );
        }

        if let Some(path) = params.get("normal_texture") {
            log::info!("Parsed normal_texture {:?} for body {:?}", path, body_name);

            mat_mut.set_texture(
                TextureUnit::Normal,
                res_man.get_or_request_texture(path, false),
            );
        }

        if let Some(path) = params.get("metal_rough_texture") {
            log::info!(
                "Parsed metal_rough_texture {:?} for body {:?}",
                path,
                body_name
            );

            mat_mut.set_texture(
                TextureUnit::MetallicRoughness,
                res_man.get_or_request_texture(path, false),
            );
        }

        if let Some(path) = params.get("emissive_texture") {
            log::info!(
                "Parsed emissive_texture {:?} for body {:?}",
                path,
                body_name
            );

            mat_mut.set_texture(
                TextureUnit::Emissive,
                res_man.get_or_request_texture(path, false),
            );
        }

        if let Some(float_vec_str) = params.get("emissive_factor") {
            log::info!(
                "Parsed emissive_factor {:?} for body {:?}",
                float_vec_str,
                body_name
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
    }

    return Some(mat);
}

pub fn fetch_default_motion_if_needed(
    body_id: &str,
    orbit_man: &OrbitManager,
    default_time: Jdn,
) -> Option<StateVector> {
    let vectors = orbit_man.get_state_vectors().get(body_id);
    if vectors.is_none() {
        return None;
    };

    let vectors = vectors.unwrap();
    if vectors.len() < 1 {
        return None;
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

    return Some(vectors[lowest_index].clone());
}
