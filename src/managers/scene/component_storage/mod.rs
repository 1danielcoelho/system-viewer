use crate::components::Component;
use crate::managers::scene::Entity;

pub mod hash_storage;
pub mod packed_storage;

pub trait ComponentStorage<T: Component> {
    fn add_component(&mut self, entity: Entity) -> &T;
    fn get_component(&self, entity: Entity) -> Option<&T>;
    fn get_component_mut(&mut self, entity: Entity) -> Option<&mut T>;
    fn has_component(&self, entity: Entity) -> bool;
    fn remove_component(&mut self, entity: Entity);
}
