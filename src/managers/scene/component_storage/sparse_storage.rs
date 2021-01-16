use crate::components::Component;
use crate::managers::scene::component_storage::ComponentStorage;
use crate::managers::scene::Entity;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct SparseStorage<T: Component> {
    storage: Vec<T>,

    #[serde(skip)]
    entity_to_index: Rc<RefCell<HashMap<Entity, u32>>>,
}
impl<T: Component> SparseStorage<T> {
    pub fn new(entity_to_index: Rc<RefCell<HashMap<Entity, u32>>>) -> Self {
        let mut sparse_storage = Self::default();
        sparse_storage.entity_to_index = entity_to_index;
        return sparse_storage;
    }

    pub fn iter(&self) -> std::slice::Iter<T> {
        return self.storage.iter();
    }

    pub fn iter_mut(&mut self) -> std::slice::IterMut<T> {
        return self.storage.iter_mut();
    }

    pub fn resize(&mut self, target_size: u32) {
        self.storage.resize_with(target_size as usize, T::default);
    }

    pub fn set_entity_to_index(&mut self, entity_to_index: Rc<RefCell<HashMap<Entity, u32>>>) {
        self.entity_to_index = entity_to_index;
    }

    pub fn get_component_from_index(&self, index: u32) -> Option<&T> {
        return Some(&self.storage[index as usize]);
    }

    pub fn get_component_from_index_mut(&mut self, index: u32) -> Option<&mut T> {
        return Some(&mut self.storage[index as usize]);
    }

    /// This assumes that entity_to_index will be updated by the scene
    pub fn copy_from_other(&mut self, other: &SparseStorage<T>, other_index_to_index: &Vec<u32>) {        
        let highest_new_id = other_index_to_index.iter().max().unwrap();        
        self.resize(highest_new_id + 1);

        for (other_index, this_index) in other_index_to_index.iter().enumerate() {
            self.storage[*this_index as usize] = other.storage[other_index].clone();
        }
    }

    /// This assumes that entity_to_index will be updated by the scene
    pub fn move_from_other(&mut self, mut other: SparseStorage<T>, other_index_to_index: &Vec<u32>) {        
        let highest_new_id = other_index_to_index.iter().max().unwrap();        
        self.resize(highest_new_id + 1);

        for this_index in other_index_to_index.iter().rev() {
            self.storage[*this_index as usize] = other.storage.pop().unwrap();
        }
    }
}

impl<T: Component> ComponentStorage<T> for SparseStorage<T> {
    fn add_component(&mut self, entity: Entity) -> &mut T {
        let index = self.entity_to_index.borrow()[&entity];

        let comp = &mut self.storage[index as usize];
        *comp = T::default();
        comp.set_enabled(true);

        return comp;
    }

    fn get_component(&self, entity: Entity) -> Option<&T> {
        if let Some(index) = self.entity_to_index.borrow().get(&entity) {
            return Some(&self.storage[*index as usize]);
        }

        return None;
    }

    fn get_component_mut(&mut self, entity: Entity) -> Option<&mut T> {
        if let Some(index) = self.entity_to_index.borrow().get(&entity) {
            return Some(&mut self.storage[*index as usize]);
        }

        return None;
    }

    fn has_component(&self, entity: Entity) -> bool {
        if let Some(index) = self.entity_to_index.borrow().get(&entity) {
            return self.storage[*index as usize].get_enabled();
        }

        return false;
    }

    fn remove_component(&mut self, entity: Entity) {
        if let Some(index) = self.entity_to_index.borrow().get(&entity) {
            self.storage[*index as usize].set_enabled(false);
        }
    }

    fn reserve_for_n_more(&mut self, n: usize) {
        self.storage.reserve(n);
    }

    fn swap_components(&mut self, entity_a: Entity, entity_b: Entity) {
        let entity_to_index = self.entity_to_index.borrow();
        let index_a = entity_to_index[&entity_a];
        let index_b = entity_to_index[&entity_b];

        self.storage.swap(index_a as usize, index_b as usize);
    }

    fn get_num_components(&self) -> u32 {
        return self.storage.len() as u32;
    }
}
