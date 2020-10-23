use std::{collections::HashMap, sync::Arc};

use crate::{
    components::{Component, NUM_COMPONENTS},
    materials::SimpleMaterial,
    mesh::Mesh,
};

pub static EntityManagerInstance: EntityManager = EntityManager::new();

pub trait Drawable3D {
    fn draw(&self);
}

pub trait DrawableUI {
    fn draw(&self, ui: &egui::Ui);
}

pub struct Entity {
    id: u32,
    component_ids: [u32; NUM_COMPONENTS],
    name: String,
}
impl Entity {
    pub fn new(name: &str) -> &Self {
        let new_entity = Self {
            id: 0,
            component_ids: [0; NUM_COMPONENTS],
            name: String::from(name),
        };

        let ent_man = &EntityManagerInstance;
        return ent_man.register(new_entity).unwrap();
    }

    pub fn get_component<T: Component + Component<ComponentType = T> + 'static>(
        &self,
    ) -> Option<&T> {
        let comp_id = self.component_ids[T::get_component_index() as usize];
        if comp_id == 0 {
            return None;
        };

        return T::get_components_vector().get(comp_id as usize);
    }

    pub fn add_component<T: Default + Component + Component<ComponentType = T> + 'static>(
        &self,
    ) -> Option<&T> {
        let comp_id = self.component_ids[T::get_component_index() as usize];
        if comp_id != 0 {
            log::info!("Tried to add a repeated component");
            return self.get_component();
        };

        let comp_vec = T::get_components_vector();
        comp_vec.push(T::default());

        self.component_ids[T::get_component_index() as usize] = (comp_vec.len() - 1) as u32;

        return comp_vec.last();
    }

    // For builder pattern
    pub fn with_component<T: Default + Component + Component<ComponentType = T> + 'static>(
        &self,
    ) -> &Self {
        self.add_component::<T>();
        return &self;
    }
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

    pub fn register(&self, entity: Entity) -> Option<&Entity> {
        entity.id = self.last_id;
        self.entities.insert(self.last_id, entity);

        self.last_id += 1;
        return self.entities.get(&(self.last_id - 1));
    }
    pub fn get_entity(&self, id: &u32) -> Option<&Entity> {
        return self.entities.get(id);
    }
}
