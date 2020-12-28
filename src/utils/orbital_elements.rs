use crate::utils::transform::Transform;
use na::{UnitQuaternion, Vector3};
use regex::Regex;

lazy_static! {
    static ref TARGET_BODY_NAME_RE: Regex = Regex::new(r"Target body name: ([^;]+?) \(").unwrap();
    static ref CENTER_BODY_NAME_RE: Regex = Regex::new(r"Center body name: ([^;]+?) \(").unwrap();
    static ref ECCENTRICITY_RE: Regex = Regex::new(r" EC= ([\d\-+eE.]+)").unwrap();
    static ref PERIAPSIS_DISTANCE_RE: Regex = Regex::new(r" QR= ([\d\-+eE.]+)").unwrap();
    static ref INCLINATION_RE: Regex = Regex::new(r" IN= ([\d\-+eE.]+)").unwrap();
    static ref LONG_ASC_NODE_RE: Regex = Regex::new(r" OM= ([\d\-+eE.]+)").unwrap();
    static ref ARG_PERIAPSIS_RE: Regex = Regex::new(r" W = ([\d\-+eE.]+)").unwrap();
    static ref TIME_OF_PERIAPSIS_RE: Regex = Regex::new(r" Tp= ([\d\-+eE.]+)").unwrap();
    static ref MEAN_MOTION_RE: Regex = Regex::new(r" N = ([\d\-+eE.]+)").unwrap();
    static ref MEAN_ANOMALY_RE: Regex = Regex::new(r" MA= ([\d-+eE.]+)").unwrap();
    static ref TRUE_ANOMALY_RE: Regex = Regex::new(r" TA= ([\d\-+eE.]+)").unwrap();
    static ref SEMI_MAJOR_AXIS_RE: Regex = Regex::new(r" A = ([\d\-+eE.]+)").unwrap();
    static ref APOAPSIS_DISTANCE_RE: Regex = Regex::new(r" AD= ([\d\-+eE.]+)").unwrap();
    static ref SIDERAL_ORBIT_PERIOD_RE: Regex = Regex::new(r" PR= ([\d\-+eE.]+)").unwrap();
    static ref MEAN_RADIUS_RE: Regex =
        Regex::new(r"[R,r]adius[ \t\(\)IAU,]+km[ \t\)=]+([\d.x ]+)").unwrap();
}

#[derive(Debug)]
pub struct BodyDescription {
    pub id: String,
    pub mean_radius: f64, // Km, 0 for e.g. barycenters
                          // Rotation, rotation axis?
}

#[derive(Debug)]
pub struct OrbitalElements {
    pub semi_major_axis: f64, // Km
    pub eccentricity: f64,
    pub inclination: f64,   // Deg
    pub long_asc_node: f64, // Deg
    pub arg_periapsis: f64, // Deg
    pub true_anomaly: f64,  // Deg
}

#[derive(Debug)]
pub struct Ellipse {}

fn float_from_match(s: &str, regex: &Regex) -> Option<f64> {
    return regex
        .captures(s)
        .and_then(|c| c.get(1))
        .and_then(|m| Some(m.as_str()))
        .and_then(|s| s.parse().ok());
}

fn string_from_match(s: &str, regex: &Regex) -> Option<String> {
    return regex
        .captures(s)
        .and_then(|c| c.get(1))
        .and_then(|m| Some(m.as_str().to_owned()));
}

pub fn parse_ephemerides(file_str: &str) -> Result<(OrbitalElements, BodyDescription), String> {
    let semi_major_axis = float_from_match(file_str, &SEMI_MAJOR_AXIS_RE).ok_or(format!(
        "Failed to extract semi major axis from this file!\n{}",
        file_str
    ))?;
    let eccentricity = float_from_match(file_str, &ECCENTRICITY_RE).ok_or(format!(
        "Failed to extract eccentricity from this file!\n{}",
        file_str
    ))?;
    let inclination = float_from_match(file_str, &INCLINATION_RE).ok_or(format!(
        "Failed to extract inclination from this file!\n{}",
        file_str
    ))?;
    let long_asc_node = float_from_match(file_str, &LONG_ASC_NODE_RE).ok_or(format!(
        "Failed to extract longitude of ascending node from this file!\n{}",
        file_str
    ))?;
    let arg_periapsis = float_from_match(file_str, &ARG_PERIAPSIS_RE).ok_or(format!(
        "Failed to extract argument of periapsis from this file!\n{}",
        file_str
    ))?;
    let true_anomaly = float_from_match(file_str, &TRUE_ANOMALY_RE).ok_or(format!(
        "Failed to extract true anomaly from this file!\n{}",
        file_str
    ))?;

    // TODO: Obliquity to orbit seems like what I need but it feels like it's missing one degree of freedom

    let id = string_from_match(file_str, &TARGET_BODY_NAME_RE).ok_or(format!(
        "Failed to extract body name from this file!\n{}",
        file_str
    ))?;
    let mean_radius = float_from_match(file_str, &MEAN_RADIUS_RE).unwrap_or(0.0);

    // We use 1000 km per unit
    return Ok((
        OrbitalElements {
            semi_major_axis: semi_major_axis * 149597.8707, // AU to 1000 km
            eccentricity,
            inclination,
            long_asc_node,
            arg_periapsis,
            true_anomaly,
        },
        BodyDescription {
            id,
            mean_radius: mean_radius / 1000.0,
        },
    ));
}

pub fn elements_to_circle_transform(elements: &OrbitalElements) -> Transform {
    let mut result = Transform::identity();

    let b = elements.semi_major_axis * (1.0 - elements.eccentricity.powi(2)).sqrt();

    // Shaping transform for semi-major and minor axes
    result.concat(&Transform {
        scale: Vector3::new(elements.semi_major_axis as f32, b as f32, 1.0),
        ..Transform::identity()
    });

    // Apply inclination around world axes
    result = Transform {
        rot: UnitQuaternion::from_axis_angle(
            &Vector3::x_axis(),
            elements.inclination.to_radians() as f32,
        ),
        ..Transform::identity()
    }
    .concat_clone(&result);

    // Rotate for longitude of ascending node around world axes
    result = Transform {
        rot: UnitQuaternion::from_axis_angle(
            &Vector3::z_axis(),
            elements.long_asc_node.to_radians() as f32,
        ),
        ..Transform::identity()
    }
    .concat_clone(&result);

    // Rotate for argument of periapsis around local axes
    result.concat(&Transform {
        rot: UnitQuaternion::from_axis_angle(
            &Vector3::z_axis(),
            elements.arg_periapsis.to_radians() as f32,
        ),
        ..Transform::identity()
    });

    result.concat(&Transform {
        trans: Vector3::new(
            -elements.eccentricity as f32, // This is a move by -a*e, but we already have our x axis scaled by 'a'
            0.0,
            0.0,
        ),
        ..Transform::identity()
    });

    return result;
}
