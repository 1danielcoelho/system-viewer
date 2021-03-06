use crate::managers::resource::body_description::{OrbitalElements, StateVector};
use na::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum BodyMotionType {
    DefaultVector,
    DefaultElements,
    CustomVector,
    CustomElements,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BodyInstanceDescription {
    pub motion_type: BodyMotionType,
    pub state_vector: Option<StateVector>,
    pub initial_rot: Option<Vector3<f64>>,
    pub scale: Option<Vector3<f64>>,
    pub orbital_elements: Option<OrbitalElements>,
    pub angular_velocity: Option<Vector3<f64>>,
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
    pub bodies: HashMap<String, HashMap<String, BodyInstanceDescription>>,
}
