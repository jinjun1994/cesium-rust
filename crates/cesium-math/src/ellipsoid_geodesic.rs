//! Ellipsoid geodesic matching CesiumJS `Core/EllipsoidGeodesic`.
//! Adapted from CesiumJS (Apache-2.0).

use crate::cartesian::Cartesian3;
use crate::core_math::EPSILON12;
use crate::ellipsoid::Ellipsoid;
use crate::errors::DeveloperError;
use crate::Cartographic;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct GeodesicConstants {
    pub a: f64,
    pub b: f64,
    pub f: f64,
    pub cosine_heading: f64,
    pub sine_heading: f64,
    pub tan_u: f64,
    pub cosine_u: f64,
    pub sine_u: f64,
    pub sigma: f64,
    pub sine_alpha: f64,
    pub sine_squared_alpha: f64,
    pub cosine_squared_alpha: f64,
    pub cosine_alpha: f64,
    pub u2_over_4: f64,
    pub u4_over_16: f64,
    pub u6_over_64: f64,
    pub u8_over_256: f64,
    pub a0: f64,
    pub a1: f64,
    pub a2: f64,
    pub a3: f64,
    pub distance_ratio: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct EllipsoidGeodesic {
    ellipsoid: Ellipsoid,
    start: Option<Cartographic>,
    end: Option<Cartographic>,
    constants: GeodesicConstants,
    start_heading: Option<f64>,
    end_heading: Option<f64>,
    distance: Option<f64>,
    u_squared: Option<f64>,
}

impl Default for EllipsoidGeodesic {
    fn default() -> Self {
        Self::new(None, None, None)
    }
}

fn compute_c(f: f64, cosine_squared_alpha: f64) -> f64 {
    (f * cosine_squared_alpha * (4.0 + f * (4.0 - 3.0 * cosine_squared_alpha))) / 16.0
}

fn compute_delta_lambda(
    f: f64,
    sine_alpha: f64,
    cosine_squared_alpha: f64,
    sigma: f64,
    sine_sigma: f64,
    cosine_sigma: f64,
    cosine_twice_sigma_midpoint: f64,
) -> f64 {
    let c = compute_c(f, cosine_squared_alpha);
    (1.0 - c)
        * f
        * sine_alpha
        * (sigma
            + c * sine_sigma
                * (cosine_twice_sigma_midpoint
                    + c * cosine_sigma
                        * (2.0 * cosine_twice_sigma_midpoint * cosine_twice_sigma_midpoint - 1.0)))
}

fn vincenty_inverse_formula(
    major: f64,
    minor: f64,
    first_longitude: f64,
    first_latitude: f64,
    second_longitude: f64,
    second_latitude: f64,
) -> (f64, f64, f64, f64) {
    let eff = (major - minor) / major;
    let l = second_longitude - first_longitude;

    let u1 = ((1.0 - eff) * first_latitude.tan()).atan();
    let u2 = ((1.0 - eff) * second_latitude.tan()).atan();

    let cosine_u1 = u1.cos();
    let sine_u1 = u1.sin();
    let cosine_u2 = u2.cos();
    let sine_u2 = u2.sin();

    let cc = cosine_u1 * cosine_u2;
    let cs = cosine_u1 * sine_u2;
    let ss = sine_u1 * sine_u2;
    let sc = sine_u1 * cosine_u2;

    let mut lambda = l;
    let mut lambda_dot;

    let mut cosine_lambda;
    let mut sine_lambda;
    let mut sigma;
    let mut cosine_sigma;
    let mut sine_sigma;
    let mut cosine_squared_alpha;
    let mut cosine_twice_sigma_midpoint;

    loop {
        cosine_lambda = lambda.cos();
        sine_lambda = lambda.sin();

        let temp = cs - sc * cosine_lambda;
        sine_sigma = (cosine_u2 * cosine_u2 * sine_lambda * sine_lambda + temp * temp).sqrt();
        cosine_sigma = ss + cc * cosine_lambda;

        sigma = sine_sigma.atan2(cosine_sigma);

        let sine_alpha = if sine_sigma == 0.0 {
            cosine_squared_alpha = 1.0;
            0.0
        } else {
            let sa = (cc * sine_lambda) / sine_sigma;
            cosine_squared_alpha = 1.0 - sa * sa;
            sa
        };

        lambda_dot = lambda;

        cosine_twice_sigma_midpoint = if cosine_squared_alpha != 0.0 {
            cosine_sigma - (2.0 * ss) / cosine_squared_alpha
        } else {
            0.0
        };

        if !cosine_twice_sigma_midpoint.is_finite() {
            cosine_twice_sigma_midpoint = 0.0;
        }

        lambda = l + compute_delta_lambda(
            eff,
            sine_alpha,
            cosine_squared_alpha,
            sigma,
            sine_sigma,
            cosine_sigma,
            cosine_twice_sigma_midpoint,
        );

        if (lambda - lambda_dot).abs() <= EPSILON12 {
            break;
        }
    }

    let u_squared = (cosine_squared_alpha * (major * major - minor * minor)) / (minor * minor);
    let a = 1.0
        + (u_squared * (4096.0 + u_squared * (u_squared * (320.0 - 175.0 * u_squared) - 768.0)))
            / 16384.0;

    let b_coeff = (u_squared
        * (256.0 + u_squared * (u_squared * (74.0 - 47.0 * u_squared) - 128.0)))
        / 1024.0;

    let cosine_squared_twice_sigma_midpoint =
        cosine_twice_sigma_midpoint * cosine_twice_sigma_midpoint;
    let delta_sigma = b_coeff
        * sine_sigma
        * (cosine_twice_sigma_midpoint
            + (b_coeff
                * (cosine_sigma * (2.0 * cosine_squared_twice_sigma_midpoint - 1.0)
                    - (b_coeff
                        * cosine_twice_sigma_midpoint
                        * (4.0 * sine_sigma * sine_sigma - 3.0)
                        * (4.0 * cosine_squared_twice_sigma_midpoint - 3.0))
                        / 6.0))
                / 4.0);

    let distance = minor * a * (sigma - delta_sigma);

    let start_heading = (cosine_u2 * sine_lambda).atan2(cs - sc * cosine_lambda);
    let end_heading = (cosine_u1 * sine_lambda).atan2(cs * cosine_lambda - sc);

    (distance, start_heading, end_heading, u_squared)
}

fn set_constants(
    u_squared: f64,
    ellipsoid: &Ellipsoid,
    start: &Cartographic,
    start_heading: f64,
) -> GeodesicConstants {
    let a = ellipsoid.maximum_radius();
    let b = ellipsoid.minimum_radius();
    let f = (a - b) / a;

    let cosine_heading = start_heading.cos();
    let sine_heading = start_heading.sin();

    let tan_u = (1.0 - f) * start.latitude.tan();
    let cosine_u = 1.0 / (1.0 + tan_u * tan_u).sqrt();
    let sine_u = cosine_u * tan_u;

    let sigma = tan_u.atan2(cosine_heading);

    let sine_alpha = cosine_u * sine_heading;
    let sine_squared_alpha = sine_alpha * sine_alpha;

    let cosine_squared_alpha = 1.0 - sine_squared_alpha;
    let cosine_alpha = cosine_squared_alpha.max(0.0).sqrt();

    let u2_over_4 = u_squared / 4.0;
    let u4_over_16 = u2_over_4 * u2_over_4;
    let u6_over_64 = u4_over_16 * u2_over_4;
    let u8_over_256 = u4_over_16 * u4_over_16;

    let a0 = 1.0 + u2_over_4 - (3.0 * u4_over_16) / 4.0 + (5.0 * u6_over_64) / 4.0
        - (175.0 * u8_over_256) / 64.0;
    let a1 = 1.0 - u2_over_4 + (15.0 * u4_over_16) / 8.0 - (35.0 * u6_over_64) / 8.0;
    let a2 = 1.0 - 3.0 * u2_over_4 + (35.0 * u4_over_16) / 4.0;
    let a3 = 1.0 - 5.0 * u2_over_4;

    let distance_ratio = a0 * sigma
        - (a1 * (2.0 * sigma).sin() * u2_over_4) / 2.0
        - (a2 * (4.0 * sigma).sin() * u4_over_16) / 16.0
        - (a3 * (6.0 * sigma).sin() * u6_over_64) / 48.0
        - ((8.0 * sigma).sin() * 5.0 * u8_over_256) / 512.0;

    GeodesicConstants {
        a,
        b,
        f,
        cosine_heading,
        sine_heading,
        tan_u,
        cosine_u,
        sine_u,
        sigma,
        sine_alpha,
        sine_squared_alpha,
        cosine_squared_alpha,
        cosine_alpha,
        u2_over_4,
        u4_over_16,
        u6_over_64,
        u8_over_256,
        a0,
        a1,
        a2,
        a3,
        distance_ratio,
    }
}

impl EllipsoidGeodesic {
    pub fn new(
        start: Option<Cartographic>,
        end: Option<Cartographic>,
        ellipsoid: Option<Ellipsoid>,
    ) -> Self {
        let e = ellipsoid.unwrap_or(Ellipsoid::WGS84);
        let mut geo = Self {
            ellipsoid: e,
            start: None,
            end: None,
            constants: GeodesicConstants::default(),
            start_heading: None,
            end_heading: None,
            distance: None,
            u_squared: None,
        };
        if let (Some(s), Some(nd)) = (start, end) {
            let _ = geo.set_end_points(s, nd);
        }
        geo
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

        let (dist, sh, eh, u2) = vincenty_inverse_formula(
            self.ellipsoid.maximum_radius(),
            self.ellipsoid.minimum_radius(),
            start.longitude,
            start.latitude,
            end.longitude,
            end.latitude,
        );

        let mut s_clean = start;
        s_clean.height = 0.0;
        let mut e_clean = end;
        e_clean.height = 0.0;

        self.start = Some(s_clean);
        self.end = Some(e_clean);
        self.distance = Some(dist);
        self.start_heading = Some(sh);
        self.end_heading = Some(eh);
        self.u_squared = Some(u2);
        self.constants = set_constants(u2, &self.ellipsoid, &s_clean, sh);

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

    pub fn surface_distance(&self) -> Result<f64, DeveloperError> {
        self.distance
            .ok_or_else(|| DeveloperError::new("start and end points must be defined."))
    }

    pub fn start_heading(&self) -> Result<f64, DeveloperError> {
        self.start_heading
            .ok_or_else(|| DeveloperError::new("start and end points must be defined."))
    }

    pub fn end_heading(&self) -> Result<f64, DeveloperError> {
        self.end_heading
            .ok_or_else(|| DeveloperError::new("start and end points must be defined."))
    }

    pub fn interpolate_using_fraction(&self, fraction: f64) -> Result<Cartographic, DeveloperError> {
        let dist = self.surface_distance()?;
        self.interpolate_using_surface_distance(dist * fraction)
    }

    pub fn interpolate_using_surface_distance(
        &self,
        distance: f64,
    ) -> Result<Cartographic, DeveloperError> {
        let _ = self.surface_distance()?;
        let start = self.start.unwrap();
        let constants = &self.constants;

        let s = constants.distance_ratio + distance / constants.b;

        let cosine_2s = (2.0 * s).cos();
        let cosine_4s = (4.0 * s).cos();
        let cosine_6s = (6.0 * s).cos();
        let sine_2s = (2.0 * s).sin();
        let sine_4s = (4.0 * s).sin();
        let sine_6s = (6.0 * s).sin();
        let sine_8s = (8.0 * s).sin();

        let s2 = s * s;
        let s3 = s * s2;

        let u8_over_256 = constants.u8_over_256;
        let u2_over_4 = constants.u2_over_4;
        let u6_over_64 = constants.u6_over_64;
        let u4_over_16 = constants.u4_over_16;

        let mut sigma = (2.0 * s3 * u8_over_256 * cosine_2s) / 3.0
            + s * (1.0 - u2_over_4 + (7.0 * u4_over_16) / 4.0 - (15.0 * u6_over_64) / 4.0
                + (579.0 * u8_over_256) / 64.0
                - (u4_over_16 - (15.0 * u6_over_64) / 4.0 + (187.0 * u8_over_256) / 16.0)
                    * cosine_2s
                - ((5.0 * u6_over_64) / 4.0 - (115.0 * u8_over_256) / 16.0) * cosine_4s
                - (29.0 * u8_over_256 * cosine_6s) / 16.0)
            + (u2_over_4 / 2.0 - u4_over_16 + (71.0 * u6_over_64) / 32.0
                - (85.0 * u8_over_256) / 16.0)
                * sine_2s
            + ((5.0 * u4_over_16) / 16.0 - (5.0 * u6_over_64) / 4.0 + (383.0 * u8_over_256) / 96.0)
                * sine_4s
            - s2 * ((u6_over_64 - (11.0 * u8_over_256) / 2.0) * sine_2s
                + (5.0 * u8_over_256 * sine_4s) / 2.0)
            + ((29.0 * u6_over_64) / 96.0 - (29.0 * u8_over_256) / 16.0) * sine_6s
            + (539.0 * u8_over_256 * sine_8s) / 1536.0;

        let theta = (sigma.sin() * constants.cosine_alpha).asin();
        let latitude = ((constants.a / constants.b) * theta.tan()).atan();

        sigma -= constants.sigma;
        let cosine_twice_sigma_midpoint = (2.0 * constants.sigma + sigma).cos();

        let sine_sigma = sigma.sin();
        let cosine_sigma = sigma.cos();

        let cc = constants.cosine_u * cosine_sigma;
        let ss = constants.sine_u * sine_sigma;

        let lambda = (sine_sigma * constants.sine_heading)
            .atan2(cc - ss * constants.cosine_heading);

        let l = lambda
            - compute_delta_lambda(
                constants.f,
                constants.sine_alpha,
                constants.cosine_squared_alpha,
                sigma,
                sine_sigma,
                cosine_sigma,
                cosine_twice_sigma_midpoint,
            );

        Ok(Cartographic {
            longitude: start.longitude + l,
            latitude,
            height: 0.0,
        })
    }
}
