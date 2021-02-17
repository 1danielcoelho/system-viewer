use crate::managers::details_ui::DetailsUI;
use crate::managers::scene::component_storage::ComponentStorage;
use crate::managers::scene::Scene;

pub trait Component: Default + Clone + DetailsUI {
    type ComponentType;

    fn set_enabled(&mut self, enabled: bool);
    fn get_enabled(&self) -> bool;

    fn get_storage(scene: &Scene) -> Box<&dyn ComponentStorage<Self::ComponentType>>;
    fn get_storage_mut(scene: &mut Scene) -> Box<&mut dyn ComponentStorage<Self::ComponentType>>;
}
