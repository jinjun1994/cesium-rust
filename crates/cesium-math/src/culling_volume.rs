//! Plane-based culling volumes adapted from CesiumJS (Apache-2.0).
use crate::{AxisAlignedBoundingBox, MathError, Plane, PlaneIntersection, Sphere};
use glam::DVec3;

/// A volume that can classify itself against a clipping plane.
pub trait BoundingVolume {
    fn intersect_plane(&self, plane: Plane) -> PlaneIntersection;
}

impl BoundingVolume for Sphere {
    fn intersect_plane(&self, plane: Plane) -> PlaneIntersection {
        Sphere::intersect_plane(*self, plane.normal, plane.distance)
    }
}

impl BoundingVolume for AxisAlignedBoundingBox {
    fn intersect_plane(&self, plane: Plane) -> PlaneIntersection {
        AxisAlignedBoundingBox::intersect_plane(*self, plane.normal, plane.distance)
    }
}

#[derive(Clone, Debug, Default)]
pub struct CullingVolume {
    pub planes: Vec<Plane>,
}

impl CullingVolume {
    pub const MASK_OUTSIDE: u32 = u32::MAX;
    pub const MASK_INSIDE: u32 = 0;
    pub const MASK_INDETERMINATE: u32 = 0x7fff_ffff;

    pub fn new(planes: Vec<Plane>) -> Self {
        Self { planes }
    }

    /// Creates the six axis-aligned planes of the cube enclosing the sphere.
    pub fn from_bounding_sphere(sphere: Sphere) -> Result<Self, MathError> {
        if !sphere.center.is_finite() || !sphere.radius.is_finite() || sphere.radius < 0.0 {
            return Err(MathError);
        }
        let mut planes = Vec::with_capacity(6);
        for axis in [DVec3::X, DVec3::Y, DVec3::Z] {
            planes.push(Plane::from_point_normal(
                sphere.center - axis * sphere.radius,
                axis,
            )?);
            planes.push(Plane::from_point_normal(
                sphere.center + axis * sphere.radius,
                -axis,
            )?);
        }
        Ok(Self { planes })
    }

    pub fn compute_visibility<V: BoundingVolume + ?Sized>(&self, volume: &V) -> PlaneIntersection {
        let mut intersection = false;
        for &plane in &self.planes {
            match volume.intersect_plane(plane) {
                PlaneIntersection::Outside => return PlaneIntersection::Outside,
                PlaneIntersection::Intersecting => intersection = true,
                PlaneIntersection::Inside => {}
            }
        }
        if intersection {
            PlaneIntersection::Intersecting
        } else {
            PlaneIntersection::Inside
        }
    }

    /// Parent masks are valid only for child volumes contained by that parent.
    /// As in Cesium, only the first 31 planes can be represented by mask bits.
    pub fn compute_visibility_with_plane_mask<V: BoundingVolume + ?Sized>(
        &self,
        volume: &V,
        parent_mask: u32,
    ) -> u32 {
        if parent_mask == Self::MASK_OUTSIDE || parent_mask == Self::MASK_INSIDE {
            return parent_mask;
        }
        let mut mask = Self::MASK_INSIDE;
        for (index, &plane) in self.planes.iter().enumerate() {
            let flag = if index < 31 { 1_u32 << index } else { 0 };
            if index < 31 && parent_mask & flag == 0 {
                continue;
            }
            match volume.intersect_plane(plane) {
                PlaneIntersection::Outside => return Self::MASK_OUTSIDE,
                PlaneIntersection::Intersecting => mask |= flag,
                PlaneIntersection::Inside => {}
            }
        }
        mask
    }
}
