use cesium_math::{ArcType, Cartesian3, DeveloperError, Ellipsoid, GeoRectangle, PolygonGeometry};
use std::sync::Arc;

/// Options for creating a `ClippingPolygon`.
#[derive(Debug, Clone, Default)]
pub struct ClippingPolygonOptions {
    /// Outer ring positions (minimum 3 required).
    pub positions: Option<Vec<Cartesian3>>,
    /// Interior rings (holes), each minimum 3 required.
    pub holes: Option<Vec<Vec<Cartesian3>>>,
    /// Ellipsoid for projection. Defaults to Ellipsoid::WGS84.
    pub ellipsoid: Option<Ellipsoid>,
}

/// A geodesic polygon used for selectively clipping regions in a model, tileset, or globe.
#[derive(Debug, Clone)]
pub struct ClippingPolygon {
    positions: Arc<[Cartesian3]>,
    holes: Vec<Arc<[Cartesian3]>>,
    ellipsoid: Ellipsoid,
    rectangle: GeoRectangle,
}

impl ClippingPolygon {
    /// Creates a new `ClippingPolygon`.
    pub fn new(options: ClippingPolygonOptions) -> Result<Self, DeveloperError> {
        let positions_vec = options
            .positions
            .ok_or_else(|| DeveloperError::new("options.positions is required."))?;

        if positions_vec.len() < 3 {
            return Err(DeveloperError::new(
                "options.positions.length must be greater than or equal to 3.",
            ));
        }

        let mut holes = Vec::new();
        if let Some(h_list) = options.holes {
            for (i, h) in h_list.into_iter().enumerate() {
                if h.len() < 3 {
                    return Err(DeveloperError::new(format!(
                        "options.holes[{i}].length must be greater than or equal to 3."
                    )));
                }
                holes.push(Arc::from(h.into_boxed_slice()));
            }
        }

        let ellipsoid = options.ellipsoid.unwrap_or(Ellipsoid::WGS84);
        let rectangle = PolygonGeometry::compute_rectangle_from_positions(
            &positions_vec,
            Some(&ellipsoid),
            Some(ArcType::Geodesic),
        )?;

        Ok(Self {
            positions: Arc::from(positions_vec.into_boxed_slice()),
            holes,
            ellipsoid,
            rectangle,
        })
    }

    /// Total number of vertices including outer ring and all holes.
    #[inline]
    pub fn length(&self) -> usize {
        let mut count = self.positions.len();
        for h in &self.holes {
            count += h.len();
        }
        count
    }

    /// Returns the outer ring positions.
    #[inline]
    pub fn positions(&self) -> &[Cartesian3] {
        &self.positions
    }

    /// Returns the holes.
    #[inline]
    pub fn holes(&self) -> Vec<Vec<Cartesian3>> {
        self.holes.iter().map(|h| h.to_vec()).collect()
    }

    /// Returns the ellipsoid.
    #[inline]
    pub fn ellipsoid(&self) -> Ellipsoid {
        self.ellipsoid
    }

    /// Returns the cartographic rectangle enclosing the polygon.
    #[inline]
    pub fn rectangle(&self) -> GeoRectangle {
        self.rectangle
    }

    /// Clones the ClippingPolygon.
    pub fn clone_polygon(polygon: &Self, result: Option<&mut Self>) -> Self {
        let cloned = Self {
            positions: Arc::from(polygon.positions.to_vec().into_boxed_slice()),
            holes: polygon
                .holes
                .iter()
                .map(|h| Arc::from(h.to_vec().into_boxed_slice()))
                .collect(),
            ellipsoid: polygon.ellipsoid,
            rectangle: polygon.rectangle,
        };

        if let Some(res) = result {
            *res = cloned.clone();
        }

        cloned
    }

    /// Compares two ClippingPolygons (requires pointer equality for positions as per CesiumJS spec).
    pub fn equals(left: &Self, right: &Self) -> bool {
        if left.ellipsoid != right.ellipsoid {
            return false;
        }

        if !Arc::ptr_eq(&left.positions, &right.positions) {
            return false;
        }

        if left.holes.len() != right.holes.len() {
            return false;
        }

        for (h1, h2) in left.holes.iter().zip(right.holes.iter()) {
            if !Arc::ptr_eq(h1, h2) {
                return false;
            }
        }

        true
    }

    /// Deprecated method to compute rectangle.
    pub fn compute_rectangle(&self, result: &mut GeoRectangle) -> GeoRectangle {
        *result = self.rectangle;
        self.rectangle
    }
}
