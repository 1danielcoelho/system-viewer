use crate::components::Component;
use crate::managers::scene::Entity;

mod hash_storage;
mod packed_storage;
mod sparse_storage;

pub use hash_storage::*;
pub use packed_storage::*;
pub use sparse_storage::*;

pub trait ComponentStorage<T: Component> {
    fn add_component(&mut self, entity: Entity) -> &mut T;
    fn get_component(&self, entity: Entity) -> Option<&T>;
    fn get_component_mut(&mut self, entity: Entity) -> Option<&mut T>;
    fn has_component(&self, entity: Entity) -> bool;
    fn remove_component(&mut self, entity: Entity);
    fn swap_components(&mut self, entity_a: Entity, entity_b: Entity);
    fn get_num_components(&self) -> u32;

    fn reserve_for_n_more(&mut self, n: usize);
}
