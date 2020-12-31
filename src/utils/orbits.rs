use std::f64::consts::PI;

use crate::utils::{
    transform::Transform,
    units::{Au, Deg, Jdn, Mm, Rad, RadsPerDay},
};
use na::{Point3, UnitQuaternion, Vector3};
use regex::Regex;
use serde::{Deserialize, Serialize};

lazy_static! {
    static ref TARGET_BODY_NAME_RE: Regex = Regex::new(r"Target body name: ([^;]+?) \(").unwrap();
    static ref TARGET_BODY_ID_RE: Regex =
        Regex::new(r"Target body name: [^;]+? \((\d+)\)").unwrap();
    static ref CENTER_BODY_NAME_RE: Regex = Regex::new(r"Center body name: ([^;]+?) \(").unwrap();
    static ref CENTER_BODY_ID_RE: Regex =
        Regex::new(r"Center body name: [^;]+? \((\d+)\)").unwrap();
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

const GRAVITATION_CONSTANT: f64 = 
const J2000_JDN: Jdn = Jdn(2451545.0);
const NEWTON_RAPHSON_MAX_ITER: u32 = 30;
const NEWTON_RAPHSON_DELTA: f64 = 0.000001;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum BodyType {
    Barycenter,
    Star,
    Planet,
    Moon,
    Asteroid,
    Comet,
    Artificial,
    Other,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BodyDescription {
    pub id: u32,
    pub name: String,
    pub reference_id: u32,
    pub body_type: BodyType,
    pub mean_radius: Mm,
    pub mass: f64,
    // Rotation, rotation axis?
    pub orbital_elements: OrbitalElements,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OrbitalElements {
    pub semi_major_axis: Mm,
    pub eccentricity: f64,
    pub inclination: Rad, // Rad to prevent a rogue .sin() from spewing nonsense
    pub long_asc_node: Rad,
    pub arg_periapsis: Rad,
    pub mean_anomaly_0: Rad,
}

fn float_from_match(s: &str, regex: &Regex) -> Option<f64> {
    return regex
        .captures(s)
        .and_then(|c| c.get(1))
        .and_then(|m| Some(m.as_str()))
        .and_then(|s| s.parse().ok());
}

fn int_from_match(s: &str, regex: &Regex) -> Option<u32> {
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

pub fn parse_csv_lines(file_str: &str) -> Result<Vec<BodyDescription>, String> {
    let mut results = Vec::new();
    for line in file_str.lines() {
        results.push(parse_csv_line(line)?);
    }

    return Ok(results);
}

pub fn parse_csv_line(line_str: &str) -> Result<BodyDescription, String> {
    let elements: Vec<&str> = line_str.split(",").collect();

    const EXPECTED: usize = 11;
    if elements.len() != EXPECTED {
        return Err(format!(
            "Incorrect number of elements in csv line! Expected: {}. Found: {}",
            EXPECTED,
            elements.len()
        ));
    }

    // Body data
    let id: u32 = elements[0].parse::<u32>().map_err(|err| err.to_string())?;
    let name: String = elements[1].to_owned();
    let reference_id: u32 = elements[2].parse::<u32>().map_err(|err| err.to_string())?;
    let body_type: BodyType = match elements[3] {
        "star" => BodyType::Star,
        "planet" => BodyType::Planet,
        "system barycenter" => BodyType::Barycenter,
        "satellite" => BodyType::Moon,
        _ => BodyType::Other,
    };
    let mean_radius: Mm = Mm(elements[10].parse::<f64>().map_err(|err| err.to_string())?);

    // Orbit data
    let semi_major_axis: Mm = Mm(elements[4].parse::<f64>().map_err(|err| err.to_string())?);
    let eccentricity: f64 = elements[5].parse::<f64>().map_err(|err| err.to_string())?;
    let inclination: Rad = Deg(elements[6].parse::<f64>().map_err(|err| err.to_string())?).to_rad();
    let long_asc_node: Rad =
        Deg(elements[7].parse::<f64>().map_err(|err| err.to_string())?).to_rad();
    let arg_periapsis: Rad =
        Deg(elements[8].parse::<f64>().map_err(|err| err.to_string())?).to_rad();
    let mean_anomaly_0: Rad =
        Deg(elements[9].parse::<f64>().map_err(|err| err.to_string())?).to_rad();

    return Ok(BodyDescription {
        id,
        name,
        reference_id,
        body_type,
        mass: todo!(),
        mean_radius,
        orbital_elements: OrbitalElements {
            semi_major_axis,
            eccentricity,
            inclination,
            long_asc_node,
            arg_periapsis,
            mean_anomaly_0,
        },
    });
}

pub fn parse_ephemerides(file_str: &str) -> Result<BodyDescription, String> {
    let semi_major_axis = Au(
        float_from_match(file_str, &SEMI_MAJOR_AXIS_RE).ok_or(format!(
            "Failed to extract semi major axis from this file!\n{}",
            file_str
        ))?,
    )
    .to_Mm();

    let eccentricity = float_from_match(file_str, &ECCENTRICITY_RE).ok_or(format!(
        "Failed to extract eccentricity from this file!\n{}",
        file_str
    ))?;

    let inclination = Deg(float_from_match(file_str, &INCLINATION_RE).ok_or(format!(
        "Failed to extract inclination from this file!\n{}",
        file_str
    ))?)
    .to_rad();

    let long_asc_node = Deg(float_from_match(file_str, &LONG_ASC_NODE_RE).ok_or(format!(
        "Failed to extract longitude of ascending node from this file!\n{}",
        file_str
    ))?)
    .to_rad();

    let arg_periapsis = Deg(float_from_match(file_str, &ARG_PERIAPSIS_RE).ok_or(format!(
        "Failed to extract argument of periapsis from this file!\n{}",
        file_str
    ))?)
    .to_rad();

    let mean_anomaly_0 = Deg(float_from_match(file_str, &TRUE_ANOMALY_RE).ok_or(format!(
        "Failed to extract true anomaly from this file!\n{}",
        file_str
    ))?)
    .to_rad();

    let name = string_from_match(file_str, &TARGET_BODY_NAME_RE).ok_or(format!(
        "Failed to extract body name from this file!\n{}",
        file_str
    ))?;

    let id = int_from_match(file_str, &TARGET_BODY_ID_RE).ok_or(format!(
        "Failed to extract body id from this file!\n{}",
        file_str
    ))?;

    let reference_id = int_from_match(file_str, &CENTER_BODY_ID_RE).ok_or(format!(
        "Failed to extract reference body id from this file!\n{}",
        file_str
    ))?;

    let mean_radius = Mm(float_from_match(file_str, &MEAN_RADIUS_RE).unwrap_or(0.0) / 1000.0);

    let body_type = {
        if id == 0 {
            BodyType::Star
        } else if id > 100 && (id + 1) % 100 == 0 {
            BodyType::Planet
        } else if reference_id == 0 && id < 10 {
            BodyType::Barycenter
        } else {
            BodyType::Other
        }
    };

    return Ok(BodyDescription {
        id,
        mean_radius,
        name,
        reference_id,
        mass: todo!(),
        body_type,
        orbital_elements: OrbitalElements {
            semi_major_axis,
            eccentricity,
            inclination,
            long_asc_node,
            arg_periapsis,
            mean_anomaly_0,
        },
    });
}

pub fn elements_to_circle_transform(elements: &OrbitalElements) -> Transform {
    let mut result = Transform::identity();

    let b = elements.semi_major_axis.0 * (1.0 - elements.eccentricity.powi(2)).sqrt();

    // Shaping transform for semi-major and minor axes
    result
        .concat(&Transform {
            scale: Vector3::new(elements.semi_major_axis.0 as f32, b as f32, 1.0),
            ..Transform::identity()
        })
        .concat(&elements_to_ellipse_rotation_transform(elements));

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

/// Returns the transform to rotate an ellipse with correct semi-major/minor axes into the correct
/// orientation to match it's orbital elements. Corresponds to applying, in order:
/// - Rotation around +Z with -arg_periapsis
/// - Rotation around +X with -inclination
/// - Rotation around +Z with -long_asc_node
///
/// Sources:
/// - https://downloads.rene-schwarz.com/download/M001-Keplerian_Orbit_Elements_to_Cartesian_State_Vectors.pdf
pub fn elements_to_ellipse_rotation_transform(elements: &OrbitalElements) -> Transform {
    let mut result = Transform::identity();

    // Apply inclination around world axes
    result = Transform {
        rot: UnitQuaternion::from_axis_angle(&Vector3::x_axis(), elements.inclination.0 as f32),
        ..Transform::identity()
    }
    .concat_clone(&result);

    // Rotate for longitude of ascending node around world axes
    result = Transform {
        rot: UnitQuaternion::from_axis_angle(&Vector3::z_axis(), elements.long_asc_node.0 as f32),
        ..Transform::identity()
    }
    .concat_clone(&result);

    // Rotate for argument of periapsis around local axes
    result.concat(&Transform {
        rot: UnitQuaternion::from_axis_angle(&Vector3::z_axis(), elements.arg_periapsis.0 as f32),
        ..Transform::identity()
    });

    return result;
}

/// Sources:
/// - https://space.stackexchange.com/questions/19322/converting-orbital-elements-to-cartesian-state-vectors
/// - https://downloads.rene-schwarz.com/download/M001-Keplerian_Orbit_Elements_to_Cartesian_State_Vectors.pdf
///
/// Returns (position, velocity) in world space (cartesian coordinates), in Mm and Mm / day (86400 s)
pub fn orbital_elements_to_xyz(
    elements: &OrbitalElements,
    gravitation_const: f64,
    t: Jdn,
    ellipse_rotation_transform: &Transform,
) -> (Point3<f32>, Vector3<f32>) {
    // Calculate mean anomaly at t
    let mean_anomaly = elements.mean_anomaly_0.0
        + (t.0 - J2000_JDN.0) * (gravitation_const / elements.semi_major_axis.0.powi(3)).sqrt();

    // Find eccentric anomaly by solving Kepler's equation using Newton-Raphson
    let mut eccentric_anomaly = mean_anomaly;
    let mut error =
        eccentric_anomaly - elements.eccentricity * eccentric_anomaly.sin() - mean_anomaly;
    for _ in 0..NEWTON_RAPHSON_MAX_ITER {
        if error.abs() <= NEWTON_RAPHSON_DELTA {
            break;
        }

        eccentric_anomaly =
            eccentric_anomaly - error / (1.0 - elements.eccentricity * eccentric_anomaly.cos());
        error = eccentric_anomaly - elements.eccentricity * eccentric_anomaly.sin() - mean_anomaly;
    }
    if error > NEWTON_RAPHSON_DELTA {
        log::warn!(
            "Failed to converge eccentric anomaly for time {:?} and orbital elements {:#?}",
            t,
            elements
        );
    }

    // Find true anomaly from eccentric anomaly
    let true_anomaly = 2.0
        * ((1.0 + elements.eccentricity).sqrt() * (eccentric_anomaly * 0.5).sin())
            .atan2((1.0 - elements.eccentricity).sqrt() * (eccentric_anomaly * 0.5).cos());

    // Find distance to central body
    let dist_to_body =
        elements.semi_major_axis.0 * (1.0 - elements.eccentricity * eccentric_anomaly.cos());

    // Body position in orbital frame (ellipse on XY plane, +X going from center to periapsis)
    let pos_temp = Point3::new(
        (dist_to_body * true_anomaly.cos()) as f32,
        (dist_to_body * true_anomaly.sin()) as f32,
        0.0,
    );
    let vel_temp = Vector3::new(
        -eccentric_anomaly.sin() as f32,
        ((1.0 - elements.eccentricity.powi(2)).sqrt() * eccentric_anomaly.cos()) as f32,
        0.0,
    )
    .scale(((gravitation_const * elements.semi_major_axis.0).sqrt() / dist_to_body) as f32);

    return (
        ellipse_rotation_transform.transform_point(&pos_temp),
        ellipse_rotation_transform.transform_vector(&vel_temp),
    );
}

/// It's pretty expensive to repeatedly call orbital_elements_to_xyz. Use this function
/// to find the times in JDN where the body will cross each angle in the orbit, and then
/// during execution interpolate each time to find the eccentric anomaly for it, then just
/// do the end of the calculation.
pub fn bake_eccentric_anomaly_times(
    elements: &OrbitalElements,
    gravitation_const: f64,
    num_angles: u32,
) -> Vec<f64> {
    let result: Vec<f64> = Vec::new();

    let mean_motion = RadsPerDay((gravitation_const / elements.semi_major_axis.0.powi(3)).sqrt());

    let incr = 360.0 / num_angles as f64;
    for i in 0..num_angles {
        let eccentric_anomaly = (i as f64 * incr).to_radians();

        let mean_anomaly = eccentric_anomaly - elements.eccentricity * eccentric_anomaly.sin();
        let t = (mean_anomaly + mean_motion * time_of_periapsis) / mean_motion;
    }

    todo!();
}

/// Returns the JDN time of the next periapsis crossing
pub fn time_of_next_periapsis(mean_anomaly_at_epoch: Rad, mean_motion: f64) -> Jdn {
    return Jdn((2.0 * PI - mean_anomaly_at_epoch.0) / mean_motion + J2000_JDN.0);
}

/// Returns the time of the periapsis crossing immediately before JDN
pub fn time_of_prev_periapsis(mean_anomaly_at_epoch: Rad, mean_motion: f64) -> Jdn {
    return Jdn((0.0 - mean_anomaly_at_epoch.0) / mean_motion + J2000_JDN.0);
}

pub fn orbital_period(mean_motion: RadsPerDay) -> Jdn {
    return Jdn(2.0 * PI / mean_motion.0);
}

pub fn mean_motion(orbital_period: Jdn) -> RadsPerDay {
    return RadsPerDay(2.0 * PI / orbital_period.0);
}

/// Returns u = GM, in units of (Mm^3 / day^2) so that it can be used with all other stuff
pub fn get_gravitational_parameter(reference_mass_kg: f64) -> f64 {
    
}
