use crate::{app_state::AppState, managers::{ComponentManager, EntityManager, EventManager, ResourceManager, SystemManager}, world::World};

pub struct AppUpdateContext<'a> {
    pub ent_man: &'a mut EntityManager,
    pub res_man: &'a mut ResourceManager,
    pub comp_man: &'a mut ComponentManager,
    pub sys_man: &'a mut SystemManager,
    pub event_man: &'a mut EventManager,
}
impl<'a> AppUpdateContext<'a> {
    pub fn new(w: &'a mut World) -> AppUpdateContext {
        AppUpdateContext {
            ent_man: &mut w.ent_man,
            res_man: &mut w.res_man,
            comp_man: &mut w.comp_man,
            sys_man: &mut w.sys_man,
            event_man: &mut w.event_man,
        }
    }

    pub fn update(&mut self, state: &mut AppState) {
        self.sys_man.run(state, self.comp_man, self.event_man);
    }
}
