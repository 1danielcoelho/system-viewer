use crate::components::{
    Component, MeshComponent, PhysicsComponent, TransformComponent, UIComponent,
};

use super::{Entity, EntityManager, Event, EventReceiver};

pub struct ComponentManager {
    pub physics: Vec<PhysicsComponent>,
    pub mesh: Vec<MeshComponent>,
    pub transform: Vec<TransformComponent>,
    pub interface: Vec<UIComponent>,
}
impl ComponentManager {
    pub fn new() -> Self {
        return Self {
            physics: vec![],
            mesh: vec![],
            transform: vec![],
            interface: vec![],
        };
    }

    pub fn get_component<T>(&mut self, entity: u32) -> Option<&T>
    where
        T: Default + Component + Component<ComponentType = T>,
    {
        let comp_vec = T::get_components_vector(self);
        return comp_vec.get(entity as usize);
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

    fn parent_entity(&mut self, parent: Entity, child: Entity, ent_man: &EntityManager) {
        let parent_id = ent_man.get_entity_index(&parent).unwrap();
        let child_id = ent_man.get_entity_index(&child).unwrap();

        // Update old parent
        if let Some(old_parent) = &self.transform[child_id as usize].parent {
            if *old_parent == parent {
                return;
            }

            let old_parent_id = ent_man.get_entity_index(&old_parent).unwrap();

            let old_parent_comp = &mut self.transform[old_parent_id as usize];

            let old_child_index = old_parent_comp
                .children
                .iter()
                .position(|&c| c == child)
                .unwrap();
            old_parent_comp.children.remove(old_child_index);
        }

        // Update new parent
        {
            let new_parent_comp = &mut self.transform[parent_id as usize];

            // Make sure we don't double-add an entity somehow
            let old_child_index = new_parent_comp.children.iter().position(|&c| c == child);
            assert!(
                old_child_index.is_none(),
                format!("Entity {:#?} was already child of {:#?}!", child, parent)
            );

            new_parent_comp.children.push(child);
        }

        // Update child
        {
            let new_child_comp = &mut self.transform[child_id as usize];
            new_child_comp.parent = Some(parent);
        }

        // TODO: Maybe send an event to ourselves here to remember to notify_transforms_reparented?
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
}
impl EventReceiver for ComponentManager {
    fn receive_event(&mut self, event: Event) {
        //
    }
}
