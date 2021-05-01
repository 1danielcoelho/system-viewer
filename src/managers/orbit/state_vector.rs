use crate::utils::units::Jdn;
use na::*;
use serde::{Deserialize, Serialize};
use std::fmt::Display;

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
