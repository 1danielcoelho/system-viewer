use crate::components::{
    Component, LightComponent, MeshComponent, OrbitalComponent, PhysicsComponent,
    TransformComponent,
};
use crate::managers::scene::component_storage::{
    ComponentStorage, HashStorage, PackedStorage, SparseStorage,
};
use crate::managers::scene::serialization::{SerEntity, SerScene};
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::rc::Rc;
use std::{cmp::Reverse, hash::Hash};

#[derive(Debug, Copy, Clone, Eq, Hash, Serialize, Deserialize)]
pub struct Entity(pub u32); // temp
impl PartialEq for Entity {
    fn eq(&self, other: &Self) -> bool {
        return self.0 == other.0;
    }
}

#[derive(Clone)]
pub struct EntityEntry {
    pub live: bool,
    pub current: Entity,

    pub name: Option<String>,
    pub parent: Option<Entity>,
    pub children: Vec<Entity>,
}

#[derive(Clone)]
pub struct Scene {
    pub identifier: String,

    entity_storage: Vec<EntityEntry>,
    free_indices: BinaryHeap<Reverse<u32>>,
    entity_to_index: Rc<RefCell<HashMap<Entity, u32>>>,
    last_used_entity: Entity,

    // Until there is a proper way to split member borrows in Rust I think
    // hard-coding the component types in here is the simplest way of doing things, sadly
    pub physics: PackedStorage<PhysicsComponent>,
    pub mesh: SparseStorage<MeshComponent>,
    pub transform: SparseStorage<TransformComponent>,
    pub light: HashStorage<LightComponent>,
    pub orbital: HashStorage<OrbitalComponent>,

    _private: (),
}

// Main interface
impl Scene {
    pub(super) fn new(identifier: &str) -> Self {
        let entity_to_index = Rc::new(RefCell::new(HashMap::new()));

        Self {
            identifier: identifier.to_string(),
            entity_storage: Vec::new(),
            free_indices: BinaryHeap::new(),
            entity_to_index: entity_to_index.clone(),
            last_used_entity: Entity(0),

            physics: PackedStorage::new(),
            mesh: SparseStorage::new(entity_to_index.clone()),
            transform: SparseStorage::new(entity_to_index.clone()),
            light: HashStorage::new(),
            orbital: HashStorage::new(),
            _private: (),
        }
    }

    pub fn get_entity_entries(&self) -> &Vec<EntityEntry> {
        return &self.entity_storage;
    }

    /** Used when injecting scenes into eachother */
    pub fn move_from_other(&mut self, other_man: Self) {
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
                other_index_to_new_index[other_index as usize] =
                    self.entity_to_index.borrow()[&new_ent];
                other_index_to_new_ent[other_index as usize] = new_ent;
            }

            for (other_ent, other_index) in other_man.entity_to_index.borrow().iter() {
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

        // Move component data over
        self.physics
            .move_from_other(other_man.physics, &other_ent_to_new_ent);
        self.mesh
            .move_from_other(other_man.mesh, &other_index_to_new_index);
        self.transform
            .move_from_other(other_man.transform, &other_index_to_new_index);
        self.light
            .move_from_other(other_man.light, &other_ent_to_new_ent);
        self.orbital
            .move_from_other(other_man.orbital, &other_ent_to_new_ent);

        // TODO: Update entity references in components
    }
}

// Entity interface
impl Scene {
    pub fn reserve_space_for_entities(&mut self, num_new_spaces_to_reserve: u32) {
        let num_missing =
            (num_new_spaces_to_reserve as i64 - self.free_indices.len() as i64).max(0) as usize;

        self.entity_storage.reserve(num_missing);

        self.physics.reserve_for_n_more(num_missing);
        self.transform.reserve_for_n_more(num_missing);
        self.mesh.reserve_for_n_more(num_missing);
        self.light.reserve_for_n_more(num_missing);
        self.orbital.reserve_for_n_more(num_missing);
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
            .borrow_mut()
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
                self.entity_to_index.borrow_mut().remove(&e);

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
        match self.entity_to_index.borrow().get(&e) {
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
        self.entity_to_index.borrow_mut().insert(
            self.entity_storage[source_index as usize].current,
            target_index,
        );
        self.entity_to_index.borrow_mut().insert(
            self.entity_storage[target_index as usize].current,
            source_index,
        );

        // Actually swap the entries
        self.entity_storage
            .swap(source_index as usize, target_index as usize);
    }
}

// Component interface
impl Scene {
    pub fn get_component<T>(&mut self, entity: Entity) -> Option<&mut T>
    where
        T: Component<ComponentType = T>,
    {
        return T::get_storage(self).get_component_mut(entity);
    }

    pub fn add_component<'a, T>(&'a mut self, entity: Entity) -> &'a mut T
    where
        T: Default + Component + Component<ComponentType = T>,
    {
        log::info!(
            "Adding component '{:?}' to entity '{:?}'",
            std::any::type_name::<T>(),
            entity
        );

        return T::get_storage(self).add_component(entity);
    }

    fn swap_components(&mut self, index_a: u32, index_b: u32) {
        if index_a == index_b {
            return;
        }

        // Pretty sure I'm converting to and from entities at least once too many...
        let ent_a = self.get_entity_from_index(index_a).unwrap();
        let ent_b = self.get_entity_from_index(index_b).unwrap();

        let max_index = index_a.max(index_b);
        self.resize_components(max_index + 1);

        self.physics.swap_components(ent_a, ent_b);
        self.mesh.swap_components(ent_a, ent_b);
        self.transform.swap_components(ent_a, ent_b);
        self.orbital.swap_components(ent_a, ent_b);
        self.light.swap_components(ent_a, ent_b);
    }

    /// Resizes sparse storage components to match a target min_length
    fn resize_components(&mut self, min_length: u32) {
        if min_length <= self.transform.get_num_components() {
            return;
        }

        self.transform.resize(min_length);
        self.mesh.resize(min_length);
    }
}

// Serialization interface
impl Scene {
    /** Internal use. Use SceneManager::deserialize_scene */
    pub(super) fn deserialize(ron_str: &str) -> Result<Self, String> {
        let ser_scene: SerScene = ron::de::from_str(ron_str)
            .map_err(|e| format!("RON Deserialization error:\n{}", e).to_owned())?;

        let mut new_scene = Self::new(ser_scene.identifier);
        new_scene.reserve_space_for_entities(ser_scene.entities.len() as u32);

        // First add entries for all entities
        for ser_ent in ser_scene.entities.iter() {
            new_scene.new_entity(ser_ent.name);
        }

        // Parent/child relationships
        for (index, ser_ent) in ser_scene.entities.iter().enumerate() {
            if ser_ent.parent.is_none() {
                continue;
            }

            new_scene.set_entity_parent(
                new_scene
                    .get_entity_from_index(ser_ent.parent.unwrap().0)
                    .unwrap(),
                new_scene.get_entity_from_index(index as u32).unwrap(),
            );
        }

        // Since our scene is empty, we can just use the Entities as they were in the serialized
        // scene, regardless of what that is
        new_scene.physics = ser_scene.physics;
        new_scene.mesh = ser_scene.mesh;
        new_scene
            .mesh
            .set_entity_to_index(new_scene.entity_to_index.clone());
        new_scene.transform = ser_scene.transform;
        new_scene
            .transform
            .set_entity_to_index(new_scene.entity_to_index.clone());
        new_scene.light = ser_scene.light;
        new_scene.orbital = ser_scene.orbital;

        return Ok(new_scene);
    }

    pub fn serialize(&self) -> String {
        // For the serialized scene we temporarily use Entities as if they're just indices
        let _unused = Rc::new(RefCell::new(HashMap::new()));
        let mut ser_scene = SerScene {
            identifier: &self.identifier,
            entities: Vec::new(),
            physics: PackedStorage::new(),
            mesh: SparseStorage::new(_unused.clone()),
            transform: SparseStorage::new(_unused.clone()),
            light: HashStorage::new(),
            orbital: HashStorage::new(),
        };

        let mut index_to_packed_index: Vec<u32> = Vec::new();
        let mut ent_to_packed_ent: HashMap<Entity, Entity> = HashMap::new();

        index_to_packed_index.resize(self.entity_storage.len(), 0);
        ser_scene.entities.reserve(self.entity_storage.len());

        // Add packed entities, still referencing indices of parent/childs from the non-serialized scene
        for (index, entry) in self.entity_storage.iter().enumerate() {
            if !entry.live {
                continue;
            }

            let packed_ent = Entity(index as u32);
            let parent_index = entry.parent.and_then(|p| self.get_entity_index(p));
            let child_indices: Vec<Entity> = entry
                .children
                .iter()
                .map(|c| Entity(self.get_entity_index(*c).unwrap()))
                .collect();

            index_to_packed_index[index] = ser_scene.entities.len() as u32;

            ser_scene.entities.push(SerEntity {
                name: entry.name.as_ref().and_then(|n| Some(n.as_ref())),
                entity: packed_ent,
                parent: parent_index.and_then(|p| Some(Entity(p))),
                children: child_indices,
            });

            ent_to_packed_ent.insert(entry.current, packed_ent);
        }

        // Now that all our entities are packed, update the parent/child references to the packed entities vec
        for ent in ser_scene.entities.iter_mut() {
            ent.parent = ent
                .parent
                .and_then(|p| Some(Entity(index_to_packed_index[p.0 as usize])));

            ent.children = ent
                .children
                .iter()
                .map(|c| Entity(index_to_packed_index[c.0 as usize]))
                .collect();
        }

        ser_scene
            .physics
            .copy_from_other(&self.physics, &ent_to_packed_ent);
        ser_scene
            .mesh
            .copy_from_other(&self.mesh, &index_to_packed_index);
        ser_scene
            .transform
            .copy_from_other(&self.transform, &index_to_packed_index);
        ser_scene
            .light
            .copy_from_other(&self.light, &ent_to_packed_ent);
        ser_scene
            .orbital
            .copy_from_other(&self.orbital, &ent_to_packed_ent);

        return ron::ser::to_string_pretty(&ser_scene, ron::ser::PrettyConfig::new()).unwrap();
    }
}
