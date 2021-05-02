use na::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Copy)]
#[serde(rename_all = "lowercase")]
pub enum BodyType {
    Star,
    Planet,
    Satellite,
    Asteroid,
    Comet,
    Artificial,
    Barycenter,
    Other,
}
impl Default for BodyType {
    fn default() -> Self {
        BodyType::Other
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BodyDescription {
    pub id: Option<String>,
    pub name: String,

    #[serde(rename = "type")]
    pub body_type: BodyType,

    #[serde(default)]
    pub meta: HashMap<String, String>,

    pub mass: Option<f32>,               // Kg
    pub radius: Option<f32>,             // Mm
    pub albedo: Option<f32>,             // Abs
    pub magnitude: Option<f32>,          // Abs
    pub brightness: Option<f32>,         // Candela intensity value for light sources
    pub rotation_period: Option<f32>,    // Days (86400s)
    pub rotation_axis: Option<[f64; 3]>, // J2000 ecliptic rectangular right-handed normalized
    pub spec_smassii: Option<String>,    // Spectral class
    pub spec_tholen: Option<String>,     // Spectral class

    #[serde(default)]
    pub mesh: Option<String>,
    #[serde(default)]
    pub mesh_params: Option<HashMap<String, String>>,

    #[serde(default)]
    pub material: Option<String>,
    #[serde(default)]
    pub material_params: Option<HashMap<String, String>>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BodyInstanceDescription {
    // IDs
    pub name: Option<String>,
    pub source: Option<String>,
    pub parent: Option<String>,

    // Transform
    pub pos: Option<Point3<f64>>,
    pub rot: Option<Vector3<f64>>,
    pub scale: Option<Vector3<f64>>,

    // Physics
    pub linvel: Option<Vector3<f64>>,
    pub angvel: Option<Vector3<f64>>,

    // BodyDescription overrides
    pub mass: Option<f32>,
    pub radius: Option<f32>,
    pub brightness: Option<f32>,
    pub mesh: Option<String>,
    pub mesh_params: Option<HashMap<String, String>>,
    pub material: Option<String>,
    pub material_params: Option<HashMap<String, String>>,
}
