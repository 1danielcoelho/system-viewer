use crate::utils::{
    transform::Transform,
    units::{Au, Deg, Jdn, Mm, Rad},
};
use na::{Point3, UnitQuaternion, Vector3};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::f64::consts::PI;

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

const GRAVITATION_CONSTANT: f64 = 4.9823382528e-19; // Mm3 / (kg day2)
const J2000_JDN: Jdn = Jdn(2451545.0);
const NEWTON_RAPHSON_MAX_ITER: u32 = 30;
const NEWTON_RAPHSON_DELTA: f64 = 0.00000001;

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

pub fn elements_to_circle_transform(elements: &OrbitalElements) -> Transform<f64> {
    let mut result = Transform::identity();

    let b = elements.semi_major_axis.0 * (1.0 - elements.eccentricity.powi(2)).sqrt();

    // Shaping transform for semi-major and minor axes
    result
        .concat(&Transform {
            scale: Vector3::new(elements.semi_major_axis.0, b, 1.0),
            ..Transform::identity()
        })
        .concat(&elements_to_ellipse_rotation_transform(elements));

    result.concat(&Transform {
        trans: Vector3::new(
            -elements.eccentricity, // This is a move by -a*e, but we already have our x axis scaled by 'a'
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
pub fn elements_to_ellipse_rotation_transform(elements: &OrbitalElements) -> Transform<f64> {
    let mut result = Transform::identity();

    // Apply inclination around world axes
    result = Transform {
        rot: UnitQuaternion::from_axis_angle(&Vector3::x_axis(), elements.inclination.0),
        ..Transform::identity()
    }
    .concat_clone(&result);

    // Rotate for longitude of ascending node around world axes
    result = Transform {
        rot: UnitQuaternion::from_axis_angle(&Vector3::z_axis(), elements.long_asc_node.0),
        ..Transform::identity()
    }
    .concat_clone(&result);

    // Rotate for argument of periapsis around local axes
    result.concat(&Transform {
        rot: UnitQuaternion::from_axis_angle(&Vector3::z_axis(), elements.arg_periapsis.0),
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
    sidereal_orbit_period_days: f64,
    t: Jdn,
    ellipse_rotation_transform: &Transform<f64>,
) -> (Point3<f64>, Vector3<f64>) {
    let mean_motion = 2.0 * PI / sidereal_orbit_period_days; // Rads/day
    let gravitation_const = mean_motion * mean_motion * elements.semi_major_axis.0.powi(3);

    // Calculate mean anomaly at t
    let mean_anomaly = elements.mean_anomaly_0.0 + mean_motion * (t.0 - J2000_JDN.0);

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
        dist_to_body * true_anomaly.cos(),
        dist_to_body * true_anomaly.sin(),
        0.0,
    );
    let vel_temp = Vector3::new(
        -eccentric_anomaly.sin(),
        (1.0 - elements.eccentricity.powi(2)).sqrt() * eccentric_anomaly.cos(),
        0.0,
    )
    .scale((gravitation_const * elements.semi_major_axis.0).sqrt() / dist_to_body);

    // Reference calculations, should also work:
    // let mut pos_x = dist_to_body * true_anomaly.cos();
    // let mut pos_y = dist_to_body * true_anomaly.sin();
    // let mut vel_x = -eccentric_anomaly.sin();
    // let mut vel_y = (1.0 - elements.eccentricity.powi(2)).sqrt() * eccentric_anomaly.cos();
    // let scaling = (gravitation_const * elements.semi_major_axis.0).sqrt() / dist_to_body;
    // vel_x *= scaling;
    // vel_y *= scaling;
    // let cosw = elements.arg_periapsis.0.cos();
    // let sinw = elements.arg_periapsis.0.sin();
    // let coso = elements.long_asc_node.0.cos();
    // let sino = elements.long_asc_node.0.sin();
    // let cosi = elements.inclination.0.cos();
    // let sini = elements.inclination.0.sin();
    // let final_pos_x = pos_x * (cosw * coso - sinw * cosi * sino) - pos_y * (sinw * coso + cosw * cosi * sino);
    // let final_pos_y = pos_x * (cosw * sino + sinw * cosi * coso) + pos_y * (cosw * cosi * coso - sinw * sino);
    // let final_pos_z =                      pos_x * (sinw * sini) + pos_y * (cosw * sini);
    // let final_vel_x = vel_x * (cosw * coso - sinw * cosi * sino) - vel_y * (sinw * coso + cosw * cosi * sino);
    // let final_vel_y = vel_x * (cosw * sino + sinw * cosi * coso) + vel_y * (cosw * cosi * coso - sinw * sino);
    // let final_vel_z =                      vel_x * (sinw * sini) + vel_y * (cosw * sini);

    return (
        ellipse_rotation_transform.transform_point(&pos_temp),
        ellipse_rotation_transform.transform_vector(&vel_temp),
    );
}

/// It's pretty expensive to repeatedly call orbital_elements_to_xyz. Use this function
/// to find the times in JDN where the body will cross each angle in the orbit, and then
/// during execution interpolate each time to find the eccentric anomaly for it, then just
/// do the end of the calculation.
///
/// Note: if num_angles is N, N+1 values will be returned, because we want the time for angle 0 and also for 2pi
pub fn bake_eccentric_anomaly_times(
    elements: &OrbitalElements,
    sidereal_orbit_period_days: f64,
    num_angles: u32,
) -> Vec<f64> {
    let mut result: Vec<f64> = Vec::new();
    result.reserve((num_angles + 1) as usize);

    let mean_motion = 2.0 * PI / sidereal_orbit_period_days; // Rads/day
    let time_of_periapsis: Jdn = time_of_prev_periapsis(elements.mean_anomaly_0, mean_motion);

    log::info!(
        "Mean motion: {}, time of periapsis: {}",
        mean_motion,
        time_of_periapsis.0
    );

    let incr = 360.0 / num_angles as f64;
    for i in 0..=num_angles {
        let eccentric_anomaly = (i as f64 * incr).to_radians();

        let mean_anomaly = eccentric_anomaly - elements.eccentricity * eccentric_anomaly.sin(); // Rad
        let t = (mean_anomaly + mean_motion * time_of_periapsis.0) / mean_motion; // day

        result.push(t);
    }

    return result;
}

/// Returns the JDN time of the next periapsis crossing
pub fn time_of_next_periapsis(mean_anomaly_at_epoch: Rad, mean_motion: f64) -> Jdn {
    return Jdn((2.0 * PI - mean_anomaly_at_epoch.0) / mean_motion + J2000_JDN.0);
}

/// Returns the time of the periapsis crossing immediately before JDN
pub fn time_of_prev_periapsis(mean_anomaly_at_epoch: Rad, mean_motion: f64) -> Jdn {
    return Jdn((0.0 - mean_anomaly_at_epoch.0) / mean_motion + J2000_JDN.0);
}

// pub use tests::*;
pub mod tests {
    extern crate wasm_bindgen_test;
    use wasm_bindgen_test::*;
    wasm_bindgen_test_configure!(run_in_browser);
    use super::*;

    const ACCEPTABLE_DELTA: f64 = 0.0000000001;

    #[wasm_bindgen_test]
    pub fn elements_to_cartesian_numerical() {
        // Venus heliocentric
        let elements = OrbitalElements {
            semi_major_axis: Au(7.233269274790103E-01).to_Mm(),
            eccentricity: 6.755786250503024E-03,
            inclination: Deg(3.394589648659516E+00).to_rad(),
            long_asc_node: Deg(7.667837463924961E+01).to_rad(),
            arg_periapsis: Deg(5.518596653686583E+01).to_rad(),
            mean_anomaly_0: Deg(5.011477187351476E+01).to_rad(),
        };
        let orbit_period = 2.246983300739057E+02;

        let trans = elements_to_ellipse_rotation_transform(&elements);
        let (pos, vel) = orbital_elements_to_xyz(&elements, orbit_period, J2000_JDN, &trans);

        // Values from HORIZONS (converted to Mm from km):
        let expected_pos: Point3<f64> = Point3::new(
            -1.074564940489116E+05,
            -4.885015029930510E+03,
            6.135634314000621E+03,
        );
        let expected_vel: Vector3<f64> = Vector3::new(
            1.193966825403014E+02,
            -3.036121503211865E+03,
            -4.838765653005392E+01,
        );

        assert!((pos - expected_pos).magnitude() < ACCEPTABLE_DELTA);
        assert!((vel - expected_vel).magnitude() < ACCEPTABLE_DELTA);

        // Callisto vs Jupiter system barycenter
        let elements = OrbitalElements {
            semi_major_axis: Au(1.258537659089199E-02).to_Mm(),
            eccentricity: 7.423685220918853E-03,
            inclination: Deg(2.016919351362485E+00).to_rad(),
            long_asc_node: Deg(3.379426810412668E+02).to_rad(),
            arg_periapsis: Deg(1.619056899622573E+01).to_rad(),
            mean_anomaly_0: Deg(8.505552940851534E+01).to_rad(),
        };
        let orbit_period = 1.669092070644565E+01;

        let trans = elements_to_ellipse_rotation_transform(&elements);
        let (pos, vel) = orbital_elements_to_xyz(&elements, orbit_period, J2000_JDN, &trans);

        // Values from HORIZONS (converted to Mm from km):
        let expected_pos: Point3<f64> = Point3::new(
            3.251207975783225E+02,
            1.852211480158530E+03,
            6.475384249692666E+01,
        );
        let expected_vel: Vector3<f64> = Vector3::new(
            -6.975067606364888E+02,
            1.279422738733358E+02,
            -5.048609078863432E+00,
        );

        assert!((pos - expected_pos).magnitude() < ACCEPTABLE_DELTA);
        assert!((vel - expected_vel).magnitude() < ACCEPTABLE_DELTA);
    }

    #[wasm_bindgen_test]
    pub fn test_bake_eccentric_anomaly_times() {
        // Mercury
        let elements = OrbitalElements {
            semi_major_axis: Au(0.3870982121840369).to_Mm(),
            eccentricity: 0.2056302929816634,
            inclination: Deg(7.00501414069919).to_rad(),
            long_asc_node: Deg(48.3305373398104).to_rad(),
            arg_periapsis: Deg(29.12428280936123).to_rad(),
            mean_anomaly_0: Deg(174.7958829506606).to_rad(),
        };
        let orbit_period = 87.96909804182887;
        let expected = vec![
            2451502.287121765,
            2451502.48123539,
            2451502.6753643197,
            2451502.869523856,
            2451503.063729288,
            2451503.257995894,
            2451503.4523389325,
            2451503.6467736387,
            2451503.8413152196,
            2451504.0359788504,
            2451504.230779668,
            2451504.4257327686,
            2451504.6208532015,
            2451504.816155965,
            2451505.0116560026,
            2451505.207368197,
            2451505.403307365,
            2451505.5994882574,
            2451505.7959255488,
            2451505.992633836,
            2451506.1896276344,
            2451506.3869213723,
            2451506.584529385,
            2451506.782465914,
            2451506.9807450995,
            2451507.1793809775,
            2451507.3783874763,
            2451507.5777784097,
            2451507.777567476,
            2451507.9777682517,
            2451508.1783941872,
            2451508.379458604,
            2451508.5809746897,
            2451508.7829554947,
            2451508.985413928,
            2451509.1883627526,
            2451509.391814582,
            2451509.595781877,
            2451509.8002769416,
            2451510.0053119184,
            2451510.210898786,
            2451510.4170493535,
            2451510.62377526,
            2451510.83108797,
            2451511.0389987663,
            2451511.2475187522,
            2451511.456658845,
            2451511.666429771,
            2451511.8768420676,
            2451512.087906074,
            2451512.299631932,
            2451512.512029583,
            2451512.725108762,
            2451512.9388789963,
            2451513.1533496045,
            2451513.3685296904,
            2451513.584428142,
            2451513.801053629,
            2451514.018414598,
            2451514.2365192734,
            2451514.4553756528,
            2451514.6749915043,
            2451514.895374364,
            2451515.1165315364,
            2451515.338470088,
            2451515.561196848,
            2451515.7847184064,
            2451516.00904111,
            2451516.234171062,
            2451516.460114119,
            2451516.686875892,
            2451516.91446174,
            2451517.1428767717,
            2451517.372125845,
            2451517.602213562,
            2451517.833144269,
            2451518.064922057,
            2451518.2975507583,
            2451518.531033946,
            2451518.765374932,
            2451519.000576769,
            2451519.2366422447,
            2451519.473573887,
            2451519.7113739564,
            2451519.950044452,
            2451520.1895871065,
            2451520.4300033865,
            2451520.6712944927,
            2451520.9134613597,
            2451521.156504655,
            2451521.4004247794,
            2451521.645221866,
            2451521.8908957825,
            2451522.137446127,
            2451522.384872232,
            2451522.6331731635,
            2451522.8823477207,
            2451523.132394436,
            2451523.3833115776,
            2451523.635097147,
            2451523.887748882,
            2451524.141264257,
            2451524.3956404817,
            2451524.650874506,
            2451524.906963016,
            2451525.163902439,
            2451525.421688943,
            2451525.680318438,
            2451525.9397865757,
            2451526.200088755,
            2451526.4612201187,
            2451526.7231755573,
            2451526.9859497114,
            2451527.2495369706,
            2451527.513931479,
            2451527.7791271317,
            2451528.0451175827,
            2451528.311896242,
            2451528.579456282,
            2451528.847790633,
            2451529.116891993,
            2451529.3867528248,
            2451529.6573653608,
            2451529.928721603,
            2451530.200813328,
            2451530.4736320875,
            2451530.747169213,
            2451531.021415816,
            2451531.2963627926,
            2451531.5720008253,
            2451531.848320386,
            2451532.1253117383,
            2451532.4029649436,
            2451532.681269859,
            2451532.960216144,
            2451533.239793263,
            2451533.519990488,
            2451533.8007969027,
            2451534.0822014045,
            2451534.3641927093,
            2451534.6467593526,
            2451534.929889697,
            2451535.2135719312,
            2451535.4977940787,
            2451535.7825439945,
            2451536.0678093764,
            2451536.3535777633,
            2451536.639836541,
            2451536.9265729478,
            2451537.213774073,
            2451537.5014268677,
            2451537.7895181435,
            2451538.0780345793,
            2451538.3669627244,
            2451538.656289002,
            2451538.9459997145,
            2451539.236081048,
            2451539.526519074,
            2451539.8172997567,
            2451540.108408955,
            2451540.399832429,
            2451540.6915558414,
            2451540.983564765,
            2451541.2758446853,
            2451541.5683810045,
            2451541.861159047,
            2451542.1541640647,
            2451542.4473812385,
            2451542.740795686,
            2451543.0343924644,
            2451543.3281565756,
            2451543.6220729686,
            2451543.91612655,
            2451544.21030218,
            2451544.5045846854,
            2451544.7989588585,
            2451545.0934094638,
            2451545.3879212425,
            2451545.682478918,
            2451545.9770671995,
            2451546.2716707857,
            2451546.5662743724,
            2451546.8608626537,
            2451547.155420329,
            2451547.449932108,
            2451547.7443827135,
            2451548.038756886,
            2451548.3330393913,
            2451548.6272150218,
            2451548.921268603,
            2451549.2151849964,
            2451549.508949107,
            2451549.8025458856,
            2451550.095960333,
            2451550.389177507,
            2451550.6821825244,
            2451550.9749605674,
            2451551.2674968867,
            2451551.5597768067,
            2451551.8517857306,
            2451552.143509143,
            2451552.434932617,
            2451552.7260418152,
            2451553.016822498,
            2451553.3072605236,
            2451553.597341857,
            2451553.8870525695,
            2451554.176378847,
            2451554.465306992,
            2451554.753823428,
            2451555.041914704,
            2451555.3295674985,
            2451555.616768624,
            2451555.9035050306,
            2451556.1897638086,
            2451556.4755321955,
            2451556.760797577,
            2451557.0455474933,
            2451557.32976964,
            2451557.613451875,
            2451557.896582219,
            2451558.179148862,
            2451558.461140167,
            2451558.7425446687,
            2451559.0233510835,
            2451559.3035483086,
            2451559.583125428,
            2451559.862071713,
            2451560.1403766284,
            2451560.418029833,
            2451560.695021186,
            2451560.9713407466,
            2451561.246978779,
            2451561.521925756,
            2451561.796172359,
            2451562.069709484,
            2451562.342528244,
            2451562.614619969,
            2451562.8859762107,
            2451563.1565887467,
            2451563.4264495783,
            2451563.6955509386,
            2451563.9638852896,
            2451564.2314453293,
            2451564.498223989,
            2451564.7642144402,
            2451565.029410093,
            2451565.2938046013,
            2451565.5573918605,
            2451565.8201660146,
            2451566.0821214532,
            2451566.3432528167,
            2451566.6035549957,
            2451566.8630231335,
            2451567.1216526283,
            2451567.3794391328,
            2451567.6363785556,
            2451567.8924670657,
            2451568.1477010897,
            2451568.402077315,
            2451568.6555926898,
            2451568.908244425,
            2451569.1600299943,
            2451569.4109471356,
            2451569.6609938513,
            2451569.9101684084,
            2451570.15846934,
            2451570.4058954446,
            2451570.6524457894,
            2451570.8981197053,
            2451571.1429167925,
            2451571.3868369167,
            2451571.629880212,
            2451571.872047079,
            2451572.113338185,
            2451572.353754465,
            2451572.5932971193,
            2451572.831967615,
            2451573.069767685,
            2451573.3066993267,
            2451573.542764803,
            2451573.7779666395,
            2451574.0123076257,
            2451574.245790813,
            2451574.4784195144,
            2451574.7101973025,
            2451574.94112801,
            2451575.1712157265,
            2451575.4004647997,
            2451575.628879832,
            2451575.8564656796,
            2451576.083227453,
            2451576.30917051,
            2451576.534300462,
            2451576.758623165,
            2451576.9821447236,
            2451577.204871484,
            2451577.4268100355,
            2451577.6479672072,
            2451577.8683500676,
            2451578.0879659187,
            2451578.306822298,
            2451578.524926974,
            2451578.742287943,
            2451578.9589134296,
            2451579.174811881,
            2451579.389991967,
            2451579.6044625756,
            2451579.8182328097,
            2451580.031311989,
            2451580.2437096396,
            2451580.455435498,
            2451580.6664995044,
            2451580.876911801,
            2451581.0866827266,
            2451581.295822819,
            2451581.504342805,
            2451581.7122536018,
            2451581.9195663114,
            2451582.1262922185,
            2451582.332442786,
            2451582.538029653,
            2451582.74306463,
            2451582.9475596943,
            2451583.15152699,
            2451583.3549788194,
            2451583.557927644,
            2451583.760386077,
            2451583.9623668822,
            2451584.163882968,
            2451584.3649473847,
            2451584.5655733203,
            2451584.7657740954,
            2451584.9655631618,
            2451585.1649540956,
            2451585.3639605944,
            2451585.5625964724,
            2451585.7608756577,
            2451585.9588121865,
            2451586.1564201997,
            2451586.353713937,
            2451586.5507077356,
            2451586.747416023,
            2451586.9438533145,
            2451587.1400342067,
            2451587.335973375,
            2451587.5316855693,
            2451587.7271856065,
            2451587.9224883704,
            2451588.117608803,
            2451588.312561904,
            2451588.5073627215,
            2451588.7020263523,
            2451588.8965679333,
            2451589.091002639,
            2451589.2853456773,
            2451589.4796122834,
            2451589.673817716,
            2451589.8679772518,
            2451590.062106182,
            2451590.2562198066,
        ];

        let res = bake_eccentric_anomaly_times(&elements, orbit_period, 1);
        assert_eq!(res, vec![expected[0], expected[expected.len() - 1]]);

        let res = bake_eccentric_anomaly_times(&elements, orbit_period, 4);
        assert_eq!(
            res,
            vec![
                expected[0],
                expected[90],
                expected[180],
                expected[270],
                expected[360]
            ]
        );

        let res = bake_eccentric_anomaly_times(&elements, orbit_period, 360);
        assert_eq!(res, expected);

        // Charon
        let elements = OrbitalElements {
            semi_major_axis: Au(0.0001165004117181425).to_Mm(),
            eccentricity: 0.002072743604774027,
            inclination: Deg(112.8984926230046).to_rad(),
            long_asc_node: Deg(227.4012844469266).to_rad(),
            arg_periapsis: Deg(144.5907325672654).to_rad(),
            mean_anomaly_0: Deg(176.6725371869484).to_rad(),
        };
        let orbit_period = 6.362099643049675;
        let expected = vec![
            2451541.877754762,
            2451541.895390632,
            2451541.913026514,
            2451541.9306624173,
            2451541.9482983546,
            2451541.9659343367,
            2451541.983570374,
            2451542.0012064786,
            2451542.018842661,
            2451542.036478932,
            2451542.0541153033,
            2451542.0717517855,
            2451542.08938839,
            2451542.107025127,
            2451542.1246620077,
            2451542.1422990435,
            2451542.1599362446,
            2451542.177573622,
            2451542.1952111865,
            2451542.2128489483,
            2451542.2304869187,
            2451542.248125107,
            2451542.2657635245,
            2451542.2834021817,
            2451542.301041089,
            2451542.3186802557,
            2451542.336319693,
            2451542.3539594105,
            2451542.371599418,
            2451542.389239726,
            2451542.406880344,
            2451542.4245212814,
            2451542.442162548,
            2451542.459804154,
            2451542.477446107,
            2451542.4950884185,
            2451542.5127310967,
            2451542.5303741503,
            2451542.5480175884,
            2451542.5656614206,
            2451542.5833056546,
            2451542.6009502998,
            2451542.6185953645,
            2451542.636240857,
            2451542.6538867857,
            2451542.671533158,
            2451542.689179983,
            2451542.706827267,
            2451542.724475019,
            2451542.7421232467,
            2451542.7597719566,
            2451542.777421156,
            2451542.795070852,
            2451542.812721052,
            2451542.8303717626,
            2451542.8480229904,
            2451542.8656747416,
            2451542.883327023,
            2451542.900979841,
            2451542.918633201,
            2451542.936287109,
            2451542.95394157,
            2451542.971596591,
            2451542.9892521757,
            2451543.0069083306,
            2451543.02456506,
            2451543.0422223685,
            2451543.059880262,
            2451543.077538743,
            2451543.095197817,
            2451543.112857488,
            2451543.1305177594,
            2451543.148178635,
            2451543.1658401196,
            2451543.1835022154,
            2451543.201164925,
            2451543.2188282525,
            2451543.2364922008,
            2451543.2541567716,
            2451543.271821968,
            2451543.2894877912,
            2451543.3071542447,
            2451543.324821329,
            2451543.3424890474,
            2451543.3601573994,
            2451543.377826388,
            2451543.3954960126,
            2451543.4131662752,
            2451543.4308371767,
            2451543.448508717,
            2451543.4661808964,
            2451543.4838537145,
            2451543.5015271725,
            2451543.5192012694,
            2451543.5368760047,
            2451543.554551378,
            2451543.5722273877,
            2451543.5899040336,
            2451543.6075813132,
            2451543.6252592267,
            2451543.6429377715,
            2451543.660616946,
            2451543.6782967476,
            2451543.6959771747,
            2451543.713658225,
            2451543.7313398956,
            2451543.7490221835,
            2451543.766705086,
            2451543.7843886,
            2451543.8020727215,
            2451543.819757448,
            2451543.8374427753,
            2451543.8551286994,
            2451543.872815216,
            2451543.890502321,
            2451543.9081900106,
            2451543.9258782794,
            2451543.9435671223,
            2451543.9612565353,
            2451543.9789465126,
            2451543.996637049,
            2451544.0143281394,
            2451544.0320197777,
            2451544.049711958,
            2451544.0674046744,
            2451544.085097921,
            2451544.102791691,
            2451544.1204859787,
            2451544.1381807765,
            2451544.1558760786,
            2451544.173571877,
            2451544.1912681656,
            2451544.2089649364,
            2451544.226662182,
            2451544.2443598956,
            2451544.2620580695,
            2451544.2797566946,
            2451544.297455764,
            2451544.3151552696,
            2451544.332855203,
            2451544.3505555554,
            2451544.3682563193,
            2451544.385957485,
            2451544.403659045,
            2451544.421360989,
            2451544.439063309,
            2451544.4567659963,
            2451544.474469041,
            2451544.492172433,
            2451544.5098761646,
            2451544.527580225,
            2451544.5452846056,
            2451544.562989295,
            2451544.5806942857,
            2451544.5983995665,
            2451544.616105127,
            2451544.633810958,
            2451544.651517049,
            2451544.6692233896,
            2451544.68692997,
            2451544.704636779,
            2451544.7223438076,
            2451544.7400510437,
            2451544.7577584772,
            2451544.775466098,
            2451544.793173895,
            2451544.8108818578,
            2451544.8285899744,
            2451544.8462982355,
            2451544.8640066287,
            2451544.881715145,
            2451544.8994237715,
            2451544.917132498,
            2451544.934841314,
            2451544.9525502073,
            2451544.970259168,
            2451544.987968184,
            2451545.0056772446,
            2451545.023386339,
            2451545.0410954556,
            2451545.0588045833,
            2451545.076513711,
            2451545.094222828,
            2451545.111931922,
            2451545.1296409825,
            2451545.147349999,
            2451545.1650589593,
            2451545.182767853,
            2451545.200476669,
            2451545.2181853955,
            2451545.235894022,
            2451545.253602538,
            2451545.271310932,
            2451545.2890191926,
            2451545.3067273097,
            2451545.3244352723,
            2451545.3421430686,
            2451545.3598506893,
            2451545.3775581233,
            2451545.3952653594,
            2451545.4129723874,
            2451545.430679197,
            2451545.448385777,
            2451545.466092118,
            2451545.4837982086,
            2451545.50150404,
            2451545.5192096005,
            2451545.5369148813,
            2451545.5546198715,
            2451545.572324562,
            2451545.5900289416,
            2451545.6077330024,
            2451545.625436734,
            2451545.6431401265,
            2451545.6608431707,
            2451545.6785458573,
            2451545.6962481774,
            2451545.7139501222,
            2451545.7316516815,
            2451545.7493528477,
            2451545.767053611,
            2451545.7847539643,
            2451545.8024538974,
            2451545.8201534026,
            2451545.8378524724,
            2451545.855551098,
            2451545.8732492714,
            2451545.8909469848,
            2451545.908644231,
            2451545.9263410014,
            2451545.94403729,
            2451545.9617330884,
            2451545.97942839,
            2451545.997123188,
            2451546.014817476,
            2451546.032511246,
            2451546.050204492,
            2451546.067897209,
            2451546.0855893893,
            2451546.1032810276,
            2451546.1209721174,
            2451546.138662654,
            2451546.1563526317,
            2451546.1740420447,
            2451546.191730888,
            2451546.2094191564,
            2451546.2271068455,
            2451546.2447939506,
            2451546.2624804676,
            2451546.2801663917,
            2451546.2978517185,
            2451546.315536445,
            2451546.333220567,
            2451546.350904081,
            2451546.368586983,
            2451546.386269272,
            2451546.4039509417,
            2451546.421631992,
            2451546.4393124194,
            2451546.456992221,
            2451546.4746713955,
            2451546.49234994,
            2451546.5100278533,
            2451546.527705134,
            2451546.5453817793,
            2451546.563057789,
            2451546.580733162,
            2451546.598407897,
            2451546.616081994,
            2451546.633755452,
            2451546.6514282706,
            2451546.6691004504,
            2451546.68677199,
            2451546.7044428913,
            2451546.7221131544,
            2451546.7397827795,
            2451546.7574517676,
            2451546.7751201196,
            2451546.7927878373,
            2451546.8104549227,
            2451546.8281213758,
            2451546.845787199,
            2451546.8634523954,
            2451546.8811169663,
            2451546.898780914,
            2451546.9164442415,
            2451546.934106952,
            2451546.9517690474,
            2451546.9694305314,
            2451546.9870914076,
            2451547.004751679,
            2451547.02241135,
            2451547.0400704243,
            2451547.0577289057,
            2451547.075386798,
            2451547.093044107,
            2451547.1107008364,
            2451547.128356991,
            2451547.1460125763,
            2451547.1636675964,
            2451547.181322058,
            2451547.198975966,
            2451547.216629326,
            2451547.2342821434,
            2451547.251934425,
            2451547.2695861766,
            2451547.2872374044,
            2451547.304888115,
            2451547.322538315,
            2451547.3401880115,
            2451547.3578372104,
            2451547.3754859203,
            2451547.3931341474,
            2451547.4107818995,
            2451547.428429184,
            2451547.4460760085,
            2451547.4637223813,
            2451547.48136831,
            2451547.499013802,
            2451547.516658867,
            2451547.534303512,
            2451547.5519477464,
            2451547.569591578,
            2451547.587235017,
            2451547.6048780708,
            2451547.6225207485,
            2451547.64016306,
            2451547.6578050135,
            2451547.675446619,
            2451547.6930878856,
            2451547.7107288227,
            2451547.7283694404,
            2451547.7460097484,
            2451547.763649756,
            2451547.7812894736,
            2451547.798928911,
            2451547.816568078,
            2451547.8342069853,
            2451547.8518456426,
            2451547.86948406,
            2451547.8871222488,
            2451547.904760218,
            2451547.92239798,
            2451547.940035545,
            2451547.957672922,
            2451547.975310123,
            2451547.992947159,
            2451548.0105840396,
            2451548.028220777,
            2451548.045857381,
            2451548.063493863,
            2451548.0811302345,
            2451548.098766506,
            2451548.1164026884,
            2451548.134038793,
            2451548.1516748304,
            2451548.169310812,
            2451548.1869467497,
            2451548.204582653,
            2451548.2222185344,
            2451548.239854405,
        ];

        let res = bake_eccentric_anomaly_times(&elements, orbit_period, 1);
        assert_eq!(res, vec![expected[0], expected[expected.len() - 1]]);

        let res = bake_eccentric_anomaly_times(&elements, orbit_period, 4);
        assert_eq!(
            res,
            vec![
                expected[0],
                expected[90],
                expected[180],
                expected[270],
                expected[360]
            ]
        );

        let res = bake_eccentric_anomaly_times(&elements, orbit_period, 360);
        assert_eq!(res, expected);
    }
}
