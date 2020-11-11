use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap};

use super::ComponentManager;

#[derive(Debug, Copy, Clone, Eq)]
pub struct Entity {
    id: u32, // Actual component index in the component arrays. Not pub so user has to go through manager
    gen: u32, // Generation of the currently live entity
    uuid: u32, // Unique index for this entity, follows entities when they're sorted
}
impl PartialEq for Entity {
    // Completely ignore id and gen, as we may be comparing an old reference to this entity
    // with a newer reference, even though both reference the same entity
    fn eq(&self, other: &Self) -> bool {
        return self.uuid == other.uuid;
    }
}

pub struct EntityEntry {
    live: bool,
    gen: u32,
    uuid: u32,
}

pub struct EntityManager {
    // Current entities and spaces are available here
    entity_storage: Vec<EntityEntry>,

    // Deallocated spaces of entity_storage: These will be reused first when allocating
    free_indices: BinaryHeap<Reverse<u32>>,

    // Maps from an entity uuid to it's current index in entity_storage
    uuid_to_index: HashMap<u32, u32>,
    last_used_uuid: u32,
}

impl EntityManager {
    pub fn new() -> Self {
        Self {
            entity_storage: Vec::new(),
            free_indices: BinaryHeap::new(),
            uuid_to_index: HashMap::new(),
            last_used_uuid: 0,
        }
    }

    // Returns a new Entity. It may occupy an existing, vacant spot or be a brand new
    // reallocation. Repeatedly calling this always returns entities with increasing indices
    pub fn new_entity(&mut self) -> Entity {
        let target_index = match self.free_indices.pop() {
            Some(Reverse { 0: vacant_index }) => {
                self.entity_storage[vacant_index as usize].gen += 1;
                vacant_index
            }
            None => {
                self.entity_storage.push(EntityEntry {
                    live: true,
                    gen: 0,
                    uuid: 0,
                });
                (self.entity_storage.len() - 1) as u32
            }
        };

        let new_entity: &mut EntityEntry = &mut self.entity_storage[target_index as usize];

        self.last_used_uuid += 1;
        new_entity.uuid = self.last_used_uuid;
        self.uuid_to_index.insert(self.last_used_uuid, target_index);

        return Entity {
            id: target_index,
            gen: new_entity.gen,
            uuid: new_entity.uuid,
        };
    }

    pub fn delete_entity(&mut self, e: &Entity) -> bool {
        match self.get_entity_index(e) {
            Some(index) => {
                self.entity_storage[index as usize].live = false;
                self.free_indices.push(Reverse(index));
                self.uuid_to_index.remove(&e.uuid);
                return true;
            }
            None => return false,
        };
    }

    pub fn get_entity_index(&self, e: &Entity) -> Option<u32> {
        let mut stored_entity = self.entity_storage.get(e.id as usize);
        match stored_entity {
            Some(se) => {
                if se.gen == e.gen {
                    return Some(e.id);
                } else {
                    stored_entity = None;
                }
            }
            None => {}
        }

        // Stale reference, try searching the right entity via uuid
        if stored_entity.is_none() || stored_entity.unwrap().gen != e.gen {
            return self.get_entity_index_by_uuid(e.uuid);
        };

        return None;
    }

    // Also updates Entity reference so that it doesn't have to re-check by uuid every time
    pub fn get_entity_index_update_ref(&self, e: &mut Entity) -> Option<u32> {
        let index_op = self.get_entity_index(e);

        if let Some(index) = index_op {
            e.id = index;
            e.gen = self.entity_storage[index as usize].gen;
        }

        return index_op;
    }

    pub fn get_entity_index_by_uuid(&self, uuid: u32) -> Option<u32> {
        match self.uuid_to_index.get(&uuid) {
            Some(index) => {
                let ent = &self.entity_storage[(*index) as usize];
                if ent.live {
                    return Some(*index);
                } else {
                    return None;
                }
            }
            None => return None,
        }
    }

    pub fn is_live(&self, id: u32) -> bool {
        match self.entity_storage.get(id as usize) {
            Some(entry) => entry.live,
            None => false,
        }
    }

    pub fn get_num_entities(&self) -> u32 {
        return self.entity_storage.len() as u32;
    }

    pub fn swap_entity_indices(&mut self, a: &Entity, b: &Entity, comp_man: &mut ComponentManager) {
        let index_a = self.get_entity_index(a).unwrap();
        let index_b = self.get_entity_index(b).unwrap();
        if index_a == index_b {
            return;
        }

        self.entity_storage.swap(index_a as usize, index_b as usize);
        self.entity_storage[index_a as usize].gen += 1;
        self.entity_storage[index_b as usize].gen += 1;

        comp_man.swap_components(index_a, index_b);
    }
    
    // Moves `a` into `stomped`'s location in the storage, effectively deleting `stomped` and vacating `a`'s previous location
    fn stomp_entity(&mut self, a: &Entity, stomped: &Entity) {
        todo!();

        // Don't forget to clear uuid and its entry in the map. Maybe call delete entity
    }

    // Temp solution to make sure that all parent transforms come before child transforms
    // I think the final solution to this will involve using events, but I'm not sure how that
    // will work until I have some more use cases
    fn sort_entities(&mut self, comp_man: &mut ComponentManager) {

        // Collect hierarchy parents that need to be relocated
        
        // Breadth first traversal on each hierarchy parent to store their indices

        // Reserve size for entities at once

        // For each hierarchy's sorted indices
            // Repeatedly call new_entity and stomp_entity in order 
    }
}
