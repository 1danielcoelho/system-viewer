use crate::{app_state::AppState, managers::ECManager};
use web_sys::WebGl2RenderingContext;

type GL = WebGl2RenderingContext;

pub struct MouseInteractionSystem {}
impl MouseInteractionSystem {
    pub fn new() -> Self {
        return Self {};
    }

    pub fn run(&self, state: &AppState, ent_man: &mut ECManager) {
        // Early out if mouse is over UI (somehow detect that)

        // Raycast over all components

        // Check the raycast hit entity: If it has a MouseInteraction component then call its component's functions
    }
}
