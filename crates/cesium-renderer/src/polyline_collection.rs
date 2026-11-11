//! Polyline and PolylineCollection matching CesiumJS `Scene/PolylineCollection.js` and `Scene/Polyline.js` (Apache-2.0).

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::material::Material;
use cesium_math::{
    BoundingSphere, Cartesian3, DeveloperError, DistanceDisplayCondition, Ellipsoid,
};
use glam::{DMat4, DVec3};

static NEXT_POLYLINE_ID: AtomicU64 = AtomicU64::new(1);
static NEXT_COLLECTION_ID: AtomicU64 = AtomicU64::new(1);

/// Construction options for creating a Polyline.
#[derive(Clone, Debug, PartialEq)]
pub struct PolylineOptions {
    pub show: Option<bool>,
    pub positions: Option<Vec<Cartesian3>>,
    pub loop_: Option<bool>,
    pub width: Option<f64>,
    pub material: Option<Material>,
    pub id: Option<String>,
    pub distance_display_condition: Option<DistanceDisplayCondition>,
}

impl Default for PolylineOptions {
    fn default() -> Self {
        Self {
            show: Some(true),
            positions: Some(Vec::new()),
            loop_: Some(false),
            width: Some(1.0),
            material: None,
            id: None,
            distance_display_condition: None,
        }
    }
}

/// A renderable polyline primitive.
#[derive(Clone, Debug)]
pub struct Polyline {
    pub internal_id: u64,
    pub collection_id: Option<u64>,
    pub show: bool,
    pub positions: Vec<Cartesian3>,
    pub loop_: bool,
    pub width: f64,
    pub material: Material,
    pub id: Option<String>,
    pub distance_display_condition: Option<DistanceDisplayCondition>,
    pub dirty: bool,
}

impl PartialEq for Polyline {
    fn eq(&self, other: &Self) -> bool {
        self.internal_id == other.internal_id
    }
}

impl Default for Polyline {
    fn default() -> Self {
        let mut mat = Material::default();
        let mut color_map = HashMap::new();
        color_map.insert("red".to_string(), serde_json::json!(1.0));
        color_map.insert("green".to_string(), serde_json::json!(1.0));
        color_map.insert("blue".to_string(), serde_json::json!(1.0));
        color_map.insert("alpha".to_string(), serde_json::json!(1.0));
        mat.uniforms.insert("color".to_string(), serde_json::json!(color_map));

        Self {
            internal_id: NEXT_POLYLINE_ID.fetch_add(1, Ordering::Relaxed),
            collection_id: None,
            show: true,
            positions: Vec::new(),
            loop_: false,
            width: 1.0,
            material: mat,
            id: None,
            distance_display_condition: None,
            dirty: true,
        }
    }
}

impl Polyline {
    /// Sets polyline positions. If positions count <= 2, loop is automatically disabled.
    pub fn set_positions(&mut self, positions: Vec<Cartesian3>) {
        if positions.len() <= 2 {
            self.loop_ = false;
        }
        self.positions = positions;
        self.dirty = true;
    }

    /// Sets the loop property. Loop can only be true if number of positions > 2.
    pub fn set_loop(&mut self, loop_: bool) {
        if self.positions.len() > 2 {
            self.loop_ = loop_;
        } else {
            self.loop_ = false;
        }
        self.dirty = true;
    }

    pub fn set_width(&mut self, width: f64) {
        self.width = width;
        self.dirty = true;
    }

    pub fn set_show(&mut self, show: bool) {
        self.show = show;
        self.dirty = true;
    }

    pub fn set_material(&mut self, material: Material) {
        self.material = material;
        self.dirty = true;
    }

    pub fn set_id(&mut self, id: Option<String>) {
        self.id = id;
        self.dirty = true;
    }

    pub fn set_distance_display_condition(
        &mut self,
        condition: Option<DistanceDisplayCondition>,
    ) {
        self.distance_display_condition = condition;
        self.dirty = true;
    }

    /// Checks if this polyline has an alpha value greater than zero.
    pub fn is_visible_for_picking(&self) -> bool {
        if !self.show {
            return false;
        }
        if let Some(serde_json::Value::Object(map)) = self.material.uniforms.get("color") {
            if let Some(alpha_val) = map.get("alpha").and_then(|v| v.as_f64()) {
                if alpha_val <= 0.0 {
                    return false;
                }
            }
        }
        true
    }
}

/// A renderable collection of polylines.
#[derive(Clone, Debug)]
pub struct PolylineCollection {
    pub id: u64,
    pub polylines: Vec<Polyline>,
    pub model_matrix: Option<DMat4>,
    pub debug_show_bounding_volume: bool,
    pub is_destroyed: bool,
}

impl Default for PolylineCollection {
    fn default() -> Self {
        Self::new()
    }
}

impl PolylineCollection {
    /// Constructs an empty PolylineCollection.
    pub fn new() -> Self {
        Self {
            id: NEXT_COLLECTION_ID.fetch_add(1, Ordering::Relaxed),
            polylines: Vec::new(),
            model_matrix: None,
            debug_show_bounding_volume: false,
            is_destroyed: false,
        }
    }

    pub fn len(&self) -> usize {
        self.polylines.len()
    }

    pub fn is_empty(&self) -> bool {
        self.polylines.is_empty()
    }

    pub fn is_destroyed(&self) -> bool {
        self.is_destroyed
    }

    /// Adds a polyline with options and returns its index in the collection.
    pub fn add(&mut self, options: Option<PolylineOptions>) -> usize {
        let opts = options.unwrap_or_default();
        let mut p = Polyline::default();

        if let Some(show) = opts.show {
            p.show = show;
        }
        if let Some(pos) = opts.positions {
            p.set_positions(pos);
        }
        if let Some(loop_) = opts.loop_ {
            p.set_loop(loop_);
        }
        if let Some(width) = opts.width {
            p.width = width;
        }
        if let Some(mat) = opts.material {
            p.material = mat;
        }
        p.id = opts.id;
        p.distance_display_condition = opts.distance_display_condition;
        p.collection_id = Some(self.id);

        self.polylines.push(p);
        self.polylines.len() - 1
    }

    /// Gets a polyline by index. Throws DeveloperError if index is None or out of range.
    pub fn get<I: Into<Option<usize>>>(&self, index: I) -> Result<&Polyline, DeveloperError> {
        let idx = index
            .into()
            .ok_or_else(|| DeveloperError::new("index is required."))?;
        self.polylines
            .get(idx)
            .ok_or_else(|| DeveloperError::new("index out of range."))
    }

    /// Gets a mutable reference to a polyline by index.
    pub fn get_mut<I: Into<Option<usize>>>(
        &mut self,
        index: I,
    ) -> Result<&mut Polyline, DeveloperError> {
        let idx = index
            .into()
            .ok_or_else(|| DeveloperError::new("index is required."))?;
        self.polylines
            .get_mut(idx)
            .ok_or_else(|| DeveloperError::new("index out of range."))
    }

    /// Removes the polyline at the specified index.
    pub fn remove(&mut self, index: usize) -> bool {
        if index < self.polylines.len() {
            let mut removed = self.polylines.remove(index);
            removed.collection_id = None;
            true
        } else {
            false
        }
    }

    /// Removes the specified polyline from the collection.
    pub fn remove_polyline(&mut self, polyline: Option<&Polyline>) -> bool {
        let p = match polyline {
            Some(pt) => pt,
            None => return false,
        };
        let target_id = p.internal_id;
        if let Some(idx) = self
            .polylines
            .iter()
            .position(|it| it.internal_id == target_id)
        {
            let mut removed = self.polylines.remove(idx);
            removed.collection_id = None;
            true
        } else {
            false
        }
    }

    /// Removes all polylines from the collection.
    pub fn remove_all(&mut self) {
        for p in &mut self.polylines {
            p.collection_id = None;
        }
        self.polylines.clear();
    }

    /// Checks whether the collection contains the specified polyline.
    pub fn contains(&self, polyline: Option<&Polyline>) -> bool {
        let p = match polyline {
            Some(pt) => pt,
            None => return false,
        };
        self.polylines.iter().any(|it| it.internal_id == p.internal_id)
    }

    /// Computes the 3D bounding sphere encompassing all visible polylines.
    pub fn compute_bounding_sphere_3d(&self) -> Option<BoundingSphere> {
        let mut dvecs = Vec::new();
        let matrix = self.model_matrix.unwrap_or(DMat4::IDENTITY);

        for p in &self.polylines {
            if !p.show || p.positions.is_empty() {
                continue;
            }
            for pos in &p.positions {
                let transformed = matrix.transform_point3(DVec3::new(pos.x, pos.y, pos.z));
                dvecs.push(transformed);
            }
        }

        if dvecs.is_empty() {
            return None;
        }

        let mut sphere = BoundingSphere::from_points(&dvecs).ok()?;
        let max_width = self
            .polylines
            .iter()
            .map(|p| p.width)
            .fold(0.0f64, f64::max);
        sphere.radius += max_width * 0.5;
        Some(sphere)
    }

    /// Computes the 2D bounding sphere.
    pub fn compute_bounding_sphere_2d(&self, ellipsoid: &Ellipsoid) -> Option<BoundingSphere> {
        let mut projected = Vec::new();
        for p in &self.polylines {
            if !p.show || p.positions.is_empty() {
                continue;
            }
            for pos in &p.positions {
                if let Some(carto) = ellipsoid.cartesian_to_cartographic(DVec3::new(
                    pos.x, pos.y, pos.z,
                )) {
                    let x = carto.longitude * ellipsoid.radii().x;
                    let y = carto.latitude * ellipsoid.radii().x;
                    projected.push(DVec3::new(0.0, x, y));
                }
            }
        }

        if projected.is_empty() {
            return None;
        }

        let mut sphere = BoundingSphere::from_points(&projected).ok()?;
        sphere.radius += 10.0;
        Some(sphere)
    }

    /// Computes the Columbus View bounding sphere.
    pub fn compute_bounding_sphere_columbus_view(
        &self,
        ellipsoid: &Ellipsoid,
    ) -> Option<BoundingSphere> {
        let mut projected = Vec::new();
        for p in &self.polylines {
            if !p.show || p.positions.is_empty() {
                continue;
            }
            for pos in &p.positions {
                if let Some(carto) = ellipsoid.cartesian_to_cartographic(DVec3::new(
                    pos.x, pos.y, pos.z,
                )) {
                    let x = carto.longitude * ellipsoid.radii().x;
                    let y = carto.latitude * ellipsoid.radii().x;
                    projected.push(DVec3::new(x, y, carto.height));
                }
            }
        }

        if projected.is_empty() {
            return None;
        }

        let mut sphere = BoundingSphere::from_points(&projected).ok()?;
        sphere.radius += 10.0;
        Some(sphere)
    }

    /// Computes optimized bounding volumes partitioned by material.
    pub fn compute_bounding_spheres_per_material(&self) -> HashMap<String, BoundingSphere> {
        let mut groups: HashMap<String, Vec<DVec3>> = HashMap::new();
        for p in &self.polylines {
            if !p.show || p.positions.is_empty() {
                continue;
            }
            let key = p.material.material_type.clone();
            let entry = groups.entry(key).or_default();
            for pos in &p.positions {
                entry.push(DVec3::new(pos.x, pos.y, pos.z));
            }
        }

        let mut results = HashMap::new();
        for (mat_type, points) in groups {
            if let Ok(sphere) = BoundingSphere::from_points(&points) {
                results.insert(mat_type, sphere);
            }
        }
        results
    }

    /// Destroys WebGL resources.
    pub fn destroy(&mut self) {
        if !self.is_destroyed {
            self.remove_all();
            self.is_destroyed = true;
        }
    }
}
