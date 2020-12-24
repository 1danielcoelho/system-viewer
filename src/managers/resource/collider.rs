use super::Mesh;
use crate::utils::Ray;
use cgmath::*;
use std::rc::Rc;

const RAY_TRIANGLE_EPSILON: f32 = 1e-6;

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
    // Source: https://tavianator.com/2015/ray_box_nan.html
    fn intersects(&self, ray: &Ray) -> f32 {
        let mut t1: f32 = (self.mins[0] - ray.start[0]) / ray.direction[0];
        let mut t2: f32 = (self.maxes[0] - ray.start[0]) / ray.direction[0];

        let mut tmin = t1.min(t2);
        let mut tmax = t1.max(t2);

        for i in 1..3 {
            t1 = (self.mins[i] - ray.start[i]) / ray.direction[i];
            t2 = (self.maxes[i] - ray.start[i]) / ray.direction[i];

            tmin = tmin.max(t1.min(t2).min(tmax));
            tmax = tmax.min(t1.max(t2).max(tmin));
        }

        if tmax <= tmin.max(0.0) {
            return std::f32::INFINITY;
        }

        return tmin;
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
    // Source: https://www.scratchapixel.com/lessons/3d-basic-rendering/minimal-ray-tracer-rendering-simple-shapes/ray-sphere-intersection
    fn intersects(&self, ray: &Ray) -> f32 {
        let start_to_center = self.center - ray.start;
        let projection_on_dir = start_to_center.dot(ray.direction);

        let dist2_center_nearest_pt =
            start_to_center.dot(start_to_center) - projection_on_dir.powi(2);
        if dist2_center_nearest_pt > self.radius2 {
            return std::f32::INFINITY;
        }

        let delta = (self.radius2 - dist2_center_nearest_pt).sqrt();
        let mut t0 = projection_on_dir - delta;
        let mut t1 = projection_on_dir + delta;

        if t0 > t1 {
            std::mem::swap(&mut t0, &mut t1);
        }

        if t0 < 0.0 {
            t0 = t1;
            if t0 < 0.0 {
                return std::f32::INFINITY;
            }
        }

        return t0;
    }

    fn contains(&self, point: &Point3<f32>) -> bool {
        return self.center.distance2(*point) <= self.radius2;
    }
}

#[derive(Clone, PartialEq)]
pub struct MeshCollider {
    pub mesh: Rc<Mesh>,
}
impl Collider for MeshCollider {
    fn intersects(&self, ray: &Ray) -> f32 {
        let mut min_t: f32 = std::f32::INFINITY;

        // Early out if our mesh has a collider and it's not just pointing back at the mesh
        if let Some(collider) = &self.mesh.collider {
            if self.as_mesh() != collider.as_mesh() {
                if collider.intersects(&ray) == min_t {
                    return min_t;
                }
            }
        }

        for primitive in &self.mesh.primitives {
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
        let test_ray = Ray {
            start: *point,
            direction: Vector3::new(1.0, 0.0, 0.0),
        };

        let mut num_intersections = 0;
        for primitive in &self.mesh.primitives {
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

// MT algorithm as described here: https://www.scratchapixel.com/lessons/3d-basic-rendering/ray-tracing-rendering-a-triangle/moller-trumbore-ray-triangle-intersection
fn triangle_ray_intersection(
    p0: &Vector3<f32>,
    p1: &Vector3<f32>,
    p2: &Vector3<f32>,
    ray: &Ray,
) -> Option<f32> {
    let p0p1 = p1 - p0;
    let p0p2 = p2 - p0;
    let pvec = ray.direction.cross(p0p2);
    let det = p0p1.dot(pvec);
    if det.abs() < RAY_TRIANGLE_EPSILON {
        return None;
    }

    let inv_det = 1.0 / det;

    let tvec = ray.start - p0;
    let u = tvec.dot(pvec) * inv_det;
    if u < 0.0 || u > 1.0 {
        return None;
    }

    let qvec = tvec.to_vec().cross(p0p1);
    let v = ray.direction.dot(qvec) * inv_det;
    if v < 0.0 || u + v > 1.0 {
        return None;
    }

    let t = p0p2.dot(qvec) * inv_det;
    return Some(t);
}
