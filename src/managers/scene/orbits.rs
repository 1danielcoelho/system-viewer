use crate::components::light::LightType;
use crate::components::{
    LightComponent, MeshComponent, MetadataComponent, PhysicsComponent, TransformComponent,
};
use crate::managers::resource::body_description::{BodyDescription, BodyType};
use crate::managers::resource::material::{Material, UniformName, UniformValue};
use crate::managers::resource::mesh::Mesh;
use crate::managers::resource::texture::TextureUnit;
use crate::managers::scene::description::{BodyInstanceDescription, BodyMotionType};
use crate::managers::scene::{Scene, SceneManager};
use crate::managers::ResourceManager;
use crate::utils::string::decode_hex;
use crate::utils::units::Jdn;
use na::*;
use nalgebra::Vector3;
use std::cell::RefCell;
use std::rc::Rc;

impl SceneManager {
    // pub fn load_bodies_into_scene(
    //     &mut self,
    //     bodies: &Vec<BodyDescription>,
    //     res_man: &mut ResourceManager,
    // ) {
    //     log::info!("Loading {} bodies...", bodies.len());

    // The idea here is that bodies are only ever parented to barycenters
    // Barycenters will themselves translate, but never rotate or scale, so nested orbits look ok-for-now-I-guess

    // let mut id_to_entity: HashMap<u32, Entity> = HashMap::new();
    // for body in bodies {
    //     let parent = {
    //         if body.reference_id == body.id {
    //             None
    //         } else {
    //             // We expect these bodies to be in order, so we should already have
    //             // parsed our parent
    //             Some(id_to_entity[&body.reference_id])
    //         }
    //     };

    //     let body_ent = self.add_body_entities(body, parent, res_man);
    //     if let Some(_) = id_to_entity.insert(body.id, body_ent) {
    //         log::warn!("Body collision when parsing csv for body: '{:#?}'", body);
    //     }
    // }

    // log::info!("Loaded {} bodies into the scene", id_to_entity.len());
    // }

    // Adds these entities to the scene
    // 1) Barycenter/Body entity (+geometry) around the parent barycenter (if available), parented to it (or free body);
    // 2) Orbit entity (+geometry) around the parent barycenter, parented to it (optional),
    //
    // Returns the barycenter/body entity so that we can parent other stuff to it.
    // fn add_body_entities(
    //     &mut self,
    //     body: &BodyDescription,
    //     parent_bary: Option<Entity>,
    //     res_man: &mut ResourceManager,
    // ) -> Entity {
    //     let scene = self.get_main_scene_mut().unwrap();

    //     let body_ent = scene.new_entity(Some(&body.name));
    //     if let Some(parent) = parent_bary {
    //         scene.set_entity_parent(parent, body_ent);
    //     }

    //     let trans_comp = scene.add_component::<TransformComponent>(body_ent);

    // // Sphere mesh
    // if body.mean_radius.0 > 0.0 {
    //     let radius = body.mean_radius.0;

    //     trans_comp.get_local_transform_mut().scale = Vector3::new(radius, radius, radius);

    //     let mesh_comp = scene.add_component::<MeshComponent>(body_ent);
    //     mesh_comp.set_mesh(res_man.get_or_create_mesh("ico_sphere"));
    //     mesh_comp.set_material_override(res_man.get_or_create_material("phong"), 0);
    // }

    // if body.body_type == BodyType::Star {
    //     let light_comp = scene.add_component::<LightComponent>(body_ent);
    //     light_comp.color = Vector3::new(1.0, 1.0, 1.0);
    //     light_comp.intensity = 5E10;
    //     light_comp.light_type = LightType::Point;
    // }

    // // Orbit
    // if body.orbital_elements.semi_major_axis.0 > 0.0 {
    //     let trans = elements_to_circle_transform(&body.orbital_elements);

    //     let orbit_comp = scene.add_component::<OrbitalComponent>(body_ent);
    //     orbit_comp.desc = body.clone(); // TODO: I could probably move this in
    //     orbit_comp.circle_to_final_ellipse = trans.clone();

    //     // Bake eccentric anomalies into the body
    //     if body.orbital_elements.sidereal_orbit_period_days > 0.0 {
    //         const NUM_ANGLES: u32 = 360;

    //         // Add eccentric anomaly interpolation values
    //         orbit_comp.baked_eccentric_anomaly_times =
    //             bake_eccentric_anomaly_times(&body.orbital_elements, NUM_ANGLES);
    //     }

    //     // Orbit mesh entity
    //     {
    //         let orbit = scene.new_entity(Some(&(body.name.clone() + "'s orbit")));
    //         if let Some(parent) = parent_bary {
    //             scene.set_entity_parent(parent, orbit);
    //         }

    //         let trans_comp = scene.add_component::<TransformComponent>(orbit);
    //         *trans_comp.get_local_transform_mut() = trans;

    //         let mesh_comp = scene.add_component::<MeshComponent>(orbit);
    //         mesh_comp.set_mesh(res_man.get_or_create_mesh("circle"));
    //     }
    // }

    //     return body_ent;
    // }
}

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

    log::info!(
        "Adding body '{}' to scene '{}'",
        body.name,
        scene.identifier
    );

    match body_instance.motion_type {
        BodyMotionType::DefaultElements | BodyMotionType::CustomElements => todo!(),
        _ => {}
    };

    let state_vector = body_instance.state_vector.as_ref().unwrap();

    // Entity
    let body_ent = scene.new_entity(Some(&body.name));
    let trans_comp = scene.add_component::<TransformComponent>(body_ent);
    let trans = trans_comp.get_local_transform_mut();
    trans.trans = state_vector.pos.coords;
    if let Some(rot) = body_instance.initial_rot {
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
        trans_comp
            .get_local_transform_mut()
            .scale
            .scale_mut(radius as f64);

        let mesh = get_body_mesh(body, res_man);
        let num_slots = mesh
            .as_ref()
            .and_then(|m| Some(m.borrow()))
            .and_then(|m| Some(m.primitives.len()));

        let mesh_comp = scene.add_component::<MeshComponent>(body_ent);
        mesh_comp.set_mesh(mesh);

        if let Some(num_slots) = num_slots {
            let mat_over = get_body_material(body, res_man);

            for slot_index in 0..num_slots {
                log::info!("Overriding slot {} with material '{:?}'", slot_index, mat_over);
                mesh_comp.set_material_override(mat_over.clone(), slot_index);
            }
        }
    }

    // Physics
    let phys_comp = scene.add_component::<PhysicsComponent>(body_ent);
    phys_comp.mass = body.mass.unwrap() as f64;
    phys_comp.lin_mom = state_vector.vel.scale(body.mass.unwrap() as f64);
    if let Some(ang_vel) = body_instance.angular_velocity {
        phys_comp.ang_mom += phys_comp.mass * ang_vel; // TODO: VERY WRONG! Needs to be moment of inertia instead of mass here
    }

    // Light
    if body.body_type == BodyType::Star {
        let light_comp = scene.add_component::<LightComponent>(body_ent);
        light_comp.color = Vector3::new(1.0, 1.0, 1.0);
        light_comp.intensity = 5E10;
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

pub fn get_body_material(
    body: &BodyDescription,
    res_man: &mut ResourceManager,
) -> Option<Rc<RefCell<Material>>> {
    let mat = match body.material.clone().unwrap_or(String::from("")).as_str() {
        // TODO: Actual different materials with different uniforms and so on
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
        _ => match body.body_type {
            // TODO
            // BodyType::Star => {}
            // BodyType::Planet => {}
            // BodyType::Satellite => {}
            // BodyType::Asteroid => {}
            // BodyType::Comet => {}
            BodyType::Artificial => None,
            // BodyType::Barycenter => {}
            // BodyType::Other => {}
            _ => Some(
                res_man
                    .instantiate_material(
                        "gltf_metal_rough",
                        &("gas_".to_owned() + &body.id.as_ref().unwrap()),
                    )
                    .unwrap(),
            ),
        },
    };

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
    };

    return mat;
}

pub fn fetch_default_motion_if_needed(
    body_id: &str,
    body_instance: &mut BodyInstanceDescription,
    res_man: &mut ResourceManager,
    default_time: Jdn,
) {
    match body_instance.motion_type {
        BodyMotionType::DefaultVector => {
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

            body_instance.state_vector = Some(vectors[lowest_index].clone());
        }
        BodyMotionType::DefaultElements => {
            let elements = res_man.get_osc_elements().get(body_id);
            if elements.is_none() {
                return;
            };

            let elements = elements.unwrap();
            if elements.len() < 1 {
                return;
            }

            // Search for elements closest to default_time
            // Have to do min_by_key manually because Rust
            // Technically we could early out here when delta starts increasing but what the hell, this is one-off code
            let mut lowest_index: usize = 0;
            let mut lowest_delta: f64 = std::f64::INFINITY;
            for (index, el) in elements.iter().enumerate() {
                let delta = (el.epoch.0 - default_time.0).abs();
                if delta < lowest_delta {
                    lowest_index = index;
                    lowest_delta = delta;
                }
            }

            if lowest_delta > 0.1 {
                log::warn!("Using orbital elements '{:?}' with time delta of '{}' days to scene time '{}', for used for body '{}'", elements[lowest_index], lowest_delta, default_time.0, body_id);
            }

            body_instance.orbital_elements = Some(elements[lowest_index].clone());
        }
        _ => {}
    };
}
