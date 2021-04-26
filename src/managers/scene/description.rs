use na::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BodyInstanceDescription {
    pub name: Option<String>,
    pub source: Option<String>,
    pub pos: Option<Vector3<f64>>,
    pub rot: Option<Vector3<f64>>,
    pub scale: Option<Vector3<f64>>,
    pub linvel: Option<Vector3<f64>>,
    pub angvel: Option<Vector3<f64>>,
    pub mass: Option<f32>,
    pub parent: Option<String>,
}

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
