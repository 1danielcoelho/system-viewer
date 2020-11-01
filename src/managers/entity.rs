use std::collections::HashMap;

pub struct Entity {
    pub id: u32,
    pub name: String,
}

pub struct EntityManager {
    last_id: u32,
    entities: HashMap<u32, Entity>,
}

impl EntityManager {
    pub fn new() -> Self {
        Self {
            last_id: 0,
            entities: HashMap::new(),
        }
    }

    pub fn new_entity(&mut self, name: &str) -> &mut Entity {
        log::info!("new_entity with name '{}' and id {}", name, self.last_id);
        
        self.entities.insert(
            self.last_id,
            Entity {
                id: self.last_id,
                name: String::from(name),
            },
        );
        self.last_id += 1;
        
        return self
            .get_entity(self.last_id - 1)
            .expect("Weirdness in new_entity");
    }

    pub fn get_entity(&mut self, id: u32) -> Option<&mut Entity> {
        return self.entities.get_mut(&id);
    }
}
