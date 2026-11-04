//! Spherical horizon occlusion adapted from CesiumJS (Apache-2.0).
use crate::{Ellipsoid, GeoRectangle, MathError, Sphere};
use glam::DVec3;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Visibility {
    None,
    Partial,
    Full,
}

#[derive(Clone, Copy, Debug)]
pub struct Occluder {
    sphere: Sphere,
    camera: DVec3,
    horizon: Option<f64>,
    plane_normal: DVec3,
    plane_position: DVec3,
}

impl Occluder {
    /// Computes a conservative occludee point for fixed positions surrounding a sphere.
    /// Invalid required inputs are errors; geometrically unrepresentable sets return None.
    pub fn compute_occludee_point(
        sphere: Sphere,
        position: DVec3,
        points: &[DVec3],
    ) -> Result<Option<DVec3>, MathError> {
        if !sphere.center.is_finite()
            || !sphere.radius.is_finite()
            || sphere.radius <= 0.0
            || !position.is_finite()
            || points.is_empty()
            || points.iter().any(|p| !p.is_finite())
        {
            return Err(MathError);
        }
        let normal = (position - sphere.center)
            .try_normalize()
            .ok_or(MathError)?;
        let abs = normal.abs();
        let major = if abs.x > abs.y {
            if abs.z > abs.x {
                2
            } else {
                0
            }
        } else if abs.z > abs.y {
            2
        } else {
            1
        };
        let mut in_plane = DVec3::ONE;
        in_plane[major] = -(normal.dot(in_plane) - normal[major]) / normal[major];
        let fallback = in_plane.try_normalize().ok_or(MathError)?;
        let mut minimum_dot = f64::INFINITY;
        for point in points {
            let delta = *point - sphere.center;
            let d2 = delta.length_squared();
            let radius2 = sphere.radius * sphere.radius;
            if d2 < radius2 || !d2.is_finite() || d2 == 0.0 {
                return Ok(None);
            }
            let distance = d2.sqrt();
            let point_dir = delta / distance;
            let cross = normal.cross(point_dir);
            let axis = if normal.dot(point_dir) < 0.999_999_984_769_129 && cross.length() > 1e-13 {
                cross.normalize()
            } else {
                fallback
            };
            let toward = -point_dir;
            let horizon2 = d2 - radius2;
            let plane_distance = horizon2 / distance;
            let horizon_center = *point + toward * plane_distance;
            let cross_distance = (horizon2 - plane_distance * plane_distance).max(0.0).sqrt();
            let rotated = (axis.cross(toward) + axis * axis.dot(toward)).try_normalize();
            let Some(rotated) = rotated else {
                return Ok(None);
            };
            let offset = rotated * cross_distance;
            let Some(a) = (horizon_center + offset - sphere.center).try_normalize() else {
                return Ok(None);
            };
            let Some(b) = (horizon_center - offset - sphere.center).try_normalize() else {
                return Ok(None);
            };
            minimum_dot = minimum_dot.min(normal.dot(a).min(normal.dot(b)));
        }
        if !minimum_dot.is_finite() || minimum_dot < 0.001_745_328_365_898_308_8 {
            return Ok(None);
        }
        let point = sphere.center + normal * (sphere.radius / minimum_dot);
        Ok(point.is_finite().then_some(point))
    }

    pub fn compute_occludee_point_from_rectangle(
        rectangle: GeoRectangle,
        ellipsoid: Ellipsoid,
    ) -> Option<DVec3> {
        let points = rectangle.subsample(ellipsoid, 0.0)?;
        let center = Sphere::from_points(&points).ok()?.center;
        if center == DVec3::ZERO {
            return None;
        }
        Self::compute_occludee_point(
            Sphere {
                center: DVec3::ZERO,
                radius: ellipsoid.minimum_radius(),
            },
            center,
            &points,
        )
        .ok()?
    }

    pub fn new(sphere: Sphere, camera: DVec3) -> Result<Self, MathError> {
        if !sphere.center.is_finite()
            || !sphere.radius.is_finite()
            || sphere.radius < 0.0
            || !camera.is_finite()
        {
            return Err(MathError);
        }
        let v = sphere.center - camera;
        let d2 = v.length_squared();
        let r2 = sphere.radius * sphere.radius;
        if !d2.is_finite() || !r2.is_finite() {
            return Err(MathError);
        }
        let (horizon, plane_normal, plane_position) = if d2 > r2 {
            let horizon = (d2 - r2).sqrt();
            let inv_distance = 1.0 / d2.sqrt();
            let normal = v * inv_distance;
            (
                Some(horizon),
                normal,
                camera + normal * (horizon * horizon * inv_distance),
            )
        } else {
            (None, DVec3::ZERO, DVec3::ZERO)
        };
        Ok(Self {
            sphere,
            camera,
            horizon,
            plane_normal,
            plane_position,
        })
    }
    pub fn position(self) -> DVec3 {
        self.sphere.center
    }
    pub fn radius(self) -> f64 {
        self.sphere.radius
    }
    pub fn camera_position(self) -> DVec3 {
        self.camera
    }
    pub fn set_camera_position(&mut self, camera: DVec3) -> Result<(), MathError> {
        *self = Self::new(self.sphere, camera)?;
        Ok(())
    }

    pub fn is_point_visible(self, point: DVec3) -> bool {
        let Some(horizon) = self.horizon else {
            return false;
        };
        let d =
            point.distance_squared(self.sphere.center) - self.sphere.radius * self.sphere.radius;
        d > 0.0 && (d.sqrt() + horizon).powi(2) > point.distance_squared(self.camera)
    }

    pub fn is_sphere_visible(self, occludee: Sphere) -> bool {
        let Some(horizon) = self.horizon else {
            return false;
        };
        let r = occludee.radius;
        let radius = self.sphere.radius;
        let d = occludee.center.distance_squared(self.sphere.center) - (radius - r).powi(2);
        if r < radius {
            return d > 0.0
                && (d.sqrt() + horizon).powi(2) + r * r
                    > occludee.center.distance_squared(self.camera);
        }
        if d <= 0.0 {
            return true;
        }
        let camera_distance = occludee.center.distance_squared(self.camera);
        let radius_squared = radius * radius;
        (horizon * horizon + radius_squared) * r * r > camera_distance * radius_squared
            || (d.sqrt() + horizon).powi(2) + r * r > camera_distance
    }

    pub fn compute_visibility(self, occludee: Sphere) -> Visibility {
        let radius = self.sphere.radius;
        let r = occludee.radius;
        if r > radius {
            return Visibility::Full;
        }
        let Some(horizon) = self.horizon else {
            return Visibility::None;
        };
        let distance = occludee.center.distance_squared(self.sphere.center);
        let d = distance - (radius - r).powi(2);
        if d <= 0.0 {
            return Visibility::None;
        }
        let camera_distance = occludee.center.distance_squared(self.camera);
        if (d.sqrt() + horizon).powi(2) + r * r < camera_distance {
            return Visibility::None;
        }
        let d = distance - (radius + r).powi(2);
        if d > 0.0 {
            return if camera_distance < (d.sqrt() + horizon).powi(2) + r * r {
                Visibility::Full
            } else {
                Visibility::Partial
            };
        }
        if (occludee.center - self.plane_position).dot(self.plane_normal) > -r {
            Visibility::Partial
        } else {
            Visibility::Full
        }
    }
}
