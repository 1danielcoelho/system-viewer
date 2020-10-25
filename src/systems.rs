use egui::Ui;
use web_sys::WebGlRenderingContext;

use crate::components::{ComponentManager, MeshComponent, TransformComponent, UIComponent};

pub struct SystemManager {
    render: RenderingSystem,
    interface: InterfaceSystem,
}
impl SystemManager {
    pub fn new() -> Self {
        return Self {
            render: RenderingSystem {},
            interface: InterfaceSystem {},
        };
    }

    // TODO: Make some "context" object that has mut refs to everything and is created every frame
    pub fn run(&self, ui: &egui::Ui, ctx: &WebGlRenderingContext, comp_man: &mut ComponentManager) {
        self.render.run(ctx, &comp_man.transform, &comp_man.mesh);
        self.interface.run(ui, &comp_man.transform, &comp_man.interface);
    }
}

pub struct RenderingSystem {}
impl RenderingSystem {
    pub fn run(&self, ctx: &WebGlRenderingContext, transforms: &Vec<TransformComponent>, meshes: &Vec<MeshComponent>) {
        // do the drawing
    }
}

pub struct InterfaceSystem {}
impl InterfaceSystem {
    pub fn run(&self, ui: &Ui, transforms: &Vec<TransformComponent>, uis: &Vec<UIComponent>) {
        // do the UI drawing
    }
}
