use crate::components::Component;
use crate::managers::scene::component_storage::ComponentStorage;
use crate::managers::scene::Entity;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct HashStorage<T: Component> {
    storage: HashMap<Entity, T>,
}
impl<T: Component> HashStorage<T> {
    pub fn new() -> Self {
        Self::default()
    }

    fn iter_components(&self) -> std::collections::hash_map::Iter<'_, Entity, T> {
        return self.storage.iter();
    }

    fn iter_components_mut(&mut self) -> std::collections::hash_map::IterMut<'_, Entity, T> {
        return self.storage.iter_mut();
    }
}

impl<T: Component> ComponentStorage<T> for HashStorage<T> {
    fn add_component(&mut self, entity: Entity) -> &T {
        assert!(!self.storage.contains_key(&entity));

        self.storage.insert(entity, T::default());

        return self.storage.get(&entity).unwrap();
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
}
