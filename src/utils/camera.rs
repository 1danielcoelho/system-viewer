use na::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Camera {
    pub pos: Point3<f64>,
    pub up: Unit<Vector3<f64>>,
    pub target: Point3<f64>,
    pub fov_v: f64,
    pub near: f64,
    pub far: f64,

    // Calculated once per frame after inputs are accounted for
    #[serde(skip)]
    pub v: Matrix4<f64>,
    #[serde(skip)]
    pub p: Matrix4<f64>,
    #[serde(skip)]
    pub v_inv: Matrix4<f64>,
    #[serde(skip)]
    pub p_inv: Matrix4<f64>,
}
impl Camera {
    /// Converts from pixels (with 0,0 on top left of canvas) into world space coordinates
    pub fn canvas_to_world(
        &self,
        x: i32,
        y: i32,
        canvas_width: u32,
        canvas_height: u32,
    ) -> Point3<f64> {
        let ndc_to_world: Matrix4<f64> = self.v_inv * self.p_inv;

        let ndc_near_pos = Point3::from(Vector3::new(
            -1.0 + 2.0 * x as f64 / (canvas_width - 1) as f64,
            1.0 - 2.0 * y as f64 / (canvas_height - 1) as f64,
            -1.0,
        ));

        return ndc_to_world.transform_point(&ndc_near_pos);
    }

    /// Converts from Mm xyz to pixel xy (with 0,0 on top left of canvas). Also returns whether the point is in front of camera or not
    pub fn world_to_canvas(
        &self,
        pt: &Point3<f64>,
        canvas_width: u32,
        canvas_height: u32,
    ) -> (i32, i32, bool) {
        let world_to_ndc = self.p * self.v;

        let ndc = world_to_ndc.transform_point(pt);

        return (
            (canvas_width as f64 * (ndc.x + 1.0) / 2.0) as i32 + 1,
            (canvas_height as f64 * (1.0 - ndc.y) / 2.0) as i32 + 1,
            ndc.z > 0.0 && ndc.z < 1.0,
        );
    }

    pub fn update_transforms(
        &mut self,
        aspect_ratio: f64,
        reference_translation: Option<Vector3<f64>>,
    ) {
        self.p = Matrix4::new_perspective(
            aspect_ratio,
            self.fov_v.to_radians() as f64,
            self.near as f64,
            self.far as f64,
        );
        self.p_inv = self.p.try_inverse().unwrap();

        self.v = Matrix4::look_at_rh(&self.pos, &self.target, &self.up);
        if let Some(trans) = reference_translation {
            self.v *= Translation3::from(-trans).to_homogeneous();
        }
        self.v_inv = self.v.try_inverse().unwrap();
    }
}
