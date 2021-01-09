use crate::{
    app_state::AppState,
    components::{MeshComponent, TransformComponent},
    glc,
    managers::resource::material::{FrameUniformValues, UniformName, UniformValue},
    utils::gl::GL,
};
use crate::{managers::scene::Scene, GLCTX};
use na::*;
use std::convert::TryInto;
use web_sys::WebGl2RenderingContext;

pub const NUM_LIGHTS: usize = 8;

pub struct RenderingSystem {}
impl RenderingSystem {
    pub fn new() -> Self {
        return Self::default();
    }

    pub fn run(&mut self, state: &AppState, scene: &mut Scene) {
        GLCTX.with(|gl| {
            let ref_mut = gl.borrow_mut();
            let gl = ref_mut.as_ref().unwrap();

            let mut uniform_data = self.pre_draw(state, gl, scene);
            self.draw(gl, &mut uniform_data, scene);
            self.post_draw(gl);
        });
    }

    fn pre_draw(
        &mut self,
        state: &AppState,
        gl: &WebGl2RenderingContext,
        scene: &mut Scene,
    ) -> FrameUniformValues {
        // Setup GL state
        glc!(gl, gl.enable(GL::CULL_FACE));
        glc!(gl, gl.disable(GL::SCISSOR_TEST));
        glc!(gl, gl.enable(GL::DEPTH_TEST));
        glc!(
            gl,
            gl.viewport(0, 0, state.canvas_width as i32, state.canvas_height as i32,)
        );
        glc!(gl, gl.clear_color(0.1, 0.1, 0.2, 1.0));
        glc!(gl, gl.clear(GL::COLOR_BUFFER_BIT | GL::DEPTH_BUFFER_BIT));

        // Initialize VP transform for frame
        let p = Matrix4::new_perspective(
            state.canvas_width as f64 / state.canvas_height as f64,
            state.camera.fov_v.to_radians(),
            state.camera.near,
            state.camera.far,
        );

        let mut v = Matrix4::look_at_rh(&state.camera.pos, &state.camera.target, &state.camera.up);

        if let Some(reference) = state.camera.reference_entity {
            let trans = &scene
                .get_component::<TransformComponent>(reference)
                .unwrap()
                .get_world_transform()
                .trans;

            let mat: Matrix4<f64> =
                Translation3::new(-trans.x, -trans.y, -trans.z).to_homogeneous();
            v = v * mat;
        }

        let mut result = FrameUniformValues {
            vp: (na::convert::<Matrix4<f64>, Matrix4<f32>>(p * v))
                .as_slice()
                .try_into()
                .unwrap(),
            p_v_mat: p * v,
            light_types: Vec::new(),
            light_colors: Vec::new(),
            light_pos_or_dir: Vec::new(),
            light_intensities: Vec::new(),
            camera_pos: [
                state.camera.pos.x as f32,
                state.camera.pos.y as f32,
                state.camera.pos.z as f32,
            ],
        };

        result.light_types.reserve(NUM_LIGHTS);
        result.light_colors.reserve(NUM_LIGHTS * 3);
        result.light_pos_or_dir.reserve(NUM_LIGHTS * 3);
        result.light_intensities.reserve(NUM_LIGHTS);

        // Pick lights that will affect the scene (randomly for now)
        let mut index = 0;
        for (ent, light) in scene.light.iter() {
            let ent_index = scene.get_entity_index(*ent).unwrap();
            let pos = &scene.transform[ent_index as usize]
                .get_world_transform()
                .trans;

            result.light_types.push(light.light_type as i32);

            result.light_colors.push(light.color.x);
            result.light_colors.push(light.color.y);
            result.light_colors.push(light.color.z);

            result
                .light_intensities
                .push(light.intensity.powf(state.light_intensity));

            result.light_pos_or_dir.push(pos.x as f32);
            result.light_pos_or_dir.push(pos.y as f32);
            result.light_pos_or_dir.push(pos.z as f32);

            // log::info!("Setting light {} with pos: '{:?}', intensity: '{}' and color: '{:?}'", index, pos, light.intensity, light.color);

            index += 1;
            if index >= NUM_LIGHTS {
                break;
            }
        }

        return result;
    }

    fn draw(
        &self,
        gl: &WebGl2RenderingContext,
        uniform_data: &mut FrameUniformValues,
        scene: &mut Scene,
    ) {
        for (t, m) in scene.transform.iter().zip(scene.mesh.iter()) {
            self.draw_one(gl, uniform_data, t, m);
        }
    }

    fn post_draw(&self, gl: &WebGl2RenderingContext) {
        // Egui needs this disabled for now
        glc!(gl, gl.disable(GL::DEPTH_TEST));
    }

    fn draw_one(
        &self,
        gl: &WebGl2RenderingContext,
        uniform_data: &FrameUniformValues,
        tc: &TransformComponent,
        mc: &MeshComponent,
    ) {
        let trans = tc.get_world_transform();
        let w: Matrix4<f32> = na::convert(trans.to_matrix4());
        let world_trans_uniform_data: [f32; 16] = w.as_slice().try_into().unwrap();

        let w_inv_trans: Matrix4<f32> = w.try_inverse().unwrap_or(Matrix4::identity()).transpose();
        let w_inv_trans_uniform_data: [f32; 16] = w_inv_trans.as_slice().try_into().unwrap();

        if let Some(mesh) = mc.get_mesh() {
            for (primitive_index, primitive) in mesh.borrow().primitives.iter().enumerate() {
                let resolved_mat = mc.get_resolved_material(primitive_index);

                if let Some(mat) = &resolved_mat {
                    let mut mat_mut = mat.borrow_mut();

                    log::info!("world_trans: {}", w);

                    // TODO: I shouldn't need to clone these...
                    mat_mut.set_uniform_value(
                        UniformName::WorldTrans,
                        UniformValue::Matrix(world_trans_uniform_data),
                    );
                    mat_mut.set_uniform_value(
                        UniformName::WorldTransInvTranspose,
                        UniformValue::Matrix(w_inv_trans_uniform_data),
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
                    mat_mut.set_uniform_value(
                        UniformName::CameraPos,
                        UniformValue::Vec3(uniform_data.camera_pos),
                    );

                    // log::info!("Drawing mesh {} with material {}", mesh.name, mat_mut.name);
                    mat_mut.bind_for_drawing(gl);
                }

                primitive.draw(gl);

                if let Some(mat) = &resolved_mat {
                    mat.borrow().unbind_from_drawing(gl);
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
