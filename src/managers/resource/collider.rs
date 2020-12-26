use super::Mesh;
use crate::utils::raycasting::{
    aabb_ray_intersection, sphere_ray_intersection, triangle_ray_intersection, Ray,
};
use na::{Point3, Vector3};
use std::{cell::RefCell, rc::Weak};

// Trick from https://stackoverflow.com/questions/30353462/how-to-clone-a-struct-storing-a-boxed-trait-object
// to have clonable trait objects
pub trait ColliderClone {
    fn clone_box(&self) -> Box<dyn Collider>;
}
impl<T> ColliderClone for T
where
    T: 'static + Collider + Clone,
{
    fn clone_box(&self) -> Box<dyn Collider> {
        Box::new(self.clone())
    }
}
impl Clone for Box<dyn Collider> {
    fn clone(&self) -> Box<dyn Collider> {
        self.clone_box()
    }
}

pub trait Collider: ColliderClone {
    fn intersects(&self, ray: &Ray) -> f32; // Distance along ray to the intersection point
    fn contains(&self, point: &Point3<f32>) -> bool;

    fn as_mesh(&self) -> Option<MeshCollider> {
        None
    }
    fn as_compound(&self) -> CompoundCollider {
        CompoundCollider {
            colliders: vec![self.clone_box()],
        }
    }
    fn combine(&self, other: Box<dyn Collider>) -> CompoundCollider {
        let mut result = self.as_compound();
        let mut other_compound = other.as_compound();
        result.colliders.append(&mut other_compound.colliders);
        return result;
    }
}

#[derive(Clone)]
pub struct CompoundCollider {
    pub colliders: Vec<Box<dyn Collider>>,
}
impl Collider for CompoundCollider {
    fn intersects(&self, ray: &Ray) -> f32 {
        let mut min_t = std::f32::INFINITY;
        for collider in &self.colliders {
            min_t = min_t.min(collider.intersects(ray));
        }
        return min_t;
    }

    fn contains(&self, point: &Point3<f32>) -> bool {
        return self
            .colliders
            .iter()
            .any(|collider| collider.contains(point));
    }

    fn as_compound(&self) -> CompoundCollider {
        return self.clone();
    }
}

#[derive(Clone)]
pub struct AxisAlignedBoxCollider {
    pub mins: Point3<f32>,
    pub maxes: Point3<f32>,
}
impl Collider for AxisAlignedBoxCollider {
    fn intersects(&self, ray: &Ray) -> f32 {
        return aabb_ray_intersection(&self.mins, &self.maxes, ray);
    }

    fn contains(&self, point: &Point3<f32>) -> bool {
        return point.x <= self.maxes.x
            && point.x >= self.mins.x
            && point.y <= self.maxes.y
            && point.y >= self.mins.y
            && point.z <= self.maxes.z
            && point.z >= self.mins.z;
    }
}

#[derive(Clone)]
pub struct SphereCollider {
    pub center: Point3<f32>,
    pub radius2: f32,
}
impl Collider for SphereCollider {
    fn intersects(&self, ray: &Ray) -> f32 {
        return sphere_ray_intersection(&self.center, self.radius2, ray);
    }

    fn contains(&self, point: &Point3<f32>) -> bool {
        return (self.center - point).magnitude_squared() <= self.radius2;
    }
}

#[derive(Clone)]
pub struct MeshCollider {
    // This can't be Rc as meshes can use themselves as their colliders
    pub mesh: Weak<RefCell<Mesh>>,

    // When we have a mesh use itself as a MeshCollider, we can use this additional member
    // as an early-out check. In other cases, when we use another mesh as a collider the
    // mesh's own collider will also be used as an early out
    pub additional_outer_collider: Option<Box<dyn Collider>>,
}
impl Collider for MeshCollider {
    fn intersects(&self, ray: &Ray) -> f32 {
        let mut min_t: f32 = std::f32::INFINITY;

        let mesh = self.mesh.upgrade();
        if mesh.is_none() {
            log::warn!(
                "Failed to reach target mesh when calculating intersection for a MeshCollider!"
            );
            return min_t;
        }

        // Additional early out if we have one
        if let Some(additional_collider) = &self.additional_outer_collider {
            if additional_collider.intersects(&ray) == min_t {
                return min_t;
            }
        }

        // Early out if our mesh has a collider and it's not just pointing back at the mesh
        if let Some(collider) = &mesh.as_ref().unwrap().borrow().collider {
            let other_as_mesh = collider.as_mesh();

            if other_as_mesh.is_none() || other_as_mesh.unwrap().mesh.upgrade() != mesh {
                if collider.intersects(&ray) == min_t {
                    log::warn!("Earlying out due to recursive MeshCollider!");
                    return min_t;
                }
            }
        }

        for primitive in &mesh.unwrap().borrow().primitives {
            if let Some(ref source_data) = primitive.source_data {
                for i in 0..source_data.indices.len() / 3 {
                    let p0 = source_data.positions[source_data.indices[i * 3 + 0] as usize];
                    let p1 = source_data.positions[source_data.indices[i * 3 + 1] as usize];
                    let p2 = source_data.positions[source_data.indices[i * 3 + 2] as usize];

                    if let Some(inter_t) = triangle_ray_intersection(&p0, &p1, &p2, ray) {
                        min_t = min_t.min(inter_t);
                    }
                }
            }
        }

        return min_t;
    }

    fn contains(&self, point: &Point3<f32>) -> bool {
        let mesh = self.mesh.upgrade();
        if mesh.is_none() {
            log::warn!("Failed to reach target mesh when calculating contains for a MeshCollider!");
            return false;
        }

        // Additional early out if we have one
        if let Some(additional_collider) = &self.additional_outer_collider {
            if !additional_collider.contains(point) {
                return false;
            }
        }

        // Early out if our mesh has a collider and it's not just pointing back at the mesh
        if let Some(collider) = &mesh.as_ref().unwrap().borrow().collider {
            let other_as_mesh = collider.as_mesh();

            if other_as_mesh.is_none() || other_as_mesh.unwrap().mesh.upgrade() != mesh {
                if !collider.contains(point) {
                    return false;
                }
            }
        }

        let test_ray = Ray {
            start: *point,
            direction: Vector3::new(1.0, 0.0, 0.0),
        };

        let mut num_intersections = 0;
        for primitive in &mesh.unwrap().borrow().primitives {
            if let Some(ref source_data) = primitive.source_data {
                for i in 0..source_data.indices.len() / 3 {
                    let p0 = source_data.positions[source_data.indices[i * 3 + 0] as usize];
                    let p1 = source_data.positions[source_data.indices[i * 3 + 1] as usize];
                    let p2 = source_data.positions[source_data.indices[i * 3 + 2] as usize];

                    if let Some(_) = triangle_ray_intersection(&p0, &p1, &p2, &test_ray) {
                        num_intersections += 1;
                    }
                }
            }
        }

        return num_intersections % 2 != 0;
    }

    fn as_mesh(&self) -> Option<MeshCollider> {
        return Some(self.clone());
    }
}
