//! Hessian normal-form planes adapted from CesiumJS (Apache-2.0).
use crate::{ray_plane, AxisAlignedBoundingBox, MathError, PlaneIntersection, Sphere};
use glam::{DMat4, DVec3, DVec4};

/// A plane `normal.dot(point) + distance = 0` with unit normal.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Plane {
    pub normal: DVec3,
    pub distance: f64,
}

impl Plane {
    pub const ORIGIN_XY: Self = Self {
        normal: DVec3::Z,
        distance: 0.0,
    };
    pub const ORIGIN_YZ: Self = Self {
        normal: DVec3::X,
        distance: 0.0,
    };
    pub const ORIGIN_ZX: Self = Self {
        normal: DVec3::Y,
        distance: 0.0,
    };

    /// Requires a finite normal normalized within Cesium's 1e-6 tolerance.
    pub fn new(normal: DVec3, distance: f64) -> Result<Self, MathError> {
        let length = normal.length();
        if !normal.is_finite()
            || !distance.is_finite()
            || (length - 1.0).abs() > 1e-6 * length.max(1.0)
        {
            return Err(MathError);
        }
        Ok(Self { normal, distance })
    }

    pub fn from_point_normal(point: DVec3, normal: DVec3) -> Result<Self, MathError> {
        Self::new(normal, -normal.dot(point))
    }

    pub fn from_coefficients(coefficients: DVec4) -> Result<Self, MathError> {
        Self::new(coefficients.truncate(), coefficients.w)
    }

    pub fn point_distance(self, point: DVec3) -> f64 {
        self.normal.dot(point) + self.distance
    }

    pub fn project_point(self, point: DVec3) -> DVec3 {
        point - self.normal * self.point_distance(point)
    }

    /// Transform the plane by the inverse transpose and restore unit normal.
    pub fn transform(self, transform: DMat4) -> Result<Self, MathError> {
        let determinant = transform.determinant();
        if !transform.is_finite() || !determinant.is_finite() || determinant == 0.0 {
            return Err(MathError);
        }
        let coefficients = transform.inverse().transpose() * self.normal.extend(self.distance);
        let length = coefficients.truncate().length();
        if !length.is_finite() || length == 0.0 {
            return Err(MathError);
        }
        Self::from_coefficients(coefficients / length)
    }

    pub fn intersect_ray(self, origin: DVec3, direction: DVec3) -> Option<DVec3> {
        ray_plane(origin, direction, self.normal, self.distance)
    }

    pub fn classify_box(self, bounds: AxisAlignedBoundingBox) -> PlaneIntersection {
        bounds.intersect_plane(self.normal, self.distance)
    }

    pub fn classify_sphere(self, sphere: Sphere) -> PlaneIntersection {
        sphere.intersect_plane(self.normal, self.distance)
    }
}
