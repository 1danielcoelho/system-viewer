use crate::components::{
    Component, MeshComponent, PhysicsComponent, TransformComponent, UIComponent,
};

use super::{Entity, Event, EventReceiver};

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

    pub fn get_component<T>(&mut self, entity: &Entity) -> Option<&T>
    where
        T: Default + Component + Component<ComponentType = T>,
    {
        let comp_vec = T::get_components_vector(self);
        return comp_vec.get(entity.id as usize);
    }

    pub fn add_component<'a, T>(&'a mut self, entity: &mut Entity) -> Option<&'a mut T>
    where
        T: Default + Component + Component<ComponentType = T>,
    {
        // Ensure size. Very temp for now, never shrinks...
        self.resize_components((entity.id + 1) as usize);

        let comp_vec = T::get_components_vector(self);
        comp_vec[entity.id as usize].set_enabled(true);

        return Some(&mut comp_vec[entity.id as usize]);
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
