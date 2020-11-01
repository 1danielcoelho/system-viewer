use crate::managers::ComponentManager;

pub enum ComponentIndex {
    Transform = 0,
    Mesh = 1,
    Physics = 2,
    Ui = 3,
}

pub trait Component: Default {
    type ComponentType;

    fn set_enabled(&mut self, enabled: bool);
    fn get_enabled(&mut self) -> bool;

    fn get_component_index() -> ComponentIndex;
    fn get_components_vector<'a>(w: &'a mut ComponentManager) -> &'a mut Vec<Self::ComponentType>;
}