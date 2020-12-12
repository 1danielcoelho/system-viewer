use std::collections::HashMap;

use cgmath::Matrix4;

use web_sys::WebGlRenderingContext;
use web_sys::WebGlRenderingContext as GL;

use crate::{
    app_state::AppState,
    components::LightComponent,
    components::{MeshComponent, TransformComponent},
    managers::resource::UniformData,
    managers::ECManager,
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

pub struct RenderingSystem {}
impl RenderingSystem {
    pub fn new() -> Self {
        return Self::default();
    }

    pub fn run(&mut self, state: &AppState, em: &mut ECManager) {
        if state.gl.is_none() {
            return;
        }

        let mut uniform_data = self.pre_draw(state, em);
        self.draw(state, &mut uniform_data, em);
        self.post_draw(state);
    }

    fn pre_draw(&mut self, state: &AppState, em: &mut ECManager) -> UniformData {
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

        let mut result = UniformData {
            w: [0.0; 16], // This will be filled in later
            vp: *(p * v).as_ref(),
            light_types: [0; NUM_LIGHTS],
            light_colors: [0.0; NUM_LIGHTS * 3],
            light_pos_or_dir: [0.0; NUM_LIGHTS * 3],
            light_intensities: [0.0; NUM_LIGHTS],
        };

        // Pick lights that will affect the scene (randomly for now)
        let mut index = 0;
        for (ent, light) in em.light.iter() {
            let ent_index = em.get_entity_index(*ent).unwrap();
            let pos = &em.transform[ent_index as usize].get_world_transform().disp;

            result.light_types[index] = light.light_type as i32;

            result.light_colors[index * 3 + 0] = light.color.x;
            result.light_colors[index * 3 + 1] = light.color.y;
            result.light_colors[index * 3 + 2] = light.color.z;

            result.light_intensities[index] = light.intensity;

            result.light_pos_or_dir[index * 3 + 0] = pos.x;
            result.light_pos_or_dir[index * 3 + 1] = pos.y;
            result.light_pos_or_dir[index * 3 + 2] = pos.z;

            // log::info!("Setting light {} with pos: '{:?}', intensity: '{}' and color: '{:?}'", index, pos, light.intensity, light.color);

            index += 1;
            if index >= NUM_LIGHTS {
                break;
            }
        }

        return result;
    }

    fn draw(&self, state: &AppState, uniform_data: &mut UniformData, em: &mut ECManager) {
        for (t, m) in em.transform.iter().zip(em.mesh.iter()) {
            self.draw_one(state, uniform_data, t, m);
        }
    }

    fn post_draw(&self, state: &AppState) {
        let gl: &WebGlRenderingContext = (state.gl.as_ref()).unwrap();

        // Egui needs this disabled for now
        glc!(gl, gl.disable(GL::DEPTH_TEST));
    }

    fn draw_one(
        &self,
        state: &AppState,
        uniform_data: &mut UniformData,
        tc: &TransformComponent,
        mc: &MeshComponent,
    ) {
        let trans = tc.get_world_transform();
        let w: Matrix4<f32> = (*trans).into(); // TODO: Is this the right way of doing it?
        uniform_data.w = *w.as_ref();

        if let Some(mesh) = mc.get_mesh() {
            for (primitive_index, primitive) in mesh.primitives.iter().enumerate() {
                let resolved_mat = mc.get_resolved_material(primitive_index);
                if let Some(mat) = &resolved_mat {
                    mat.bind_for_drawing(state, uniform_data);
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
        Self {}
    }
}
