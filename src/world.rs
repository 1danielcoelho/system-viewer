use crate::{
    components::ComponentManager, entity::EntityManager, resources::ResourceManager,
    systems::SystemManager,
};

pub struct World {
    pub ent_man: EntityManager,
    pub res_man: ResourceManager,
    pub comp_man: ComponentManager,
    pub sys_man: SystemManager,
    //pub scene_man: SceneManager,
}
impl World {
    pub fn new() -> Self {
        let new_world = Self {
            ent_man: EntityManager::new(),
            res_man: ResourceManager::new(),
            comp_man: ComponentManager::new(),
            sys_man: SystemManager::new(),
        };

        return new_world;
    }
}

// let world = self.world.unwrap().upgrade().unwrap().borrow_mut();
//         let entity = world.ent_man.get_entity(entity_id);
