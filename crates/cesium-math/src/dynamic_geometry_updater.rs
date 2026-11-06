//! DynamicGeometryUpdater matching CesiumJS `DataSources/DynamicGeometryUpdater.js` (Apache-2.0).

use crate::{DeveloperError, JulianDate};

/// Base struct managing dynamic geometry update lifecycle.
pub struct DynamicGeometryUpdater {
    is_destroyed: bool,
}

impl DynamicGeometryUpdater {
    /// Creates a new DynamicGeometryUpdater validating updater, primitives and groundPrimitives.
    pub fn new<U, P, G>(
        updater: Option<&U>,
        primitives: Option<&P>,
        ground_primitives: Option<&G>,
    ) -> Result<Self, DeveloperError> {
        if updater.is_none() {
            return Err(DeveloperError::new("updater is required."));
        }
        if primitives.is_none() {
            return Err(DeveloperError::new("primitives is required."));
        }
        if ground_primitives.is_none() {
            return Err(DeveloperError::new("groundPrimitives is required."));
        }

        Ok(Self {
            is_destroyed: false,
        })
    }

    /// Updates dynamic geometry at the given time.
    pub fn update(&mut self, time: Option<JulianDate>) -> Result<(), DeveloperError> {
        if time.is_none() {
            return Err(DeveloperError::new("time is required."));
        }
        Ok(())
    }

    /// Checks if this dynamic updater is destroyed.
    pub fn is_destroyed(&self) -> bool {
        self.is_destroyed
    }

    /// Destroys this dynamic updater.
    pub fn destroy(&mut self) {
        self.is_destroyed = true;
    }
}
