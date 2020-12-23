use cgmath::*;

use crate::components::MeshComponent;

pub struct Ray {
    pub start: Point3<f32>,
    pub direction: Vector3<f32>,
}

pub struct RaycastHit {
    pub entity_index: u32,
    pub hit_position_world: Point3<f32>,
}

pub fn raycast(ray: &Ray, components: &Vec<MeshComponent>) -> Option<RaycastHit> {
    return None;
}
