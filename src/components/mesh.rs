use std::{cell::RefCell, rc::Rc};

use crate::managers::{
    resource::{Material, Mesh},
    ECManager,
};

use super::{
    component::{ComponentIndex, ComponentStorageType},
    Component,
};

#[derive(Clone)]
pub struct MeshComponent {
    enabled: bool,

    pub aabb_min: cgmath::Vector3<f32>,
    pub aabb_max: cgmath::Vector3<f32>,
    pub raycasting_visible: bool,
    pub visible: bool,
    mesh: Option<Rc<Mesh>>,
    material_overrides: Vec<Option<Rc<RefCell<dyn Material>>>>,
}
impl MeshComponent {
    fn new() -> Self {
        return Self::default();
    }

    pub fn get_mesh(&self) -> Option<Rc<Mesh>> {
        return self.mesh.clone();
    }

    pub fn set_mesh(&mut self, mesh: Option<Rc<Mesh>>) {
        self.mesh = mesh;

        if let Some(mesh) = &self.mesh {
            self.material_overrides.resize(mesh.primitives.len(), None);
        } else {
            self.material_overrides.resize(0, None);
        }
    }

    pub fn get_material_override(&self, index: usize) -> Option<Rc<RefCell<dyn Material>>> {
        if let Some(material_override) = self.material_overrides.get(index) {
            return material_override.clone();
        } else {
            return None;
        }
    }

    pub fn set_material_override(&mut self, material: Option<Rc<RefCell<dyn Material>>>, index: usize) {
        self.material_overrides[index] = material;
    }

    pub fn get_resolved_material(&self, index: usize) -> Option<Rc<RefCell<dyn Material>>> {
        if self.material_overrides.len() <= index {
            return None;
        } else if let Some(material_override) = self.get_material_override(index) {
            return Some(material_override);
        } else if let Some(mesh) = &self.mesh {
            return mesh.primitives[index].default_material.clone();
        }
        return None;
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
            material_overrides: Vec::new(),
        };
    }
}
impl Component for MeshComponent {
    type ComponentType = MeshComponent;
    const STORAGE_TYPE: ComponentStorageType = ComponentStorageType::Vec;

    fn get_component_index() -> ComponentIndex {
        return ComponentIndex::Mesh;
    }

    fn get_components_vector<'a>(w: &'a mut ECManager) -> Option<&'a mut Vec<MeshComponent>> {
        return Some(&mut w.mesh);
    }

    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    fn get_enabled(&mut self) -> bool {
        return self.enabled;
    }
}
