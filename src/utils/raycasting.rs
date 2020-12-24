use cgmath::*;

use crate::components::{MeshComponent, TransformComponent};

#[derive(Debug)]
pub struct Ray {
    pub start: Point3<f32>,
    pub direction: Vector3<f32>,
}

#[derive(Debug)]
pub struct RaycastHit {
    pub entity_index: u32,
    pub hit_position_world: Point3<f32>,
}

pub fn raycast(
    ray: &Ray,
    meshes: &Vec<MeshComponent>,
    transforms: &Vec<TransformComponent>,
) -> Option<RaycastHit> {
    let mut min_t2: f32 = std::f32::INFINITY;
    let mut entity_index: Option<u32> = None;

    for (index, (mesh_comp, trans_comp)) in meshes.iter().zip(transforms.iter()).enumerate() {
        if !mesh_comp.raycasting_visible {
            continue;
        }

        let mesh = mesh_comp.get_mesh();
        if mesh.is_none() {
            continue;
        }
        let mesh = mesh.unwrap();

        if mesh.bb.is_none() {
            continue;
        }
        let bb = mesh.bb.as_ref().unwrap();

        let world_trans = trans_comp.get_world_transform();
        let inv_world_trans = world_trans.inverse_transform().unwrap();

        // @Performance: Maybe it would be faster to transform the bb instead?
        // But then we'd get some weirdness like AABB not really being axis-aligned anymore,
        // or spheres behaving weirdly on non-uniform scaling
        let bb_space_ray = Ray {
            start: inv_world_trans.transform_point(ray.start),
            direction: inv_world_trans.transform_vector(ray.direction).normalize(),
        };

        let bb_space_t = bb.intersects(&bb_space_ray);
        let bb_space_delta = bb_space_t * bb_space_ray.direction;
        let world_space_delta = world_trans.transform_vector(bb_space_delta);
        let world_space_t2 = world_space_delta.magnitude2();

        if world_space_t2 < min_t2 {
            min_t2 = world_space_t2;
            entity_index = Some(index as u32);
        }
    }

    if let Some(entity_index) = entity_index {
        return Some(RaycastHit {
            entity_index,
            hit_position_world: ray.start + min_t2.sqrt() * ray.direction,
        });
    }

    return None;
}
