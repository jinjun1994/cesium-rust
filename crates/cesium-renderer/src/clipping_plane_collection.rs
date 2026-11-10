//! ClippingPlaneCollection matching CesiumJS `Scene/ClippingPlaneCollection.js` (Apache-2.0).

use crate::clipping_plane::ClippingPlane;
use crate::clipping_polygon_collection::Intersect;
use cesium_math::{Cartesian2, Color, Event, Matrix4, Sphere};
use glam::DMat4;

/// Initialization options for `ClippingPlaneCollection`.
#[derive(Debug, Clone, Default)]
pub struct ClippingPlaneCollectionOptions {
    /// An array of ClippingPlane objects used to initialize the collection.
    pub planes: Option<Vec<ClippingPlane>>,
    /// Determines whether the clipping planes are active. Defaults to true.
    pub enabled: Option<bool>,
    /// The 4x4 matrix used to transform the planes into world coordinates. Defaults to Matrix4::IDENTITY.
    pub model_matrix: Option<Matrix4>,
    /// The color applied to highlights on edges created by clipping planes. Defaults to Color::WHITE.
    pub edge_color: Option<Color>,
    /// The width in pixels of edges created by clipping planes. Defaults to 0.0.
    pub edge_width: Option<f64>,
    /// Determines whether regions are unioned. Defaults to false (intersection).
    pub union_clipping_regions: Option<bool>,
    /// Optional explicit texture resolution.
    pub texture_resolution: Option<Cartesian2>,
}

/// A collection of ClippingPlane objects that defines planar boundaries for clipping models or 3D Tilesets.
#[derive(Debug, Clone)]
pub struct ClippingPlaneCollection {
    planes: Vec<ClippingPlane>,
    /// Determines whether the clipping planes are active.
    pub enabled: bool,
    /// The 4x4 matrix used to transform the planes into world coordinates.
    pub model_matrix: Matrix4,
    /// The color applied to highlights on edges created by clipping planes.
    pub edge_color: Color,
    /// The width in pixels of edges created by clipping planes.
    pub edge_width: f64,
    /// Determines whether regions are unioned.
    pub union_clipping_regions: bool,
    /// Explicit texture resolution override if set.
    pub texture_resolution: Option<Cartesian2>,

    /// Event fired when a plane is added to the collection.
    pub plane_added: Event<usize>,
    /// Event fired when a plane is removed from the collection.
    pub plane_removed: Event<usize>,

    is_destroyed: bool,
}

impl Default for ClippingPlaneCollection {
    fn default() -> Self {
        Self::new(None)
    }
}

impl ClippingPlaneCollection {
    /// Creates a new `ClippingPlaneCollection`.
    pub fn new(options: Option<ClippingPlaneCollectionOptions>) -> Self {
        let opts = options.unwrap_or_default();
        let mut collection = Self {
            planes: Vec::new(),
            enabled: opts.enabled.unwrap_or(true),
            model_matrix: opts.model_matrix.unwrap_or(Matrix4::IDENTITY),
            edge_color: opts.edge_color.unwrap_or(Color::WHITE),
            edge_width: opts.edge_width.unwrap_or(0.0),
            union_clipping_regions: opts.union_clipping_regions.unwrap_or(false),
            texture_resolution: opts.texture_resolution,
            plane_added: Event::new(),
            plane_removed: Event::new(),
            is_destroyed: false,
        };

        if let Some(initial_planes) = opts.planes {
            for plane in initial_planes {
                collection.add(plane);
            }
        }

        collection
    }

    /// Number of planes in the collection.
    pub fn len(&self) -> usize {
        self.planes.len()
    }

    /// Returns true if the collection has no planes.
    pub fn is_empty(&self) -> bool {
        self.planes.is_empty()
    }

    /// Adds a ClippingPlane to the collection.
    pub fn add(&mut self, plane: ClippingPlane) {
        let index = self.planes.len();
        self.planes.push(plane);
        self.plane_added.raise_event(&index);
    }

    /// Returns the plane at the specified index.
    pub fn get(&self, index: usize) -> Option<&ClippingPlane> {
        self.planes.get(index)
    }

    /// Returns mutable reference to the plane at the specified index.
    pub fn get_mut(&mut self, index: usize) -> Option<&mut ClippingPlane> {
        self.planes.get_mut(index)
    }

    /// Determines whether this collection contains the specified plane.
    pub fn contains(&self, plane: &ClippingPlane) -> bool {
        self.planes.iter().any(|p| p == plane)
    }

    /// Removes the first occurrence of the specified plane from the collection.
    pub fn remove(&mut self, plane: &ClippingPlane) -> bool {
        if let Some(pos) = self.planes.iter().position(|p| p == plane) {
            self.planes.remove(pos);
            self.plane_removed.raise_event(&pos);
            true
        } else {
            false
        }
    }

    /// Removes all planes from the collection.
    pub fn remove_all(&mut self) {
        if !self.planes.is_empty() {
            let count = self.planes.len();
            self.planes.clear();
            self.plane_removed.raise_event(&count);
        }
    }

    /// Computes a number representing the clipping planes state for shaders.
    pub fn clipping_planes_state(&self) -> i32 {
        let count = self.planes.len() as i32;
        if count == 0 {
            0
        } else if self.union_clipping_regions {
            count
        } else {
            -count
        }
    }

    /// Computes intersection between the bounding sphere and all clipping planes in the collection.
    pub fn compute_intersection_with_bounding_volume(
        &self,
        bounding_volume: &Sphere,
        transform: Option<&Matrix4>,
    ) -> Intersect {
        let length = self.planes.len();
        if length == 0 {
            return Intersect::Inside;
        }

        let dmat = match transform {
            Some(t) => DMat4::from(*t) * DMat4::from(self.model_matrix),
            None => DMat4::from(self.model_matrix),
        };

        let mut intersection = if !self.union_clipping_regions {
            Intersect::Outside
        } else {
            Intersect::Inside
        };

        for plane in &self.planes {
            let Ok(p) = plane.to_plane() else {
                continue;
            };
            let transformed_plane = p.transform(dmat).unwrap_or(p);

            let intersect = transformed_plane.classify_sphere(*bounding_volume);
            let val = match intersect {
                cesium_math::PlaneIntersection::Outside => Intersect::Outside,
                cesium_math::PlaneIntersection::Intersecting => Intersect::Intersecting,
                cesium_math::PlaneIntersection::Inside => Intersect::Inside,
            };

            if val == Intersect::Intersecting {
                intersection = Intersect::Intersecting;
            } else if self.test_intersection(val) {
                return val;
            }
        }

        intersection
    }

    fn test_intersection(&self, value: Intersect) -> bool {
        if self.union_clipping_regions {
            value == Intersect::Outside
        } else {
            value == Intersect::Inside
        }
    }

    /// Calculates expected texture resolution given plane count and texture format.
    pub fn get_texture_resolution(
        &self,
        use_float_texture: bool,
        max_size: Option<u32>,
    ) -> Cartesian2 {
        if let Some(res) = self.texture_resolution {
            return res;
        }
        let max_size = max_size.unwrap_or(2048) as f64;
        let count = self.planes.len() as f64;
        let pixels_needed = if use_float_texture { count } else { count * 2.0 };

        if pixels_needed == 0.0 {
            return Cartesian2::ZERO;
        }

        let width = pixels_needed.min(max_size);
        let height = (pixels_needed / width).ceil() * 2.0;

        Cartesian2::new(width, height)
    }

    /// Returns true if this object was destroyed.
    pub fn is_destroyed(&self) -> bool {
        self.is_destroyed
    }

    /// Destroys resources.
    pub fn destroy(&mut self) {
        self.planes.clear();
        self.is_destroyed = true;
    }
}
