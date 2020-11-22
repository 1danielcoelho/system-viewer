use web_sys::WebGlRenderingContext;
use web_sys::WebGlRenderingContext as GL;

mod materials;
mod mesh;
mod shaders;
mod texture;

pub use materials::*;
pub use mesh::*;
pub use shaders::*;
pub use texture::*;

use crate::{
    app_state::AppState,
    components::{MeshComponent, TransformComponent},
};

#[macro_export]
macro_rules! glc {
    ($ctx:expr, $any:expr) => {
        #[cfg(debug_assertions)]
        while $ctx.get_error() != 0 {} // Not sure why he did this
        $any;
        #[cfg(debug_assertions)]
        while match $ctx.get_error() {
            0 => false,
            err => {
                log::error!("[OpenGL Error] {}", err);
                true
            }
        } {}
    };
}

pub struct RenderingSystem {}
impl RenderingSystem {
    pub fn run(
        &self,
        state: &AppState,
        transforms: &Vec<TransformComponent>,
        meshes: &Vec<MeshComponent>,
    ) {
        if state.gl.is_none() {
            return;
        }

        RenderingSystem::pre_draw(state);
        RenderingSystem::draw(state, transforms, meshes);
        RenderingSystem::post_draw(state);
    }

    fn pre_draw(state: &AppState) {
        let gl: &WebGlRenderingContext = (state.gl.as_ref()).unwrap();

        // Egui needs this disabled for now
        glc!(gl, gl.enable(GL::CULL_FACE));
        glc!(gl, gl.disable(GL::SCISSOR_TEST));
        glc!(gl, gl.enable(GL::DEPTH_TEST));

        glc!(
            gl,
            gl.viewport(0, 0, state.canvas_width as i32, state.canvas_height as i32,)
        );

        glc!(gl, gl.clear_color(0.1, 0.1, 0.2, 1.0));
        glc!(gl, gl.clear(GL::COLOR_BUFFER_BIT | GL::DEPTH_BUFFER_BIT));
    }

    fn draw(state: &AppState, transforms: &Vec<TransformComponent>, meshes: &Vec<MeshComponent>) {
        assert_eq!(
            transforms.len(),
            meshes.len(),
            "RenderingSystem::draw: Different number of trans and meshes"
        );

        for (t, m) in transforms.iter().zip(meshes.iter()) {
            RenderingSystem::draw_one(state, t, m);
        }
    }

    fn post_draw(state: &AppState) {
        let gl: &WebGlRenderingContext = (state.gl.as_ref()).unwrap();

        // Egui needs this disabled for now
        glc!(gl, gl.disable(GL::DEPTH_TEST));
    }

    fn draw_one(state: &AppState, tc: &TransformComponent, mc: &MeshComponent) {
        let trans = &tc.get_world_transform();
        if let Some(mesh) = mc.get_mesh() {
            for (primitive_index, primitive) in mesh.primitives.iter().enumerate() {
                if let Some(mat) = mc.get_resolved_material(primitive_index) {
                    mat.bind_for_drawing(state, trans);
                }

                primitive.draw(state.gl.as_ref().unwrap());
            }
        }
    }
}
