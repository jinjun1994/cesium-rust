use crate::{HeadingPitchRoll, JulianDate, Matrix3};

const GMST_CONSTANT0: f64 = 6.0 * 3600.0 + 41.0 * 60.0 + 50.54841;
const GMST_CONSTANT1: f64 = 8640184.812866;
const GMST_CONSTANT2: f64 = 0.093104;
const GMST_CONSTANT3: f64 = -6.2e-6;
const RATE_COEF: f64 = 1.1772758384668e-19;
const WGS84_WR_PRECESSING: f64 = 7.2921158553e-5;
const TWO_PI_OVER_SECONDS_IN_DAY: f64 = std::f64::consts::TAU / 86400.0;
const DAYS_PER_JULIAN_CENTURY: f64 = 36525.0;

const TDT_MINUS_TAI: f64 = 32.184;
const J2000D: f64 = 2451545.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CelestialFrameTransforms;

impl CelestialFrameTransforms {
    /// Computes a rotation matrix to transform a point or vector from True Equator Mean Equinox (TEME)
    /// axes to the pseudo-fixed axes at a given time. Treats UT1 standard as equivalent to UTC.
    pub fn compute_teme_to_pseudo_fixed_matrix(date: &JulianDate) -> Matrix3 {
        let tai_minus_utc = date.compute_tai_minus_utc();
        let date_in_utc = date.add_seconds(-tai_minus_utc);
        let utc_day_number = date_in_utc.day_number;
        let utc_seconds_into_day = date_in_utc.seconds_of_day;

        let diff_days = (utc_day_number - 2451545) as f64;
        let t = if utc_seconds_into_day >= 43200.0 {
            (diff_days + 0.5) / DAYS_PER_JULIAN_CENTURY
        } else {
            (diff_days - 0.5) / DAYS_PER_JULIAN_CENTURY
        };

        let gmst0 =
            GMST_CONSTANT0 + t * (GMST_CONSTANT1 + t * (GMST_CONSTANT2 + t * GMST_CONSTANT3));
        let angle = (gmst0 * TWO_PI_OVER_SECONDS_IN_DAY) % std::f64::consts::TAU;
        let ratio = WGS84_WR_PRECESSING + RATE_COEF * (utc_day_number as f64 - 2451545.5);
        let percent_of_day = if utc_seconds_into_day < 43200.0 {
            utc_seconds_into_day / 86400.0 + 0.5
        } else {
            utc_seconds_into_day / 86400.0 - 0.5
        };
        let gmst = (angle + ratio * percent_of_day * 86400.0) % std::f64::consts::TAU;

        Matrix3::from_rotation_z(gmst)
    }

    /// Computes a rotation matrix to transform a point or vector from Moon-Fixed frame axes
    /// to the International Celestial Reference Frame (GCRF/ICRF) inertial frame axes.
    pub fn compute_moon_fixed_to_icrf_matrix(date: &JulianDate) -> Matrix3 {
        let seconds_tt = date.add_seconds(TDT_MINUS_TAI);
        let d = seconds_tt.total_days() - J2000D;

        let e1 = 12.112f64.to_radians() - 0.052992f64.to_radians() * d;
        let e2 = 24.224f64.to_radians() - 0.105984f64.to_radians() * d;
        let e3 = 227.645f64.to_radians() + 13.012f64.to_radians() * d;
        let e4 = 261.105f64.to_radians() + 13.340716f64.to_radians() * d;
        let e5 = 358.0f64.to_radians() + 0.9856f64.to_radians() * d;

        let pitch = (270.0_f64 - 90.0).to_radians()
            - 3.878_f64.to_radians() * e1.sin()
            - 0.12_f64.to_radians() * e2.sin()
            + 0.07_f64.to_radians() * e3.sin()
            - 0.017_f64.to_radians() * e4.sin();

        let roll = (66.53_f64 - 90.0).to_radians()
            + 1.543_f64.to_radians() * e1.cos()
            + 0.24_f64.to_radians() * e2.cos()
            - 0.028_f64.to_radians() * e3.cos()
            + 0.007_f64.to_radians() * e4.cos();

        let heading = (244.375_f64 - 90.0).to_radians()
            + 13.17635831_f64.to_radians() * d
            + 3.558_f64.to_radians() * e1.sin()
            + 0.121_f64.to_radians() * e2.sin()
            - 0.064_f64.to_radians() * e3.sin()
            + 0.016_f64.to_radians() * e4.sin()
            + 0.025_f64.to_radians() * e5.sin();

        let hpr = HeadingPitchRoll::new(heading, pitch, roll);
        Matrix3::from_heading_pitch_roll(&hpr)
    }

    /// Computes a rotation matrix from ICRF to the Moon Fixed axes at the given date.
    pub fn compute_icrf_to_moon_fixed_matrix(date: &JulianDate) -> Matrix3 {
        let fixed_to_icrf = Self::compute_moon_fixed_to_icrf_matrix(date);
        fixed_to_icrf.transpose()
    }

    /// Computes a rotation matrix from ICRF to Central Body Fixed (Earth/Moon) axes.
    pub fn compute_icrf_to_central_body_fixed_matrix(date: &JulianDate) -> Matrix3 {
        Self::compute_teme_to_pseudo_fixed_matrix(date)
    }
}
