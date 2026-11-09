use crate::clipping_polygon::ClippingPolygon;
use cesium_math::{DeveloperError, Ellipsoid, GeoRectangle};

/// Intersect type matching CesiumJS `Core/Intersect.js`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Intersect {
    /// Outside the volume.
    Outside = -1,
    /// Intersecting the volume.
    Intersecting = 0,
    /// Inside the volume.
    Inside = 1,
}

/// Options for creating a `ClippingPolygonCollection`.
#[derive(Debug, Clone, Default)]
pub struct ClippingPolygonCollectionOptions {
    /// Initial polygons in the collection.
    pub polygons: Option<Vec<ClippingPolygon>>,
    /// Determines whether clipping polygons are active. Defaults to true.
    pub enabled: Option<bool>,
    /// If true, regions outside every polygon are clipped. Defaults to false.
    pub inverse: Option<bool>,
    /// Quality factor. Defaults to 1.0.
    pub quality: Option<f64>,
    /// Ellipsoid for projection. Defaults to Ellipsoid::WGS84.
    pub ellipsoid: Option<Ellipsoid>,
}

/// A collection of `ClippingPolygon` objects that selectively disable rendering in specified regions.
#[derive(Debug, Clone)]
pub struct ClippingPolygonCollection {
    polygons: Vec<ClippingPolygon>,
    /// Determines whether clipping polygons are active.
    pub enabled: bool,
    /// If true, regions outside every polygon are clipped.
    pub inverse: bool,
    /// Quality scalar.
    pub quality: f64,
    /// The ellipsoid.
    pub ellipsoid: Ellipsoid,
    is_destroyed: bool,
}

impl Default for ClippingPolygonCollection {
    fn default() -> Self {
        Self::new(ClippingPolygonCollectionOptions::default())
    }
}

impl ClippingPolygonCollection {
    /// Creates a new `ClippingPolygonCollection`.
    pub fn new(options: ClippingPolygonCollectionOptions) -> Self {
        let polygons = options.polygons.unwrap_or_default();
        Self {
            polygons,
            enabled: options.enabled.unwrap_or(true),
            inverse: options.inverse.unwrap_or(false),
            quality: options.quality.unwrap_or(1.0),
            ellipsoid: options.ellipsoid.unwrap_or(Ellipsoid::WGS84),
            is_destroyed: false,
        }
    }

    /// Number of polygons in the collection.
    #[inline]
    pub fn length(&self) -> usize {
        self.polygons.len()
    }

    /// Total vertices across all polygons including holes.
    pub fn vertex_count(&self) -> usize {
        self.polygons.iter().map(|p| p.length()).sum()
    }

    /// Total number of holes across all polygons.
    pub fn hole_count(&self) -> usize {
        self.polygons.iter().map(|p| p.holes().len()).sum()
    }

    /// Encodes enabled, presence, and inverse into a state number:
    /// - 0: inactive (disabled or empty)
    /// - 1: normal clipping
    /// - -1: inverse clipping
    #[inline]
    pub fn clipping_polygons_state(&self) -> i32 {
        if !self.enabled || self.polygons.is_empty() {
            0
        } else if self.inverse {
            -1
        } else {
            1
        }
    }

    /// Adds a `ClippingPolygon` to the collection.
    pub fn add(&mut self, polygon: ClippingPolygon) -> usize {
        let index = self.polygons.len();
        self.polygons.push(polygon);
        index
    }

    /// Gets the polygon at the specified index.
    pub fn get(&self, index: usize) -> Option<&ClippingPolygon> {
        self.polygons.get(index)
    }

    /// Checks whether this collection contains the given polygon.
    pub fn contains(&self, polygon: &ClippingPolygon) -> bool {
        self.polygons
            .iter()
            .any(|p| ClippingPolygon::equals(p, polygon))
    }

    /// Removes the first occurrence of the given polygon.
    pub fn remove(&mut self, polygon: &ClippingPolygon) -> bool {
        if let Some(pos) = self
            .polygons
            .iter()
            .position(|p| ClippingPolygon::equals(p, polygon))
        {
            self.polygons.remove(pos);
            true
        } else {
            false
        }
    }

    /// Removes all polygons from the collection.
    pub fn remove_all(&mut self) {
        self.polygons.clear();
    }

    /// Determines the intersection type with a bounding rectangle.
    pub fn compute_intersection_with_rectangle(
        &self,
        rect: &GeoRectangle,
    ) -> Intersect {
        for poly in &self.polygons {
            let r = poly.rectangle();
            // Check bounding rectangle overlap
            if rect.west <= r.east
                && rect.east >= r.west
                && rect.south <= r.north
                && rect.north >= r.south
            {
                return Intersect::Intersecting;
            }
        }

        if self.inverse {
            Intersect::Inside
        } else {
            Intersect::Outside
        }
    }

    /// Validates request rectangle data parameters.
    pub fn validate_request_data_params(
        &self,
        rectangle: Option<&GeoRectangle>,
    ) -> Result<(), DeveloperError> {
        if rectangle.is_none() {
            return Err(DeveloperError::new("rectangle is required."));
        }
        Ok(())
    }

    /// Returns true if destroyed.
    pub fn is_destroyed(&self) -> bool {
        self.is_destroyed
    }

    /// Destroys resources.
    pub fn destroy(&mut self) {
        self.is_destroyed = true;
    }
}
