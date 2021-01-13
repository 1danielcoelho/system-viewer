use crate::components::Component;
use crate::managers::scene::component_storage::ComponentStorage;
use crate::managers::scene::Entity;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct HashStorage<T: Component> {
    storage: HashMap<Entity, T>,
}
impl<T: Component> HashStorage<T> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn iter(&self) -> std::collections::hash_map::Iter<'_, Entity, T> {
        return self.storage.iter();
    }

    pub fn iter_mut(&mut self) -> std::collections::hash_map::IterMut<'_, Entity, T> {
        return self.storage.iter_mut();
    }

    pub fn copy_from_other(&mut self, other: &HashStorage<T>, other_entity_to_entity: &HashMap<Entity, Entity>) {
        let num_new_entries = other.storage.len();
        
        self.storage.reserve(num_new_entries);
        
        for (other_ent, comp) in other.storage.drain() {
            let our_ent = other_entity_to_entity[&other_ent];

            self.storage.insert(our_ent, comp.clone());
        }
    }

    pub fn move_from_other(&mut self, other: HashStorage<T>, other_entity_to_entity: &HashMap<Entity, Entity>) {
        let num_new_entries = other.storage.len();
        
        self.storage.reserve(num_new_entries);
        
        for (other_ent, comp) in other.storage.drain() {
            let our_ent = other_entity_to_entity[&other_ent];

            self.storage.insert(our_ent, comp);
        }
    }
}

impl<T: Component> ComponentStorage<T> for HashStorage<T> {
    fn add_component(&mut self, entity: Entity) -> &mut T {
        assert!(!self.storage.contains_key(&entity));

        self.storage.insert(entity, T::default());

        return self.storage.get_mut(&entity).unwrap();
    }

    fn get_component(&self, entity: Entity) -> Option<&T> {
        return self.storage.get(&entity);
    }

    fn get_component_mut(&mut self, entity: Entity) -> Option<&mut T> {
        return self.storage.get_mut(&entity);
    }

    fn has_component(&self, entity: Entity) -> bool {
        return self.storage.contains_key(&entity);
    }

    fn remove_component(&mut self, entity: Entity) {
        self.storage.remove(&entity);
    }

    fn reserve_for_n_more(&mut self, n: usize) {
        self.storage.reserve(n);
    }

    fn swap_components(&mut self, entity_a: Entity, entity_b: Entity) {
        let comp_a = self.storage.remove(&entity_a);
        let comp_b = self.storage.remove(&entity_b);
        
        if let Some(comp_a) = comp_a {
            self.storage.insert(entity_b, comp_a);
        };

        if let Some(comp_b) = comp_b {
            self.storage.insert(entity_a, comp_b);
        };        
    }

    fn get_num_components(&self) -> u32 {
        return self.storage.len() as u32;
    }
}
