//! Ellipsoid tangent plane projections adapted from CesiumJS (Apache-2.0).
use crate::{AxisAlignedBoundingBox, Ellipsoid, MathError, Plane};
use glam::{DVec2, DVec3};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EllipsoidTangentPlane {
    pub ellipsoid: Ellipsoid,
    pub origin: DVec3,
    pub plane: Plane,
    pub x_axis: DVec3,
    pub y_axis: DVec3,
}

impl EllipsoidTangentPlane {
    pub fn from_origin(origin: DVec3) -> Result<Self, MathError> {
        Self::new(origin, Ellipsoid::WGS84)
    }

    pub fn new(origin: DVec3, ellipsoid: Ellipsoid) -> Result<Self, MathError> {
        let origin = ellipsoid
            .scale_to_geodetic_surface(origin)
            .ok_or(MathError)?;
        let frame = ellipsoid
            .east_north_up_to_fixed_frame(origin)
            .ok_or(MathError)?;
        let plane = Plane::from_point_normal(origin, frame.z_axis.truncate())?;
        Ok(Self {
            ellipsoid,
            origin,
            plane,
            x_axis: frame.x_axis.truncate(),
            y_axis: frame.y_axis.truncate(),
        })
    }

    pub fn from_points(points: &[DVec3], ellipsoid: Ellipsoid) -> Result<Self, MathError> {
        Self::new(
            AxisAlignedBoundingBox::from_points(points)?.center,
            ellipsoid,
        )
    }

    pub fn z_axis(self) -> DVec3 {
        self.plane.normal
    }

    fn coordinates(self, position: DVec3) -> DVec2 {
        let offset = position - self.origin;
        DVec2::new(self.x_axis.dot(offset), self.y_axis.dot(offset))
    }

    /// Radial projection onto the infinite plane; parallel rays return None.
    pub fn project_point_onto_plane(self, position: DVec3) -> Option<DVec2> {
        let direction = position.try_normalize()?;
        let intersection = self
            .plane
            .intersect_ray(position, direction)
            .or_else(|| self.plane.intersect_ray(position, -direction))?;
        Some(self.coordinates(intersection))
    }

    /// Unprojectable inputs are omitted, matching Cesium's batch operation.
    pub fn project_points_onto_plane(self, points: &[DVec3]) -> Vec<DVec2> {
        points
            .iter()
            .filter_map(|&point| self.project_point_onto_plane(point))
            .collect()
    }

    pub fn project_point_to_nearest_on_plane(self, position: DVec3) -> Option<DVec2> {
        if !position.is_finite() {
            return None;
        }
        let result = self.coordinates(self.plane.project_point(position));
        result.is_finite().then_some(result)
    }

    pub fn project_points_to_nearest_on_plane(self, points: &[DVec3]) -> Option<Vec<DVec2>> {
        points
            .iter()
            .map(|&point| self.project_point_to_nearest_on_plane(point))
            .collect()
    }

    pub fn project_point_onto_ellipsoid(self, point: DVec2) -> Option<DVec3> {
        if !point.is_finite() {
            return None;
        }
        self.ellipsoid.scale_to_geocentric_surface(
            self.origin + self.x_axis * point.x + self.y_axis * point.y,
        )
    }

    pub fn project_points_onto_ellipsoid(self, points: &[DVec2]) -> Option<Vec<DVec3>> {
        points
            .iter()
            .map(|&point| self.project_point_onto_ellipsoid(point))
            .collect()
    }

    pub fn equals(&self, other: Option<&Self>) -> bool {
        match other {
            Some(o) => {
                self.ellipsoid == o.ellipsoid
                    && self.origin == o.origin
                    && self.plane == o.plane
                    && self.x_axis == o.x_axis
                    && self.y_axis == o.y_axis
            }
            None => false,
        }
    }
}
