//! Helper for viewing and tracking an Entity with camera and reference frames.
//! Direct port of CesiumJS `DataSources/EntityView.js`.

use std::sync::atomic::{AtomicI64, Ordering};

use crate::{
    BoundingSphere, Cartesian3, Ellipsoid, Entity, FixedFrameTransforms, JulianDate, Matrix3,
    Matrix4, TrackingReferenceFrame,
};

// Fixed-point atomic representation for defaultOffset3D (x, y, z scaled by 1000)
static DEFAULT_OFFSET_X: AtomicI64 = AtomicI64::new(-14_000_000);
static DEFAULT_OFFSET_Y: AtomicI64 = AtomicI64::new(3_500_000);
static DEFAULT_OFFSET_Z: AtomicI64 = AtomicI64::new(3_500_000);

/// Helper for viewing and tracking an Entity.
#[derive(Debug, Clone)]
pub struct EntityView {
    pub entity: Entity,
    pub ellipsoid: Ellipsoid,
    pub bounding_sphere: Option<BoundingSphere>,
    pub offset_3d: Cartesian3,
}

impl EntityView {
    /// Gets the static camera offset used to initialize EntityViews.
    pub fn default_offset_3d() -> Cartesian3 {
        Cartesian3::new(
            DEFAULT_OFFSET_X.load(Ordering::Relaxed) as f64 / 1000.0,
            DEFAULT_OFFSET_Y.load(Ordering::Relaxed) as f64 / 1000.0,
            DEFAULT_OFFSET_Z.load(Ordering::Relaxed) as f64 / 1000.0,
        )
    }

    /// Sets the static camera offset used to initialize EntityViews.
    pub fn set_default_offset_3d(offset: Cartesian3) {
        DEFAULT_OFFSET_X.store((offset.x * 1000.0).round() as i64, Ordering::Relaxed);
        DEFAULT_OFFSET_Y.store((offset.y * 1000.0).round() as i64, Ordering::Relaxed);
        DEFAULT_OFFSET_Z.store((offset.z * 1000.0).round() as i64, Ordering::Relaxed);
    }

    /// Creates a new EntityView for the given entity.
    pub fn new(entity: Entity, ellipsoid: Option<Ellipsoid>) -> Self {
        let offset = Self::default_offset_3d();
        Self {
            entity,
            ellipsoid: ellipsoid.unwrap_or(Ellipsoid::WGS84),
            bounding_sphere: None,
            offset_3d: offset,
        }
    }

    /// Computes the camera offset to use for tracking the entity at the given time.
    pub fn compute_offset(&self, time: &JulianDate) -> Cartesian3 {
        if let Some(ref view_from_prop) = self.entity.view_from {
            if let Some(view_from) = view_from_prop.get_value(Some(time)) {
                return view_from;
            }
        }
        self.offset_3d
    }

    /// Computes the transform matrix for tracking reference frame at the given time.
    pub fn compute_transform(&self, time: &JulianDate) -> Option<Matrix4> {
        let position = self.entity.position.as_ref()?.get_value(Some(time))?;

        match self.entity.tracking_reference_frame {
            TrackingReferenceFrame::Inertial => {
                if let Some(ref orientation_prop) = self.entity.orientation {
                    if let Some(orientation) = orientation_prop.get_value(Some(time)) {
                        let rot = Matrix3::from_quaternion(&orientation);
                        return Some(Matrix4::from_rotation_translation(&rot, position));
                    }
                }
                FixedFrameTransforms::east_north_up_to_fixed_frame(position, Some(self.ellipsoid)).ok()
            }
            TrackingReferenceFrame::Velocity | TrackingReferenceFrame::Enu | TrackingReferenceFrame::Autodetect => {
                FixedFrameTransforms::east_north_up_to_fixed_frame(position, Some(self.ellipsoid)).ok()
            }
        }
    }
}
