use crate::components::{Component, TransformComponent};
use crate::managers::scene::Entity;
use anymap::any::CloneAny;
use std::cell::{Ref, RefCell};
use std::collections::HashMap;

#[derive(Clone)]
pub struct VecStorage<T> {
    storage: Vec<T>,
}
impl<T> VecStorage<T> {
    fn new() -> Self {
        Self {
            storage: Vec::new(),
        }
    }

    fn add_component(&mut self, entity: Entity) {
        todo!()
    }

    fn remove_component(&mut self, entity: Entity) {
        todo!()
    }

    fn has_component(&self, entity: Entity) -> bool {
        todo!()
    }

    fn get_component(&self, entity: Entity) -> Option<&T> {
        todo!()
    }

    fn get_component_mut(&mut self, entity: Entity) -> Option<&mut T> {
        todo!()
    }

    fn get_num_entries(&self) -> u32 {
        todo!()
    }

    fn clear(&self) {
        todo!()
    }

    fn iter(&self) -> std::slice::Iter<T> {
        return self.storage.iter();
    }

    fn iter_mut(&mut self) -> std::slice::IterMut<T> {
        return self.storage.iter_mut();
    }
}

#[derive(Clone)]
pub struct PackedVecStorage<T> {
    storage: Vec<T>,
}
impl<T> PackedVecStorage<T> {
    fn new() -> Self {
        Self {
            storage: Vec::new(),
        }
    }

    fn add_component(&mut self, entity: Entity) {
        todo!()
    }

    fn remove_component(&mut self, entity: Entity) {
        todo!()
    }

    fn has_component(&self, entity: Entity) -> bool {
        todo!()
    }

    fn get_component(&self, entity: Entity) -> Option<&T> {
        todo!()
    }

    fn get_component_mut(&mut self, entity: Entity) -> Option<&mut T> {
        todo!()
    }

    fn get_num_entries(&self) -> u32 {
        todo!()
    }

    fn clear(&self) {
        todo!()
    }

    fn iter(&self) -> std::slice::Iter<T> {
        return self.storage.iter();
    }

    fn iter_mut(&mut self) -> std::slice::IterMut<T> {
        return self.storage.iter_mut();
    }
}

#[derive(Clone)]
pub struct HashStorage<T> {
    storage: HashMap<Entity, T>,
}
impl<T> HashStorage<T> {
    fn new() -> Self {
        Self {
            storage: HashMap::new(),
        }
    }

    fn add_component(&mut self, entity: Entity) {
        todo!()
    }

    fn remove_component(&mut self, entity: Entity) {
        todo!()
    }

    fn has_component(&self, entity: Entity) -> bool {
        todo!()
    }

    fn get_component(&self, entity: Entity) -> Option<&T> {
        todo!()
    }

    fn get_component_mut(&mut self, entity: Entity) -> Option<&mut T> {
        todo!()
    }

    fn get_num_entries(&self) -> u32 {
        todo!()
    }

    fn clear(&self) {
        todo!()
    }

    fn iter(&self) -> std::collections::hash_map::Iter<Entity, T> {
        return self.storage.iter();
    }

    fn iter_mut(&mut self) -> std::collections::hash_map::IterMut<Entity, T> {
        return self.storage.iter_mut();
    }
}

pub enum ComponentIterator<'a, T: 'a> {
    VecIterator(std::slice::Iter<'a, T>),
    HashIterator(std::collections::hash_map::Iter<'a, Entity, T>),
}

pub enum ComponentIteratorMut<'a, T: 'a> {
    VecIterator(std::slice::IterMut<'a, T>),
    HashIterator(std::collections::hash_map::IterMut<'a, Entity, T>),
}

#[derive(Clone)]
pub struct FakeScene {
    pub components: anymap::Map<dyn CloneAny>,
}

impl<'a> FakeScene {
    pub fn register_vec<T: Component + 'static>(&mut self) {
        self.components.insert(RefCell::new(VecStorage::<T>::new()));
    }

    pub fn register_packed_vec<T: Component + 'static>(&mut self) {
        self.components
            .insert(RefCell::new(PackedVecStorage::<T>::new()));
    }

    pub fn register_hash<T: Component + 'static>(&mut self) {
        self.components
            .insert(RefCell::new(HashStorage::<T>::new()));
    }

    pub fn add_component<T: Component + 'static>(&self, ent: Entity) {
        if let Some(vec_storage) = self.components.get::<RefCell<VecStorage<T>>>() {
            vec_storage.borrow_mut().add_component(ent);
        } else if let Some(hash_storage) = self.components.get::<RefCell<HashStorage<T>>>() {
            hash_storage.borrow_mut().add_component(ent);
        }
    }

    pub fn remove_component<T: Component + 'static>(&self, ent: Entity) {
        if let Some(vec_storage) = self.components.get::<RefCell<VecStorage<T>>>() {
            vec_storage.borrow_mut().remove_component(ent);
        } else if let Some(hash_storage) = self.components.get::<RefCell<HashStorage<T>>>() {
            hash_storage.borrow_mut().remove_component(ent);
        }
    }

    pub fn has_component<T: Component + 'static>(&self, ent: Entity) -> bool {
        if let Some(vec_storage) = self.components.get::<VecStorage<T>>() {
            return vec_storage.has_component(ent);
        } else if let Some(hash_storage) = self.components.get::<HashStorage<T>>() {
            return hash_storage.has_component(ent);
        }

        return false;
    }

    pub fn get_component<T: Component + 'static>(&'a self, ent: Entity) -> Option<Ref<T>> {
        if let Some(vec_storage) = self.components.get::<RefCell<VecStorage<T>>>() {
            let storage = vec_storage.borrow();
            let ref_to_opt = Ref::map(storage, |s| &s.get_component(ent));            

            return Some(ref_to_opt);
        }
        // else if let Some(hash_storage) = self.components.get::<RefCell<HashStorage<T>>>() {
        //     return hash_storage.borrow().get_component(ent);
        // }

        return None;
    }

    // pub fn get_component_mut<T: Component + 'static>(&self, ent: Entity) -> Option<&mut T> {
    //     if let Some(vec_storage) = self.components.get::<RefCell<VecStorage<T>>>() {
    //         return vec_storage.borrow_mut().get_component_mut(ent);
    //     } else if let Some(hash_storage) = self.components.get::<RefCell<HashStorage<T>>>() {
    //         return hash_storage.borrow_mut().get_component_mut(ent);
    //     }

    //     return None;
    // }

    pub fn iter_components<T: Component + 'static>(&self) -> Option<ComponentIterator<T>> {
        if let Some(vec_storage) = self.components.get::<VecStorage<T>>() {
            return Some(ComponentIterator::VecIterator(vec_storage.iter()));
        } else if let Some(hash_storage) = self.components.get::<HashStorage<T>>() {
            return Some(ComponentIterator::HashIterator(hash_storage.iter()));
        }

        return None;
    }

    pub fn test(&mut self) {
        self.register_vec::<TransformComponent>();
        self.add_component::<TransformComponent>(Entity(2));

        let comp = self.get_component::<TransformComponent>(Entity(2));

        // TODO: Try making a ComponentStorage iterator, have FakeScene just deal with dyn ComponentStorage instead
        // return ComponentStorage references instead of iter_components

        // match self.iter_components::<TransformComponent>().unwrap() {
        //     ComponentIterator::VecIterator(iter) => {
        //         for comp in iter {
        //             //
        //         }
        //     }
        //     ComponentIterator::HashIterator(iter) => {
        //         for (key, comp) in iter {
        //             //
        //         }
        //     }
        // }
    }
}
