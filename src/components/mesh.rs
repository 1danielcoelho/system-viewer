use super::Component;
use crate::managers::details_ui::DetailsUI;
use crate::managers::resource::material::Material;
use crate::managers::resource::mesh::Mesh;
use crate::managers::scene::component_storage::ComponentStorage;
use crate::managers::scene::Scene;
use egui::Ui;
use serde::{Deserialize, Serialize};
use std::{cell::RefCell, rc::Rc};

#[derive(Debug, Clone)]
pub struct MeshComponent {
    enabled: bool,

    pub raycasting_visible: bool,
    pub visible: bool,
    mesh: Option<Rc<RefCell<Mesh>>>,
    material_overrides: Vec<Option<Rc<RefCell<Material>>>>,
}

impl MeshComponent {
    #[allow(dead_code)]
    fn new() -> Self {
        return Self::default();
    }

    pub fn get_mesh(&self) -> Option<Rc<RefCell<Mesh>>> {
        return self.mesh.clone();
    }

    pub fn set_mesh(&mut self, mesh: Option<Rc<RefCell<Mesh>>>) {
        self.mesh = mesh;
    }

    pub fn get_material_override(&self, index: usize) -> Option<Rc<RefCell<Material>>> {
        if let Some(material_override) = self.material_overrides.get(index) {
            return material_override.clone();
        } else {
            return None;
        }
    }

    pub fn set_material_override(&mut self, material: Option<Rc<RefCell<Material>>>, index: usize) {
        if self.material_overrides.len() <= index {
            self.material_overrides.resize(index + 1, None);
        }
        self.material_overrides[index] = material;
    }

    pub fn get_resolved_material(&self, index: usize) -> Option<Rc<RefCell<Material>>> {
        if let Some(material_override) = self.get_material_override(index) {
            return Some(material_override);
        }

        if let Some(mesh) = self.mesh.as_ref().and_then(|m| Some(m.borrow())) {
            if let Some(prim) = mesh.primitives.get(index) {
                return prim.default_material.clone();
            }
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

    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    fn get_enabled(&self) -> bool {
        return self.enabled;
    }

    fn get_storage(scene: &Scene) -> Box<&dyn ComponentStorage<Self::ComponentType>> {
        return Box::new(&scene.mesh);
    }

    fn get_storage_mut(scene: &mut Scene) -> Box<&mut dyn ComponentStorage<Self::ComponentType>> {
        return Box::new(&mut scene.mesh);
    }
}

impl DetailsUI for MeshComponent {
    fn draw_details_ui(&mut self, ui: &mut Ui) {
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
