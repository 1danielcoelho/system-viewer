use std::any::Any;
use std::collections::HashMap;

use anymap::any::CloneAny;
use anymap::Map;

use crate::components::{
    component::ComponentIndex, Component, MeshComponent, PhysicsComponent, TransformComponent,
    UIComponent,
};

use super::{Event, EventReceiver};

pub trait ComponentStorage {
    type ComponentType: Component;

    fn get_component(&self, id: u32) -> Option<&Self::ComponentType>;
    fn get_component_mut(&mut self, id: u32) -> Option<&mut Self::ComponentType>;
    fn add_component(&mut self, id: u32) -> Option<&mut Self::ComponentType>;
}

#[derive(Clone)]
pub struct VecStorage<T: Component> {
    storage: Vec<T>,
}
impl<T: Component> ComponentStorage for VecStorage<T> {
    type ComponentType = T;

    fn get_component(&self, id: u32) -> Option<&Self::ComponentType> {
        return self.storage.get(id as usize);
    }

    fn get_component_mut(&mut self, id: u32) -> Option<&mut Self::ComponentType> {
        return self.storage.get_mut(id as usize);
    }

    fn add_component(&mut self, id: u32) -> Option<&mut Self::ComponentType> {
        let min_length = self.storage.len().max((id + 1) as usize);
        if min_length <= self.storage.len() {
            self.storage
                .resize_with(min_length, Self::ComponentType::default);
        };

        self.storage[id as usize].set_enabled(true);
        return Some(&mut self.storage[id as usize]);
    }
}

#[derive(Clone)]
pub struct HashMapStorage<T: Component> {
    storage: HashMap<usize, T>,
}
impl<T: Component> ComponentStorage for HashMapStorage<T> {
    type ComponentType = T;

    fn get_component(&self, id: u32) -> Option<&Self::ComponentType> {
        return self.storage.get(&(id as usize));
    }

    fn get_component_mut(&mut self, id: u32) -> Option<&mut Self::ComponentType> {
        return self.storage.get_mut(&(id as usize));
    }

    fn add_component(&mut self, id: u32) -> Option<&mut Self::ComponentType> {
        self.storage
            .insert(id as usize, Self::ComponentType::default());
        return self.get_component_mut(id);
    }
}

#[derive(Clone)]
enum StorageType {
    Vec,
    HashMap,
}

#[derive(Clone)]
pub struct ComponentManager {
    pub physics: Vec<PhysicsComponent>,
    pub mesh: Vec<MeshComponent>,
    pub transform: Vec<TransformComponent>,
    pub interface: Vec<UIComponent>,

    pub map: Map<dyn CloneAny>,

    // Lazy way of finding out which storage type we have for which component
    // Ideally we'd store ComponentStorage<T> in self.map, but AnyMap only works for concrete types
    storage_type: HashMap<ComponentIndex, StorageType>,
}
impl ComponentManager {
    pub fn new() -> Self {
        return Self {
            physics: vec![],
            mesh: vec![],
            transform: vec![],
            interface: vec![],
            map: Map::new(),
            storage_type: HashMap::new(),
        };
    }

    pub fn register_vec_storage<T: 'static + Component>(&mut self) -> Result<(), String> {
        if self.storage_type.contains_key(&T::INDEX) {
            return Err(format!(
                "Failed to register vec storage for type {}: Type already registered!",
                T::INDEX as u32
            ));
        }

        let storage: Vec<T> = Vec::new();
        self.map.insert(VecStorage { storage });

        self.storage_type.insert(T::INDEX, StorageType::Vec);

        return Ok(());
    }

    pub fn register_hashmap_storage<T: 'static + Component>(&mut self) -> Result<(), String> {
        if self.storage_type.contains_key(&T::INDEX) {
            return Err(format!(
                "Failed to register vec storage for type {}: Type already registered!",
                T::INDEX as u32
            ));
        }

        let storage: HashMap<usize, T> = HashMap::new();
        self.map.insert(HashMapStorage { storage });

        self.storage_type.insert(T::INDEX, StorageType::HashMap);

        return Ok(());
    }

    pub fn get_component<T: 'static + Component>(&mut self, id: u32) -> Option<&mut T> {
        match self.storage_type.get(&T::INDEX) {
            Some(StorageType::Vec) => {
                return self
                    .map
                    .get_mut::<VecStorage<T>>()
                    .unwrap()
                    .get_component_mut(id)
            }
            Some(StorageType::HashMap) => {
                return self
                    .map
                    .get_mut::<HashMapStorage<T>>()
                    .unwrap()
                    .get_component_mut(id)
            }
            None => return None,
        }
    }

    pub fn add_component<'a, T: 'static + Component>(&'a mut self, id: u32) -> Option<&'a mut T>
    where
        T: Default + Component + Component<ComponentType = T>,
    {
        match self.storage_type.get(&T::INDEX) {
            Some(StorageType::Vec) => {
                return self
                    .map
                    .get_mut::<VecStorage<T>>()
                    .unwrap()
                    .add_component(id)
            }
            Some(StorageType::HashMap) => {
                return self
                    .map
                    .get_mut::<HashMapStorage<T>>()
                    .unwrap()
                    .add_component(id)
            }
            None => return None,
        }
    }

    pub fn swap_components(&mut self, id_a: u32, id_b: u32) {
        if id_a == id_b {
            return;
        }

        // let keys: Vec<&dyn CloneAny> = self.map.as_ref().iter().collect();
        // for key in keys {
        //     let entry = self.map.as_mut().entry(key.type_id());
        // }

        let max_index = id_a.max(id_b);
        self.resize_components((max_index + 1) as usize);

        self.physics.swap(id_a as usize, id_b as usize);
        self.mesh.swap(id_a as usize, id_b as usize);
        self.transform.swap(id_a as usize, id_b as usize);
        self.interface.swap(id_a as usize, id_b as usize);
    }

    fn resize_components(&mut self, min_length: usize) {
        if min_length <= self.physics.len() {
            return;
        }

        self.physics.resize_with(min_length, Default::default);
        self.mesh.resize_with(min_length, Default::default);
        self.transform.resize_with(min_length, Default::default);
        self.interface.resize_with(min_length, Default::default);
    }

    pub fn move_from_other(&mut self, mut other: Self, other_to_this_index: &Vec<u32>) {
        let highest_new_index = other_to_this_index.iter().max().unwrap();
        self.resize_components((highest_new_index + 1) as usize);

        for this_index in other_to_this_index.iter().rev() {
            self.physics[*this_index as usize] = other.physics.pop().unwrap();
            self.mesh[*this_index as usize] = other.mesh.pop().unwrap();
            self.transform[*this_index as usize] = other.transform.pop().unwrap();
            self.interface[*this_index as usize] = other.interface.pop().unwrap();
        }
    }
}

// impl Clone for ComponentManager {
//     fn clone(&self) -> Self {
//         return Self {
//             interface: self.interface.clone(),
//             mesh: self.mesh.clone(),
//             physics: self.physics.clone(),
//             transform: self.transform.clone(),
//             vec_components: self.vec_components.clone_from_slice(src)
//         }
//     }
// }

impl EventReceiver for ComponentManager {
    fn receive_event(&mut self, _event: Event) {
        //
    }
}
