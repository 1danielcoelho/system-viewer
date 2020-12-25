use super::{
    component::{ComponentStorageType, ComponentType},
    Component,
};
use crate::managers::{
    details_ui::DetailsUI,
    resource::{Material, Mesh},
    ECManager,
};
use egui::Ui;
use std::{cell::RefCell, rc::Rc};

#[derive(Clone)]
pub struct MeshComponent {
    enabled: bool,

    pub raycasting_visible: bool,
    pub visible: bool,
    mesh: Option<Rc<RefCell<Mesh>>>,
    material_overrides: Vec<Option<Rc<RefCell<Material>>>>,
}
impl MeshComponent {
    fn new() -> Self {
        return Self::default();
    }

    pub fn get_mesh(&self) -> Option<Rc<RefCell<Mesh>>> {
        return self.mesh.clone();
    }

    pub fn set_mesh(&mut self, mesh: Option<Rc<RefCell<Mesh>>>) {
        self.mesh = mesh;

        if let Some(mesh) = &self.mesh {
            self.material_overrides
                .resize(mesh.borrow().primitives.len(), None);
        } else {
            self.material_overrides.resize(0, None);
        }
    }

    pub fn get_material_override(&self, index: usize) -> Option<Rc<RefCell<Material>>> {
        if let Some(material_override) = self.material_overrides.get(index) {
            return material_override.clone();
        } else {
            return None;
        }
    }

    pub fn set_material_override(&mut self, material: Option<Rc<RefCell<Material>>>, index: usize) {
        self.material_overrides[index] = material;
    }

    pub fn get_resolved_material(&self, index: usize) -> Option<Rc<RefCell<Material>>> {
        if self.material_overrides.len() <= index {
            return None;
        } else if let Some(material_override) = self.get_material_override(index) {
            return Some(material_override);
        } else if let Some(mesh) = &self.mesh {
            return mesh.borrow().primitives[index].default_material.clone();
        }
        return None;
    }
}
impl Default for MeshComponent {
    fn default() -> Self {
        return Self {
            enabled: false,
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
    const COMPONENT_TYPE: ComponentType = ComponentType::Mesh;

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

impl DetailsUI for MeshComponent {
    fn draw_details_ui(&mut self, ui: &mut Ui) {
        ui.label("Mesh component:");

        ui.columns(2, |cols| {
            cols[0].label("Mesh:");
            cols[1].label(
                self.mesh
                    .as_ref()
                    .and_then(|m| Some(m.borrow().name.clone()))
                    .unwrap_or_default(),
            );
        });

        for i in 0..self.material_overrides.len() {
            if let Some(mat) = self.get_resolved_material(i) {
                mat.borrow_mut().draw_details_ui(ui);
            }
        }
    }
}
