use crate::{components::component::ComponentIndex, engine::Engine};
use std::collections::VecDeque;

pub enum EventTransmitter {
    ComponentManager(ComponentIndex),
    EntityManager,
    ResourceManager,
    SceneManager,
    EventManager,
    SystemsEtc,
}

pub enum EventData {
    SetBoundingBox {
        entity: u32,
        mins: cgmath::Vector3<f32>,
        maxs: cgmath::Vector3<f32>,
    },
}

pub struct Event {
    source: EventTransmitter,
    dest: EventTransmitter,
    data: EventData,
}

pub trait EventReceiver {
    fn receive_event(&mut self, event: Event); // Can even add a response here if I wanted!
}

pub struct EventManager {
    queue: VecDeque<Event>,
}
impl EventManager {
    pub fn new() -> EventManager {
        Self {
            queue: VecDeque::new(),
        }
    }

    pub fn push_event(&mut self, event: Event) {
        self.queue.push_front(event);
    }

    pub fn pump_events(&mut self, engine: &mut Engine) {
        while !self.queue.is_empty() {
            if let Some(event) = self.queue.pop_back() {
                EventManager::deliver_event(event, engine);
            }
        }
    }

    fn deliver_event(event: Event, engine: &mut Engine) {
        // match event.dest {
        //     EventTransmitter::ComponentManager(_) => engine.comp_man.receive_event(event),
        //     EventTransmitter::EntityManager => {}
        //     EventTransmitter::ResourceManager => {}
        //     EventTransmitter::SceneManager => {}
        //     EventTransmitter::EventManager => {}
        //     EventTransmitter::SystemsEtc => {}
        // }
    }
}
