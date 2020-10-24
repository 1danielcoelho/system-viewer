use std::{collections::HashMap};

use crate::{components::NUM_COMPONENTS};

pub struct Entity {
    pub id: u32,
    pub component_ids: [u32; NUM_COMPONENTS],
    pub name: String,
}

pub struct EntityManager {
    last_id: u32,
    entities: HashMap<u32, Entity>,
}

impl EntityManager {
    pub fn new() -> Self {
        Self {
            last_id: 1,
            entities: HashMap::new(),
        }
    }

    pub fn new_entity(&mut self, name: &str) -> &mut Entity {
        self.last_id += 1;

        self.entities.insert(
            self.last_id,
            Entity {
                id: self.last_id,
                component_ids: [0; NUM_COMPONENTS],
                name: String::from(name),
            },
        );
        return self
            .get_entity(&(self.last_id - 1))
            .expect("Weirdness in new_entity");
    }

    pub fn get_entity(&mut self, id: &u32) -> Option<&mut Entity> {
        return self.entities.get_mut(id);
    }
}
