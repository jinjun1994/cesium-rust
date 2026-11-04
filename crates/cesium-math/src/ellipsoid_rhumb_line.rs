//! Ellipsoid rhumb line matching CesiumJS `Core/EllipsoidRhumbLine`.
//! Adapted from CesiumJS (Apache-2.0).

use crate::cartesian::Cartesian3;
use crate::core_math::{negative_pi_to_pi, EPSILON10, EPSILON8};
use crate::ellipsoid::Ellipsoid;
use crate::errors::DeveloperError;
use crate::Cartographic;

#[derive(Clone, Debug, PartialEq)]
pub struct EllipsoidRhumbLine {
    ellipsoid: Ellipsoid,
    start: Option<Cartographic>,
    end: Option<Cartographic>,
    heading: Option<f64>,
    distance: Option<f64>,
    ellipticity: f64,
    ellipticity_squared: f64,
}

impl Default for EllipsoidRhumbLine {
    fn default() -> Self {
        Self::new(None, None, None)
    }
}

fn calculate_m(ellipticity: f64, major: f64, latitude: f64) -> f64 {
    if ellipticity == 0.0 {
        return major * latitude;
    }

    let e2 = ellipticity * ellipticity;
    let e4 = e2 * e2;
    let e6 = e4 * e2;
    let e8 = e6 * e2;
    let e10 = e8 * e2;
    let e12 = e10 * e2;
    let phi = latitude;
    let sin_2phi = (2.0 * phi).sin();
    let sin_4phi = (4.0 * phi).sin();
    let sin_6phi = (6.0 * phi).sin();
    let sin_8phi = (8.0 * phi).sin();
    let sin_10phi = (10.0 * phi).sin();
    let sin_12phi = (12.0 * phi).sin();

    major
        * ((1.0
            - e2 / 4.0
            - (3.0 * e4) / 64.0
            - (5.0 * e6) / 256.0
            - (175.0 * e8) / 16384.0
            - (441.0 * e10) / 65536.0
            - (4851.0 * e12) / 1048576.0)
            * phi
            - ((3.0 * e2) / 8.0
                + (3.0 * e4) / 32.0
                + (45.0 * e6) / 1024.0
                + (105.0 * e8) / 4096.0
                + (2205.0 * e10) / 131072.0
                + (6237.0 * e12) / 524288.0)
                * sin_2phi
            + ((15.0 * e4) / 256.0
                + (45.0 * e6) / 1024.0
                + (525.0 * e8) / 16384.0
                + (1575.0 * e10) / 65536.0
                + (155925.0 * e12) / 8388608.0)
                * sin_4phi
            - ((35.0 * e6) / 3072.0
                + (175.0 * e8) / 12288.0
                + (3675.0 * e10) / 262144.0
                + (13475.0 * e12) / 1048576.0)
                * sin_6phi
            + ((315.0 * e8) / 131072.0
                + (2205.0 * e10) / 524288.0
                + (43659.0 * e12) / 8388608.0)
                * sin_8phi
            - ((693.0 * e10) / 1310720.0 + (6237.0 * e12) / 5242880.0) * sin_10phi
            + ((1001.0 * e12) / 8388608.0) * sin_12phi)
}

fn calculate_inverse_m(m: f64, ellipticity: f64, major: f64) -> f64 {
    let d = m / major;

    if ellipticity == 0.0 {
        return d;
    }

    let d2 = d * d;
    let d3 = d2 * d;
    let d4 = d3 * d;
    let e = ellipticity;
    let e2 = e * e;
    let e4 = e2 * e2;
    let e6 = e4 * e2;
    let e8 = e6 * e2;
    let e10 = e8 * e2;
    let e12 = e10 * e2;
    let sin_2d = (2.0 * d).sin();
    let cos_2d = (2.0 * d).cos();
    let sin_4d = (4.0 * d).sin();
    let cos_4d = (4.0 * d).cos();
    let sin_6d = (6.0 * d).sin();
    let cos_6d = (6.0 * d).cos();
    let sin_8d = (8.0 * d).sin();
    let cos_8d = (8.0 * d).cos();
    let sin_10d = (10.0 * d).sin();
    let cos_10d = (10.0 * d).cos();
    let sin_12d = (12.0 * d).sin();

    d + (d * e2) / 4.0
        + (7.0 * d * e4) / 64.0
        + (15.0 * d * e6) / 256.0
        + (579.0 * d * e8) / 16384.0
        + (1515.0 * d * e10) / 65536.0
        + (16837.0 * d * e12) / 1048576.0
        + ((3.0 * d * e4) / 16.0 + (45.0 * d * e6) / 256.0
            - (d * (32.0 * d2 - 561.0) * e8) / 4096.0
            - (d * (232.0 * d2 - 1677.0) * e10) / 16384.0
            + (d * (399985.0 - 90560.0 * d2 + 512.0 * d4) * e12) / 5242880.0)
            * cos_2d
        + ((21.0 * d * e6) / 256.0 + (483.0 * d * e8) / 4096.0
            - (d * (224.0 * d2 - 1969.0) * e10) / 16384.0
            - (d * (33152.0 * d2 - 112599.0) * e12) / 1048576.0)
            * cos_4d
        + ((151.0 * d * e8) / 4096.0
            + (4681.0 * d * e10) / 65536.0
            + (1479.0 * d * e12) / 16384.0
            - (453.0 * d3 * e12) / 32768.0)
            * cos_6d
        + ((1097.0 * d * e10) / 65536.0 + (42783.0 * d * e12) / 1048576.0) * cos_8d
        + ((8011.0 * d * e12) / 1048576.0) * cos_10d
        + ((3.0 * e2) / 8.0
            + (3.0 * e4) / 16.0
            + (213.0 * e6) / 2048.0
            - (3.0 * d2 * e6) / 64.0
            + (255.0 * e8) / 4096.0
            - (33.0 * d2 * e8) / 512.0
            + (20861.0 * e10) / 524288.0
            - (33.0 * d2 * e10) / 512.0
            + (d4 * e10) / 1024.0
            + (28273.0 * e12) / 1048576.0
            - (471.0 * d2 * e12) / 8192.0
            + (9.0 * d4 * e12) / 4096.0)
            * sin_2d
        + ((21.0 * e4) / 256.0
            + (21.0 * e6) / 256.0
            + (533.0 * e8) / 8192.0
            - (21.0 * d2 * e8) / 512.0
            + (197.0 * e10) / 4096.0
            - (315.0 * d2 * e10) / 4096.0
            + (584039.0 * e12) / 16777216.0
            - (12517.0 * d2 * e12) / 131072.0
            + (7.0 * d4 * e12) / 2048.0)
            * sin_4d
        + ((151.0 * e6) / 6144.0
            + (151.0 * e8) / 4096.0
            + (5019.0 * e10) / 131072.0
            - (453.0 * d2 * e10) / 16384.0
            + (26965.0 * e12) / 786432.0
            - (8607.0 * d2 * e12) / 131072.0)
            * sin_6d
        + ((1097.0 * e8) / 131072.0
            + (1097.0 * e10) / 65536.0
            + (225797.0 * e12) / 10485760.0
            - (1097.0 * d2 * e12) / 65536.0)
            * sin_8d
        + ((8011.0 * e10) / 2621440.0 + (8011.0 * e12) / 1048576.0) * sin_10d
        + ((293393.0 * e12) / 251658240.0) * sin_12d
}

fn calculate_sigma(ellipticity: f64, latitude: f64) -> f64 {
    if ellipticity == 0.0 {
        return (0.5 * (std::f64::consts::FRAC_PI_2 + latitude)).tan().ln();
    }

    let e_sin_l = ellipticity * latitude.sin();
    (0.5 * (std::f64::consts::FRAC_PI_2 + latitude)).tan().ln()
        - (ellipticity / 2.0) * ((1.0 + e_sin_l) / (1.0 - e_sin_l)).ln()
}

fn calculate_heading(
    ellipticity: f64,
    first_longitude: f64,
    first_latitude: f64,
    second_longitude: f64,
    second_latitude: f64,
) -> f64 {
    let sigma1 = calculate_sigma(ellipticity, first_latitude);
    let sigma2 = calculate_sigma(ellipticity, second_latitude);
    negative_pi_to_pi(second_longitude - first_longitude).atan2(sigma2 - sigma1)
}

#[allow(clippy::too_many_arguments)]
fn calculate_arc_length(
    heading: f64,
    ellipticity: f64,
    ellipticity_squared: f64,
    major: f64,
    minor: f64,
    first_longitude: f64,
    first_latitude: f64,
    second_longitude: f64,
    second_latitude: f64,
) -> f64 {
    let delta_longitude = second_longitude - first_longitude;

    let distance = if (heading.abs() - std::f64::consts::FRAC_PI_2).abs() < EPSILON8 {
        if major == minor {
            major * first_latitude.cos() * negative_pi_to_pi(delta_longitude)
        } else {
            let sin_phi = first_latitude.sin();
            (major * first_latitude.cos() * negative_pi_to_pi(delta_longitude))
                / (1.0 - ellipticity_squared * sin_phi * sin_phi).sqrt()
        }
    } else {
        let m1 = calculate_m(ellipticity, major, first_latitude);
        let m2 = calculate_m(ellipticity, major, second_latitude);
        (m2 - m1) / heading.cos()
    };

    distance.abs()
}

impl EllipsoidRhumbLine {
    pub fn new(
        start: Option<Cartographic>,
        end: Option<Cartographic>,
        ellipsoid: Option<Ellipsoid>,
    ) -> Self {
        let e = ellipsoid.unwrap_or(Ellipsoid::WGS84);
        let major = e.maximum_radius();
        let minor = e.minimum_radius();
        let major_sq = major * major;
        let minor_sq = minor * minor;
        let ellipticity_squared = (major_sq - minor_sq) / major_sq;
        let ellipticity = ellipticity_squared.max(0.0).sqrt();

        let mut rhumb = Self {
            ellipsoid: e,
            start: None,
            end: None,
            heading: None,
            distance: None,
            ellipticity,
            ellipticity_squared,
        };

        if let (Some(s), Some(nd)) = (start, end) {
            let _ = rhumb.set_end_points(s, nd);
        }

        rhumb
    }

    pub fn from_start_heading_distance(
        start: Cartographic,
        heading: f64,
        distance: f64,
        ellipsoid: Option<Ellipsoid>,
    ) -> Self {
        let mut rhumb = Self::new(None, None, ellipsoid);
        let end = rhumb
            .interpolate_internal(
                &start,
                heading,
                distance,
                rhumb.ellipsoid.maximum_radius(),
                rhumb.ellipticity,
            );
        let _ = rhumb.set_end_points(start, end);
        rhumb
    }

    pub fn set_end_points(
        &mut self,
        start: Cartographic,
        end: Cartographic,
    ) -> Result<(), DeveloperError> {
        let p1 = Cartesian3::from(
            self.ellipsoid
                .cartographic_to_cartesian(start)
                .ok_or_else(|| DeveloperError::new("Invalid start cartographic"))?,
        )
        .normalize();
        let p2 = Cartesian3::from(
            self.ellipsoid
                .cartographic_to_cartesian(end)
                .ok_or_else(|| DeveloperError::new("Invalid end cartographic"))?,
        )
        .normalize();

        let angle = p1.angle_between(&p2);
        if (angle.abs() - std::f64::consts::PI).abs() < 0.0125 {
            return Err(DeveloperError::new(
                "Angle between start and end points is too close to antipodal.",
            ));
        }

        let mut s_clean = start;
        s_clean.height = 0.0;
        let mut e_clean = end;
        e_clean.height = 0.0;

        let h = calculate_heading(
            self.ellipticity,
            s_clean.longitude,
            s_clean.latitude,
            e_clean.longitude,
            e_clean.latitude,
        );

        let d = calculate_arc_length(
            h,
            self.ellipticity,
            self.ellipticity_squared,
            self.ellipsoid.maximum_radius(),
            self.ellipsoid.minimum_radius(),
            s_clean.longitude,
            s_clean.latitude,
            e_clean.longitude,
            e_clean.latitude,
        );

        self.start = Some(s_clean);
        self.end = Some(e_clean);
        self.heading = Some(h);
        self.distance = Some(d);

        Ok(())
    }

    pub fn ellipsoid(&self) -> &Ellipsoid {
        &self.ellipsoid
    }

    pub fn start(&self) -> Option<Cartographic> {
        self.start
    }

    pub fn end(&self) -> Option<Cartographic> {
        self.end
    }

    pub fn heading(&self) -> Result<f64, DeveloperError> {
        self.heading
            .ok_or_else(|| DeveloperError::new("start and end points must be defined."))
    }

    pub fn surface_distance(&self) -> Result<f64, DeveloperError> {
        self.distance
            .ok_or_else(|| DeveloperError::new("start and end points must be defined."))
    }

    pub fn interpolate_using_fraction(&self, fraction: f64) -> Result<Cartographic, DeveloperError> {
        let dist = self.surface_distance()?;
        self.interpolate_using_surface_distance(dist * fraction)
    }

    fn interpolate_internal(
        &self,
        start: &Cartographic,
        heading: f64,
        distance: f64,
        major: f64,
        ellipticity: f64,
    ) -> Cartographic {
        if distance == 0.0 {
            return *start;
        }

        let ellipticity_squared = ellipticity * ellipticity;

        let (latitude, longitude) = if (std::f64::consts::FRAC_PI_2 - heading.abs()).abs() > EPSILON8 {
            let m1 = calculate_m(ellipticity, major, start.latitude);
            let delta_m = distance * heading.cos();
            let m2 = m1 + delta_m;
            let lat = calculate_inverse_m(m2, ellipticity, major);

            let lon = if heading.abs() < EPSILON10 {
                negative_pi_to_pi(start.longitude)
            } else {
                let sigma1 = calculate_sigma(ellipticity, start.latitude);
                let sigma2 = calculate_sigma(ellipticity, lat);
                let delta_lon = heading.tan() * (sigma2 - sigma1);
                negative_pi_to_pi(start.longitude + delta_lon)
            };
            (lat, lon)
        } else {
            let lat = start.latitude;
            let local_rad = if ellipticity == 0.0 {
                major * start.latitude.cos()
            } else {
                let sin_phi = start.latitude.sin();
                (major * start.latitude.cos()) / (1.0 - ellipticity_squared * sin_phi * sin_phi).sqrt()
            };

            let delta_lon = distance / local_rad;
            let lon = if heading > 0.0 {
                negative_pi_to_pi(start.longitude + delta_lon)
            } else {
                negative_pi_to_pi(start.longitude - delta_lon)
            };
            (lat, lon)
        };

        Cartographic {
            longitude,
            latitude,
            height: 0.0,
        }
    }

    pub fn interpolate_using_surface_distance(
        &self,
        distance: f64,
    ) -> Result<Cartographic, DeveloperError> {
        let start = self
            .start
            .ok_or_else(|| DeveloperError::new("start point must be defined."))?;
        let heading = self.heading()?;
        Ok(self.interpolate_internal(
            &start,
            heading,
            distance,
            self.ellipsoid.maximum_radius(),
            self.ellipticity,
        ))
    }

    pub fn find_intersection_with_latitude(
        &self,
        intersection_latitude: f64,
    ) -> Option<Cartographic> {
        let dist = self.surface_distance().ok()?;
        if dist == 0.0 {
            return None;
        }
        let heading = self.heading().ok()?;
        let start = self.start?;

        if (heading.abs() - std::f64::consts::FRAC_PI_2).abs() < EPSILON8 {
            return None;
        }

        let sigma1 = calculate_sigma(self.ellipticity, start.latitude);
        let sigma2 = calculate_sigma(self.ellipticity, intersection_latitude);
        let delta_lon = heading.tan() * (sigma2 - sigma1);
        let longitude = negative_pi_to_pi(start.longitude + delta_lon);

        Some(Cartographic {
            longitude,
            latitude: intersection_latitude,
            height: 0.0,
        })
    }
}
