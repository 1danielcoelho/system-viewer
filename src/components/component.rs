use crate::managers::ComponentManager;

#[repr(u32)]
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ComponentIndex {
    Transform = 0,
    Mesh = 1,
    Physics = 2,
    Ui = 3,
}

pub trait Component: Default + Clone {
    type ComponentType;
    const INDEX: ComponentIndex;

    fn set_enabled(&mut self, enabled: bool);
    fn get_enabled(&mut self) -> bool;

    fn get_components_vector<'a>(w: &'a mut ComponentManager) -> &'a mut Vec<Self::ComponentType>;
}
