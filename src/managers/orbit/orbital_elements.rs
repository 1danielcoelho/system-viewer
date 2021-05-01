use serde::{Deserialize, Serialize};
use crate::utils::units::{Jdn, Mm, Rad};

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
