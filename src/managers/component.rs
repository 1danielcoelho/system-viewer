use std::collections::HashMap;

use crate::components::{
    Component, MeshComponent, PhysicsComponent, TransformComponent, UIComponent,
};

use super::{Entity, Event, EventReceiver};

// Blob of weird stuff so that we can clone ComponentStorage
//https://stackoverflow.com/questions/30353462/how-to-clone-a-struct-storing-a-boxed-trait-object
pub trait ComponentStorageClone {
    fn clone_box(&self) -> Box<dyn ComponentStorageBase>;
}
impl<T: 'static + ComponentStorageBase + Clone> ComponentStorageClone for T {
    fn clone_box(&self) -> Box<dyn ComponentStorageBase> {
        Box::new(self.clone())
    }
}
impl Clone for Box<dyn ComponentStorageBase> {
    fn clone(&self) -> Box<dyn ComponentStorageBase> {
        self.clone_box()
    }
}

// Base trait so that we can store type-erased storages in ComponentManager
pub trait ComponentStorageBase : ComponentStorageClone {}
impl<T: ComponentStorage> ComponentStorageBase for T {}

// Trait implemented by any struct that stores cmoponents
pub trait ComponentStorage: ComponentStorageBase {
    type ComponentType: Component;

    fn get_component(&self, index: usize) -> &Self::ComponentType;
}

#[derive(Clone)]
pub struct VecStorage<T: Component> {
    storage: Vec<T>,
}
impl<T: 'static + Component> ComponentStorage for VecStorage<T> {
    type ComponentType = T;

    fn get_component(&self, index: usize) -> &Self::ComponentType {
        return &self.storage[index];
    }
}

#[derive(Clone)]
pub struct HashMapStorage<T: Component> {
    storage: HashMap<usize, T>,
}
impl<T: 'static + Component> ComponentStorage for HashMapStorage<T> {
    type ComponentType = T;

    fn get_component(&self, index: usize) -> &Self::ComponentType {
        return &self.storage[&index];
    }
}

#[derive(Clone)]
pub struct ComponentManager {
    pub physics: Vec<PhysicsComponent>,
    pub mesh: Vec<MeshComponent>,
    pub transform: Vec<TransformComponent>,
    pub interface: Vec<UIComponent>,

    pub storages: HashMap<u32, Box<dyn ComponentStorageBase>>,
}
impl ComponentManager {
    pub fn new() -> Self {
        let storage1: Vec<PhysicsComponent> = Vec::new();
        let storage2: Vec<TransformComponent> = Vec::new();
        let storage3: HashMap<usize, TransformComponent> = HashMap::new();

        let mut storages: HashMap<u32, Box<dyn ComponentStorageBase>> = HashMap::new();
        storages.insert(0, Box::new(VecStorage { storage: storage1 }));
        storages.insert(1, Box::new(VecStorage { storage: storage2 }));
        storages.insert(2, Box::new(HashMapStorage { storage: storage3 }));

        return Self {
            physics: vec![],
            mesh: vec![],
            transform: vec![],
            interface: vec![],
            storages,
        };
    }

    pub fn get_component<T>(&mut self, entity: u32) -> Option<&mut T>
    where
        T: Default + Component + Component<ComponentType = T>,
    {
        let comp_vec = T::get_components_vector(self);
        return comp_vec.get_mut(entity as usize);
    }

    pub fn add_component<'a, T>(&'a mut self, entity: u32) -> Option<&'a mut T>
    where
        T: Default + Component + Component<ComponentType = T>,
    {
        // Ensure size. Very temp for now, never shrinks...
        self.resize_components((entity + 1) as usize);

        let comp_vec = T::get_components_vector(self);
        comp_vec[entity as usize].set_enabled(true);

        return Some(&mut comp_vec[entity as usize]);
    }

    pub fn swap_components(&mut self, index_a: u32, index_b: u32) {
        if index_a == index_b {
            return;
        }

        let max_index = index_a.max(index_b);
        self.resize_components((max_index + 1) as usize);

        self.physics.swap(index_a as usize, index_b as usize);
        self.mesh.swap(index_a as usize, index_b as usize);
        self.transform.swap(index_a as usize, index_b as usize);
        self.interface.swap(index_a as usize, index_b as usize);
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
