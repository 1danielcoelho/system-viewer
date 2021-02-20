use na::*;

use crate::app_state::{AppState, ButtonState};
use crate::components::TransformComponent;

pub struct InputManager {
    last_mouse_x: i32,
    last_mouse_y: i32,
}
impl InputManager {
    pub fn new() -> Self {
        return Self {
            last_mouse_x: 0,
            last_mouse_y: 0,
        };
    }

    pub fn run(&mut self, state: &mut AppState) {
        process_input(state, self.last_mouse_x, self.last_mouse_y);
        update_camera_transforms(state);

        self.last_mouse_x = state.input.mouse_x;
        self.last_mouse_y = state.input.mouse_y;
    }
}

fn process_input(state: &mut AppState, last_mouse_x: i32, last_mouse_y: i32) {
    if state.input.spacebar == ButtonState::Pressed {
        state.simulation_paused = !state.simulation_paused;
        state.input.spacebar = ButtonState::Handled;
    }

    state.input.delta_x = state.input.mouse_x - last_mouse_x;
    state.input.delta_y = state.input.mouse_y - last_mouse_y;

    let aspect = state.canvas_width as f64 / state.canvas_height as f64;

    let cam_forward = (state.camera.target - state.camera.pos).normalize();
    let cam_right = cam_forward.cross(&state.camera.up).normalize();
    let cam_up = cam_right.cross(&cam_forward).normalize();

    let lock_pitch = true;

    if state.input.scroll_delta_y < 0 {
        state.move_speed *= 1.1;
    } else if state.input.scroll_delta_y > 0 {
        state.move_speed *= 0.9;
    }
    state.input.scroll_delta_y = 0; // We have to "consume" this as it isn't cleared otherwise

    let mut incr: Vector3<f64> = Vector3::new(0.0, 0.0, 0.0);
    if state.input.forward == ButtonState::Pressed {
        incr += cam_forward * state.real_delta_time_s * state.move_speed;
    }
    if state.input.back == ButtonState::Pressed {
        incr -= cam_forward * state.real_delta_time_s * state.move_speed;
    }
    if state.input.left == ButtonState::Pressed {
        incr -= cam_right * state.real_delta_time_s * state.move_speed;
    }
    if state.input.right == ButtonState::Pressed {
        incr += cam_right * state.real_delta_time_s * state.move_speed;
    }
    if state.input.up == ButtonState::Pressed {
        incr += cam_up * state.real_delta_time_s * state.move_speed;
    }
    if state.input.down == ButtonState::Pressed {
        incr -= cam_up * state.real_delta_time_s * state.move_speed;
    }

    if state.input.m1 == ButtonState::Pressed
        && (state.input.delta_y.abs() > 0 || state.input.delta_x.abs() > 0)
    {
        // Orbit
        if state.input.modifiers.alt && state.camera.reference_entity.is_some() {
        } else {
            let half_canvas_height_world =
                state.camera.near * (state.camera.fov_v.to_radians() / 2.0).tan();
            let half_canvas_width_world = aspect * half_canvas_height_world;

            let delta_x_world = -half_canvas_width_world
                * (state.input.delta_x as f64 / (state.canvas_width as f64 / 2.0));
            let delta_y_world = -half_canvas_height_world
                * (state.input.delta_y as f64 / (state.canvas_height as f64 / 2.0));

            let mut x_angle = (delta_x_world / state.camera.near).atan();
            let mut y_angle = (delta_y_world / state.camera.near).atan();
            x_angle *= state.rotate_speed;
            y_angle *= state.rotate_speed;

            let curr_pitch_angle = (cam_forward.cross(&state.camera.up).magnitude())
                .atan2(cam_forward.dot(&state.camera.up));

            if lock_pitch {
                if curr_pitch_angle - y_angle < 0.0001 {
                    y_angle = curr_pitch_angle - 0.0001;
                } else if curr_pitch_angle - y_angle > (std::f64::consts::PI - 0.0001) {
                    y_angle = -(std::f64::consts::PI - 0.0001) + curr_pitch_angle;
                };
            }

            let rot_z = Rotation3::from_axis_angle(&state.camera.up, x_angle);
            let rot_x = Rotation3::from_axis_angle(&Unit::new_unchecked(cam_right), y_angle);

            let new_cam_forward = rot_z.transform_vector(&rot_x.transform_vector(&cam_forward));
            let prev_targ_dist = (state.camera.target - state.camera.pos).magnitude();
            let new_targ = state.camera.pos + new_cam_forward * prev_targ_dist;
            state.camera.target = new_targ;
        }
    }

    state.camera.pos += incr;
    state.camera.target += incr;

    if !lock_pitch {
        state.camera.up = Unit::new_unchecked(cam_up);
    }
}

fn update_camera_transforms(state: &mut AppState) {
    state.camera.p = Matrix4::new_perspective(
        state.canvas_width as f64 / state.canvas_height as f64,
        state.camera.fov_v.to_radians() as f64,
        state.camera.near as f64,
        state.camera.far as f64,
    );
    state.camera.p_inv = state.camera.p.try_inverse().unwrap();

    state.camera.v = Matrix4::look_at_rh(&state.camera.pos, &state.camera.target, &state.camera.up);
    if let Some(trans) = state.camera.reference_translation {
        state.camera.v *= Translation3::from(-trans).to_homogeneous();
    }
    state.camera.v_inv = state.camera.v.try_inverse().unwrap();
}
