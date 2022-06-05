use crate::app_state::AppState;
use crate::components::light_component::LightType;
use crate::components::{Component, MeshComponent, TransformComponent};
use crate::managers::resource::material::{
    FrameUniformValues, Material, UniformName, UniformValue,
};
use crate::managers::resource::mesh::Mesh;
use crate::managers::resource::texture::TextureUnit;
use crate::managers::scene::component_storage::ComponentStorage;
use crate::managers::scene::Scene;
use crate::managers::ResourceManager;
use crate::systems::Framebuffer;
use crate::utils::gl::GL;
use crate::utils::log::*;
use crate::utils::string::decode_hex;
use crate::{GLCTX, STATE};
use glow::*;
use na::*;
use std::cell::RefCell;
use std::convert::TryInto;
use std::rc::Rc;

pub const NUM_LIGHTS: usize = 8;

fn exposure_factor(ev100: f32) -> f32 {
    return 1.0 / (2.0.powf(ev100) * 1.2);
}

pub struct RenderingSystem {
    framebuffer: Framebuffer,
    screenspace_quad: Option<Rc<RefCell<Mesh>>>,
    blit_framebuffer_mat: Option<Rc<RefCell<Material>>>,
}
impl RenderingSystem {
    pub fn new(res_man: &mut ResourceManager) -> Self {
        // Fetch canvas width and height
        let mut canvas_width: u32 = 0;
        let mut canvas_height: u32 = 0;
        STATE.with(|s| {
            if let Ok(mut ref_mut_s) = s.try_borrow_mut() {
                let s = ref_mut_s.as_mut().unwrap();
                canvas_width = s.canvas_width;
                canvas_height = s.canvas_height;
            }
        });

        // Fetch resources we'll need for our render passes
        let screenspace_quad = res_man.get_or_create_mesh("quad");
        let blit_framebuffer_mat =
            res_man.instantiate_material("default_screenspace", "default_screenspace");

        // Create framebuffer
        let framebuffer = GLCTX.with(|gl| {
            return Framebuffer::new(canvas_width, canvas_height, gl);
        });

        // Set framebuffer color texture into our material so that it's automatically bound when we draw
        {
            let mut mat_mut = blit_framebuffer_mat.as_ref().unwrap().borrow_mut();
            mat_mut.set_texture(
                TextureUnit::BaseColor,
                Some(framebuffer.get_color_tex().clone()),
            );
        }

        return Self {
            framebuffer,
            screenspace_quad,
            blit_framebuffer_mat,
        };
    }

    pub fn resize(&mut self, width: u32, height: u32, gl: &glow::Context) {
        self.framebuffer.resize(width, height, gl);
    }

    pub fn run(&mut self, state: &AppState, scene: &mut Scene) {
        GLCTX.with(|gl| {
            // Main pass
            self.framebuffer.bind(gl);
            let mut uniform_data = pre_draw(state, gl, scene);
            draw(gl, &mut uniform_data, scene);
            draw_points(state, gl, &mut uniform_data, scene);
            draw_skybox(state, gl, &mut uniform_data, scene);
            self.framebuffer.unbind(gl);

            // Blit to main render target
            post_draw(
                state.canvas_width,
                state.canvas_height,
                gl,
                self.blit_framebuffer_mat.as_ref(),
                self.screenspace_quad.as_ref(),
            );
        });
    }
}

fn pre_draw(state: &AppState, gl: &glow::Context, scene: &mut Scene) -> FrameUniformValues {
    unsafe {
        gl.enable(GL::CULL_FACE);
        gl.disable(GL::SCISSOR_TEST);

        gl.enable(GL::DEPTH_TEST);
        gl.depth_func(GL::LESS);

        gl.clear_color(0.1, 0.1, 0.2, 1.0);
        gl.clear(GL::COLOR_BUFFER_BIT | GL::DEPTH_BUFFER_BIT);
    }

    let mut result = FrameUniformValues {
        v: state.camera.v,
        pv: state.camera.p * state.camera.v,
        light_types: Vec::new(),
        light_colors: Vec::new(),
        light_pos_or_dir_c: Vec::new(),
        light_intensities: Vec::new(),
        exposure_factor: exposure_factor(state.ev100),
        f_coef: (2.0 / (state.camera.far + 1.0).log2()) as f32,
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

        result.light_intensities.push(light.intensity);

        result.light_pos_or_dir_c.push(pos.x as f32);
        result.light_pos_or_dir_c.push(pos.y as f32);
        result.light_pos_or_dir_c.push(pos.z as f32);

        // info!("Setting light {} with pos: '{:?}', intensity: '{}' and color: '{:?}'", index, pos, light.intensity, light.color);

        index += 1;
        if index >= NUM_LIGHTS {
            break;
        }
    }

    return result;
}

fn draw(gl: &glow::Context, uniform_data: &mut FrameUniformValues, scene: &mut Scene) {
    for (t, m) in scene.transform.iter().zip(scene.mesh.iter_mut()) {
        draw_one(gl, uniform_data, t, m);
    }
}

fn post_draw(
    width: u32,
    height: u32,
    gl: &glow::Context,
    mat: Option<&Rc<RefCell<Material>>>,
    quad: Option<&Rc<RefCell<Mesh>>>,
) {
    unsafe {
        gl.viewport(0, 0, width as i32, height as i32);
        gl.clear_color(0.1, 0.1, 0.2, 1.0);
        gl.clear(GL::COLOR_BUFFER_BIT | GL::DEPTH_BUFFER_BIT);
        gl.disable(GL::DEPTH_TEST);
    }

    // Blit the main framebuffer to the default (canvas) framebuffer
    // We could actually blit here but I like having this actual draw call as it's helpful for debugging
    if mat.is_some() && quad.is_some() {
        let mut mat = RefCell::borrow_mut(mat.unwrap());
        let quad = quad.unwrap();

        mat.bind_for_drawing(gl);

        for prim in &RefCell::borrow(quad).primitives {
            prim.draw(gl);
        }

        mat.unbind_from_drawing(gl);
    }
}

fn draw_one(
    gl: &glow::Context,
    uniform_data: &FrameUniformValues,
    tc: &TransformComponent,
    mc: &mut MeshComponent,
) {
    // We never do calculations in world space to evade precision problems:
    // If we're at 15557283 and our target is at 15557284 we'd get tons of triangle jittering,
    // but if we do it in camera space then we're at 0 and our target is at 1, which is fine
    let w = tc.get_world_transform().to_matrix4();
    let wv = uniform_data.v * w;
    let wvp = uniform_data.pv * w;

    // TODO: This seems very expensive just to support non-uniform normal scaling...
    let mut wv_no_trans = wv.clone();
    wv_no_trans[(0, 3)] = 0.0;
    wv_no_trans[(1, 3)] = 0.0;
    wv_no_trans[(2, 3)] = 0.0;
    if let None = wv_no_trans.try_inverse() {
        // TODO. Usually happens when radius/scale is zero by some mistake
        error!(
            LogCat::Resources,
            "Failed to invert trans '{:#?}'", wv_no_trans
        );
        return;
    }

    // Store position to draw a point on top of this later
    mc.last_ndc_position =
        na::convert::<Vector4<f64>, Vector4<f32>>(wvp * Vector4::new(0.0, 0.0, 0.0, 1.0));

    let wv_inv_trans = wv_no_trans.try_inverse().unwrap().transpose(); // Note: This is correct, it's not meant to be v * w.inv().trans()

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

                // Make sure we're up to date in case this mesh has just been replaced with another that now
                // has normals or something like that
                // TODO: Find a better way of doing this: One check per draw call is not the best...
                if mat_mut.get_compatible_prim_hash() != primitive.compatible_hash {
                    mat_mut.set_prim_defines(primitive);
                }

                mat_mut.set_uniform_value(UniformName::WVTrans, UniformValue::Matrix(wv_arr));
                mat_mut.set_uniform_value(
                    UniformName::WVInvTranspTrans,
                    UniformValue::Matrix(wv_inv_trans_arr),
                );
                mat_mut.set_uniform_value(UniformName::WVPTrans, UniformValue::Matrix(wvp_arr));

                mat_mut.set_uniform_value(
                    UniformName::ExposureFactor,
                    UniformValue::Float(uniform_data.exposure_factor),
                );

                mat_mut.set_uniform_value(
                    UniformName::Fcoef,
                    UniformValue::Float(uniform_data.f_coef),
                );

                if uniform_data.light_types.len() > 0 {
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
                }

                // info!("Drawing mesh {} with material {}", mesh.name, mat_mut.name);
                mat_mut.bind_for_drawing(gl);
            }

            primitive.draw(gl);

            if let Some(mat) = &resolved_mat {
                mat.borrow().unbind_from_drawing(gl);
            }
        }
    }
}

fn draw_points(
    _state: &AppState,
    gl: &glow::Context,
    uniform_data: &mut FrameUniformValues,
    scene: &mut Scene,
) {
    if scene.points_mesh.is_none() || scene.points_mat.is_none() {
        return;
    }

    let mut pts = RefCell::borrow_mut(scene.points_mesh.as_ref().unwrap());
    if let Some(prim) = &mut pts.dynamic_primitive {
        let num_bodies = scene.mesh.get_num_components() as usize;

        // Update color buffer only when entity number changes (expensive)
        if prim.get_num_elements() != num_bodies {
            prim.set_num_elements(num_bodies);

            let buf = prim.get_color_buffer_mut();
            for (ent, metadata) in scene.metadata.iter() {
                // We need to get the index of the point, which will follow the mesh component
                // So here we assume the mesh component storage is sparse
                let ent_index = scene.get_entity_index(*ent).unwrap() as usize;

                // We may have entities that have metadata components and no mesh components,
                // but we obviously don't care about those here
                if ent_index >= num_bodies {
                    continue;
                }

                let mut color: [f32; 4] = match metadata
                    .get_metadata("body_type")
                    .unwrap_or(&String::from(""))
                    .as_str()
                {
                    "Satellite" => [0.8, 0.8, 0.8, 1.5],
                    "Asteroid" => [1.0, 1.0, 1.0, 1.0],
                    "Star" => [1.0, 0.5, 0.0, 2.5],
                    "Planet" => [0.6, 0.8, 0.6, 2.0],
                    "Comet" => [0.4, 0.6, 0.8, 1.0],
                    "Artificial" => [1.0, 1.0, 1.0, 1.0],
                    "Barycenter" => [0.5, 0.5, 0.5, 0.5],
                    _ => [1.0, 1.0, 1.0, 1.0],
                };

                if let Some(base_color) = metadata.get_metadata("base_color") {
                    let mut bytes: Vec<f32> = decode_hex(base_color)
                        .unwrap()
                        .iter()
                        .map(|u| *u as f32 / 255.0)
                        .collect();
                    bytes.resize(4, 1.0);

                    color[0] = bytes[0];
                    color[1] = bytes[1];
                    color[2] = bytes[2];
                }

                buf[ent_index * 4 + 0] = color[0];
                buf[ent_index * 4 + 1] = color[1];
                buf[ent_index * 4 + 2] = color[2];
                buf[ent_index * 4 + 3] = color[3];
            }
        }

        // Update pos buffer every frame
        let buf = prim.get_pos_buffer_mut();
        for (ent_index, mesh_comp) in scene.mesh.iter().enumerate() {
            if mesh_comp.get_enabled() {
                buf[ent_index * 4 + 0] = mesh_comp.last_ndc_position.x;
                buf[ent_index * 4 + 1] = mesh_comp.last_ndc_position.y;
                buf[ent_index * 4 + 2] = mesh_comp.last_ndc_position.z;
                buf[ent_index * 4 + 3] = mesh_comp.last_ndc_position.w;
            } else {
                // Just put disabled components off screen
                // TODO: Optimize this somehow
                buf[ent_index * 4 + 0] = -10.0;
                buf[ent_index * 4 + 1] = -10.0;
                buf[ent_index * 4 + 2] = -10.0;
                buf[ent_index * 4 + 3] = 1.0;
            }
        }

        let mut scene_mat_mut = scene.points_mat.as_ref().unwrap().borrow_mut();

        scene_mat_mut
            .set_uniform_value(UniformName::Fcoef, UniformValue::Float(uniform_data.f_coef));

        scene_mat_mut.bind_for_drawing(gl);
        prim.upload_buffers(gl);
        prim.draw(gl);
        scene_mat_mut.unbind_from_drawing(gl);
    }
}

fn draw_skybox(
    state: &AppState,
    gl: &glow::Context,
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

    let old_depth_func;
    unsafe {
        old_depth_func = gl.get_parameter_i32(GL::DEPTH_FUNC) as u32;
        gl.depth_func(GL::LEQUAL);
    }

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

            mat_mut.set_uniform_value(
                UniformName::ExposureFactor,
                UniformValue::Float(uniform_data.exposure_factor),
            );

            mat_mut.bind_for_drawing(gl);
        }

        primitive.draw(gl);

        if let Some(mat) = &scene.skybox_mat {
            mat.borrow().unbind_from_drawing(gl);
        }
    }

    unsafe {
        gl.depth_func(old_depth_func);
    }
}
