#![allow(clippy::inconsistent_digit_grouping, clippy::excessive_precision)]

use std::f64::consts::PI;

use crate::cartesian::Cartesian3;
use crate::core_math::{zero_to_two_pi, RADIANS_PER_ARCSECOND, RADIANS_PER_DEGREE, TWO_PI};
use crate::julian_date::JulianDate;
use crate::matrices::Matrix3;
use crate::time_constants::TimeConstants;

/// Contains functions for finding the Cartesian coordinates of the Sun and the Moon in the Earth-centered inertial frame.
pub struct Simon1994PlanetaryPositions;

fn compute_tdb_minus_tt_spice(days_since_j2000_in_tt: f64) -> f64 {
    let g = 6.239_996 + 0.017_201_969_654_4 * days_since_j2000_in_tt;
    1.657e-3 * (g + 1.671e-2 * g.sin()).sin()
}

const TDT_MINUS_TAI: f64 = 32.184;
const J2000D: f64 = 2451545.0;

fn tai_to_tdb(date: &JulianDate) -> JulianDate {
    // Converts TAI to TT
    let tt = date.add_seconds(TDT_MINUS_TAI);
    let days = tt.total_days() - J2000D;
    tt.add_seconds(compute_tdb_minus_tt_spice(days))
}

const METERS_PER_KILOMETER: f64 = 1000.0;
const METERS_PER_ASTRONOMICAL_UNIT: f64 = 1.495_978_7e11; // IAU 1976 value

fn elements_to_cartesian(
    semimajor_axis: f64,
    eccentricity: f64,
    mut inclination: f64,
    longitude_of_perigee: f64,
    mut longitude_of_node: f64,
    mean_longitude: f64,
) -> Cartesian3 {
    if inclination < 0.0 {
        inclination = -inclination;
        longitude_of_node += PI;
    }

    let radius_of_periapsis = semimajor_axis * (1.0 - eccentricity);
    let argument_of_periapsis = longitude_of_perigee - longitude_of_node;
    let right_ascension_of_ascending_node = longitude_of_node;
    let true_anomaly =
        mean_anomaly_to_true_anomaly(mean_longitude - longitude_of_perigee, eccentricity);

    let perifocal_to_equatorial = perifocal_to_cartesian_matrix(
        argument_of_periapsis,
        inclination,
        right_ascension_of_ascending_node,
    );
    let semilatus = radius_of_periapsis * (1.0 + eccentricity);
    let costheta = true_anomaly.cos();
    let sintheta = true_anomaly.sin();

    let denom = 1.0 + eccentricity * costheta;
    let radius = semilatus / denom;

    let v = Cartesian3::new(radius * costheta, radius * sintheta, 0.0);
    perifocal_to_equatorial.multiply_by_vector(v)
}

fn mean_anomaly_to_true_anomaly(mean_anomaly: f64, eccentricity: f64) -> f64 {
    let eccentric_anomaly = mean_anomaly_to_eccentric_anomaly(mean_anomaly, eccentricity);
    eccentric_anomaly_to_true_anomaly(eccentric_anomaly, eccentricity)
}

const MAX_ITERATION_COUNT: usize = 50;
const KEPLER_EQ_CONVERGENCE: f64 = 1e-8;

fn mean_anomaly_to_eccentric_anomaly(mut mean_anomaly: f64, eccentricity: f64) -> f64 {
    let revs = (mean_anomaly / TWO_PI).floor();
    mean_anomaly -= revs * TWO_PI;

    let mut iteration_value = mean_anomaly
        + (eccentricity * mean_anomaly.sin())
            / (1.0 - (mean_anomaly + eccentricity).sin() + mean_anomaly.sin());

    let mut eccentric_anomaly = f64::MAX;
    for _ in 0..MAX_ITERATION_COUNT {
        if (eccentric_anomaly - iteration_value).abs() <= KEPLER_EQ_CONVERGENCE {
            break;
        }
        eccentric_anomaly = iteration_value;
        let nr_function = eccentric_anomaly - eccentricity * eccentric_anomaly.sin() - mean_anomaly;
        let d_nr_function = 1.0 - eccentricity * eccentric_anomaly.cos();
        iteration_value = eccentric_anomaly - nr_function / d_nr_function;
    }

    iteration_value + revs * TWO_PI
}

fn eccentric_anomaly_to_true_anomaly(mut eccentric_anomaly: f64, eccentricity: f64) -> f64 {
    let revs = (eccentric_anomaly / TWO_PI).floor();
    eccentric_anomaly -= revs * TWO_PI;

    let true_anomaly_x = eccentric_anomaly.cos() - eccentricity;
    let true_anomaly_y = eccentric_anomaly.sin() * (1.0 - eccentricity * eccentricity).sqrt();

    let mut true_anomaly = true_anomaly_y.atan2(true_anomaly_x);
    true_anomaly = zero_to_two_pi(true_anomaly);
    if eccentric_anomaly < 0.0 {
        true_anomaly -= TWO_PI;
    }

    true_anomaly + revs * TWO_PI
}

fn perifocal_to_cartesian_matrix(
    argument_of_periapsis: f64,
    inclination: f64,
    right_ascension: f64,
) -> Matrix3 {
    let cosap = argument_of_periapsis.cos();
    let sinap = argument_of_periapsis.sin();
    let cosi = inclination.cos();
    let sini = inclination.sin();
    let cosraan = right_ascension.cos();
    let sinraan = right_ascension.sin();

    Matrix3::new(
        cosraan * cosap - sinraan * sinap * cosi,
        -cosraan * sinap - sinraan * cosap * cosi,
        sinraan * sini,
        sinraan * cosap + cosraan * sinap * cosi,
        -sinraan * sinap + cosraan * cosap * cosi,
        -cosraan * sini,
        sinap * sini,
        cosap * sini,
        cosi,
    )
}

// Section 5.8
const SEMI_MAJOR_AXIS_0: f64 = 1.000_001_017_8 * METERS_PER_ASTRONOMICAL_UNIT;
const MEAN_LONGITUDE_0: f64 = 100.466_456_83 * RADIANS_PER_DEGREE;
const MEAN_LONGITUDE_1: f64 = 1_295_977_422.834_29 * RADIANS_PER_ARCSECOND;

// Table 6
const P1U: f64 = 16002.0;
const P2U: f64 = 21863.0;
const P3U: f64 = 32004.0;
const P4U: f64 = 10931.0;
const P5U: f64 = 14529.0;
const P6U: f64 = 16368.0;
const P7U: f64 = 15318.0;
const P8U: f64 = 32794.0;

const CA1: f64 = 64.0 * 1e-7 * METERS_PER_ASTRONOMICAL_UNIT;
const CA2: f64 = -152.0 * 1e-7 * METERS_PER_ASTRONOMICAL_UNIT;
const CA3: f64 = 62.0 * 1e-7 * METERS_PER_ASTRONOMICAL_UNIT;
const CA4: f64 = -8.0 * 1e-7 * METERS_PER_ASTRONOMICAL_UNIT;
const CA5: f64 = 32.0 * 1e-7 * METERS_PER_ASTRONOMICAL_UNIT;
const CA6: f64 = -41.0 * 1e-7 * METERS_PER_ASTRONOMICAL_UNIT;
const CA7: f64 = 19.0 * 1e-7 * METERS_PER_ASTRONOMICAL_UNIT;
const CA8: f64 = -11.0 * 1e-7 * METERS_PER_ASTRONOMICAL_UNIT;

const SA1: f64 = -150.0 * 1e-7 * METERS_PER_ASTRONOMICAL_UNIT;
const SA2: f64 = -46.0 * 1e-7 * METERS_PER_ASTRONOMICAL_UNIT;
const SA3: f64 = 68.0 * 1e-7 * METERS_PER_ASTRONOMICAL_UNIT;
const SA4: f64 = 54.0 * 1e-7 * METERS_PER_ASTRONOMICAL_UNIT;
const SA5: f64 = 14.0 * 1e-7 * METERS_PER_ASTRONOMICAL_UNIT;
const SA6: f64 = 24.0 * 1e-7 * METERS_PER_ASTRONOMICAL_UNIT;
const SA7: f64 = -28.0 * 1e-7 * METERS_PER_ASTRONOMICAL_UNIT;
const SA8: f64 = 22.0 * 1e-7 * METERS_PER_ASTRONOMICAL_UNIT;

const Q1U: f64 = 10.0;
const Q2U: f64 = 16002.0;
const Q3U: f64 = 21863.0;
const Q4U: f64 = 10931.0;
const Q5U: f64 = 1473.0;
const Q6U: f64 = 32004.0;
const Q7U: f64 = 4387.0;
const Q8U: f64 = 73.0;

const CL1: f64 = -325.0 * 1e-7;
const CL2: f64 = -322.0 * 1e-7;
const CL3: f64 = -79.0 * 1e-7;
const CL4: f64 = 232.0 * 1e-7;
const CL5: f64 = -52.0 * 1e-7;
const CL6: f64 = 97.0 * 1e-7;
const CL7: f64 = 55.0 * 1e-7;
const CL8: f64 = -41.0 * 1e-7;

const SL1: f64 = -105.0 * 1e-7;
const SL2: f64 = -137.0 * 1e-7;
const SL3: f64 = 258.0 * 1e-7;
const SL4: f64 = 35.0 * 1e-7;
const SL5: f64 = -116.0 * 1e-7;
const SL6: f64 = -88.0 * 1e-7;
const SL7: f64 = -112.0 * 1e-7;
const SL8: f64 = -80.0 * 1e-7;

fn compute_simon_earth_moon_barycenter(date: &JulianDate) -> Cartesian3 {
    let tdb_date = tai_to_tdb(date);
    let epoch = JulianDate::new_tai(2451545, 0.0);
    let x = (tdb_date.day_number as f64 - epoch.day_number as f64)
        + (tdb_date.seconds_of_day - epoch.seconds_of_day) / TimeConstants::SECONDS_PER_DAY;
    let t = x / (TimeConstants::DAYS_PER_JULIAN_CENTURY * 10.0);

    let u = 0.359_536_2 * t;
    let semimajor_axis = SEMI_MAJOR_AXIS_0
        + CA1 * (P1U * u).cos()
        + SA1 * (P1U * u).sin()
        + CA2 * (P2U * u).cos()
        + SA2 * (P2U * u).sin()
        + CA3 * (P3U * u).cos()
        + SA3 * (P3U * u).sin()
        + CA4 * (P4U * u).cos()
        + SA4 * (P4U * u).sin()
        + CA5 * (P5U * u).cos()
        + SA5 * (P5U * u).sin()
        + CA6 * (P6U * u).cos()
        + SA6 * (P6U * u).sin()
        + CA7 * (P7U * u).cos()
        + SA7 * (P7U * u).sin()
        + CA8 * (P8U * u).cos()
        + SA8 * (P8U * u).sin();

    let mean_longitude = MEAN_LONGITUDE_0
        + MEAN_LONGITUDE_1 * t
        + CL1 * (Q1U * u).cos()
        + SL1 * (Q1U * u).sin()
        + CL2 * (Q2U * u).cos()
        + SL2 * (Q2U * u).sin()
        + CL3 * (Q3U * u).cos()
        + SL3 * (Q3U * u).sin()
        + CL4 * (Q4U * u).cos()
        + SL4 * (Q4U * u).sin()
        + CL5 * (Q5U * u).cos()
        + SL5 * (Q5U * u).sin()
        + CL6 * (Q6U * u).cos()
        + SL6 * (Q6U * u).sin()
        + CL7 * (Q7U * u).cos()
        + SL7 * (Q7U * u).sin()
        + CL8 * (Q8U * u).cos()
        + SL8 * (Q8U * u).sin();

    let eccentricity = 0.016_708_634_2 - 0.000_420_365_4 * t;
    let longitude_of_perigee =
        102.937_348_08 * RADIANS_PER_DEGREE + 11612.3529 * RADIANS_PER_ARCSECOND * t;
    let inclination = 469.972_89 * RADIANS_PER_ARCSECOND * t;
    let longitude_of_node =
        174.873_175_77 * RADIANS_PER_DEGREE - 8679.270_34 * RADIANS_PER_ARCSECOND * t;

    elements_to_cartesian(
        semimajor_axis,
        eccentricity,
        inclination,
        longitude_of_perigee,
        longitude_of_node,
        mean_longitude,
    )
}

fn compute_simon_moon(date: &JulianDate) -> Cartesian3 {
    let tdb_date = tai_to_tdb(date);
    let epoch = JulianDate::new_tai(2451545, 0.0);
    let x = (tdb_date.day_number as f64 - epoch.day_number as f64)
        + (tdb_date.seconds_of_day - epoch.seconds_of_day) / TimeConstants::SECONDS_PER_DAY;
    let t = x / TimeConstants::DAYS_PER_JULIAN_CENTURY;
    let t2 = t * t;
    let t3 = t2 * t;
    let t4 = t3 * t;

    let mut semimajor_axis = 383_397.7725 + 0.004 * t;
    let mut eccentricity = 0.055_545_526 - 0.000_000_016 * t;
    let inclination_constant = 5.156_689_83 * RADIANS_PER_DEGREE;
    let mut inclination_sec_part =
        -0.00008 * t + 0.02966 * t2 - 0.000_042 * t3 - 0.000_000_13 * t4;
    let longitude_of_perigee_constant = 83.353_243_12 * RADIANS_PER_DEGREE;
    let mut longitude_of_perigee_sec_part =
        14_643_420.2669 * t - 38.2702 * t2 - 0.045_047 * t3 + 0.000_213_01 * t4;
    let longitude_of_node_constant = 125.044_555_01 * RADIANS_PER_DEGREE;
    let mut longitude_of_node_sec_part =
        -6_967_919.3631 * t + 6.3602 * t2 + 0.007_625 * t3 - 0.000_035_86 * t4;
    let mean_longitude_constant = 218.316_645_63 * RADIANS_PER_DEGREE;
    let mut mean_longitude_sec_part =
        1_732_559_343.4847 * t - 6.391 * t2 + 0.006_588 * t3 - 0.000_031_69 * t4;

    let d = 297.850_195_47 * RADIANS_PER_DEGREE
        + RADIANS_PER_ARCSECOND
            * (1_602_961_601.209 * t - 6.3706 * t2 + 0.006_593 * t3 - 0.000_031_69 * t4);
    let f = 93.272_090_62 * RADIANS_PER_DEGREE
        + RADIANS_PER_ARCSECOND
            * (1_739_527_262.8478 * t - 12.7512 * t2 - 0.001_037 * t3 + 0.000_004_17 * t4);
    let l = 134.963_402_51 * RADIANS_PER_DEGREE
        + RADIANS_PER_ARCSECOND
            * (1_717_915_923.2178 * t + 31.8792 * t2 + 0.051_635 * t3 - 0.000_244_7 * t4);
    let lprime = 357.529_109_18 * RADIANS_PER_DEGREE
        + RADIANS_PER_ARCSECOND
            * (129_596_581.0481 * t - 0.5532 * t2 + 0.000_136 * t3 - 0.000_011_49 * t4);
    let psi = 310.171_379_18 * RADIANS_PER_DEGREE
        - RADIANS_PER_ARCSECOND
            * (6_967_051.436 * t + 6.2068 * t2 + 0.007_618 * t3 - 0.000_032_19 * t4);

    let two_d = 2.0 * d;
    let four_d = 4.0 * d;
    let six_d = 6.0 * d;
    let two_l = 2.0 * l;
    let three_l = 3.0 * l;
    let four_l = 4.0 * l;
    let two_f = 2.0 * f;

    semimajor_axis += 3400.4 * two_d.cos()
        - 635.6 * (two_d - l).cos()
        - 235.6 * l.cos()
        + 218.1 * (two_d - lprime).cos()
        + 181.0 * (two_d + l).cos();

    eccentricity += 0.014_216 * (two_d - l).cos()
        + 0.008_551 * (two_d - two_l).cos()
        - 0.001_383 * l.cos()
        + 0.001_356 * (two_d + l).cos()
        - 0.001_147 * (four_d - three_l).cos()
        - 0.000_914 * (four_d - two_l).cos()
        + 0.000_869 * (two_d - lprime - l).cos()
        - 0.000_627 * two_d.cos()
        - 0.000_394 * (four_d - four_l).cos()
        + 0.000_282 * (two_d - lprime - two_l).cos()
        - 0.000_279 * (d - l).cos()
        - 0.000_236 * two_l.cos()
        + 0.000_231 * four_d.cos()
        + 0.000_229 * (six_d - four_l).cos()
        - 0.000_201 * (two_l - two_f).cos();

    inclination_sec_part += 486.26 * (two_d - two_f).cos()
        - 40.13 * two_d.cos()
        + 37.51 * two_f.cos()
        + 25.73 * (two_l - two_f).cos()
        + 19.97 * (two_d - lprime - two_f).cos();

    longitude_of_perigee_sec_part += -55609.0 * (two_d - l).sin()
        - 34711.0 * (two_d - two_l).sin()
        - 9792.0 * l.sin()
        + 9385.0 * (four_d - three_l).sin()
        + 7505.0 * (four_d - two_l).sin()
        + 5318.0 * (two_d + l).sin()
        + 3484.0 * (four_d - four_l).sin()
        - 3417.0 * (two_d - lprime - l).sin()
        - 2530.0 * (six_d - four_l).sin()
        - 2376.0 * two_d.sin()
        - 2075.0 * (two_d - three_l).sin()
        - 1883.0 * two_l.sin()
        - 1736.0 * (six_d - 5.0 * l).sin()
        + 1626.0 * lprime.sin()
        - 1370.0 * (six_d - three_l).sin();

    longitude_of_node_sec_part += -5392.0 * (two_d - two_f).sin()
        - 540.0 * lprime.sin()
        - 441.0 * two_d.sin()
        + 423.0 * two_f.sin()
        - 288.0 * (two_l - two_f).sin();

    mean_longitude_sec_part += -3332.9 * two_d.sin()
        + 1197.4 * (two_d - l).sin()
        - 662.5 * lprime.sin()
        + 396.3 * l.sin()
        - 218.0 * (two_d - lprime).sin();

    let two_psi = 2.0 * psi;
    let three_psi = 3.0 * psi;
    inclination_sec_part += 46.997 * psi.cos() * t
        - 0.614 * (two_d - two_f + psi).cos() * t
        + 0.614 * (two_d - two_f - psi).cos() * t
        - 0.0297 * two_psi.cos() * t2
        - 0.0335 * psi.cos() * t2
        + 0.0012 * (two_d - two_f + two_psi).cos() * t2
        - 0.000_16 * psi.cos() * t3
        + 0.000_04 * three_psi.cos() * t3
        + 0.000_04 * two_psi.cos() * t3;

    let perigee_and_mean =
        2.116 * psi.sin() * t - 0.111 * (two_d - two_f - psi).sin() * t - 0.0015 * psi.sin() * t2;
    longitude_of_perigee_sec_part += perigee_and_mean;
    mean_longitude_sec_part += perigee_and_mean;

    longitude_of_node_sec_part += -520.77 * psi.sin() * t
        + 13.66 * (two_d - two_f + psi).sin() * t
        + 1.12 * (two_d - psi).sin() * t
        - 1.06 * (two_f - psi).sin() * t
        + 0.66 * two_psi.sin() * t2
        + 0.371 * psi.sin() * t2
        - 0.035 * (two_d - two_f + two_psi).sin() * t2
        - 0.015 * (two_d - two_f + psi).sin() * t2
        + 0.0014 * psi.sin() * t3
        - 0.0011 * three_psi.sin() * t3
        - 0.0009 * two_psi.sin() * t3;

    semimajor_axis *= METERS_PER_KILOMETER;
    let inclination = inclination_constant + inclination_sec_part * RADIANS_PER_ARCSECOND;
    let longitude_of_perigee =
        longitude_of_perigee_constant + longitude_of_perigee_sec_part * RADIANS_PER_ARCSECOND;
    let mean_longitude =
        mean_longitude_constant + mean_longitude_sec_part * RADIANS_PER_ARCSECOND;
    let longitude_of_node =
        longitude_of_node_constant + longitude_of_node_sec_part * RADIANS_PER_ARCSECOND;

    elements_to_cartesian(
        semimajor_axis,
        eccentricity,
        inclination,
        longitude_of_perigee,
        longitude_of_node,
        mean_longitude,
    )
}

const MOON_EARTH_MASS_RATIO: f64 = 0.012_300_034;
const EARTH_OFFSET_FACTOR: f64 = -(MOON_EARTH_MASS_RATIO / (MOON_EARTH_MASS_RATIO + 1.0));

fn compute_simon_earth(date: &JulianDate) -> Cartesian3 {
    let moon = compute_simon_moon(date);
    moon * EARTH_OFFSET_FACTOR
}

const AXES_TRANSFORMATION: Matrix3 = Matrix3::new(
    1.000_000_000_000_000_2,
    5.619_723_173_785_822e-16,
    4.690_511_510_146_299e-19,
    -5.154_129_427_414_611e-16,
    0.917_482_062_069_181_9,
    -0.397_777_155_931_913_76,
    -2.239_700_961_365_68e-16,
    0.397_777_155_931_913_76,
    0.917_482_062_069_181_9,
);

impl Simon1994PlanetaryPositions {
    /// Computes the position of the Sun in the Earth-centered inertial (J2000/GCRF) frame.
    pub fn compute_sun_position_in_earth_inertial_frame(
        date: Option<&JulianDate>,
    ) -> Cartesian3 {
        let now_holder;
        let d = match date {
            Some(dt) => dt,
            None => {
                now_holder = JulianDate::now();
                &now_holder
            }
        };

        let barycenter = compute_simon_earth_moon_barycenter(d);
        let neg_barycenter = -barycenter;
        let earth = compute_simon_earth(d);
        let diff = neg_barycenter - earth;

        AXES_TRANSFORMATION.multiply_by_vector(diff)
    }

    /// Computes the position of the Moon in the Earth-centered inertial (J2000/GCRF) frame.
    pub fn compute_moon_position_in_earth_inertial_frame(
        date: Option<&JulianDate>,
    ) -> Cartesian3 {
        let now_holder;
        let d = match date {
            Some(dt) => dt,
            None => {
                now_holder = JulianDate::now();
                &now_holder
            }
        };

        let moon = compute_simon_moon(d);
        AXES_TRANSFORMATION.multiply_by_vector(moon)
    }
}
