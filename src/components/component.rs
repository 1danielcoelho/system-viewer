use std::collections::HashMap;

use crate::managers::{ECManager, Entity};

#[derive(Debug)]
pub enum ComponentIndex {
    Transform,
    Mesh,
    Physics,
    Ui,
    Light,
}

pub enum ComponentStorageType {
    Vec,
    HashMap,
}

pub trait Component: Default + Clone {
    type ComponentType;
    const STORAGE_TYPE: ComponentStorageType;

    fn set_enabled(&mut self, enabled: bool);
    fn get_enabled(&mut self) -> bool;

    fn get_component_index() -> ComponentIndex;

    fn get_components_vector<'a>(w: &'a mut ECManager) -> Option<&'a mut Vec<Self::ComponentType>> {
        return None;
    }

    fn get_components_map<'a>(
        w: &'a mut ECManager,
    ) -> Option<&'a mut HashMap<Entity, Self::ComponentType>> {
        return None;
    }
}
