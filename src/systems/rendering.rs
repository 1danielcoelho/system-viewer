use crate::app_state::AppState;
use crate::components::light::LightType;
use crate::components::{MeshComponent, TransformComponent};
use crate::glc;
use crate::managers::resource::material::{FrameUniformValues, UniformName, UniformValue};
use crate::managers::scene::component_storage::ComponentStorage;
use crate::managers::scene::Scene;
use crate::utils::gl::GL;
use crate::GLCTX;
use na::*;
use std::convert::TryInto;
use web_sys::WebGl2RenderingContext;

pub const NUM_LIGHTS: usize = 8;

#[derive(Default)]
pub struct RenderingSystem {}
impl RenderingSystem {
    pub fn new() -> Self {
        return Self::default();
    }

    pub fn run(&mut self, state: &AppState, scene: &mut Scene) {
        GLCTX.with(|gl| {
            let ref_mut = gl.borrow_mut();
            let gl = ref_mut.as_ref().unwrap();

            let mut uniform_data = pre_draw(state, gl, scene);
            draw(gl, &mut uniform_data, scene);
            draw_skybox(state, gl, &mut uniform_data, scene);
            post_draw(gl);
        });
    }
}

fn pre_draw(
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

    let mut result = FrameUniformValues {
        v: state.camera.v,
        pv: state.camera.p * state.camera.v,
        light_types: Vec::new(),
        light_colors: Vec::new(),
        light_pos_or_dir_c: Vec::new(),
        light_intensities: Vec::new(),
    };

    result.light_types.reserve(NUM_LIGHTS);
    result.light_colors.reserve(NUM_LIGHTS * 3);
    result.light_pos_or_dir_c.reserve(NUM_LIGHTS * 3);
    result.light_intensities.reserve(NUM_LIGHTS);

    // Pick lights that will affect the scene (randomly for now)
    let mut index = 0;
    for (ent, light) in scene.light.iter() {
        let pos = &scene
            .transform
            .get_component(*ent)
            .unwrap()
            .get_world_transform()
            .trans;

        let pos = match light.light_type {
            LightType::Point => result.v.transform_point(&Point3::from(*pos)).coords,
            LightType::Directional => result.v.transform_vector(pos),
        };

        result.light_types.push(light.light_type as i32);

        result.light_colors.push(light.color.x);
        result.light_colors.push(light.color.y);
        result.light_colors.push(light.color.z);

        result
            .light_intensities
            .push(light.intensity.powf(state.light_intensity));

        result.light_pos_or_dir_c.push(pos.x as f32);
        result.light_pos_or_dir_c.push(pos.y as f32);
        result.light_pos_or_dir_c.push(pos.z as f32);

        // log::info!("Setting light {} with pos: '{:?}', intensity: '{}' and color: '{:?}'", index, pos, light.intensity, light.color);

        index += 1;
        if index >= NUM_LIGHTS {
            break;
        }
    }

    return result;
}

fn draw(gl: &WebGl2RenderingContext, uniform_data: &mut FrameUniformValues, scene: &mut Scene) {
    for (t, m) in scene.transform.iter().zip(scene.mesh.iter()) {
        draw_one(gl, uniform_data, t, m);
    }
}

fn post_draw(gl: &WebGl2RenderingContext) {
    // Egui needs this disabled for now
    glc!(gl, gl.disable(GL::DEPTH_TEST));
}

fn draw_one(
    gl: &WebGl2RenderingContext,
    uniform_data: &FrameUniformValues,
    tc: &TransformComponent,
    mc: &MeshComponent,
) {
    // We never do calculations in world space to evade precision problems:
    // If we're at 15557283 and our target is at 15557284 we'd get tons of triangle jittering,
    // but if we do it in camera space then we're at 0 and our target is at 1, which is fine
    let w = tc.get_world_transform().to_matrix4();

    let wv = uniform_data.v * w;
    let wv_inv_trans = wv.try_inverse().unwrap().transpose(); // Note: This is correct, it's not meant to be v * w.inv().trans()
    let wvp = uniform_data.pv * w;

    let wv_arr: [f32; 16] = na::convert::<Matrix4<f64>, Matrix4<f32>>(wv)
        .as_slice()
        .try_into()
        .unwrap();
    let wv_inv_trans_arr: [f32; 16] = na::convert::<Matrix4<f64>, Matrix4<f32>>(wv_inv_trans)
        .as_slice()
        .try_into()
        .unwrap();
    let wvp_arr: [f32; 16] = na::convert::<Matrix4<f64>, Matrix4<f32>>(wvp)
        .as_slice()
        .try_into()
        .unwrap();

    if let Some(mesh) = mc.get_mesh() {
        for (primitive_index, primitive) in mesh.borrow().primitives.iter().enumerate() {
            let resolved_mat = mc.get_resolved_material(primitive_index);

            if let Some(mat) = &resolved_mat {
                let mut mat_mut = mat.borrow_mut();

                mat_mut.set_uniform_value(UniformName::WVTrans, UniformValue::Matrix(wv_arr));
                mat_mut.set_uniform_value(
                    UniformName::WVInvTranspTrans,
                    UniformValue::Matrix(wv_inv_trans_arr),
                );
                mat_mut.set_uniform_value(UniformName::WVPTrans, UniformValue::Matrix(wvp_arr));

                mat_mut.set_uniform_value(
                    UniformName::LightTypes,
                    UniformValue::IntArr(uniform_data.light_types.clone()),
                );

                mat_mut.set_uniform_value(
                    UniformName::LightPosDir,
                    UniformValue::Vec3Arr(uniform_data.light_pos_or_dir_c.clone()),
                );

                mat_mut.set_uniform_value(
                    UniformName::LightColors,
                    UniformValue::Vec3Arr(uniform_data.light_colors.clone()),
                );

                mat_mut.set_uniform_value(
                    UniformName::LightIntensities,
                    UniformValue::FloatArr(uniform_data.light_intensities.clone()),
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

fn draw_skybox(
    state: &AppState,
    gl: &WebGl2RenderingContext,
    uniform_data: &mut FrameUniformValues,
    scene: &mut Scene,
) {
    if scene.skybox_mesh.is_none() || scene.skybox_trans.is_none() {
        return;
    }

    // Remove translation or else we can get precision issues on large coordinates
    let mut v_no_trans = state.camera.v_inv.clone();
    v_no_trans.set_column(3, &Vector4::new(0.0, 0.0, 0.0, 1.0));

    let vp_inv_arr: [f32; 16] = na::convert::<Matrix4<f64>, Matrix4<f32>>(
        scene.skybox_trans.unwrap() * v_no_trans * state.camera.p_inv,
    )
    .as_slice()
    .try_into()
    .unwrap();

    let old_depth_func = gl.get_parameter(GL::DEPTH_FUNC).unwrap().as_f64().unwrap() as u32;
    gl.depth_func(GL::LEQUAL);

    for primitive in scene
        .skybox_mesh
        .as_ref()
        .unwrap()
        .borrow()
        .primitives
        .iter()
    {
        if let Some(mat) = &scene.skybox_mat {
            let mut mat_mut = mat.borrow_mut();

            mat_mut.set_uniform_value(UniformName::VPInvTrans, UniformValue::Matrix(vp_inv_arr));

            mat_mut.bind_for_drawing(gl);
        }

        primitive.draw(gl);

        if let Some(mat) = &scene.skybox_mat {
            mat.borrow().unbind_from_drawing(gl);
        }
    }

    gl.depth_func(old_depth_func);
}
