//! Triaxial ellipsoid operations adapted from CesiumJS Ellipsoid (Apache-2.0).
use crate::errors::DeveloperError;
use crate::{Cartographic, MathError, WGS84_A, WGS84_B};
use glam::{DMat4, DVec3};

// Ten-point Gauss-Legendre quadrature, as used by CesiumJS Ellipsoid.surfaceArea.
fn gauss_legendre(a: f64, b: f64, f: impl Fn(f64) -> f64) -> f64 {
    const X: [f64; 5] = [
        0.14887433898163,
        0.43339539412925,
        0.67940956829902,
        0.86506336668898,
        0.97390652851717,
    ];
    const W: [f64; 5] = [
        0.29552422471475,
        0.26926671930999,
        0.21908636251598,
        0.14945134915058,
        0.066671344308684,
    ];
    let mid = (a + b) * 0.5;
    let half = (b - a) * 0.5;
    let mut sum = 0.0;
    for i in 0..5 {
        let dx = half * X[i];
        sum += W[i] * (f(mid + dx) + f(mid - dx));
    }
    sum * half
}

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Ellipsoid {
    radii: DVec3,
}

impl Ellipsoid {
    pub const WGS84: Self = Self {
        radii: DVec3::new(WGS84_A, WGS84_A, WGS84_B),
    };
    pub const UNIT_SPHERE: Self = Self { radii: DVec3::ONE };
    pub const MOON: Self = Self {
        radii: DVec3::splat(1_737_400.0),
    };
    pub const MARS: Self = Self {
        radii: DVec3::new(3_396_190.0, 3_396_190.0, 3_376_200.0),
    };

    /// Cesium permits zero radii; operations requiring an inverse may return None.
    pub fn new(radii: DVec3) -> Result<Self, MathError> {
        if !radii.is_finite() || radii.min_element() < 0.0 {
            return Err(MathError);
        }
        Ok(Self { radii })
    }

    pub fn from_cartesian3(cartesian: Option<DVec3>) -> Result<Self, MathError> {
        match cartesian {
            Some(radii) => Self::new(radii),
            None => Ok(Self::default()),
        }
    }

    pub fn equals(&self, other: Option<&Self>) -> bool {
        match other {
            Some(o) => self.radii == o.radii,
            None => false,
        }
    }

    pub fn geocentric_surface_normal(cartesian: DVec3) -> DVec3 {
        cartesian.normalize()
    }
    pub fn radii(self) -> DVec3 {
        self.radii
    }
    pub fn radii_squared(self) -> DVec3 {
        self.radii * self.radii
    }
    pub fn radii_to_fourth(self) -> DVec3 {
        self.radii_squared() * self.radii_squared()
    }
    pub fn one_over_radii(self) -> DVec3 {
        self.radii.map(|x| if x == 0.0 { 0.0 } else { 1.0 / x })
    }
    pub fn one_over_radii_squared(self) -> DVec3 {
        self.radii_squared()
            .map(|x| if x == 0.0 { 0.0 } else { 1.0 / x })
    }
    pub fn minimum_radius(self) -> f64 {
        self.radii.min_element()
    }
    pub fn maximum_radius(self) -> f64 {
        self.radii.max_element()
    }
    pub fn to_scaled_space(self, p: DVec3) -> DVec3 {
        p * self.one_over_radii()
    }
    pub fn from_scaled_space(self, p: DVec3) -> DVec3 {
        p * self.radii
    }
    pub fn transform_position_to_scaled_space(self, p: DVec3) -> DVec3 {
        self.to_scaled_space(p)
    }
    pub fn transform_position_from_scaled_space(self, p: DVec3) -> DVec3 {
        self.from_scaled_space(p)
    }

    /// East, north and up basis at an ECEF origin, with Cesium's pole and
    /// origin conventions. Columns describe the local-to-fixed transform.
    /// For unequal X/Y radii Cesium derives east from the origin rather than
    /// the geodetic normal, so the resulting axes need not be orthogonal.
    pub fn east_north_up_to_fixed_frame(self, origin: DVec3) -> Option<DMat4> {
        if !origin.is_finite() {
            return None;
        }
        let (east, north, up) = if origin.abs().max_element() <= 1e-14 {
            (DVec3::Y, -DVec3::X, DVec3::Z)
        } else if origin.x.abs() <= 1e-14 && origin.y.abs() <= 1e-14 {
            let sign = origin.z.signum();
            (DVec3::Y, -sign * DVec3::X, sign * DVec3::Z)
        } else {
            let up = self.geodetic_surface_normal(origin)?;
            let east = DVec3::new(-origin.y, origin.x, 0.0).try_normalize()?;
            (east, up.cross(east), up)
        };
        Some(DMat4::from_cols(
            east.extend(0.0),
            north.extend(0.0),
            up.extend(0.0),
            origin.extend(1.0),
        ))
    }

    pub fn geodetic_surface_normal_cartographic(c: Cartographic) -> DVec3 {
        let (sin_lat, cos_lat) = c.latitude.sin_cos();
        let (sin_lon, cos_lon) = c.longitude.sin_cos();
        let n = DVec3::new(cos_lat * cos_lon, cos_lat * sin_lon, sin_lat);
        n / n.length()
    }
    pub fn geodetic_surface_normal(self, position: DVec3) -> Option<DVec3> {
        if !position.is_finite() || position.abs().max_element() <= 1e-14 {
            return None;
        }
        let n = position * self.one_over_radii_squared();
        let length = n.length();
        if length == 0.0 || !length.is_finite() {
            return None;
        }
        Some(n / length)
    }
    pub fn cartographic_to_cartesian(self, c: Cartographic) -> Option<DVec3> {
        if !c.longitude.is_finite() || !c.latitude.is_finite() || !c.height.is_finite() {
            return None;
        }
        let n = Self::geodetic_surface_normal_cartographic(c);
        let k = self.radii_squared() * n;
        let gamma = n.dot(k).sqrt();
        if gamma == 0.0 || !gamma.is_finite() {
            return None;
        }
        Some(k / gamma + n * c.height)
    }
    pub fn scale_to_geocentric_surface(self, position: DVec3) -> Option<DVec3> {
        if !position.is_finite() {
            return None;
        }
        let scaled = position * self.one_over_radii();
        let norm = scaled.length();
        if norm == 0.0 || !norm.is_finite() {
            return None;
        }
        Some(position / norm)
    }
    /// Iterative geodetic projection. The center has no unique projection.
    pub fn scale_to_geodetic_surface(self, position: DVec3) -> Option<DVec3> {
        if !position.is_finite() || self.minimum_radius() == 0.0 {
            return None;
        }
        let inv = self.one_over_radii();
        let inv2 = self.one_over_radii_squared();
        let squared = position * position * inv * inv;
        let norm = squared.element_sum();
        let ratio = (1.0 / norm).sqrt();
        if !ratio.is_finite() {
            return None;
        }
        let intersection = position * ratio;
        if norm < 0.1 {
            return Some(intersection);
        }
        let gradient = intersection * inv2 * 2.0;
        let mut lambda = (1.0 - ratio) * position.length() / (0.5 * gradient.length());
        for _ in 0..100 {
            let multiplier = DVec3::ONE / (DVec3::ONE + lambda * inv2);
            let m2 = multiplier * multiplier;
            let func = (squared * m2).element_sum() - 1.0;
            if !func.is_finite() {
                return None;
            }
            if func.abs() <= 1e-12 {
                return Some(position * multiplier);
            }
            let derivative = -2.0 * (squared * m2 * multiplier * inv2).element_sum();
            if derivative == 0.0 || !derivative.is_finite() {
                return None;
            }
            lambda -= func / derivative;
        }
        None
    }
    pub fn cartesian_to_cartographic(self, position: DVec3) -> Option<Cartographic> {
        let surface = self.scale_to_geodetic_surface(position)?;
        let normal = self.geodetic_surface_normal(surface)?;
        let offset = position - surface;
        let height = if offset.dot(position) < 0.0 {
            -offset.length()
        } else {
            offset.length()
        };
        Cartographic::radians(
            normal.y.atan2(normal.x),
            normal.z.clamp(-1.0, 1.0).asin(),
            height,
        )
        .ok()
    }

    /// Intersection of the surface normal with the rotation axis. Invalid
    /// ellipsoids return `Err`; outside the buffered polar radius returns `None`.
    pub fn surface_normal_intersection_with_z_axis(
        self,
        position: DVec3,
        buffer: f64,
    ) -> Result<Option<DVec3>, MathError> {
        let r = self.radii;
        if !position.is_finite()
            || !buffer.is_finite()
            || r.z <= 0.0
            || (r.x - r.y).abs() > 1e-15_f64.max(1e-15 * r.x.abs().max(r.y.abs()))
        {
            return Err(MathError);
        }
        let z = position.z * (1.0 - r.x * r.x / (r.z * r.z));
        if !z.is_finite() {
            return Err(MathError);
        }
        Ok((z.abs() < r.z - buffer).then_some(DVec3::new(0.0, 0.0, z)))
    }

    /// East and north principal curvatures on an ellipsoid of revolution.
    pub fn local_curvature(self, surface_position: DVec3) -> Option<(f64, f64)> {
        let endpoint = self
            .surface_normal_intersection_with_z_axis(surface_position, 0.0)
            .ok()??;
        let prime_vertical = surface_position.distance(endpoint);
        let ratio = self.minimum_radius() * prime_vertical / self.maximum_radius().powi(2);
        let meridional = prime_vertical * ratio * ratio;
        if prime_vertical == 0.0 || meridional == 0.0 || !meridional.is_finite() {
            return None;
        }
        Some((1.0 / prime_vertical, 1.0 / meridional))
    }

    /// Cesium-compatible approximate area over a longitude/latitude rectangle,
    /// in radians. An east longitude less than west crosses the antimeridian.
    /// Longitude must be within [-π, π] and latitude within [-π/2, π/2].
    pub fn surface_area(
        self,
        west: f64,
        south: f64,
        east: f64,
        north: f64,
    ) -> Result<f64, MathError> {
        use std::f64::consts::{FRAC_PI_2, PI, TAU};
        if ![west, south, east, north].iter().all(|x| x.is_finite())
            || !(-PI..=PI).contains(&west)
            || !(-PI..=PI).contains(&east)
            || !(-FRAC_PI_2..=FRAC_PI_2).contains(&south)
            || !(-FRAC_PI_2..=FRAC_PI_2).contains(&north)
            || south > north
        {
            return Err(MathError);
        }
        let east = if east < west { east + TAU } else { east };
        let squared = self.radii_squared();
        let (a2, b2, c2) = (squared.x, squared.y, squared.z);
        let area = gauss_legendre(south, north, |lat| {
            let (sin_lat, cos_lat) = lat.sin_cos();
            cos_lat
                * gauss_legendre(west, east, |lon| {
                    let (sin_lon, cos_lon) = lon.sin_cos();
                    (a2 * b2 * sin_lat * sin_lat
                        + c2 * (b2 * cos_lon * cos_lon + a2 * sin_lon * sin_lon)
                            * cos_lat
                            * cos_lat)
                        .sqrt()
                })
        });
        if area.is_finite() {
            Ok(area)
        } else {
            Err(MathError)
        }
    }

    /// Equivalent to `surface_area` using a geographic rectangle.
    pub fn surface_area_rectangle(self, rectangle: crate::GeoRectangle) -> Result<f64, MathError> {
        self.surface_area(
            rectangle.west,
            rectangle.south,
            rectangle.east,
            rectangle.north,
        )
    }

    pub fn squared_x_over_squared_z(self) -> f64 {
        let r2 = self.radii_squared();
        r2.x / r2.z
    }

    pub fn cartographic_array_to_cartesian_array(
        &self,
        cartographics: &[Cartographic],
    ) -> Vec<Option<DVec3>> {
        cartographics
            .iter()
            .map(|&c| self.cartographic_to_cartesian(c))
            .collect()
    }

    pub fn cartesian_array_to_cartographic_array(
        &self,
        cartesians: &[DVec3],
    ) -> Vec<Option<Cartographic>> {
        cartesians
            .iter()
            .map(|&p| self.cartesian_to_cartographic(p))
            .collect()
    }

    pub fn get_surface_normal_intersection_with_z_axis(
        self,
        position: DVec3,
        buffer: Option<f64>,
    ) -> Result<Option<DVec3>, MathError> {
        self.surface_normal_intersection_with_z_axis(position, buffer.unwrap_or(0.0))
    }

    pub fn get_local_curvature(self, surface_position: DVec3) -> Option<(f64, f64)> {
        self.local_curvature(surface_position)
    }

    pub const PACKED_LENGTH: usize = 3;

    pub fn pack(&self, array: &mut [f64], starting_index: usize) -> Result<(), DeveloperError> {
        if array.len() < starting_index + Self::PACKED_LENGTH {
            return Err(DeveloperError::new("Array is too short"));
        }
        array[starting_index] = self.radii.x;
        array[starting_index + 1] = self.radii.y;
        array[starting_index + 2] = self.radii.z;
        Ok(())
    }

    pub fn unpack(array: &[f64], starting_index: usize) -> Result<Self, DeveloperError> {
        if array.len() < starting_index + Self::PACKED_LENGTH {
            return Err(DeveloperError::new("Array is too short"));
        }
        let radii = DVec3::new(
            array[starting_index],
            array[starting_index + 1],
            array[starting_index + 2],
        );
        Self::new(radii).map_err(|_| DeveloperError::new("Failed to unpack Ellipsoid"))
    }
}

impl std::fmt::Display for Ellipsoid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {}, {})", self.radii.x, self.radii.y, self.radii.z)
    }
}
