use crate::app_state::{AppState, ButtonState, ReferenceChange};
use na::*;

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
        state
            .camera
            .update_transforms(state.canvas_width as f64 / state.canvas_height as f64);

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

    if let Some(selected) = state.selection {
        if state.input.f == ButtonState::Pressed {
            state.camera.next_reference_entity = Some(ReferenceChange::FocusKeepLocation(selected));
        }
    }

    if let Some(selected) = state.selection {
        if state.input.g == ButtonState::Pressed {
            state.camera.entity_going_to = Some(selected);
        }
    }

    if state.input.esc == ButtonState::Pressed {
        state.camera.next_reference_entity = Some(ReferenceChange::Clear);
    }

    // Zoom in/out
    if state.input.modifiers.alt && state.camera.reference_translation.is_some() {
        if state.input.scroll_delta_y < 0 {
            state.camera.pos *= 0.9;
            state.camera.target *= 0.9;
        } else if state.input.scroll_delta_y > 0 {
            state.camera.pos *= 1.1;
            state.camera.target *= 1.1;
        }
    }
    // Change speed
    else {
        if state.input.scroll_delta_y < 0 {
            state.move_speed *= 1.1;
        } else if state.input.scroll_delta_y > 0 {
            state.move_speed *= 0.9;
        }

        state.move_speed = state.move_speed.clamp(0.0, 1000000.0);
    }
    state.input.scroll_delta_y = 0; // We have to "consume" this as it isn't cleared otherwise

    let mut incr: Vector3<f64> = Vector3::new(0.0, 0.0, 0.0);
    if state.input.forward == ButtonState::Pressed {
        incr += cam_forward;
    }
    if state.input.back == ButtonState::Pressed {
        incr -= cam_forward;
    }
    if state.input.left == ButtonState::Pressed {
        incr -= cam_right;
    }
    if state.input.right == ButtonState::Pressed {
        incr += cam_right;
    }
    if state.input.up == ButtonState::Pressed {
        incr += cam_up;
    }
    if state.input.down == ButtonState::Pressed {
        incr -= cam_up;
    }
    if incr.magnitude_squared() > 0.0 {
        incr = incr.normalize() * state.real_delta_time_s * state.move_speed;
    }

    // Orbit
    if state.input.m0 == ButtonState::Pressed
        && (state.input.delta_y.abs() > 0 || state.input.delta_x.abs() > 0)
        && state.input.modifiers.alt
        && state.camera.reference_translation.is_some()
    {
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

        let cam_to_center = -state.camera.pos.coords.normalize();
        let cam_right_tangent = cam_to_center.cross(&state.camera.up).normalize();

        let rot_z = Rotation3::from_axis_angle(&state.camera.up, x_angle);
        let rot_x = Rotation3::from_axis_angle(&Unit::new_unchecked(cam_right_tangent), y_angle);

        // Always orbit about (0, 0, 0), which is where the focused object is
        let new_cam_to_center = rot_z.transform_vector(&rot_x.transform_vector(&cam_to_center));
        let cam_dist_from_center = state.camera.pos.coords.magnitude();
        state.camera.pos = Point3::from(-new_cam_to_center * cam_dist_from_center);
        state.camera.target = Point3::new(0.0, 0.0, 0.0);
    }
    // Look around
    else if state.input.m1 == ButtonState::Pressed
        && (state.input.delta_y.abs() > 0 || state.input.delta_x.abs() > 0)
    {
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

    state.camera.pos += incr;
    state.camera.target += incr;

    if !lock_pitch {
        state.camera.up = Unit::new_unchecked(cam_up);
    }
}
