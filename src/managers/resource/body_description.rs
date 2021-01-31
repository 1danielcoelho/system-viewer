use crate::utils::units::{Jdn, Mm, Rad, J2000_JDN};
use na::{Point3, Vector3};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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
pub struct OrbitalElements {
    pub ref_id: String,
    pub epoch: Jdn,

    #[serde(rename = "a")]
    pub semi_major_axis: Mm,

    #[serde(rename = "e")]
    pub eccentricity: f64,

    #[serde(rename = "i")]
    pub inclination: Rad,

    #[serde(rename = "O")]
    pub long_asc_node: Rad,

    #[serde(rename = "w")]
    pub arg_periapsis: Rad,

    #[serde(rename = "M")]
    pub mean_anomaly_0: Rad,

    #[serde(rename = "p")]
    pub sidereal_orbit_period_days: f64,
}

#[derive(Debug, Clone)]
pub struct StateVector {
    pub jdn_date: Jdn,
    pub pos: Point3<f64>,  // Mm
    pub vel: Vector3<f64>, // Mm/s
}
impl<'a> From<&'a SerializedStateVector> for StateVector {
    fn from(other: &'a SerializedStateVector) -> Self {
        Self {
            jdn_date: Jdn(other.0[0]),
            pos: Point3::new(other.0[1], other.0[2], other.0[3]),
            vel: Vector3::new(other.0[4], other.0[5], other.0[6]),
        }
    }
}
impl Serialize for StateVector {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        SerializedStateVector::from(self).serialize(serializer)
    }
}
impl<'de> Deserialize<'de> for StateVector {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let ser: SerializedStateVector = serde::de::Deserialize::deserialize(deserializer)?;
        return Ok(StateVector::from(&ser));
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SerializedStateVector([f64; 7]);
impl<'a> From<&'a StateVector> for SerializedStateVector {
    fn from(other: &'a StateVector) -> Self {
        return SerializedStateVector([
            other.jdn_date.0,
            other.pos.x,
            other.pos.y,
            other.pos.z,
            other.vel.x,
            other.vel.y,
            other.vel.z,
        ]);
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
    
    pub mass: Option<f32>,                   // Kg
    pub radius: Option<f32>,                 // Mm
    pub albedo: Option<f32>,                 // Abs
    pub magnitude: Option<f32>,              // Abs
    pub rotation_period: Option<f32>,        // Days (86400s)
    pub rotation_axis: Option<Vector3<f64>>, // J2000 ecliptic rectangular right-handed normalized
    pub spec_smassii: Option<String>,        // Spectral class
    pub spec_tholen: Option<String>,         // Spectral class
    
    #[serde(default)]
    pub osc_elements: Vec<OrbitalElements>,
    
    #[serde(default)]
    pub state_vectors: Vec<StateVector>,
}
