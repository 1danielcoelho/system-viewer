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
        self.interface
            .run(ui, &comp_man.transform, &comp_man.interface);
    }
}

pub struct RenderingSystem {}
impl RenderingSystem {
    pub fn run(
        &self,
        ctx: &WebGlRenderingContext,
        transforms: &Vec<TransformComponent>,
        meshes: &Vec<MeshComponent>,
    ) {
        self.pre_render(ctx);

        
    }

    fn pre_render(&self, ctx: &WebGlRenderingContext) {
        // Egui needs this disabled for now
        ctx.enable(GL::CULL_FACE);
        ctx.disable(GL::SCISSOR_TEST);

        ctx.viewport(
            0,
            0,
            canvas_width_on_screen as i32,
            canvas_height_on_screen as i32,
        );

        glc!(ctx, ctx.clear_color(0.1, 0.1, 0.2, 1.0));
        glc!(ctx, ctx.clear(GL::COLOR_BUFFER_BIT));
    }
}

pub struct InterfaceSystem {}
impl InterfaceSystem {
    pub fn run(&self, ui: &Ui, transforms: &Vec<TransformComponent>, uis: &Vec<UIComponent>) {
        // do the UI drawing
    }
}
