use crate::managers::resource::body_description::BodyInstanceDescription;
use na::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct SceneDescription {
    pub name: String,
    pub description: String,
    pub time: String,
    pub simulation_scale: f64,

    #[serde(default)]
    pub focus: Option<String>,

    #[serde(default)]
    pub camera_pos: Option<Point3<f64>>,

    #[serde(default)]
    pub camera_up: Option<Unit<Vector3<f64>>>,

    #[serde(default)]
    pub camera_target: Option<Point3<f64>>,
    pub bodies: Vec<BodyInstanceDescription>,
}
