use crate::components::Component;
use crate::managers::scene::component_storage::ComponentStorage;
use crate::managers::scene::Entity;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct PackedStorage<T: Component> {
    storage: Vec<T>,
    index_to_entity: Vec<Entity>,

    #[serde(skip)] // Can rebuild this one from the inverse map
    entity_to_index: HashMap<Entity, u32>,
}
impl<T: Component> PackedStorage<T> {
    pub fn new() -> Self {
        Self::default()
    }

    fn get_component_owner(&self, index: u32) -> Option<Entity> {
        return self.index_to_entity.get(index as usize).cloned();
    }

    pub fn iter(&self) -> std::slice::Iter<T> {
        return self.storage.iter();
    }

    pub fn iter_mut(&mut self) -> std::slice::IterMut<T> {
        return self.storage.iter_mut();
    }

    pub fn copy_from_other(&mut self, other: &PackedStorage<T>, other_entity_to_entity: &HashMap<Entity, Entity>) {
        let num_new_entries = other.storage.len();
        
        self.storage.reserve(num_new_entries);
        self.index_to_entity.reserve(num_new_entries);
        self.entity_to_index.reserve(num_new_entries);
        
        self.storage.append(&mut other.storage.clone());

        for other_ent in other.index_to_entity {
            let our_ent = other_entity_to_entity[&other_ent];

            self.index_to_entity.push(our_ent);
            self.entity_to_index.insert(our_ent, (self.entity_to_index.len() - 1) as u32);
        }
    }

    pub fn move_from_other(&mut self, other: PackedStorage<T>, other_entity_to_entity: &HashMap<Entity, Entity>) {
        let num_new_entries = other.storage.len();
        
        self.storage.reserve(num_new_entries);
        self.index_to_entity.reserve(num_new_entries);
        self.entity_to_index.reserve(num_new_entries);
        
        self.storage.append(&mut other.storage);

        for other_ent in other.index_to_entity {
            let our_ent = other_entity_to_entity[&other_ent];

            self.index_to_entity.push(our_ent);
            self.entity_to_index.insert(our_ent, (self.entity_to_index.len() - 1) as u32);
        }
    }
}

impl<T: Component> ComponentStorage<T> for PackedStorage<T> {
    fn add_component(&mut self, entity: Entity) -> &mut T {
        assert!(!self.entity_to_index.contains_key(&entity));

        self.storage.push(T::default());
        self.index_to_entity.push(entity);

        let new_index = (self.storage.len() - 1) as u32;
        self.entity_to_index.insert(entity, new_index);

        return &mut self.storage[new_index as usize];
    }

    fn get_component(&self, entity: Entity) -> Option<&T> {
        if let Some(index) = self.entity_to_index.get(&entity) {
            return Some(&self.storage[*index as usize]);
        }

        return None;
    }

    fn get_component_mut(&mut self, entity: Entity) -> Option<&mut T> {
        if let Some(index) = self.entity_to_index.get(&entity) {
            return Some(&mut self.storage[*index as usize]);
        }

        return None;
    }

    fn has_component(&self, entity: Entity) -> bool {
        return self.entity_to_index.contains_key(&entity);
    }

    fn remove_component(&mut self, entity: Entity) {
        if let Some(index) = self.entity_to_index.get(&entity) {
            self.storage.remove(*index as usize);
            self.index_to_entity.remove(*index as usize);
            self.entity_to_index.remove(&entity);
        }
    }

    fn reserve_for_n_more(&mut self, n: usize) {
        self.storage.reserve(n);
        self.index_to_entity.reserve(n);
        self.entity_to_index.reserve(n);
    }

    fn swap_components(&mut self, entity_a: Entity, entity_b: Entity) {
        let index_a = self.entity_to_index[&entity_a];
        let index_b = self.entity_to_index[&entity_b];

        self.storage.swap(index_a as usize, index_b as usize);
    }

    fn get_num_components(&self) -> u32 {
        return self.storage.len() as u32;
    }
}
