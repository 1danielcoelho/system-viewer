use na::{Rotation3, Unit, Vector3};

use crate::app_state::{AppState, ButtonState};

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
        if state.input.spacebar == ButtonState::Pressed {
            state.simulation_paused = !state.simulation_paused;
            state.input.spacebar = ButtonState::Handled;
        }

        state.input.delta_x = state.input.mouse_x - self.last_mouse_x;
        state.input.delta_y = state.input.mouse_y - self.last_mouse_y;

        self.last_mouse_x = state.input.mouse_x;
        self.last_mouse_y = state.input.mouse_y;

        let aspect = state.canvas_width as f32 / state.canvas_height as f32;

        let cam_forward = ((state.camera.target - state.camera.pos) as Vector3<f32>).normalize();
        let cam_right: Vector3<f32> = cam_forward.cross(&state.camera.up).normalize();
        let cam_up: Vector3<f32> = cam_right.cross(&cam_forward).normalize();

        let lock_pitch = true;

        if state.input.scroll_delta_y < 0 {
            state.move_speed *= 1.1;
        } else if state.input.scroll_delta_y > 0 {
            state.move_speed *= 0.9;
        }
        state.input.scroll_delta_y = 0; // We have to "consume" this as it isn't cleared otherwise

        let move_speed = state.move_speed;
        let rotate_speed = state.rotate_speed;

        let mut incr: Vector3<f32> = Vector3::new(0.0, 0.0, 0.0);
        if state.input.forward == ButtonState::Pressed {
            incr += cam_forward * (state.real_delta_time_s as f32) * move_speed;
        }
        if state.input.back == ButtonState::Pressed {
            incr -= cam_forward * (state.real_delta_time_s as f32) * move_speed;
        }
        if state.input.left == ButtonState::Pressed {
            incr -= cam_right * (state.real_delta_time_s as f32) * move_speed;
        }
        if state.input.right == ButtonState::Pressed {
            incr += cam_right * (state.real_delta_time_s as f32) * move_speed;
        }
        if state.input.up == ButtonState::Pressed {
            incr += cam_up * (state.real_delta_time_s as f32) * move_speed;
        }
        if state.input.down == ButtonState::Pressed {
            incr -= cam_up * (state.real_delta_time_s as f32) * move_speed;
        }

        if state.input.m1 == ButtonState::Pressed
            && (state.input.delta_y.abs() > 0 || state.input.delta_x.abs() > 0)
        {
            let half_canvas_height_world =
                state.camera.near * (state.camera.fov_v.to_radians() / 2.0).tan();
            let half_canvas_width_world = aspect * half_canvas_height_world;

            let delta_x_world = -half_canvas_width_world
                * (state.input.delta_x as f32 / (state.canvas_width as f32 / 2.0));
            let delta_y_world = -half_canvas_height_world
                * (state.input.delta_y as f32 / (state.canvas_height as f32 / 2.0));

            let mut x_angle = (delta_x_world / state.camera.near).atan();
            let mut y_angle = (delta_y_world / state.camera.near).atan();
            x_angle *= rotate_speed;
            y_angle *= rotate_speed;

            let curr_pitch_angle = (cam_forward.cross(&state.camera.up).magnitude())
                .atan2(cam_forward.dot(&state.camera.up));

            if lock_pitch {
                if curr_pitch_angle - y_angle < 0.0001 {
                    y_angle = curr_pitch_angle - 0.0001;
                } else if curr_pitch_angle - y_angle > (std::f32::consts::PI - 0.0001) {
                    y_angle = -(std::f32::consts::PI - 0.0001) + curr_pitch_angle;
                };
            }

            let rot_z = Rotation3::from_axis_angle(&state.camera.up, x_angle);
            let rot_x = Rotation3::from_axis_angle(&Unit::new_unchecked(cam_right), y_angle);

            let new_cam_forward = rot_z.transform_vector(&rot_x.transform_vector(&cam_forward));
            let prev_targ_dist: f32 = (state.camera.target - state.camera.pos).magnitude();
            let new_targ = state.camera.pos + new_cam_forward * prev_targ_dist;
            state.camera.target = new_targ;
        }

        state.camera.pos += incr;
        state.camera.target += incr;

        if !lock_pitch {
            state.camera.up = Unit::new_unchecked(cam_up);
        }
    }
}
