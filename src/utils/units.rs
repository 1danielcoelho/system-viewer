use chrono::{TimeZone, Utc};
use serde::{Deserialize, Serialize};

pub const J2000_JDN: Jdn = Jdn(2451545.0);

#[derive(Clone, Copy, Default, Debug, Serialize, Deserialize)]
pub struct Rad(pub f64);
impl Rad {
    pub fn to_deg(&self) -> Deg {
        return Deg(self.0.to_degrees());
    }
}

#[derive(Clone, Copy, Default, Debug, Serialize, Deserialize)]
pub struct Deg(pub f64);
impl Deg {
    pub fn to_rad(&self) -> Rad {
        return Rad(self.0.to_radians());
    }
}

/// 1000 km
#[derive(Clone, Copy, Default, Debug, Serialize, Deserialize)]
pub struct Mm(pub f64);
impl Mm {
    #[allow(non_snake_case)]
    pub fn to_AU(&self) -> Au {
        return Au(self.0 / 149597.8707);
    }
}

/// 149597.8707 Mm
#[derive(Clone, Copy, Default, Debug, Serialize, Deserialize)]
pub struct Au(pub f64);
impl Au {
    #[allow(non_snake_case)]
    pub fn to_Mm(&self) -> Mm {
        return Mm(self.0 * 149597.8707);
    }
}

/// Julian Day Number, fractional number of days since noon of jan 1st, 4713 BC
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Jdn(pub f64);
impl Default for Jdn {
    fn default() -> Self {
        J2000_JDN
    }
}

/// Returns the corresponding Julian Day Number (JDN) for an instant in time.
/// See https://en.wikipedia.org/wiki/Julian_day
pub fn date_to_julian_date_number(date: &chrono::DateTime<Utc>) -> Jdn {
    return Jdn((date.timestamp() as f64) / 86400.0 + 2440587.5);
}

/// Returns the date object corresponding to a particular Julian Day Number (JDN)
/// See https://en.wikipedia.org/wiki/Julian_day
pub fn julian_date_number_to_date(jdn: Jdn) -> chrono::DateTime<Utc> {
    Utc.timestamp(((jdn.0 - 2440587.5) * 86400.0).round() as i64, 0)
}
