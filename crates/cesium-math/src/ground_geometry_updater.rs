//! GroundGeometryUpdater matching CesiumJS `DataSources/GroundGeometryUpdater.js` (Apache-2.0).

use crate::{GeometryOffsetAttribute, HeightReference};

/// Sentinel constant representing clamped to ground extruded height.
pub const CLAMP_TO_GROUND: f64 = -1.0;

/// Base logic for ground geometry updating and height offset calculations.
pub struct GroundGeometryUpdater {
    is_destroyed: bool,
    z_index: i32,
}

impl Default for GroundGeometryUpdater {
    fn default() -> Self {
        Self::new()
    }
}

impl GroundGeometryUpdater {
    /// Creates a new GroundGeometryUpdater.
    pub fn new() -> Self {
        Self {
            is_destroyed: false,
            z_index: 0,
        }
    }

    /// Gets the z-index for ordering ground primitives.
    pub fn z_index(&self) -> i32 {
        self.z_index
    }

    /// Sets the z-index for ordering ground primitives.
    pub fn set_z_index(&mut self, z: i32) {
        self.z_index = z;
    }

    /// Checks if updater has been destroyed.
    pub fn is_destroyed(&self) -> bool {
        self.is_destroyed
    }

    /// Destroys this updater.
    pub fn destroy(&mut self) {
        self.is_destroyed = true;
    }

    /// Calculates geometry height according to height reference.
    pub fn get_geometry_height(height: f64, height_reference: Option<HeightReference>) -> f64 {
        match height_reference {
            Some(HeightReference::ClampToGround | HeightReference::ClampToTerrain) => 0.0,
            _ => height,
        }
    }

    /// Calculates extruded geometry height according to extruded height reference.
    pub fn get_geometry_extruded_height(
        extruded_height: f64,
        extruded_height_reference: Option<HeightReference>,
    ) -> f64 {
        match extruded_height_reference {
            Some(HeightReference::ClampToGround | HeightReference::ClampToTerrain) => {
                CLAMP_TO_GROUND
            }
            _ => extruded_height,
        }
    }

    /// Computes offset attribute for geometry instances based on heights and references.
    pub fn compute_geometry_offset_attribute(
        height: Option<f64>,
        height_reference: Option<HeightReference>,
        extruded_height: Option<f64>,
        extruded_height_reference: Option<HeightReference>,
    ) -> Option<GeometryOffsetAttribute> {
        if height.is_none() && extruded_height.is_none() {
            return None;
        }

        let is_height_clamp = matches!(
            height_reference,
            Some(HeightReference::ClampToGround | HeightReference::ClampToTerrain)
        );
        let height_is_relative = matches!(
            height_reference,
            Some(HeightReference::RelativeToGround | HeightReference::RelativeToTerrain)
        );
        let extruded_height_is_relative = matches!(
            extruded_height_reference,
            Some(HeightReference::RelativeToGround | HeightReference::RelativeToTerrain)
        );

        if (height_is_relative && extruded_height_is_relative)
            || (is_height_clamp && extruded_height_is_relative)
        {
            return Some(GeometryOffsetAttribute::All);
        }

        if is_height_clamp || extruded_height_is_relative || height_is_relative {
            return Some(GeometryOffsetAttribute::Top);
        }

        None
    }
}
