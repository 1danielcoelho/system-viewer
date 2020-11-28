use std::collections::{BinaryHeap, HashMap, HashSet};

use std::{cmp::Reverse, hash::Hash};

use super::ComponentManager;

#[derive(Debug, Copy, Clone, Eq, Hash)]
pub struct EntityID(u32);
impl PartialEq for EntityID {
    fn eq(&self, other: &Self) -> bool {
        return self.0 == other.0;
    }
}

#[derive(Debug, Copy, Clone, Eq)]
pub struct Entity {
    index: u32, // Actual component index in the component arrays. Not pub so user has to go through manager
    uuid: EntityID, // Unique index for this entity, follows entities when they're sorted
}
impl PartialEq for Entity {
    // Completely ignore id and gen, as we may be comparing an old reference to this entity
    // with a newer reference, even though both reference the same entity
    fn eq(&self, other: &Self) -> bool {
        return self.uuid == other.uuid;
    }
}

#[derive(Clone)]
pub struct EntityEntry {
    live: bool,
    uuid: EntityID,

    parent: Option<Entity>,
    children: Vec<Entity>,
}

#[derive(Clone)]
pub struct EntityManager {
    // Current entities and spaces are available here
    entity_storage: Vec<EntityEntry>,

    // Deallocated spaces of entity_storage: These will be reused first when allocating
    free_indices: BinaryHeap<Reverse<u32>>,

    // Maps from an entity uuid to it's current index in entity_storage
    uuid_to_index: HashMap<EntityID, u32>,
    last_used_uuid: EntityID,
}

impl EntityManager {
    pub fn new() -> Self {
        Self {
            entity_storage: Vec::new(),
            free_indices: BinaryHeap::new(),
            uuid_to_index: HashMap::new(),
            last_used_uuid: EntityID(0),
        }
    }

    pub fn reserve_space_for_entities(&mut self, num_new_spaces_to_reserve: u32) {
        let num_missing =
            (num_new_spaces_to_reserve as i64 - self.free_indices.len() as i64).max(0);

        self.entity_storage.reserve(num_missing as usize);
    }

    fn new_entity_at_index(&mut self, entity_storage_index: u32) -> Entity {
        if entity_storage_index >= self.entity_storage.len() as u32 {
            assert!(
                entity_storage_index < 2 * self.entity_storage.len() as u32,
                "Trying to create new entity at an unreasonable index!"
            );

            self.entity_storage.resize(
                (entity_storage_index + 1) as usize,
                EntityEntry {
                    live: false,
                    uuid: EntityID(0),
                    parent: None,
                    children: Vec::new(),
                },
            );
        }

        self.last_used_uuid.0 += 1;

        let new_entity: &mut EntityEntry = &mut self.entity_storage[entity_storage_index as usize];
        new_entity.uuid = self.last_used_uuid;
        new_entity.live = true;

        self.uuid_to_index
            .insert(self.last_used_uuid, entity_storage_index);

        return Entity {
            index: entity_storage_index,
            uuid: new_entity.uuid,
        };
    }

    // Returns a new Entity. It may occupy an existing, vacant spot or be a brand new
    // reallocation. Repeatedly calling this always returns entities with increasing indices
    pub fn new_entity(&mut self) -> Entity {
        let target_index = match self.free_indices.pop() {
            Some(Reverse { 0: vacant_index }) => vacant_index,
            None => {
                self.entity_storage.push(EntityEntry {
                    live: true,
                    uuid: EntityID(0),
                    parent: None,
                    children: Vec::new(),
                });
                (self.entity_storage.len() - 1) as u32
            }
        };

        return self.new_entity_at_index(target_index);
    }

    // Returns a new Entity guaranteed to occupy a spot larger than 'reference'
    fn new_entity_larger_than(&mut self, reference: u32) -> Entity {
        let mut target_index: Option<u32> = None;

        // Checking free indices may be faster
        if (self.entity_storage.len() as u32) - reference > self.free_indices.len() as u32 {
            for i in self.free_indices.iter() {
                if i.0 > reference {
                    target_index = Some(i.0);
                    break;
                }
            }
        }
        // Checking storage directly may be faster
        else {
            for i in reference..(self.entity_storage.len() as u32) {
                if i > reference {
                    target_index = Some(i);
                    break;
                }
            }
        }

        // No vacant spots, signal that we should resize to fit it
        if target_index.is_none() {
            target_index = Some(self.entity_storage.len() as u32);
        };

        return self.new_entity_at_index(target_index.unwrap());
    }

    pub fn delete_entity(&mut self, e: &Entity) -> bool {
        match self.get_entity_index(e) {
            Some(index) => {
                let stored_clone = self.entity_storage[index as usize].clone();

                // Remove ourselves from our parent
                if let Some(parent) = self.entity_storage[index as usize].parent {
                    let parent_entry = self.get_entity_entry_mut(&parent).unwrap();
                    let child_index = parent_entry
                        .children
                        .iter()
                        .position(|&c| c.uuid == stored_clone.uuid)
                        .unwrap();
                    parent_entry.children.remove(child_index);
                }

                // Delete our children with us
                for child_entity in stored_clone.children.iter() {
                    self.delete_entity(child_entity);
                }

                let stored_entity = &mut self.entity_storage[index as usize];
                stored_entity.live = false;
                stored_entity.uuid.0 = 0;

                self.free_indices.push(Reverse(index));
                self.uuid_to_index.remove(&e.uuid);

                return true;
            }
            None => return false,
        };
    }

    pub fn is_live(&self, e: &Entity) -> bool {
        match self.get_entity_entry(e) {
            Some(entry) => entry.live,
            None => false,
        }
    }

    pub fn get_entity_index(&self, e: &Entity) -> Option<u32> {
        return self.get_entity_index_by_uuid(e.uuid);
    }

    // Checks via uuid, returns None if it's not live anymore
    fn get_entity_entry(&self, e: &Entity) -> Option<&EntityEntry> {
        // Find the actual index as this may be a stale entity reference
        let index = self.get_entity_index(e);
        if index.is_none() {
            return None;
        }

        // Fetch the guaranteed current entry for this entity
        return self.entity_storage.get(index.unwrap() as usize);
    }

    fn get_entity_entry_mut(&mut self, e: &Entity) -> Option<&mut EntityEntry> {
        // Find the actual index as this may be a stale entity reference
        let index = self.get_entity_index(e);
        if index.is_none() {
            return None;
        }

        // Fetch the guaranteed current entry for this entity
        return self.entity_storage.get_mut(index.unwrap() as usize);
    }

    fn get_entity_from_index(&self, index: u32) -> Option<Entity> {
        match self.entity_storage.get(index as usize) {
            Some(entry) => {
                if entry.live {
                    return Some(Entity {
                        index,
                        uuid: entry.uuid,
                    });
                } else {
                    return None;
                }
            }
            None => {
                return None;
            }
        };
    }

    // Weird hacky function to quickly get the parent index for TransformUpdateSystem
    pub fn get_parent_index_from_index(&self, entity_index: u32) -> Option<u32> {
        match self.entity_storage.get(entity_index as usize) {
            Some(entry) => match entry.parent {
                Some(parent) => return self.get_entity_index(&parent),
                None => return None,
            },
            None => return None,
        };
    }

    pub fn get_entity_index_by_uuid(&self, uuid: EntityID) -> Option<u32> {
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

    pub fn get_entity_parent(&self, e: &Entity) -> Option<Entity> {
        match self.get_entity_entry(e) {
            Some(entry) => entry.parent,
            None => None,
        }
    }

    // Topmost parent of the entity's hierarchy. May be the same entity if `e` has no parents
    pub fn get_entity_ancestor(&self, e: &Entity) -> Entity {
        let mut parent = self.get_entity_parent(&e);

        while parent.is_some() {
            let parent_parent = self.get_entity_parent(parent.as_ref().unwrap());

            if parent_parent.is_none() {
                return parent.unwrap();
            }

            parent = parent_parent;
        }

        return e.clone();
    }

    pub fn get_entity_children(&self, e: &Entity) -> Option<&Vec<Entity>> {
        match self.get_entity_entry(e) {
            Some(entry) => Some(&entry.children),
            None => None,
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

        comp_man.swap_components(index_a, index_b);
    }

    pub fn set_entity_parent(
        &mut self,
        parent: &Entity,
        child: &Entity,
        comp_man: &mut ComponentManager,
    ) {
        let parent_index = self.get_entity_index(parent);
        let child_index = self.get_entity_index(child);
        if parent_index.is_none() || child_index.is_none() {
            return;
        }

        // Update old parent
        if let Some(old_parent) = self.get_entity_parent(child) {
            if old_parent == *parent {
                return;
            }

            if let Some(entry) = self.get_entity_entry_mut(&old_parent) {
                let old_child_index = entry.children.iter().position(|&c| c == *child).unwrap();
                entry.children.remove(old_child_index);
            };
        }

        // Update new parent
        if let Some(entry) = self.entity_storage.get_mut(parent_index.unwrap() as usize) {
            let old_child_index = entry.children.iter().position(|&c| c == *child);
            assert!(
                old_child_index.is_none(),
                format!("Entity {:#?} was already child of {:#?}!", child, parent)
            );

            entry.children.push(*child);
        };

        // Update child
        if let Some(entry) = self.entity_storage.get_mut(child_index.unwrap() as usize) {
            entry.parent = Some(*parent);
        }

        // Need to always guarantee parent > child so that when we compute transforms every frame we
        // can compute them all in one pass with minimal referencing
        self.ensure_parent_child_order(parent_index.unwrap(), child_index.unwrap(), comp_man);
    }

    // Will make sure that `e` comes after it's parent in the entity and component arrays, also acting recursively
    fn ensure_parent_child_order(
        &mut self,
        parent_index: u32,
        child_index: u32,
        comp_man: &mut ComponentManager,
    ) {
        if child_index > parent_index {
            return;
        }

        // Create a new dummy entity in a position where child would be valid (i.e. after parent)
        let dummy_child = self.new_entity_larger_than(parent_index);

        // Swap current child with dummy, along with components (this will resize components if needed too)
        let new_child_index = dummy_child.index; // Since we just made this entity we know the index is OK
        self.swap_entities(new_child_index, child_index);
        comp_man.swap_components(new_child_index, child_index);

        // Get rid of dummy (just mark it as vacant, really)
        self.delete_entity(self.get_entity_from_index(child_index).as_ref().unwrap());

        // Propagate for all children in the hierarchy
        let grand_child_indices: Vec<u32> = self.entity_storage[new_child_index as usize]
            .children
            .iter()
            .map(|e| self.get_entity_index(e).unwrap())
            .collect();
        for grand_child_index in grand_child_indices.iter() {
            self.ensure_parent_child_order(new_child_index, *grand_child_index, comp_man);
        }
    }

    // WARNING: Will obviously not swap the components along with it
    fn swap_entities(&mut self, source_index: u32, target_index: u32) {
        if source_index == target_index {
            return;
        }

        // Update free indices (ugh)
        let mut free_indices_set: HashSet<Reverse<u32>> = HashSet::new();
        free_indices_set.reserve(self.free_indices.len());
        for item in self.free_indices.iter() {
            free_indices_set.insert(*item);
        }
        let had_src = free_indices_set.contains(&Reverse(source_index));
        let had_tar = free_indices_set.contains(&Reverse(target_index));
        free_indices_set.remove(&Reverse(source_index));
        free_indices_set.remove(&Reverse(target_index));
        if had_src {
            free_indices_set.insert(Reverse(target_index));
        }
        if had_tar {
            free_indices_set.insert(Reverse(source_index));
        }
        self.free_indices.clear();
        self.free_indices.reserve(free_indices_set.len());
        for item in free_indices_set.iter() {
            self.free_indices.push(*item);
        }

        // Update uuid map
        self.uuid_to_index.insert(
            self.entity_storage[source_index as usize].uuid,
            target_index,
        );
        self.uuid_to_index.insert(
            self.entity_storage[target_index as usize].uuid,
            source_index,
        );

        // Actually swap the entries
        self.entity_storage
            .swap(source_index as usize, target_index as usize);
    }

    /** Used when injecting scenes into eachother */
    pub fn move_from_other(
        &mut self,
        other_man: Self,
        comp_man: &mut ComponentManager,
    ) -> Vec<u32> {
        // Better off going one by one, as trying to find a block large enough to fit other_man at once may be too slow,
        // and reallocating a new block every time would lead to unbounded memory usage. This way we also promote entity
        // packing

        // Allocate new entities, keep an array of their newly allocated indices
        let num_other_ents = other_man.get_num_entities();

        let mut other_to_new_index: Vec<u32> = Vec::new();
        other_to_new_index.resize(num_other_ents as usize, 0);

        self.reserve_space_for_entities(num_other_ents);
        for other_index in 0..num_other_ents {
            let new_ent = self.new_entity();
            other_to_new_index[other_index as usize] = new_ent.index;
        }

        // Replicate parent-child relationships
        for other_index in 0..num_other_ents {
            if let Some(other_parent_index) = other_man.get_parent_index_from_index(other_index) {
                let new_index = other_to_new_index[other_index as usize];
                let new_parent_index = other_to_new_index[other_parent_index as usize];

                // This could be optimized
                let child_ent = self.get_entity_from_index(new_index).unwrap();
                let parent_ent = self.get_entity_from_index(new_parent_index).unwrap();
                self.set_entity_parent(&parent_ent, &child_ent, comp_man);
            }
        }

        return other_to_new_index;
    }
}
