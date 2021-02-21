use crate::managers::resource::body_description::{OrbitalElements, StateVector};
use na::{Point3, Unit, Vector3};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum BodyMotionType {
    DefaultVector,
    DefaultElements,
    CustomVector(StateVector),
    CustomElements(OrbitalElements),
}

#[derive(Clone, Debug)]
pub enum ResolvedBodyMotionType {
    Vector(StateVector),
    Elements(OrbitalElements),
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SceneDescription {
    pub name: String,
    pub description: String,
    pub time: String,
    pub simulation_scale: f64,
    
    #[serde(default)]
    pub tracking: Option<String>,
    
    #[serde(default)]
    pub camera_pos: Option<Point3<f64>>,
    
    #[serde(default)]
    pub camera_up: Option<Unit<Vector3<f64>>>,

    #[serde(default)]
    pub camera_target: Option<Point3<f64>>,
    pub bodies: HashMap<String, HashMap<String, BodyMotionType>>,
}
