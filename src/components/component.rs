use std::collections::HashMap;

use egui::Ui;

use crate::managers::{ECManager, Entity};

#[derive(Debug)]
pub enum ComponentType {
    Transform,
    Mesh,
    Physics,
    Light,
}

pub enum ComponentStorageType {
    Vec,
    HashMap,
}

pub trait Component: Default + Clone {
    type ComponentType;
    const STORAGE_TYPE: ComponentStorageType;
    const COMPONENT_TYPE: ComponentType;

    fn set_enabled(&mut self, enabled: bool);
    fn get_enabled(&mut self) -> bool;

    fn get_components_vector<'a>(
        _w: &'a mut ECManager,
    ) -> Option<&'a mut Vec<Self::ComponentType>> {
        return None;
    }

    fn get_components_map<'a>(
        _w: &'a mut ECManager,
    ) -> Option<&'a mut HashMap<Entity, Self::ComponentType>> {
        return None;
    }

    fn draw_details_ui(&mut self, ui: &mut Ui) {        
    }
}
