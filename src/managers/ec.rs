use std::collections::{BinaryHeap, HashMap, HashSet};

use std::{cmp::Reverse, hash::Hash};

use crate::components::{
    component::ComponentStorageType, Component, LightComponent, MeshComponent, PhysicsComponent,
    TransformComponent,
};

#[derive(Debug, Copy, Clone, Eq, Hash)]
pub struct Entity(u32);
impl PartialEq for Entity {
    fn eq(&self, other: &Self) -> bool {
        return self.0 == other.0;
    }
}

#[derive(Clone)]
pub struct EntityEntry {
    live: bool,
    current: Entity,

    name: Option<String>,
    parent: Option<Entity>,
    children: Vec<Entity>,
}

#[derive(Clone)]
pub struct ECManager {
    entity_storage: Vec<EntityEntry>,
    free_indices: BinaryHeap<Reverse<u32>>,
    pub entity_to_index: HashMap<Entity, u32>,
    last_used_entity: Entity,

    // TODO: Maybe put this in a wrapper struct or?
    pub physics: Vec<PhysicsComponent>,
    pub mesh: Vec<MeshComponent>,
    pub transform: Vec<TransformComponent>,
    pub light: HashMap<Entity, LightComponent>,
}

impl ECManager {
    pub fn new() -> Self {
        Self {
            entity_storage: Vec::new(),
            free_indices: BinaryHeap::new(),
            entity_to_index: HashMap::new(),
            last_used_entity: Entity(0),

            physics: Vec::new(),
            mesh: Vec::new(),
            transform: Vec::new(),
            light: HashMap::new(),
        }
    }

    pub fn reserve_space_for_entities(&mut self, num_new_spaces_to_reserve: u32) {
        let num_missing =
            (num_new_spaces_to_reserve as i64 - self.free_indices.len() as i64).max(0);

        self.entity_storage.reserve(num_missing as usize);
    }

    fn new_entity_at_index(&mut self, entity_storage_index: u32, name: Option<&str>) -> Entity {
        if entity_storage_index >= self.entity_storage.len() as u32 {
            assert!(
                self.entity_storage.len() < 500 || entity_storage_index < (2 * self.entity_storage.len() as u32),
                format!("Trying to create new entity at an unreasonable index {}! (we currently have {})", entity_storage_index, self.entity_storage.len())
            );

            self.entity_storage.resize(
                (entity_storage_index + 1) as usize,
                EntityEntry {
                    live: false,
                    name: None,
                    current: Entity(0),
                    parent: None,
                    children: Vec::new(),
                },
            );
        }

        self.last_used_entity.0 += 1;

        let new_entry: &mut EntityEntry = &mut self.entity_storage[entity_storage_index as usize];
        new_entry.current = self.last_used_entity;
        new_entry.live = true;
        new_entry.name = Some(String::from(name.unwrap_or_default()));

        self.entity_to_index
            .insert(self.last_used_entity, entity_storage_index);

        log::info!(
            "Creating new entity {:?}: '{}'",
            new_entry.current,
            new_entry.name.as_ref().unwrap_or(&String::new()),
        );

        return new_entry.current;
    }

    // Returns a new Entity. It may occupy an existing, vacant spot or be a brand new
    // reallocation. Repeatedly calling this always returns entities with increasing indices
    pub fn new_entity(&mut self, name: Option<&str>) -> Entity {
        let target_index = match self.free_indices.pop() {
            Some(Reverse { 0: vacant_index }) => vacant_index,
            None => self.entity_storage.len() as u32,
        };

        return self.new_entity_at_index(target_index, name);
    }

    pub fn get_entity_name(&self, entity: Entity) -> Option<&str> {
        match self.get_entity_entry(entity) {
            Some(entry) => match entry.name {
                Some(_) => Some(entry.name.as_ref().unwrap()),
                None => None,
            },
            None => None,
        }
    }

    // Returns a new Entity guaranteed to occupy an index than 'reference'
    fn new_entity_larger_than(&mut self, reference: u32) -> u32 {
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
            for i in (reference + 1)..(self.entity_storage.len() as u32) {
                if !self.entity_storage[i as usize].live {
                    target_index = Some(i);
                    break;
                }
            }
        }

        // No vacant spots, signal that we should resize to fit it
        if target_index.is_none() {
            target_index = Some(self.entity_storage.len() as u32);
        };

        return target_index.unwrap();
    }

    pub fn delete_entity(&mut self, e: Entity) -> bool {
        log::info!("Deleting entity {:?}", e);
        match self.get_entity_index(e) {
            Some(index) => {
                let cloned_entry = self.entity_storage[index as usize].clone();

                // Remove ourselves from our parent
                if let Some(parent) = self.entity_storage[index as usize].parent {
                    let parent_entry = self.get_entity_entry_mut(parent).unwrap();
                    let child_index = parent_entry
                        .children
                        .iter()
                        .position(|&c| c == cloned_entry.current)
                        .unwrap();
                    parent_entry.children.remove(child_index);
                }

                // Delete our children with us
                for child_entity in cloned_entry.children.iter() {
                    self.delete_entity(*child_entity);
                }

                let entry_ref = &mut self.entity_storage[index as usize];
                entry_ref.live = false;
                entry_ref.current.0 = 0;

                self.free_indices.push(Reverse(index));
                self.entity_to_index.remove(&e);

                return true;
            }
            None => return false,
        };
    }

    pub fn is_live(&self, e: Entity) -> bool {
        match self.get_entity_entry(e) {
            Some(entry) => entry.live,
            None => false,
        }
    }

    pub fn get_entity_index(&self, e: Entity) -> Option<u32> {
        match self.entity_to_index.get(&e) {
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

    // Returns None if it's not live anymore
    fn get_entity_entry(&self, e: Entity) -> Option<&EntityEntry> {
        // Find the actual index as this may be a stale entity reference
        let index = self.get_entity_index(e);
        if index.is_none() {
            return None;
        }

        // Fetch the guaranteed current entry for this entity
        return self.entity_storage.get(index.unwrap() as usize);
    }

    fn get_entity_entry_mut(&mut self, e: Entity) -> Option<&mut EntityEntry> {
        // Find the actual index as this may be a stale entity reference
        let index = self.get_entity_index(e);
        if index.is_none() {
            return None;
        }

        // Fetch the guaranteed current entry for this entity
        return self.entity_storage.get_mut(index.unwrap() as usize);
    }

    pub fn get_entity_from_index(&self, index: u32) -> Option<Entity> {
        match self.entity_storage.get(index as usize) {
            Some(entry) => {
                if entry.live {
                    return Some(entry.current);
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
                Some(parent) => return self.get_entity_index(parent),
                None => return None,
            },
            None => return None,
        };
    }

    pub fn get_entity_parent(&self, e: Entity) -> Option<Entity> {
        match self.get_entity_entry(e) {
            Some(entry) => entry.parent,
            None => None,
        }
    }

    // Topmost parent of the entity's hierarchy. May be the same entity if `e` has no parents
    pub fn get_entity_ancestor(&self, e: Entity) -> Entity {
        let mut parent = self.get_entity_parent(e);

        while parent.is_some() {
            let parent_parent = self.get_entity_parent(parent.unwrap());

            if parent_parent.is_none() {
                return parent.unwrap();
            }

            parent = parent_parent;
        }

        return e.clone();
    }

    pub fn get_entity_children(&self, e: Entity) -> Option<&Vec<Entity>> {
        match self.get_entity_entry(e) {
            Some(entry) => Some(&entry.children),
            None => None,
        }
    }

    pub fn get_num_entities(&self) -> u32 {
        return self.entity_storage.len() as u32;
    }

    pub fn set_entity_parent(&mut self, parent: Entity, child: Entity) {
        let parent_index = self.get_entity_index(parent);
        let child_index = self.get_entity_index(child);
        if parent_index.is_none() || child_index.is_none() {
            return;
        }

        // Update old parent
        if let Some(old_parent) = self.get_entity_parent(child) {
            if old_parent == parent {
                return;
            }

            if let Some(entry) = self.get_entity_entry_mut(old_parent) {
                let old_child_index = entry.children.iter().position(|&c| c == child).unwrap();
                entry.children.remove(old_child_index);
            };
        }

        // Update new parent
        if let Some(entry) = self.entity_storage.get_mut(parent_index.unwrap() as usize) {
            let old_child_index = entry.children.iter().position(|&c| c == child);
            assert!(
                old_child_index.is_none(),
                format!("Entity {:#?} was already child of {:#?}!", child, parent)
            );

            entry.children.push(child);
        };

        // Update child
        if let Some(entry) = self.entity_storage.get_mut(child_index.unwrap() as usize) {
            entry.parent = Some(parent);
        }

        // Need to always guarantee parent < child so that when we compute transforms every frame we
        // can compute them all in one pass with minimal referencing
        self.ensure_parent_child_order(parent_index.unwrap(), child_index.unwrap());
    }

    // Will make sure that child comes after it's parent in the entity and component arrays, also acting recursively
    fn ensure_parent_child_order(&mut self, parent_index: u32, child_index: u32) {
        if child_index > parent_index {
            return;
        }

        // Create a new dummy entity in a position where child would be valid (i.e. after parent)
        let new_child_index = self.new_entity_larger_than(parent_index);

        // Swap current child with dummy, along with components (this will resize components if needed too)
        self.swap_entities(new_child_index, child_index);
        self.swap_components(new_child_index, child_index);

        // Get rid of dummy (just mark it as vacant, really)
        self.delete_entity(self.get_entity_from_index(child_index).unwrap());

        // Propagate for all grandchildren in the child hierarchy
        let grand_child_indices: Vec<u32> = self.entity_storage[new_child_index as usize]
            .children
            .iter()
            .map(|e| self.get_entity_index(*e).unwrap())
            .collect();
        for grand_child_index in grand_child_indices.iter() {
            self.ensure_parent_child_order(new_child_index, *grand_child_index);
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
        // TODO: Very slow.. reconstructing entire binary heap when only two elements changed.
        // We can't remove things from heap though
        for item in free_indices_set.iter() {
            self.free_indices.push(*item);
        }

        // Update index map
        self.entity_to_index.insert(
            self.entity_storage[source_index as usize].current,
            target_index,
        );
        self.entity_to_index.insert(
            self.entity_storage[target_index as usize].current,
            source_index,
        );

        // Actually swap the entries
        self.entity_storage
            .swap(source_index as usize, target_index as usize);
    }

    /** Used when injecting scenes into eachother */
    pub fn move_from_other(&mut self, mut other_man: Self) {
        // Better off going one by one, as trying to find a block large enough to fit other_man at once may be too slow,
        // and reallocating a new block every time would lead to unbounded memory usage. This way we also promote entity
        // packing

        log::info!("Moving from other");

        // Allocate new entities, keep an array of their newly allocated indices
        let num_other_ents = other_man.get_num_entities();
        self.reserve_space_for_entities(num_other_ents);

        // Create mapping... maps
        let mut other_index_to_new_index: Vec<u32> = Vec::new();
        let mut other_ent_to_new_ent: HashMap<Entity, Entity> = HashMap::new();
        {
            let mut other_index_to_new_ent: Vec<Entity> = Vec::new();
            other_index_to_new_index.resize(num_other_ents as usize, 0);
            other_index_to_new_ent.resize(num_other_ents as usize, Entity(0));

            for other_index in 0..num_other_ents {
                let other_name = other_man.get_entity_from_index(other_index).unwrap();
                let new_ent = self.new_entity(other_man.get_entity_name(other_name));
                other_index_to_new_index[other_index as usize] = self.entity_to_index[&new_ent];
                other_index_to_new_ent[other_index as usize] = new_ent;
            }

            for (other_ent, other_index) in other_man.entity_to_index.iter() {
                other_ent_to_new_ent
                    .insert(*other_ent, other_index_to_new_ent[*other_index as usize]);
            }
        }

        // Replicate parent-child relationships
        for other_index in 0..num_other_ents {
            if let Some(other_parent_index) = other_man.get_parent_index_from_index(other_index) {
                let new_index = other_index_to_new_index[other_index as usize];
                let new_parent_index = other_index_to_new_index[other_parent_index as usize];

                // TODO: This could be optimized
                let child_ent = self.get_entity_from_index(new_index).unwrap();
                let parent_ent = self.get_entity_from_index(new_parent_index).unwrap();
                self.set_entity_parent(parent_ent, child_ent);
            }
        }

        // Move vec component data
        let highest_new_id = other_index_to_new_index.iter().max().unwrap();
        self.resize_components((highest_new_id + 1) as usize);
        for this_index in other_index_to_new_index.iter().rev() {
            self.physics[*this_index as usize] = other_man.physics.pop().unwrap();
            self.mesh[*this_index as usize] = other_man.mesh.pop().unwrap();
            self.transform[*this_index as usize] = other_man.transform.pop().unwrap();
        }

        // Move hashmap component data
        // for (other_ent, new_ent) in other_ent_to_new_ent.iter() {
        //     if let Some(comp) = other_man.interface.remove(other_ent) {
        //         self.interface.insert(*new_ent, comp);
        //     }
        // }

        // TODO: Update entity references in components
    }
}

// Component interface
impl ECManager {
    pub fn get_component<T>(&mut self, entity: Entity) -> Option<&mut T>
    where
        T: Default + Component + Component<ComponentType = T>,
    {
        match T::STORAGE_TYPE {
            ComponentStorageType::Vec => {
                if let Some(entity_index) = self.get_entity_index(entity) {
                    let comps = T::get_components_vector(self).unwrap();
                    return comps.get_mut(entity_index as usize);
                };

                return None;
            }
            ComponentStorageType::HashMap => {
                let comps = T::get_components_map(self).unwrap();
                return comps.get_mut(&entity);
            }
        }
    }

    pub fn add_component<'a, T>(&'a mut self, entity: Entity) -> Option<&'a mut T>
    where
        T: Default + Component + Component<ComponentType = T>,
    {
        log::info!(
            "Adding component '{:?}' to entity '{:?}'",
            T::COMPONENT_TYPE,
            entity
        );

        match T::STORAGE_TYPE {
            ComponentStorageType::Vec => {
                let index = self.entity_to_index[&entity];
                self.resize_components((index + 1) as usize);

                let comps = T::get_components_vector(self).unwrap();

                comps[index as usize].set_enabled(true);
                return comps.get_mut(index as usize);
            }
            ComponentStorageType::HashMap => {
                let comps = T::get_components_map(self).unwrap();

                if !comps.contains_key(&entity) {
                    comps.insert(entity, T::default());
                }

                let existing = comps.get_mut(&entity).unwrap();
                existing.set_enabled(true);
                return Some(existing);
            }
        }
    }

    fn swap_components(&mut self, index_a: u32, index_b: u32) {
        if index_a == index_b {
            return;
        }

        // Pretty sure I'm converting to and from entities at least once too many...
        // let ent_a = self.get_entity_from_index(index_a).unwrap();
        // let ent_b = self.get_entity_from_index(index_b).unwrap();

        let max_index = index_a.max(index_b);
        self.resize_components((max_index + 1) as usize);

        self.physics.swap(index_a as usize, index_b as usize);
        self.mesh.swap(index_a as usize, index_b as usize);
        self.transform.swap(index_a as usize, index_b as usize);

        // Swap hashmap component storages
        // let int_a = self.interface.remove(&ent_a);
        // let int_b = self.interface.remove(&ent_b);
        // if let Some(int_a) = int_a {
        //     self.interface.insert(ent_b, int_a);
        // }
        // if let Some(int_b) = int_b {
        //     self.interface.insert(ent_a, int_b);
        // }
    }

    fn resize_components(&mut self, min_length: usize) {
        if min_length <= self.physics.len() {
            return;
        }

        self.physics.resize_with(min_length, Default::default);
        self.mesh.resize_with(min_length, Default::default);
        self.transform.resize_with(min_length, Default::default);
    }
}
