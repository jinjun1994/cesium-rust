//! Rays adapted from CesiumJS (Apache-2.0).
use crate::{
    ray_axis_aligned_box, ray_ellipsoid_interval, ray_sphere, ray_triangle,
    ray_triangle_parametric, AxisAlignedBoundingBox, Ellipsoid, MathError, Plane, RayInterval,
    Sphere,
};
use glam::DVec3;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Ray {
    pub origin: DVec3,
    pub direction: DVec3,
}

impl Ray {
    /// Normalizes a nonzero direction; the default zero direction is retained.
    pub fn new(origin: DVec3, direction: DVec3) -> Result<Self, MathError> {
        if !origin.is_finite() || !direction.is_finite() {
            return Err(MathError);
        }
        let direction = if direction == DVec3::ZERO {
            direction
        } else {
            direction.try_normalize().ok_or(MathError)?
        };
        Ok(Self { origin, direction })
    }

    pub fn point_at(self, t: f64) -> DVec3 {
        self.origin + self.direction * t
    }

    pub fn get_point(&self, t: f64) -> DVec3 {
        self.point_at(t)
    }

    pub fn equals(&self, other: Option<&Self>) -> bool {
        match other {
            Some(o) => self.origin == o.origin && self.direction == o.direction,
            None => false,
        }
    }

    pub fn intersect_plane(self, plane: Plane) -> Option<DVec3> {
        plane.intersect_ray(self.origin, self.direction)
    }
    pub fn intersect_sphere(self, sphere: Sphere) -> Option<RayInterval> {
        ray_sphere(self.origin, self.direction, sphere)
    }
    pub fn intersect_ellipsoid(self, ellipsoid: Ellipsoid) -> Option<RayInterval> {
        ray_ellipsoid_interval(self.origin, self.direction, ellipsoid.radii())
    }
    /// As with Cesium, this reports intersections on the supporting line,
    /// which can include negative parameters behind the origin.
    pub fn intersect_box(self, bounds: AxisAlignedBoundingBox) -> Option<RayInterval> {
        ray_axis_aligned_box(self.origin, self.direction, bounds.minimum, bounds.maximum)
    }
    pub fn intersect_triangle(self, vertices: [DVec3; 3], cull_back_faces: bool) -> Option<DVec3> {
        ray_triangle(self.origin, self.direction, vertices, cull_back_faces)
    }
    pub fn triangle_parameter(self, vertices: [DVec3; 3], cull_back_faces: bool) -> Option<f64> {
        ray_triangle_parametric(self.origin, self.direction, vertices, cull_back_faces)
    }
}
