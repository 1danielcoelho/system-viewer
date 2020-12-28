use crate::managers::{
    details_ui::DetailsUI,
    scene::{Entity, Scene},
};
use std::collections::HashMap;

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

pub trait Component: Default + Clone + DetailsUI {
    type ComponentType;
    const STORAGE_TYPE: ComponentStorageType;
    const COMPONENT_TYPE: ComponentType;

    fn set_enabled(&mut self, enabled: bool);
    fn get_enabled(&mut self) -> bool;

    fn get_components_vector<'a>(_s: &'a mut Scene) -> Option<&'a mut Vec<Self::ComponentType>> {
        return None;
    }

    fn get_components_map<'a>(
        _s: &'a mut Scene,
    ) -> Option<&'a mut HashMap<Entity, Self::ComponentType>> {
        return None;
    }
}
