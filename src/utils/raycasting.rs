use crate::managers::scene::Scene;
use na::{Matrix4, Point3, RealField, SimdRealField, Vector3};

pub const INTERSECTION_EPSILON: f32 = 1e-6;

#[derive(Debug)]
pub struct Ray<T>
where
    T: RealField + SimdRealField,
{
    pub start: Point3<T>,
    pub direction: Vector3<T>,
}

#[derive(Debug)]
pub struct RaycastHit<T>
where
    T: RealField + SimdRealField,
{
    pub entity_index: u32,
    pub hit_position_world: Point3<T>,
}

/// Raycasts are done in f64 because they are input/output in world space. Internally they'll be converted to
/// f32 after we transform them into the coordinate space of the collider, where the precision is enough.
/// We'll then be able to use f32 meshes as colliders
pub fn raycast(ray: &Ray<f64>, scene: &Scene) -> Option<RaycastHit<f64>> {
    let mut min_t2: f64 = std::f64::INFINITY;
    let mut entity_index: Option<u32> = None;

    for (index, (mesh_comp, trans_comp)) in
        scene.mesh.iter().zip(scene.transform.iter()).enumerate()
    {
        if !mesh_comp.raycasting_visible {
            continue;
        }

        let mesh = mesh_comp.get_mesh();
        if mesh.is_none() {
            continue;
        }
        let mesh = mesh.as_ref().unwrap().borrow();

        if mesh.collider.is_none() {
            continue;
        }
        let collider = mesh.collider.as_ref().unwrap();

        let world_trans: Matrix4<f64> = trans_comp.get_world_transform().to_matrix4();
        let inv_world_trans = world_trans.try_inverse().unwrap();

        let bb_space_dir_64 = inv_world_trans.transform_vector(&ray.direction).normalize();
        let bb_space_ray: Ray<f32> = Ray {
            start: na::convert(inv_world_trans.transform_point(&ray.start)),
            direction: na::convert(bb_space_dir_64),
        };

        let bb_space_t = collider.intersects(&bb_space_ray) as f64;
        let bb_space_delta = bb_space_t * bb_space_dir_64;
        let world_space_delta = world_trans.transform_vector(&bb_space_delta);
        let world_space_t2 = world_space_delta.magnitude_squared();

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

// Source: https://tavianator.com/2015/ray_box_nan.html
pub fn aabb_ray_intersection(mins: &Point3<f32>, maxes: &Point3<f32>, ray: &Ray<f32>) -> f32 {
    let mut t1: f32 = (mins[0] - ray.start[0]) / ray.direction[0];
    let mut t2: f32 = (maxes[0] - ray.start[0]) / ray.direction[0];

    let mut tmin = t1.min(t2);
    let mut tmax = t1.max(t2);

    for i in 1..3 {
        t1 = (mins[i] - ray.start[i]) / ray.direction[i];
        t2 = (maxes[i] - ray.start[i]) / ray.direction[i];

        tmin = tmin.max(t1.min(t2).min(tmax));
        tmax = tmax.min(t1.max(t2).max(tmin));
    }

    if tmax <= tmin.max(0.0) {
        return std::f32::INFINITY;
    }

    return tmin;
}

// Source: https://www.scratchapixel.com/lessons/3d-basic-rendering/minimal-ray-tracer-rendering-simple-shapes/ray-sphere-intersection
pub fn sphere_ray_intersection(center: &Point3<f32>, radius2: f32, ray: &Ray<f32>) -> f32 {
    let start_to_center = center - ray.start;
    let projection_on_dir = start_to_center.dot(&ray.direction);

    let dist2_center_nearest_pt = start_to_center.dot(&start_to_center) - projection_on_dir.powi(2);
    if dist2_center_nearest_pt > radius2 {
        return std::f32::INFINITY;
    }

    let delta = (radius2 - dist2_center_nearest_pt).sqrt();
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

// MT algorithm as described here: https://www.scratchapixel.com/lessons/3d-basic-rendering/ray-tracing-rendering-a-triangle/moller-trumbore-ray-triangle-intersection
pub fn triangle_ray_intersection(
    p0: &Vector3<f32>,
    p1: &Vector3<f32>,
    p2: &Vector3<f32>,
    ray: &Ray<f32>,
) -> Option<f32> {
    let p0p1 = p1 - p0;
    let p0p2 = p2 - p0;
    let pvec = ray.direction.cross(&p0p2);
    let det = p0p1.dot(&pvec);
    if det.abs() < INTERSECTION_EPSILON {
        return None;
    }

    let inv_det = 1.0 / det;

    let tvec = (ray.start - p0).coords;
    let u = tvec.dot(&pvec) * inv_det;
    if u < 0.0 || u > 1.0 {
        return None;
    }

    let qvec = tvec.cross(&p0p1);
    let v = ray.direction.dot(&qvec) * inv_det;
    if v < 0.0 || u + v > 1.0 {
        return None;
    }

    let t = p0p2.dot(&qvec) * inv_det;
    if t > INTERSECTION_EPSILON {
        return Some(t);
    }
    return None;
}
