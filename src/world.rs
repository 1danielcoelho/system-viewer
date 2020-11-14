use crate::{
    app_state::AppState,
    managers::{
        ComponentManager, EntityManager, EventManager, InputManager, ResourceManager, SystemManager,
    },
};

pub struct World {
    pub ent_man: EntityManager,
    pub res_man: ResourceManager,
    pub comp_man: ComponentManager,
    pub sys_man: SystemManager,
    pub event_man: EventManager,
    pub in_man: InputManager,
    //pub scene_man: SceneManager,
}
impl World {
    pub fn new() -> Self {
        let new_world = Self {
            ent_man: EntityManager::new(),
            res_man: ResourceManager::new(),
            comp_man: ComponentManager::new(),
            sys_man: SystemManager::new(),
            event_man: EventManager::new(),
            in_man: InputManager::new(),
        };

        return new_world;
    }

    pub fn init(&mut self) {
        
    }

    pub fn update(&mut self, state: &mut AppState) {
        self.in_man.run(state);
        self.sys_man
            .run(state, &mut self.comp_man, &mut self.event_man);
    }
}

// let world = self.world.unwrap().upgrade().unwrap().borrow_mut();
//         let entity = world.ent_man.get_entity(entity_id);
