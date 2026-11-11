//! TileBoundingVolume matching CesiumJS `Scene/TileBoundingVolume.js` (Apache-2.0).

use cesium_math::{Color, DeveloperError, PlaneIntersection};

use crate::scene_primitive::Primitive;

/// Defines a bounding volume for a tile.
///
/// This type describes an interface and is not intended to be instantiated directly.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TileBoundingVolume;

impl TileBoundingVolume {
    /// Creates a new `TileBoundingVolume`.
    pub fn new() -> Self {
        Self
    }

    /// Calculates the distance between the tile and the camera.
    pub fn distance_to_camera(&self) -> Result<f64, DeveloperError> {
        Err(DeveloperError::new(
            "This type describes an interface and is not intended to be instantiated directly.",
        ))
    }

    /// Determines which side of a plane this volume is located.
    pub fn intersect_plane(&self) -> Result<PlaneIntersection, DeveloperError> {
        Err(DeveloperError::new(
            "This type describes an interface and is not intended to be instantiated directly.",
        ))
    }

    /// Creates a debug primitive that shows the outline of the tile bounding volume.
    pub fn create_debug_volume(&self) -> Result<Primitive, DeveloperError> {
        Err(DeveloperError::new(
            "This type describes an interface and is not intended to be instantiated directly.",
        ))
    }

    /// Creates a debug primitive that shows the outline of the tile bounding volume with a color.
    pub fn create_debug_volume_with_color(&self, _color: Color) -> Result<Primitive, DeveloperError> {
        Err(DeveloperError::new(
            "This type describes an interface and is not intended to be instantiated directly.",
        ))
    }
}
