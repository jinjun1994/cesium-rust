//! Horizon occlusion in ellipsoid-scaled space, adapted from CesiumJS (Apache-2.0).
use crate::{Ellipsoid, GeoRectangle, MathError, Sphere};
use glam::DVec3;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EllipsoidalOccluder {
    ellipsoid: Ellipsoid,
    camera: DVec3,
    camera_scaled: DVec3,
    limb_squared: f64,
}

impl EllipsoidalOccluder {
    pub fn from_ellipsoid(ellipsoid: Ellipsoid) -> Result<Self, MathError> {
        Self::new(ellipsoid, DVec3::ZERO)
    }

    pub fn new(ellipsoid: Ellipsoid, camera: DVec3) -> Result<Self, MathError> {
        if ellipsoid.minimum_radius() <= 0.0 || !camera.is_finite() {
            return Err(MathError);
        }
        let camera_scaled = ellipsoid.to_scaled_space(camera);
        let limb_squared = camera_scaled.length_squared() - 1.0;
        if !limb_squared.is_finite() {
            return Err(MathError);
        }
        Ok(Self {
            ellipsoid,
            camera,
            camera_scaled,
            limb_squared,
        })
    }

    pub fn ellipsoid(self) -> Ellipsoid {
        self.ellipsoid
    }
    pub fn camera_position(self) -> DVec3 {
        self.camera
    }
    pub fn set_camera_position(&mut self, camera: DVec3) -> Result<(), MathError> {
        *self = Self::new(self.ellipsoid, camera)?;
        Ok(())
    }

    fn visible(point: DVec3, camera: DVec3, limb_squared: f64) -> bool {
        let vt = point - camera;
        let dot = -vt.dot(camera);
        let occluded = if limb_squared < 0.0 {
            dot > 0.0
        } else {
            dot > limb_squared && dot * dot / vt.length_squared() > limb_squared
        };
        !occluded
    }

    pub fn is_point_visible(self, point: DVec3) -> bool {
        self.is_scaled_space_point_visible(self.ellipsoid.to_scaled_space(point))
    }
    pub fn is_scaled_space_point_visible(self, point: DVec3) -> bool {
        Self::visible(point, self.camera_scaled, self.limb_squared)
    }
    fn shrunk_ellipsoid(self, minimum_height: f64) -> Ellipsoid {
        if minimum_height.is_finite()
            && minimum_height < 0.0
            && self.ellipsoid.minimum_radius() > -minimum_height
        {
            Ellipsoid::new(self.ellipsoid.radii() + DVec3::splat(minimum_height))
                .expect("positive finite radii")
        } else {
            self.ellipsoid
        }
    }
    pub fn is_scaled_space_point_visible_under_ellipsoid(
        self,
        point: DVec3,
        minimum_height: f64,
    ) -> bool {
        let camera = self
            .shrunk_ellipsoid(minimum_height)
            .to_scaled_space(self.camera);
        Self::visible(point, camera, camera.length_squared() - 1.0)
    }

    fn culling_point(
        ellipsoid: Ellipsoid,
        direction: DVec3,
        points: impl IntoIterator<Item = DVec3>,
    ) -> Option<DVec3> {
        let direction = ellipsoid.to_scaled_space(direction).try_normalize()?;
        let mut max_magnitude: f64 = 0.0;
        for point in points {
            let scaled = ellipsoid.to_scaled_space(point);
            let magnitude_squared = scaled.length_squared();
            let unit = scaled.try_normalize()?;
            let m2 = magnitude_squared.max(1.0);
            let m = magnitude_squared.sqrt().max(1.0);
            let cos_alpha = unit.dot(direction);
            let sin_alpha = unit.cross(direction).length();
            let candidate = 1.0 / (cos_alpha / m - sin_alpha * (m2 - 1.0).sqrt() / m);
            if !candidate.is_finite() || candidate < 0.0 {
                return None;
            }
            max_magnitude = max_magnitude.max(candidate);
        }
        (max_magnitude > 0.0).then_some(direction * max_magnitude)
    }

    pub fn compute_horizon_culling_point(
        self,
        direction: DVec3,
        points: &[DVec3],
    ) -> Option<DVec3> {
        Self::culling_point(self.ellipsoid, direction, points.iter().copied())
    }
    pub fn compute_horizon_culling_point_under_ellipsoid(
        self,
        direction: DVec3,
        points: &[DVec3],
        minimum_height: f64,
    ) -> Option<DVec3> {
        Self::culling_point(
            self.shrunk_ellipsoid(minimum_height),
            direction,
            points.iter().copied(),
        )
    }
    /// Vertex records may have additional attributes after xyz.
    pub fn compute_horizon_culling_point_from_vertices(
        self,
        direction: DVec3,
        vertices: &[f64],
        stride: usize,
        center: DVec3,
        minimum_height: f64,
    ) -> Option<DVec3> {
        if stride < 3 || !center.is_finite() {
            return None;
        }
        if vertices.chunks(stride).any(|chunk| chunk.len() < 3) {
            return None;
        }
        let positions = vertices
            .chunks(stride)
            .map(|chunk| DVec3::new(chunk[0], chunk[1], chunk[2]) + center);
        Self::culling_point(self.shrunk_ellipsoid(minimum_height), direction, positions)
    }
    pub fn compute_horizon_culling_point_from_rectangle(
        self,
        rectangle: GeoRectangle,
        ellipsoid: Ellipsoid,
    ) -> Option<DVec3> {
        let points = rectangle.subsample(ellipsoid, 0.0)?;
        let center = Sphere::from_points(&points).ok()?.center;
        if center.length() < 0.1 * ellipsoid.minimum_radius() {
            return None;
        }
        self.compute_horizon_culling_point(center, &points)
    }

    pub fn equals(&self, other: Option<&Self>) -> bool {
        match other {
            Some(o) => self.ellipsoid == o.ellipsoid && self.camera == o.camera,
            None => false,
        }
    }
}
