use std::{cell::RefCell, rc::Rc};

use crate::{components::ComponentManager, entity::EntityManager, resources::ResourceManager};

pub struct World {
    pub ent_man: EntityManager,
    pub res_man: ResourceManager,
    pub comp_man: ComponentManager,
    //pub scene_man: SceneManager,
}
impl World {
    pub fn new() -> Rc<RefCell<Self>> {
        let new_world = Rc::new(RefCell::new(Self {
            ent_man: EntityManager::new(),
            res_man: ResourceManager::new(),
            comp_man: ComponentManager::new(),
        }));

        new_world.borrow_mut().ent_man.set_world(Rc::downgrade(&new_world));

        return new_world;
    }
}


// let world = self.world.unwrap().upgrade().unwrap().borrow_mut();
//         let entity = world.ent_man.get_entity(entity_id);