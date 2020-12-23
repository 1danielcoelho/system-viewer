use cgmath::*;

use crate::utils::Ray;

pub trait BoundingBox {
    fn intersects(&self, ray: &Ray) -> f32; // Distance along ray to the intersection point
    fn contains(&self, point: &Point3<f32>) -> bool;
}

pub struct AxisAlignedBoundingBox {
    pub mins: Point3<f32>,
    pub maxes: Point3<f32>,
}

impl BoundingBox for AxisAlignedBoundingBox {
    // Source: https://tavianator.com/2015/ray_box_nan.html
    fn intersects(&self, ray: &Ray) -> f32 {
        let mut t1: f32 = (self.mins[0] - ray.start[0]) / ray.direction[0];
        let mut t2: f32 = (self.maxes[0] - ray.start[0]) / ray.direction[0];

        let mut tmin = t1.min(t2);
        let mut tmax = t1.max(t2);

        for i in 1..=3 {
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

pub struct SphereBoundingBox {
    pub center: Point3<f32>,
    pub radius2: f32,
}

impl BoundingBox for SphereBoundingBox {
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
