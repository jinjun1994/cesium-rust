use crate::JulianDate;

/// Orientation parameters for central celestial bodies from the IAU/IAG Working Group report (2000).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct IauOrientationParameters {
    /// Right ascension of the north pole with respect to ICRF, in radians.
    pub right_ascension: f64,
    /// Declination of the north pole with respect to ICRF, in radians.
    pub declination: f64,
    /// Rotation about the north pole, in radians.
    pub rotation: f64,
    /// Instantaneous rotation rate about the north pole, in radians per second.
    pub rotation_rate: f64,
}

impl Default for IauOrientationParameters {
    fn default() -> Self {
        Self {
            right_ascension: 0.0,
            declination: 0.0,
            rotation: 0.0,
            rotation_rate: 0.0,
        }
    }
}

impl IauOrientationParameters {
    pub fn new(
        right_ascension: f64,
        declination: f64,
        rotation: f64,
        rotation_rate: f64,
    ) -> Self {
        Self {
            right_ascension,
            declination,
            rotation,
            rotation_rate,
        }
    }
}

pub struct Iau2000Orientation;

const TDT_MINUS_TAI: f64 = 32.184;
const J2000D: f64 = 2451545.0;
const DAYS_PER_JULIAN_CENTURY: f64 = 36525.0;

const C1: f64 = -0.0529921;
const C2: f64 = -0.1059842;
const C3: f64 = 13.0120009;
const C4: f64 = 13.3407154;
const C5: f64 = 0.9856003;
const C6: f64 = 26.4057084;
const C7: f64 = 13.064993;
const C8: f64 = 0.3287146;
const C9: f64 = 1.7484877;
const C10: f64 = -0.1589763;
const C11: f64 = 0.0036096;
const C12: f64 = 0.1643573;
const C13: f64 = 12.9590088;

impl Iau2000Orientation {
    /// Compute the orientation parameters for the Moon at the given date.
    pub fn compute_moon(date: &JulianDate) -> IauOrientationParameters {
        let date_tt = date.add_seconds(TDT_MINUS_TAI);
        let d = date_tt.total_days() - J2000D;
        let t = d / DAYS_PER_JULIAN_CENTURY;

        let e1 = (125.045 + C1 * d).to_radians();
        let e2 = (250.089 + C2 * d).to_radians();
        let e3 = (260.008 + C3 * d).to_radians();
        let e4 = (176.625 + C4 * d).to_radians();
        let e5 = (357.529 + C5 * d).to_radians();
        let e6 = (311.589 + C6 * d).to_radians();
        let e7 = (134.963 + C7 * d).to_radians();
        let e8 = (276.617 + C8 * d).to_radians();
        let e9 = (34.226 + C9 * d).to_radians();
        let e10 = (15.134 + C10 * d).to_radians();
        let e11 = (119.743 + C11 * d).to_radians();
        let e12 = (239.961 + C12 * d).to_radians();
        let e13 = (25.053 + C13 * d).to_radians();

        let sin_e1 = e1.sin();
        let sin_e2 = e2.sin();
        let sin_e3 = e3.sin();
        let sin_e4 = e4.sin();
        let sin_e5 = e5.sin();
        let sin_e6 = e6.sin();
        let sin_e7 = e7.sin();
        let sin_e8 = e8.sin();
        let sin_e9 = e9.sin();
        let sin_e10 = e10.sin();
        let sin_e11 = e11.sin();
        let sin_e12 = e12.sin();
        let sin_e13 = e13.sin();

        let cos_e1 = e1.cos();
        let cos_e2 = e2.cos();
        let cos_e3 = e3.cos();
        let cos_e4 = e4.cos();
        let cos_e5 = e5.cos();
        let cos_e6 = e6.cos();
        let cos_e7 = e7.cos();
        let cos_e8 = e8.cos();
        let cos_e9 = e9.cos();
        let cos_e10 = e10.cos();
        let cos_e11 = e11.cos();
        let cos_e12 = e12.cos();
        let cos_e13 = e13.cos();

        let right_ascension = (269.9949
            + 0.0031 * t
            - 3.8787 * sin_e1
            - 0.1204 * sin_e2
            + 0.07 * sin_e3
            - 0.0172 * sin_e4
            + 0.0072 * sin_e6
            - 0.0052 * sin_e10
            + 0.0043 * sin_e13)
            .to_radians();

        let declination = (66.5392
            + 0.013 * t
            + 1.5419 * cos_e1
            + 0.0239 * cos_e2
            - 0.0278 * cos_e3
            + 0.0068 * cos_e4
            - 0.0029 * cos_e6
            + 0.0009 * cos_e7
            + 0.0008 * cos_e10
            - 0.0009 * cos_e13)
            .to_radians();

        let rotation = (38.3213 + 13.17635815 * d - 1.4e-12 * d * d
            + 3.561 * sin_e1
            + 0.1208 * sin_e2
            - 0.0642 * sin_e3
            + 0.0158 * sin_e4
            + 0.0252 * sin_e5
            - 0.0066 * sin_e6
            - 0.0047 * sin_e7
            - 0.0046 * sin_e8
            + 0.0028 * sin_e9
            + 0.0052 * sin_e10
            + 0.004 * sin_e11
            + 0.0019 * sin_e12
            - 0.0044 * sin_e13)
            .to_radians();

        let rotation_rate = ((13.17635815 - 1.4e-12 * (2.0 * d)
            + 3.561 * cos_e1 * C1
            + 0.1208 * cos_e2 * C2
            - 0.0642 * cos_e3 * C3
            + 0.0158 * cos_e4 * C4
            + 0.0252 * cos_e5 * C5
            - 0.0066 * cos_e6 * C6
            - 0.0047 * cos_e7 * C7
            - 0.0046 * cos_e8 * C8
            + 0.0028 * cos_e9 * C9
            + 0.0052 * cos_e10 * C10
            + 0.004 * cos_e11 * C11
            + 0.0019 * cos_e12 * C12
            - 0.0044 * cos_e13 * C13)
            / 86400.0)
            .to_radians();

        IauOrientationParameters {
            right_ascension,
            declination,
            rotation,
            rotation_rate,
        }
    }
}
