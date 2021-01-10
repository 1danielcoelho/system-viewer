use crate::components::Component;
use crate::managers::scene::component_storage::ComponentStorage;
use crate::managers::scene::Entity;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct PackedStorage<T: Component> {
    #[serde(skip)] // Can rebuild this one from the inverse map
    entity_to_index: HashMap<Entity, u32>,
    index_to_entity: Vec<Entity>,
    storage: Vec<T>,
}
impl<T: Component> PackedStorage<T> {
    pub fn new() -> Self {
        Self::default()
    }

    fn get_component_owner(&self, index: u32) -> Option<Entity> {
        return self.index_to_entity.get(index as usize).cloned();
    }

    fn iter_components(&self) -> std::slice::Iter<T> {
        return self.storage.iter();
    }

    fn iter_components_mut(&mut self) -> std::slice::IterMut<T> {
        return self.storage.iter_mut();
    }
}

impl<T: Component> ComponentStorage<T> for PackedStorage<T> {
    fn add_component(&mut self, entity: Entity) -> &T {
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
}
