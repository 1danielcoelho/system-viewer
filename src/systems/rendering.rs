use std::collections::HashMap;

use cgmath::Matrix4;

use web_sys::WebGlRenderingContext;
use web_sys::WebGlRenderingContext as GL;

use crate::{
    app_state::AppState,
    components::LightComponent,
    components::{MeshComponent, TransformComponent},
    managers::Entity,
};

pub const NUM_LIGHTS: usize = 8;

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

pub struct RenderingSystem {
    // TODO: Move this data to a Frame struct, created once per frame?
    vp_transform: cgmath::Matrix4<f32>,
    lights: [LightComponent; NUM_LIGHTS],
}
impl RenderingSystem {
    pub fn new() -> Self {
        return Self::default();
    }

    pub fn run(
        &mut self,
        state: &AppState,
        transforms: &Vec<TransformComponent>,
        meshes: &Vec<MeshComponent>,
        lights: &HashMap<Entity, LightComponent>,
    ) {
        if state.gl.is_none() {
            return;
        }

        self.pre_draw(state, lights);
        self.draw(state, transforms, meshes);
        RenderingSystem::post_draw(state);
    }

    fn pre_draw(&mut self, state: &AppState, lights: &HashMap<Entity, LightComponent>) {
        let gl: &WebGlRenderingContext = (state.gl.as_ref()).unwrap();

        // Setup GL state
        glc!(gl, gl.enable(GL::CULL_FACE)); // Egui needs this disabled for now
        glc!(gl, gl.disable(GL::SCISSOR_TEST));
        glc!(gl, gl.enable(GL::DEPTH_TEST));
        glc!(
            gl,
            gl.viewport(0, 0, state.canvas_width as i32, state.canvas_height as i32,)
        );
        glc!(gl, gl.clear_color(0.1, 0.1, 0.2, 1.0));
        glc!(gl, gl.clear(GL::COLOR_BUFFER_BIT | GL::DEPTH_BUFFER_BIT));

        // Initialize VP transform for frame
        let p = cgmath::perspective(
            state.camera.fov_v,
            state.canvas_width as f32 / state.canvas_height as f32,
            state.camera.near,
            state.camera.far,
        );
        let v = cgmath::Matrix4::look_at(state.camera.pos, state.camera.target, state.camera.up);
        self.vp_transform = p * v;

        // Pick lights that will affect the scene (randomly for now)
        let mut index = 0;
        for (_ent, light) in lights.iter() {
            self.lights[index] = light.clone();
            index += 1;

            if index >= NUM_LIGHTS {
                break;
            }
        }
    }

    fn draw(
        &self,
        state: &AppState,
        transforms: &Vec<TransformComponent>,
        meshes: &Vec<MeshComponent>,
    ) {
        assert_eq!(
            transforms.len(),
            meshes.len(),
            "RenderingSystem::draw: Different number of trans and meshes"
        );

        for (t, m) in transforms.iter().zip(meshes.iter()) {
            self.draw_one(state, t, m);
        }
    }

    fn post_draw(state: &AppState) {
        let gl: &WebGlRenderingContext = (state.gl.as_ref()).unwrap();

        // Egui needs this disabled for now
        glc!(gl, gl.disable(GL::DEPTH_TEST));
    }

    fn draw_one(&self, state: &AppState, tc: &TransformComponent, mc: &MeshComponent) {
        let trans = tc.get_world_transform();
        let w: Matrix4<f32> = (*trans).into(); // TODO: Is this the right way of doing it?
        let wvp = self.vp_transform * w;
        let wvp_floats: &[f32; 16] = wvp.as_ref();

        if let Some(mesh) = mc.get_mesh() {
            for (primitive_index, primitive) in mesh.primitives.iter().enumerate() {
                let resolved_mat = mc.get_resolved_material(primitive_index);
                if let Some(mat) = &resolved_mat {
                    mat.bind_for_drawing(state, wvp_floats, &self.lights);
                }

                primitive.draw(state.gl.as_ref().unwrap());

                if let Some(mat) = &resolved_mat {
                    mat.unbind_from_drawing(state);
                }
            }
        }
    }
}

impl Default for RenderingSystem {
    fn default() -> Self {
        Self {
            vp_transform: cgmath::One::one(),
            lights: Default::default(),
        }
    }
}
