use crate::utils::{
    transform::Transform,
    units::{Au, Deg, Jdn, Mm, Rad, J2000_JDN},
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

/// Mm3 / (kg s2)
pub const GRAVITATION_CONSTANT: f64 = 6.743E-29;
const NEWTON_RAPHSON_MAX_ITER: u32 = 30;
const NEWTON_RAPHSON_DELTA: f64 = 0.00000001;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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
impl Default for BodyType {
    fn default() -> Self {
        BodyType::Other
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OrbitalElements {
    pub semi_major_axis: Mm,
    pub eccentricity: f64,
    pub inclination: Rad, // Rad to prevent a rogue .sin() from spewing nonsense
    pub long_asc_node: Rad,
    pub arg_periapsis: Rad,
    pub mean_anomaly_0: Rad,
    pub sidereal_orbit_period_days: f64,
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

#[allow(dead_code)]
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

    let sidereal_orbit_period_days =
        float_from_match(file_str, &SIDERAL_ORBIT_PERIOD_RE).unwrap_or(0.0);

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
        mass: 0.0, // TODO
        body_type,
        orbital_elements: OrbitalElements {
            semi_major_axis,
            eccentricity,
            inclination,
            long_asc_node,
            arg_periapsis,
            mean_anomaly_0,
            sidereal_orbit_period_days,
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
    t: Jdn,
    ellipse_rotation_transform: &Transform<f64>,
) -> (Point3<f64>, Vector3<f64>) {
    let mean_motion = 2.0 * PI / elements.sidereal_orbit_period_days; // Rads/day
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
pub fn bake_eccentric_anomaly_times(elements: &OrbitalElements, num_angles: u32) -> Vec<Jdn> {
    let mut result: Vec<Jdn> = Vec::new();
    result.reserve((num_angles + 1) as usize);

    let mean_motion = 2.0 * PI / elements.sidereal_orbit_period_days; // Rads/day
    let time_of_periapsis: Jdn = time_of_prev_periapsis(elements.mean_anomaly_0, mean_motion);

    let incr = 360.0 / num_angles as f64;
    for i in 0..=num_angles {
        let eccentric_anomaly = (i as f64 * incr).to_radians();

        let mean_anomaly = eccentric_anomaly - elements.eccentricity * eccentric_anomaly.sin(); // Rad
        let t = (mean_anomaly + mean_motion * time_of_periapsis.0) / mean_motion; // day

        result.push(Jdn(t));
    }

    log::info!("Baked {} samples for an orbit", result.len());
    return result;
}

pub fn get_eccentric_anomaly(mut date: Jdn, orbital_period: f64, baked_times: &Vec<Jdn>) -> Rad {
    while date > baked_times[baked_times.len() - 1] {
        date.0 -= orbital_period;
    }

    // We may have 5 angles: [0, 90, 180, 270, 360], so we want an incr of 90 deg
    // so that if our binary search gives index 3 we just return 270
    let angle_incr = 2.0 * PI / ((baked_times.len() - 1) as f64);

    // TODO: We can probably speed up this search if we keep track of the last index we used, as it's likely
    // the same one
    match baked_times.binary_search_by(|p| p.partial_cmp(&date).unwrap()) {
        Ok(exact_index) => {
            return Rad(angle_incr * exact_index as f64);
        }
        #[allow(non_snake_case)]
        Err(prev_index) => {
            let next_index = (prev_index + 1) % baked_times.len();

            let prev_date = baked_times[prev_index];
            let next_date = baked_times[next_index];
            let prev_E = angle_incr * prev_index as f64;
            let next_E = angle_incr * next_index as f64;

            return Rad(
                prev_E + (next_E - prev_E) * ((date.0 - prev_date.0) / (next_date.0 - prev_date.0))
            );
        }
    }
}

/// Returns the JDN time of the next periapsis crossing
#[allow(dead_code)]
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
    use crate::utils::units::J2000_JDN;

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
            sidereal_orbit_period_days: 2.246983300739057E+02,
        };

        let trans = elements_to_ellipse_rotation_transform(&elements);
        let (pos, vel) = orbital_elements_to_xyz(&elements, J2000_JDN, &trans);

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
            sidereal_orbit_period_days: 1.669092070644565E+01,
        };

        let trans = elements_to_ellipse_rotation_transform(&elements);
        let (pos, vel) = orbital_elements_to_xyz(&elements, J2000_JDN, &trans);

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
            sidereal_orbit_period_days: 87.96909804182887,
        };
        let expected = vec![
            Jdn(2451502.287121765),
            Jdn(2451502.48123539),
            Jdn(2451502.6753643197),
            Jdn(2451502.869523856),
            Jdn(2451503.063729288),
            Jdn(2451503.257995894),
            Jdn(2451503.4523389325),
            Jdn(2451503.6467736387),
            Jdn(2451503.8413152196),
            Jdn(2451504.0359788504),
            Jdn(2451504.230779668),
            Jdn(2451504.4257327686),
            Jdn(2451504.6208532015),
            Jdn(2451504.816155965),
            Jdn(2451505.0116560026),
            Jdn(2451505.207368197),
            Jdn(2451505.403307365),
            Jdn(2451505.5994882574),
            Jdn(2451505.7959255488),
            Jdn(2451505.992633836),
            Jdn(2451506.1896276344),
            Jdn(2451506.3869213723),
            Jdn(2451506.584529385),
            Jdn(2451506.782465914),
            Jdn(2451506.9807450995),
            Jdn(2451507.1793809775),
            Jdn(2451507.3783874763),
            Jdn(2451507.5777784097),
            Jdn(2451507.777567476),
            Jdn(2451507.9777682517),
            Jdn(2451508.1783941872),
            Jdn(2451508.379458604),
            Jdn(2451508.5809746897),
            Jdn(2451508.7829554947),
            Jdn(2451508.985413928),
            Jdn(2451509.1883627526),
            Jdn(2451509.391814582),
            Jdn(2451509.595781877),
            Jdn(2451509.8002769416),
            Jdn(2451510.0053119184),
            Jdn(2451510.210898786),
            Jdn(2451510.4170493535),
            Jdn(2451510.62377526),
            Jdn(2451510.83108797),
            Jdn(2451511.0389987663),
            Jdn(2451511.2475187522),
            Jdn(2451511.456658845),
            Jdn(2451511.666429771),
            Jdn(2451511.8768420676),
            Jdn(2451512.087906074),
            Jdn(2451512.299631932),
            Jdn(2451512.512029583),
            Jdn(2451512.725108762),
            Jdn(2451512.9388789963),
            Jdn(2451513.1533496045),
            Jdn(2451513.3685296904),
            Jdn(2451513.584428142),
            Jdn(2451513.801053629),
            Jdn(2451514.018414598),
            Jdn(2451514.2365192734),
            Jdn(2451514.4553756528),
            Jdn(2451514.6749915043),
            Jdn(2451514.895374364),
            Jdn(2451515.1165315364),
            Jdn(2451515.338470088),
            Jdn(2451515.561196848),
            Jdn(2451515.7847184064),
            Jdn(2451516.00904111),
            Jdn(2451516.234171062),
            Jdn(2451516.460114119),
            Jdn(2451516.686875892),
            Jdn(2451516.91446174),
            Jdn(2451517.1428767717),
            Jdn(2451517.372125845),
            Jdn(2451517.602213562),
            Jdn(2451517.833144269),
            Jdn(2451518.064922057),
            Jdn(2451518.2975507583),
            Jdn(2451518.531033946),
            Jdn(2451518.765374932),
            Jdn(2451519.000576769),
            Jdn(2451519.2366422447),
            Jdn(2451519.473573887),
            Jdn(2451519.7113739564),
            Jdn(2451519.950044452),
            Jdn(2451520.1895871065),
            Jdn(2451520.4300033865),
            Jdn(2451520.6712944927),
            Jdn(2451520.9134613597),
            Jdn(2451521.156504655),
            Jdn(2451521.4004247794),
            Jdn(2451521.645221866),
            Jdn(2451521.8908957825),
            Jdn(2451522.137446127),
            Jdn(2451522.384872232),
            Jdn(2451522.6331731635),
            Jdn(2451522.8823477207),
            Jdn(2451523.132394436),
            Jdn(2451523.3833115776),
            Jdn(2451523.635097147),
            Jdn(2451523.887748882),
            Jdn(2451524.141264257),
            Jdn(2451524.3956404817),
            Jdn(2451524.650874506),
            Jdn(2451524.906963016),
            Jdn(2451525.163902439),
            Jdn(2451525.421688943),
            Jdn(2451525.680318438),
            Jdn(2451525.9397865757),
            Jdn(2451526.200088755),
            Jdn(2451526.4612201187),
            Jdn(2451526.7231755573),
            Jdn(2451526.9859497114),
            Jdn(2451527.2495369706),
            Jdn(2451527.513931479),
            Jdn(2451527.7791271317),
            Jdn(2451528.0451175827),
            Jdn(2451528.311896242),
            Jdn(2451528.579456282),
            Jdn(2451528.847790633),
            Jdn(2451529.116891993),
            Jdn(2451529.3867528248),
            Jdn(2451529.6573653608),
            Jdn(2451529.928721603),
            Jdn(2451530.200813328),
            Jdn(2451530.4736320875),
            Jdn(2451530.747169213),
            Jdn(2451531.021415816),
            Jdn(2451531.2963627926),
            Jdn(2451531.5720008253),
            Jdn(2451531.848320386),
            Jdn(2451532.1253117383),
            Jdn(2451532.4029649436),
            Jdn(2451532.681269859),
            Jdn(2451532.960216144),
            Jdn(2451533.239793263),
            Jdn(2451533.519990488),
            Jdn(2451533.8007969027),
            Jdn(2451534.0822014045),
            Jdn(2451534.3641927093),
            Jdn(2451534.6467593526),
            Jdn(2451534.929889697),
            Jdn(2451535.2135719312),
            Jdn(2451535.4977940787),
            Jdn(2451535.7825439945),
            Jdn(2451536.0678093764),
            Jdn(2451536.3535777633),
            Jdn(2451536.639836541),
            Jdn(2451536.9265729478),
            Jdn(2451537.213774073),
            Jdn(2451537.5014268677),
            Jdn(2451537.7895181435),
            Jdn(2451538.0780345793),
            Jdn(2451538.3669627244),
            Jdn(2451538.656289002),
            Jdn(2451538.9459997145),
            Jdn(2451539.236081048),
            Jdn(2451539.526519074),
            Jdn(2451539.8172997567),
            Jdn(2451540.108408955),
            Jdn(2451540.399832429),
            Jdn(2451540.6915558414),
            Jdn(2451540.983564765),
            Jdn(2451541.2758446853),
            Jdn(2451541.5683810045),
            Jdn(2451541.861159047),
            Jdn(2451542.1541640647),
            Jdn(2451542.4473812385),
            Jdn(2451542.740795686),
            Jdn(2451543.0343924644),
            Jdn(2451543.3281565756),
            Jdn(2451543.6220729686),
            Jdn(2451543.91612655),
            Jdn(2451544.21030218),
            Jdn(2451544.5045846854),
            Jdn(2451544.7989588585),
            Jdn(2451545.0934094638),
            Jdn(2451545.3879212425),
            Jdn(2451545.682478918),
            Jdn(2451545.9770671995),
            Jdn(2451546.2716707857),
            Jdn(2451546.5662743724),
            Jdn(2451546.8608626537),
            Jdn(2451547.155420329),
            Jdn(2451547.449932108),
            Jdn(2451547.7443827135),
            Jdn(2451548.038756886),
            Jdn(2451548.3330393913),
            Jdn(2451548.6272150218),
            Jdn(2451548.921268603),
            Jdn(2451549.2151849964),
            Jdn(2451549.508949107),
            Jdn(2451549.8025458856),
            Jdn(2451550.095960333),
            Jdn(2451550.389177507),
            Jdn(2451550.6821825244),
            Jdn(2451550.9749605674),
            Jdn(2451551.2674968867),
            Jdn(2451551.5597768067),
            Jdn(2451551.8517857306),
            Jdn(2451552.143509143),
            Jdn(2451552.434932617),
            Jdn(2451552.7260418152),
            Jdn(2451553.016822498),
            Jdn(2451553.3072605236),
            Jdn(2451553.597341857),
            Jdn(2451553.8870525695),
            Jdn(2451554.176378847),
            Jdn(2451554.465306992),
            Jdn(2451554.753823428),
            Jdn(2451555.041914704),
            Jdn(2451555.3295674985),
            Jdn(2451555.616768624),
            Jdn(2451555.9035050306),
            Jdn(2451556.1897638086),
            Jdn(2451556.4755321955),
            Jdn(2451556.760797577),
            Jdn(2451557.0455474933),
            Jdn(2451557.32976964),
            Jdn(2451557.613451875),
            Jdn(2451557.896582219),
            Jdn(2451558.179148862),
            Jdn(2451558.461140167),
            Jdn(2451558.7425446687),
            Jdn(2451559.0233510835),
            Jdn(2451559.3035483086),
            Jdn(2451559.583125428),
            Jdn(2451559.862071713),
            Jdn(2451560.1403766284),
            Jdn(2451560.418029833),
            Jdn(2451560.695021186),
            Jdn(2451560.9713407466),
            Jdn(2451561.246978779),
            Jdn(2451561.521925756),
            Jdn(2451561.796172359),
            Jdn(2451562.069709484),
            Jdn(2451562.342528244),
            Jdn(2451562.614619969),
            Jdn(2451562.8859762107),
            Jdn(2451563.1565887467),
            Jdn(2451563.4264495783),
            Jdn(2451563.6955509386),
            Jdn(2451563.9638852896),
            Jdn(2451564.2314453293),
            Jdn(2451564.498223989),
            Jdn(2451564.7642144402),
            Jdn(2451565.029410093),
            Jdn(2451565.2938046013),
            Jdn(2451565.5573918605),
            Jdn(2451565.8201660146),
            Jdn(2451566.0821214532),
            Jdn(2451566.3432528167),
            Jdn(2451566.6035549957),
            Jdn(2451566.8630231335),
            Jdn(2451567.1216526283),
            Jdn(2451567.3794391328),
            Jdn(2451567.6363785556),
            Jdn(2451567.8924670657),
            Jdn(2451568.1477010897),
            Jdn(2451568.402077315),
            Jdn(2451568.6555926898),
            Jdn(2451568.908244425),
            Jdn(2451569.1600299943),
            Jdn(2451569.4109471356),
            Jdn(2451569.6609938513),
            Jdn(2451569.9101684084),
            Jdn(2451570.15846934),
            Jdn(2451570.4058954446),
            Jdn(2451570.6524457894),
            Jdn(2451570.8981197053),
            Jdn(2451571.1429167925),
            Jdn(2451571.3868369167),
            Jdn(2451571.629880212),
            Jdn(2451571.872047079),
            Jdn(2451572.113338185),
            Jdn(2451572.353754465),
            Jdn(2451572.5932971193),
            Jdn(2451572.831967615),
            Jdn(2451573.069767685),
            Jdn(2451573.3066993267),
            Jdn(2451573.542764803),
            Jdn(2451573.7779666395),
            Jdn(2451574.0123076257),
            Jdn(2451574.245790813),
            Jdn(2451574.4784195144),
            Jdn(2451574.7101973025),
            Jdn(2451574.94112801),
            Jdn(2451575.1712157265),
            Jdn(2451575.4004647997),
            Jdn(2451575.628879832),
            Jdn(2451575.8564656796),
            Jdn(2451576.083227453),
            Jdn(2451576.30917051),
            Jdn(2451576.534300462),
            Jdn(2451576.758623165),
            Jdn(2451576.9821447236),
            Jdn(2451577.204871484),
            Jdn(2451577.4268100355),
            Jdn(2451577.6479672072),
            Jdn(2451577.8683500676),
            Jdn(2451578.0879659187),
            Jdn(2451578.306822298),
            Jdn(2451578.524926974),
            Jdn(2451578.742287943),
            Jdn(2451578.9589134296),
            Jdn(2451579.174811881),
            Jdn(2451579.389991967),
            Jdn(2451579.6044625756),
            Jdn(2451579.8182328097),
            Jdn(2451580.031311989),
            Jdn(2451580.2437096396),
            Jdn(2451580.455435498),
            Jdn(2451580.6664995044),
            Jdn(2451580.876911801),
            Jdn(2451581.0866827266),
            Jdn(2451581.295822819),
            Jdn(2451581.504342805),
            Jdn(2451581.7122536018),
            Jdn(2451581.9195663114),
            Jdn(2451582.1262922185),
            Jdn(2451582.332442786),
            Jdn(2451582.538029653),
            Jdn(2451582.74306463),
            Jdn(2451582.9475596943),
            Jdn(2451583.15152699),
            Jdn(2451583.3549788194),
            Jdn(2451583.557927644),
            Jdn(2451583.760386077),
            Jdn(2451583.9623668822),
            Jdn(2451584.163882968),
            Jdn(2451584.3649473847),
            Jdn(2451584.5655733203),
            Jdn(2451584.7657740954),
            Jdn(2451584.9655631618),
            Jdn(2451585.1649540956),
            Jdn(2451585.3639605944),
            Jdn(2451585.5625964724),
            Jdn(2451585.7608756577),
            Jdn(2451585.9588121865),
            Jdn(2451586.1564201997),
            Jdn(2451586.353713937),
            Jdn(2451586.5507077356),
            Jdn(2451586.747416023),
            Jdn(2451586.9438533145),
            Jdn(2451587.1400342067),
            Jdn(2451587.335973375),
            Jdn(2451587.5316855693),
            Jdn(2451587.7271856065),
            Jdn(2451587.9224883704),
            Jdn(2451588.117608803),
            Jdn(2451588.312561904),
            Jdn(2451588.5073627215),
            Jdn(2451588.7020263523),
            Jdn(2451588.8965679333),
            Jdn(2451589.091002639),
            Jdn(2451589.2853456773),
            Jdn(2451589.4796122834),
            Jdn(2451589.673817716),
            Jdn(2451589.8679772518),
            Jdn(2451590.062106182),
            Jdn(2451590.2562198066),
        ];

        let res = bake_eccentric_anomaly_times(&elements, 1);
        assert_eq!(res, vec![expected[0], expected[expected.len() - 1]]);

        let res = bake_eccentric_anomaly_times(&elements, 4);
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

        let res = bake_eccentric_anomaly_times(&elements, 360);
        assert_eq!(res, expected);

        // Charon
        let elements = OrbitalElements {
            semi_major_axis: Au(0.0001165004117181425).to_Mm(),
            eccentricity: 0.002072743604774027,
            inclination: Deg(112.8984926230046).to_rad(),
            long_asc_node: Deg(227.4012844469266).to_rad(),
            arg_periapsis: Deg(144.5907325672654).to_rad(),
            mean_anomaly_0: Deg(176.6725371869484).to_rad(),
            sidereal_orbit_period_days: 6.362099643049675,
        };
        let expected = vec![
            Jdn(2451541.877754762),
            Jdn(2451541.895390632),
            Jdn(2451541.913026514),
            Jdn(2451541.9306624173),
            Jdn(2451541.9482983546),
            Jdn(2451541.9659343367),
            Jdn(2451541.983570374),
            Jdn(2451542.0012064786),
            Jdn(2451542.018842661),
            Jdn(2451542.036478932),
            Jdn(2451542.0541153033),
            Jdn(2451542.0717517855),
            Jdn(2451542.08938839),
            Jdn(2451542.107025127),
            Jdn(2451542.1246620077),
            Jdn(2451542.1422990435),
            Jdn(2451542.1599362446),
            Jdn(2451542.177573622),
            Jdn(2451542.1952111865),
            Jdn(2451542.2128489483),
            Jdn(2451542.2304869187),
            Jdn(2451542.248125107),
            Jdn(2451542.2657635245),
            Jdn(2451542.2834021817),
            Jdn(2451542.301041089),
            Jdn(2451542.3186802557),
            Jdn(2451542.336319693),
            Jdn(2451542.3539594105),
            Jdn(2451542.371599418),
            Jdn(2451542.389239726),
            Jdn(2451542.406880344),
            Jdn(2451542.4245212814),
            Jdn(2451542.442162548),
            Jdn(2451542.459804154),
            Jdn(2451542.477446107),
            Jdn(2451542.4950884185),
            Jdn(2451542.5127310967),
            Jdn(2451542.5303741503),
            Jdn(2451542.5480175884),
            Jdn(2451542.5656614206),
            Jdn(2451542.5833056546),
            Jdn(2451542.6009502998),
            Jdn(2451542.6185953645),
            Jdn(2451542.636240857),
            Jdn(2451542.6538867857),
            Jdn(2451542.671533158),
            Jdn(2451542.689179983),
            Jdn(2451542.706827267),
            Jdn(2451542.724475019),
            Jdn(2451542.7421232467),
            Jdn(2451542.7597719566),
            Jdn(2451542.777421156),
            Jdn(2451542.795070852),
            Jdn(2451542.812721052),
            Jdn(2451542.8303717626),
            Jdn(2451542.8480229904),
            Jdn(2451542.8656747416),
            Jdn(2451542.883327023),
            Jdn(2451542.900979841),
            Jdn(2451542.918633201),
            Jdn(2451542.936287109),
            Jdn(2451542.95394157),
            Jdn(2451542.971596591),
            Jdn(2451542.9892521757),
            Jdn(2451543.0069083306),
            Jdn(2451543.02456506),
            Jdn(2451543.0422223685),
            Jdn(2451543.059880262),
            Jdn(2451543.077538743),
            Jdn(2451543.095197817),
            Jdn(2451543.112857488),
            Jdn(2451543.1305177594),
            Jdn(2451543.148178635),
            Jdn(2451543.1658401196),
            Jdn(2451543.1835022154),
            Jdn(2451543.201164925),
            Jdn(2451543.2188282525),
            Jdn(2451543.2364922008),
            Jdn(2451543.2541567716),
            Jdn(2451543.271821968),
            Jdn(2451543.2894877912),
            Jdn(2451543.3071542447),
            Jdn(2451543.324821329),
            Jdn(2451543.3424890474),
            Jdn(2451543.3601573994),
            Jdn(2451543.377826388),
            Jdn(2451543.3954960126),
            Jdn(2451543.4131662752),
            Jdn(2451543.4308371767),
            Jdn(2451543.448508717),
            Jdn(2451543.4661808964),
            Jdn(2451543.4838537145),
            Jdn(2451543.5015271725),
            Jdn(2451543.5192012694),
            Jdn(2451543.5368760047),
            Jdn(2451543.554551378),
            Jdn(2451543.5722273877),
            Jdn(2451543.5899040336),
            Jdn(2451543.6075813132),
            Jdn(2451543.6252592267),
            Jdn(2451543.6429377715),
            Jdn(2451543.660616946),
            Jdn(2451543.6782967476),
            Jdn(2451543.6959771747),
            Jdn(2451543.713658225),
            Jdn(2451543.7313398956),
            Jdn(2451543.7490221835),
            Jdn(2451543.766705086),
            Jdn(2451543.7843886),
            Jdn(2451543.8020727215),
            Jdn(2451543.819757448),
            Jdn(2451543.8374427753),
            Jdn(2451543.8551286994),
            Jdn(2451543.872815216),
            Jdn(2451543.890502321),
            Jdn(2451543.9081900106),
            Jdn(2451543.9258782794),
            Jdn(2451543.9435671223),
            Jdn(2451543.9612565353),
            Jdn(2451543.9789465126),
            Jdn(2451543.996637049),
            Jdn(2451544.0143281394),
            Jdn(2451544.0320197777),
            Jdn(2451544.049711958),
            Jdn(2451544.0674046744),
            Jdn(2451544.085097921),
            Jdn(2451544.102791691),
            Jdn(2451544.1204859787),
            Jdn(2451544.1381807765),
            Jdn(2451544.1558760786),
            Jdn(2451544.173571877),
            Jdn(2451544.1912681656),
            Jdn(2451544.2089649364),
            Jdn(2451544.226662182),
            Jdn(2451544.2443598956),
            Jdn(2451544.2620580695),
            Jdn(2451544.2797566946),
            Jdn(2451544.297455764),
            Jdn(2451544.3151552696),
            Jdn(2451544.332855203),
            Jdn(2451544.3505555554),
            Jdn(2451544.3682563193),
            Jdn(2451544.385957485),
            Jdn(2451544.403659045),
            Jdn(2451544.421360989),
            Jdn(2451544.439063309),
            Jdn(2451544.4567659963),
            Jdn(2451544.474469041),
            Jdn(2451544.492172433),
            Jdn(2451544.5098761646),
            Jdn(2451544.527580225),
            Jdn(2451544.5452846056),
            Jdn(2451544.562989295),
            Jdn(2451544.5806942857),
            Jdn(2451544.5983995665),
            Jdn(2451544.616105127),
            Jdn(2451544.633810958),
            Jdn(2451544.651517049),
            Jdn(2451544.6692233896),
            Jdn(2451544.68692997),
            Jdn(2451544.704636779),
            Jdn(2451544.7223438076),
            Jdn(2451544.7400510437),
            Jdn(2451544.7577584772),
            Jdn(2451544.775466098),
            Jdn(2451544.793173895),
            Jdn(2451544.8108818578),
            Jdn(2451544.8285899744),
            Jdn(2451544.8462982355),
            Jdn(2451544.8640066287),
            Jdn(2451544.881715145),
            Jdn(2451544.8994237715),
            Jdn(2451544.917132498),
            Jdn(2451544.934841314),
            Jdn(2451544.9525502073),
            Jdn(2451544.970259168),
            Jdn(2451544.987968184),
            Jdn(2451545.0056772446),
            Jdn(2451545.023386339),
            Jdn(2451545.0410954556),
            Jdn(2451545.0588045833),
            Jdn(2451545.076513711),
            Jdn(2451545.094222828),
            Jdn(2451545.111931922),
            Jdn(2451545.1296409825),
            Jdn(2451545.147349999),
            Jdn(2451545.1650589593),
            Jdn(2451545.182767853),
            Jdn(2451545.200476669),
            Jdn(2451545.2181853955),
            Jdn(2451545.235894022),
            Jdn(2451545.253602538),
            Jdn(2451545.271310932),
            Jdn(2451545.2890191926),
            Jdn(2451545.3067273097),
            Jdn(2451545.3244352723),
            Jdn(2451545.3421430686),
            Jdn(2451545.3598506893),
            Jdn(2451545.3775581233),
            Jdn(2451545.3952653594),
            Jdn(2451545.4129723874),
            Jdn(2451545.430679197),
            Jdn(2451545.448385777),
            Jdn(2451545.466092118),
            Jdn(2451545.4837982086),
            Jdn(2451545.50150404),
            Jdn(2451545.5192096005),
            Jdn(2451545.5369148813),
            Jdn(2451545.5546198715),
            Jdn(2451545.572324562),
            Jdn(2451545.5900289416),
            Jdn(2451545.6077330024),
            Jdn(2451545.625436734),
            Jdn(2451545.6431401265),
            Jdn(2451545.6608431707),
            Jdn(2451545.6785458573),
            Jdn(2451545.6962481774),
            Jdn(2451545.7139501222),
            Jdn(2451545.7316516815),
            Jdn(2451545.7493528477),
            Jdn(2451545.767053611),
            Jdn(2451545.7847539643),
            Jdn(2451545.8024538974),
            Jdn(2451545.8201534026),
            Jdn(2451545.8378524724),
            Jdn(2451545.855551098),
            Jdn(2451545.8732492714),
            Jdn(2451545.8909469848),
            Jdn(2451545.908644231),
            Jdn(2451545.9263410014),
            Jdn(2451545.94403729),
            Jdn(2451545.9617330884),
            Jdn(2451545.97942839),
            Jdn(2451545.997123188),
            Jdn(2451546.014817476),
            Jdn(2451546.032511246),
            Jdn(2451546.050204492),
            Jdn(2451546.067897209),
            Jdn(2451546.0855893893),
            Jdn(2451546.1032810276),
            Jdn(2451546.1209721174),
            Jdn(2451546.138662654),
            Jdn(2451546.1563526317),
            Jdn(2451546.1740420447),
            Jdn(2451546.191730888),
            Jdn(2451546.2094191564),
            Jdn(2451546.2271068455),
            Jdn(2451546.2447939506),
            Jdn(2451546.2624804676),
            Jdn(2451546.2801663917),
            Jdn(2451546.2978517185),
            Jdn(2451546.315536445),
            Jdn(2451546.333220567),
            Jdn(2451546.350904081),
            Jdn(2451546.368586983),
            Jdn(2451546.386269272),
            Jdn(2451546.4039509417),
            Jdn(2451546.421631992),
            Jdn(2451546.4393124194),
            Jdn(2451546.456992221),
            Jdn(2451546.4746713955),
            Jdn(2451546.49234994),
            Jdn(2451546.5100278533),
            Jdn(2451546.527705134),
            Jdn(2451546.5453817793),
            Jdn(2451546.563057789),
            Jdn(2451546.580733162),
            Jdn(2451546.598407897),
            Jdn(2451546.616081994),
            Jdn(2451546.633755452),
            Jdn(2451546.6514282706),
            Jdn(2451546.6691004504),
            Jdn(2451546.68677199),
            Jdn(2451546.7044428913),
            Jdn(2451546.7221131544),
            Jdn(2451546.7397827795),
            Jdn(2451546.7574517676),
            Jdn(2451546.7751201196),
            Jdn(2451546.7927878373),
            Jdn(2451546.8104549227),
            Jdn(2451546.8281213758),
            Jdn(2451546.845787199),
            Jdn(2451546.8634523954),
            Jdn(2451546.8811169663),
            Jdn(2451546.898780914),
            Jdn(2451546.9164442415),
            Jdn(2451546.934106952),
            Jdn(2451546.9517690474),
            Jdn(2451546.9694305314),
            Jdn(2451546.9870914076),
            Jdn(2451547.004751679),
            Jdn(2451547.02241135),
            Jdn(2451547.0400704243),
            Jdn(2451547.0577289057),
            Jdn(2451547.075386798),
            Jdn(2451547.093044107),
            Jdn(2451547.1107008364),
            Jdn(2451547.128356991),
            Jdn(2451547.1460125763),
            Jdn(2451547.1636675964),
            Jdn(2451547.181322058),
            Jdn(2451547.198975966),
            Jdn(2451547.216629326),
            Jdn(2451547.2342821434),
            Jdn(2451547.251934425),
            Jdn(2451547.2695861766),
            Jdn(2451547.2872374044),
            Jdn(2451547.304888115),
            Jdn(2451547.322538315),
            Jdn(2451547.3401880115),
            Jdn(2451547.3578372104),
            Jdn(2451547.3754859203),
            Jdn(2451547.3931341474),
            Jdn(2451547.4107818995),
            Jdn(2451547.428429184),
            Jdn(2451547.4460760085),
            Jdn(2451547.4637223813),
            Jdn(2451547.48136831),
            Jdn(2451547.499013802),
            Jdn(2451547.516658867),
            Jdn(2451547.534303512),
            Jdn(2451547.5519477464),
            Jdn(2451547.569591578),
            Jdn(2451547.587235017),
            Jdn(2451547.6048780708),
            Jdn(2451547.6225207485),
            Jdn(2451547.64016306),
            Jdn(2451547.6578050135),
            Jdn(2451547.675446619),
            Jdn(2451547.6930878856),
            Jdn(2451547.7107288227),
            Jdn(2451547.7283694404),
            Jdn(2451547.7460097484),
            Jdn(2451547.763649756),
            Jdn(2451547.7812894736),
            Jdn(2451547.798928911),
            Jdn(2451547.816568078),
            Jdn(2451547.8342069853),
            Jdn(2451547.8518456426),
            Jdn(2451547.86948406),
            Jdn(2451547.8871222488),
            Jdn(2451547.904760218),
            Jdn(2451547.92239798),
            Jdn(2451547.940035545),
            Jdn(2451547.957672922),
            Jdn(2451547.975310123),
            Jdn(2451547.992947159),
            Jdn(2451548.0105840396),
            Jdn(2451548.028220777),
            Jdn(2451548.045857381),
            Jdn(2451548.063493863),
            Jdn(2451548.0811302345),
            Jdn(2451548.098766506),
            Jdn(2451548.1164026884),
            Jdn(2451548.134038793),
            Jdn(2451548.1516748304),
            Jdn(2451548.169310812),
            Jdn(2451548.1869467497),
            Jdn(2451548.204582653),
            Jdn(2451548.2222185344),
            Jdn(2451548.239854405),
        ];

        let res = bake_eccentric_anomaly_times(&elements, 1);
        assert_eq!(res, vec![expected[0], expected[expected.len() - 1]]);

        let res = bake_eccentric_anomaly_times(&elements, 4);
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

        let res = bake_eccentric_anomaly_times(&elements, 360);
        assert_eq!(res, expected);
    }
}
