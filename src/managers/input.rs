

use cgmath::{Basis3, Deg, InnerSpace, MetricSpace, Rotation, Rotation3, Vector3};



use crate::app_state::AppState;



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
        state.input.delta_x = state.input.mouse_x - self.last_mouse_x;
        state.input.delta_y = state.input.mouse_y - self.last_mouse_y;

        self.last_mouse_x = state.input.mouse_x;
        self.last_mouse_y = state.input.mouse_y;

        let aspect = state.canvas_width as f32 / state.canvas_height as f32;

        let cam_forward = ((state.camera.target - state.camera.pos) as Vector3<f32>).normalize();
        let cam_right: Vector3<f32> = cam_forward.cross(state.camera.up).normalize();
        let cam_up: Vector3<f32> = cam_right.cross(cam_forward).normalize();

        let lock_pitch = true;

        let move_speed = state.move_speed * 0.005;
        let rotate_speed = state.rotate_speed * 0.5;

        let mut incr: cgmath::Vector3<f32> = cgmath::Vector3::new(0.0, 0.0, 0.0);
        if state.input.forward_down {
            incr += cam_forward * (state.real_delta_time_ms as f32) * move_speed;
        }
        if state.input.back_down {
            incr -= cam_forward * (state.real_delta_time_ms as f32) * move_speed;
        }
        if state.input.left_down {
            incr -= cam_right * (state.real_delta_time_ms as f32) * move_speed;
        }
        if state.input.right_down {
            incr += cam_right * (state.real_delta_time_ms as f32) * move_speed;
        }
        if state.input.up_down {
            incr += cam_up * (state.real_delta_time_ms as f32) * move_speed;
        }
        if state.input.down_down {
            incr -= cam_up * (state.real_delta_time_ms as f32) * move_speed;
        }

        if state.input.m1_down && (state.input.delta_y.abs() > 0 || state.input.delta_x.abs() > 0) {
            let half_canvas_height_world =
                state.camera.near * cgmath::Angle::tan(state.camera.fov_v / 2.0);
            let half_canvas_width_world = aspect * half_canvas_height_world;

            let delta_x_world = -half_canvas_width_world
                * (state.input.delta_x as f32 / (state.canvas_width as f32 / 2.0));
            let delta_y_world = -half_canvas_height_world
                * (state.input.delta_y as f32 / (state.canvas_height as f32 / 2.0));

            let mut x_angle: Deg<f32> = cgmath::Angle::atan(delta_x_world / state.camera.near);
            let mut y_angle: Deg<f32> = cgmath::Angle::atan(delta_y_world / state.camera.near);
            x_angle *= rotate_speed;
            y_angle *= rotate_speed;

            let curr_pitch_angle: Deg<f32> = cgmath::Angle::atan2(
                cam_forward.cross(state.camera.up).magnitude(),
                cam_forward.dot(state.camera.up),
            );

            if lock_pitch {
                if curr_pitch_angle - y_angle < Deg(0.001) {
                    y_angle = curr_pitch_angle - Deg(0.001);
                } else if curr_pitch_angle - y_angle > Deg(179.999) {
                    y_angle = -Deg(179.999) + curr_pitch_angle;
                };
            }

            let rot_z: Basis3<f32> = Rotation3::from_axis_angle(state.camera.up, x_angle);
            let rot_x: Basis3<f32> = Rotation3::from_axis_angle(cam_right, y_angle);

            let new_cam_forward = rot_z.rotate_vector(rot_x.rotate_vector(cam_forward));
            let prev_targ_dist: f32 = state.camera.target.distance(state.camera.pos);
            let new_targ = state.camera.pos + new_cam_forward * prev_targ_dist;
            state.camera.target = new_targ;
        }

        state.camera.pos += incr;
        state.camera.target += incr;

        if !lock_pitch {
            state.camera.up = cam_up;
        }
    }
}
