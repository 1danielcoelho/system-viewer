use std::rc::Rc;

use crate::{managers::ComponentManager, materials::Material, mesh::Mesh};

use super::{Component, ComponentIndex};

pub struct MeshComponent {
    enabled: bool,

    pub aabb_min: cgmath::Vector3<f32>,
    pub aabb_max: cgmath::Vector3<f32>,
    pub raycasting_visible: bool,
    pub visible: bool,
    pub mesh: Option<Rc<Mesh>>,
    pub material: Option<Rc<Material>>,
}
impl MeshComponent {
    fn new() -> Self {
        return Self::default();
    }
}
impl Default for MeshComponent {
    fn default() -> Self {
        return Self {
            enabled: false,
            aabb_min: cgmath::Vector3::new(0.0, 0.0, 0.0),
            aabb_max: cgmath::Vector3::new(0.0, 0.0, 0.0),
            raycasting_visible: true,
            visible: true,
            mesh: None,
            material: None,
        };
    }
}
impl Component for MeshComponent {
    type ComponentType = MeshComponent;

    fn get_component_index() -> ComponentIndex {
        return ComponentIndex::Mesh;
    }

    fn get_components_vector<'a>(w: &'a mut ComponentManager) -> &'a mut Vec<MeshComponent> {
        return &mut w.mesh;
    }

    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    fn get_enabled(&mut self) -> bool {
        return self.enabled;
    }
}
