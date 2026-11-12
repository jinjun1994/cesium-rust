//! Visualizer trait matching CesiumJS `DataSources/Visualizer.js` (Apache-2.0).

use crate::{BoundingSphere, BoundingSphereState, Entity, JulianDate};

/// Trait implemented by all entity visualizers.
pub trait Visualizer: std::fmt::Debug {
    /// Updates the visualizer to the given time.
    fn update(&mut self, time: &JulianDate) -> bool;

    /// Computes the bounding sphere for the entity.
    fn get_bounding_sphere(
        &self,
        entity: &Entity,
        result: &mut BoundingSphere,
    ) -> BoundingSphereState;

    /// Returns true if this visualizer has been destroyed.
    fn is_destroyed(&self) -> bool {
        false
    }

    /// Destroys the visualizer and releases its resources.
    fn destroy(&mut self) {}
}
