//! MappedPositions representing cartographic positions computed for a primitive.
//! Matches CesiumJS `Scene/Model/MappedPositions.js` (Apache-2.0).

use cesium_math::{Cartographic, DeveloperError, Ellipsoid, GeoRectangle};

/// A collection of cartographic positions (and their bounding rectangle) that
/// have been computed from cartesian positions, for a specific ellipsoid.
#[derive(Clone, Debug, PartialEq)]
pub struct MappedPositions {
    pub cartographic_positions: Vec<Cartographic>,
    pub num_positions: usize,
    pub cartographic_bounding_rectangle: GeoRectangle,
    pub ellipsoid: Ellipsoid,
}

impl MappedPositions {
    /// Creates a new `MappedPositions` instance.
    pub fn new(
        cartographic_positions: Option<Vec<Cartographic>>,
        num_positions: Option<i64>,
        cartographic_bounding_rectangle: Option<GeoRectangle>,
        ellipsoid: Option<Ellipsoid>,
    ) -> Result<Self, DeveloperError> {
        let positions = cartographic_positions
            .ok_or_else(|| DeveloperError::new("cartographicPositions is required."))?;
        let num = num_positions
            .ok_or_else(|| DeveloperError::new("numPositions is required."))?;
        if num < 0 {
            return Err(DeveloperError::new(
                "numPositions must be greater than or equal to 0.",
            ));
        }
        let rect = cartographic_bounding_rectangle
            .ok_or_else(|| DeveloperError::new("cartographicBoundingRectangle is required."))?;
        let ellip = ellipsoid.ok_or_else(|| DeveloperError::new("ellipsoid is required."))?;

        Ok(Self {
            cartographic_positions: positions,
            num_positions: num as usize,
            cartographic_bounding_rectangle: rect,
            ellipsoid: ellip,
        })
    }

    /// Convenience constructor without optional wrappers.
    pub fn from_values(
        cartographic_positions: Vec<Cartographic>,
        num_positions: usize,
        cartographic_bounding_rectangle: GeoRectangle,
        ellipsoid: Ellipsoid,
    ) -> Result<Self, DeveloperError> {
        Ok(Self {
            cartographic_positions,
            num_positions,
            cartographic_bounding_rectangle,
            ellipsoid,
        })
    }

    /// Gets the cartographic positions.
    pub fn cartographic_positions(&self) -> &[Cartographic] {
        &self.cartographic_positions
    }

    /// Gets the number of positions.
    pub fn num_positions(&self) -> usize {
        self.num_positions
    }

    /// Gets the cartographic bounding rectangle.
    pub fn cartographic_bounding_rectangle(&self) -> GeoRectangle {
        self.cartographic_bounding_rectangle
    }

    /// Gets the ellipsoid for which these positions have been created.
    pub fn ellipsoid(&self) -> Ellipsoid {
        self.ellipsoid
    }
}
