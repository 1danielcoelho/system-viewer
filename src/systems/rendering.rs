use std::collections::HashMap;

use cgmath::Matrix4;

use web_sys::WebGlRenderingContext;
use web_sys::WebGlRenderingContext as GL;

use crate::{
    app_state::AppState,
    components::LightComponent,
    components::{MeshComponent, TransformComponent},
    managers::resource::{FrameUniformValues, UniformName, UniformValue},
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

    fn pre_draw(&mut self, state: &AppState, em: &mut ECManager) -> FrameUniformValues {
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

        let mut result = FrameUniformValues {
            vp: *(p * v).as_ref(),
            light_types: Vec::new(),
            light_colors: Vec::new(),
            light_pos_or_dir: Vec::new(),
            light_intensities: Vec::new(),
        };

        result.light_types.reserve(NUM_LIGHTS);
        result.light_colors.reserve(NUM_LIGHTS * 3);
        result.light_pos_or_dir.reserve(NUM_LIGHTS * 3);
        result.light_intensities.reserve(NUM_LIGHTS);

        // Pick lights that will affect the scene (randomly for now)
        let mut index = 0;
        for (ent, light) in em.light.iter() {
            let ent_index = em.get_entity_index(*ent).unwrap();
            let pos = &em.transform[ent_index as usize].get_world_transform().disp;

            result.light_types.push(light.light_type as i32);

            result.light_colors.push(light.color.x);
            result.light_colors.push(light.color.y);
            result.light_colors.push(light.color.z);

            result.light_intensities.push(light.intensity);

            result.light_pos_or_dir.push(pos.x);
            result.light_pos_or_dir.push(pos.y);
            result.light_pos_or_dir.push(pos.z);

            // log::info!("Setting light {} with pos: '{:?}', intensity: '{}' and color: '{:?}'", index, pos, light.intensity, light.color);

            index += 1;
            if index >= NUM_LIGHTS {
                break;
            }
        }

        return result;
    }

    fn draw(&self, state: &AppState, uniform_data: &mut FrameUniformValues, em: &mut ECManager) {
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
        uniform_data: &FrameUniformValues,
        tc: &TransformComponent,
        mc: &MeshComponent,
    ) {
        let trans = tc.get_world_transform();
        let w: Matrix4<f32> = (*trans).into(); // TODO: Is this the right way of doing it?
        let world_trans_uniform_data: [f32; 16] = *w.as_ref();

        if let Some(mesh) = mc.get_mesh() {
            for (primitive_index, primitive) in mesh.primitives.iter().enumerate() {
                let resolved_mat = mc.get_resolved_material(primitive_index);

                if let Some(mat) = &resolved_mat {
                    let mut mat_mut = mat.borrow_mut();

                    // TODO: I shouldn't need to clone these...
                    mat_mut.set_uniform_value(
                        UniformName::WorldTrans,
                        UniformValue::Matrix(world_trans_uniform_data),
                    );
                    mat_mut.set_uniform_value(
                        UniformName::ViewProjTrans,
                        UniformValue::Matrix(uniform_data.vp),
                    );
                    mat_mut.set_uniform_value(
                        UniformName::LightTypes,
                        UniformValue::IntArr(uniform_data.light_types.clone()),
                    );
                    mat_mut.set_uniform_value(
                        UniformName::LightPosDir,
                        UniformValue::Vec3Arr(uniform_data.light_pos_or_dir.clone()),
                    );
                    mat_mut.set_uniform_value(
                        UniformName::LightColors,
                        UniformValue::Vec3Arr(uniform_data.light_colors.clone()),
                    );
                    mat_mut.set_uniform_value(
                        UniformName::LightIntensities,
                        UniformValue::FloatArr(uniform_data.light_intensities.clone()),
                    );

                    mat_mut.bind_for_drawing(state);
                }

                primitive.draw(state.gl.as_ref().unwrap());

                if let Some(mat) = &resolved_mat {
                    mat.borrow().unbind_from_drawing(state);
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
