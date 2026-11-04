//! Axis-aligned bounding boxes adapted from CesiumJS (Apache-2.0).
use crate::{MathError, PlaneIntersection};
use glam::DVec3;

/// Axis-aligned box with an explicitly stored center, as in CesiumJS.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct AxisAlignedBoundingBox {
    pub minimum: DVec3,
    pub maximum: DVec3,
    pub center: DVec3,
}

impl AxisAlignedBoundingBox {
    /// A supplied center is retained even when it differs from the midpoint.
    /// Reversed corners are accepted, matching the upstream constructor.
    pub fn new(minimum: DVec3, maximum: DVec3, center: Option<DVec3>) -> Self {
        Self {
            minimum,
            maximum,
            center: center.unwrap_or((minimum + maximum) * 0.5),
        }
    }

    pub fn from_corners(minimum: DVec3, maximum: DVec3) -> Self {
        Self::new(minimum, maximum, None)
    }

    /// An empty slice gives a zero box. Non-finite points are rejected.
    pub fn from_points(points: &[DVec3]) -> Result<Self, MathError> {
        let Some((&first, rest)) = points.split_first() else {
            return Ok(Self::default());
        };
        if !first.is_finite() {
            return Err(MathError);
        }
        let (mut minimum, mut maximum) = (first, first);
        for &point in rest {
            if !point.is_finite() {
                return Err(MathError);
            }
            minimum = minimum.min(point);
            maximum = maximum.max(point);
        }
        Ok(Self::from_corners(minimum, maximum))
    }

    /// Classifies this box against a plane `normal.dot(point) + distance = 0`.
    pub fn intersect_plane(self, normal: DVec3, distance: f64) -> PlaneIntersection {
        let half = (self.maximum - self.minimum) * 0.5;
        let extent = half.dot(normal.abs());
        let signed = self.center.dot(normal) + distance;
        if signed - extent > 0.0 {
            PlaneIntersection::Inside
        } else if signed + extent < 0.0 {
            PlaneIntersection::Outside
        } else {
            PlaneIntersection::Intersecting
        }
    }

    /// Touching faces, edges and corners count as intersections.
    pub fn intersects(self, other: Self) -> bool {
        self.minimum.cmple(other.maximum).all() && self.maximum.cmpge(other.minimum).all()
    }
}
