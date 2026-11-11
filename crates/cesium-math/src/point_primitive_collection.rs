//! PointPrimitive and PointPrimitiveCollection matching CesiumJS `Scene/PointPrimitiveCollection.js` and `Scene/PointPrimitive.js` (Apache-2.0).

use std::sync::atomic::{AtomicU64, Ordering};

use crate::{
    BoundingRectangle, BoundingSphere, Cartesian2, Cartesian3, Color, DeveloperError,
    DistanceDisplayCondition, Ellipsoid, NearFarScalar, SplitDirection,
};

static NEXT_POINT_ID: AtomicU64 = AtomicU64::new(1);
static NEXT_COLLECTION_ID: AtomicU64 = AtomicU64::new(1);

/// Construction options for creating a PointPrimitive.
#[derive(Clone, Debug, PartialEq)]
pub struct PointPrimitiveOptions {
    pub show: Option<bool>,
    pub position: Option<Cartesian3>,
    pub pixel_size: Option<f64>,
    pub color: Option<Color>,
    pub outline_color: Option<Color>,
    pub outline_width: Option<f64>,
    pub scale_by_distance: Option<NearFarScalar>,
    pub translucency_by_distance: Option<NearFarScalar>,
    pub distance_display_condition: Option<DistanceDisplayCondition>,
    pub disable_depth_test_distance: Option<f64>,
    pub id: Option<String>,
    pub split_direction: Option<SplitDirection>,
}

impl Default for PointPrimitiveOptions {
    fn default() -> Self {
        Self {
            show: Some(true),
            position: Some(Cartesian3::ZERO),
            pixel_size: Some(10.0),
            color: Some(Color::new(1.0, 1.0, 1.0, 1.0)),
            outline_color: Some(Color::new(0.0, 0.0, 0.0, 0.0)),
            outline_width: Some(0.0),
            scale_by_distance: None,
            translucency_by_distance: None,
            distance_display_condition: None,
            disable_depth_test_distance: Some(0.0),
            id: None,
            split_direction: Some(SplitDirection::None),
        }
    }
}

/// A single point primitive representation with attribute validations.
#[derive(Clone, Debug)]
pub struct PointPrimitive {
    pub internal_id: u64,
    pub collection_id: Option<u64>,
    pub show: bool,
    pub position: Cartesian3,
    pub pixel_size: f64,
    pub color: Color,
    pub outline_color: Color,
    pub outline_width: f64,
    pub scale_by_distance: Option<NearFarScalar>,
    pub translucency_by_distance: Option<NearFarScalar>,
    pub distance_display_condition: Option<DistanceDisplayCondition>,
    pub disable_depth_test_distance: f64,
    pub id: Option<String>,
    pub split_direction: SplitDirection,
}

impl PartialEq for PointPrimitive {
    fn eq(&self, other: &Self) -> bool {
        self.internal_id == other.internal_id
    }
}

impl Default for PointPrimitive {
    fn default() -> Self {
        Self {
            internal_id: NEXT_POINT_ID.fetch_add(1, Ordering::Relaxed),
            collection_id: None,
            show: true,
            position: Cartesian3::ZERO,
            pixel_size: 10.0,
            color: Color::new(1.0, 1.0, 1.0, 1.0),
            outline_color: Color::new(0.0, 0.0, 0.0, 0.0),
            outline_width: 0.0,
            scale_by_distance: None,
            translucency_by_distance: None,
            distance_display_condition: None,
            disable_depth_test_distance: 0.0,
            id: None,
            split_direction: SplitDirection::None,
        }
    }
}

impl PointPrimitive {
    /// Validates and sets `scale_by_distance`.
    pub fn set_scale_by_distance(
        &mut self,
        scale: Option<NearFarScalar>,
    ) -> Result<(), DeveloperError> {
        if let Some(s) = scale {
            if s.near >= s.far {
                return Err(DeveloperError::new(
                    "scaleByDistance.near must be less than scaleByDistance.far.",
                ));
            }
        }
        self.scale_by_distance = scale;
        Ok(())
    }

    /// Validates and sets `translucency_by_distance`.
    pub fn set_translucency_by_distance(
        &mut self,
        translucency: Option<NearFarScalar>,
    ) -> Result<(), DeveloperError> {
        if let Some(t) = translucency {
            if t.near >= t.far {
                return Err(DeveloperError::new(
                    "translucencyByDistance.near must be less than translucencyByDistance.far.",
                ));
            }
        }
        self.translucency_by_distance = translucency;
        Ok(())
    }

    /// Validates and sets `distance_display_condition`.
    pub fn set_distance_display_condition(
        &mut self,
        condition: Option<DistanceDisplayCondition>,
    ) -> Result<(), DeveloperError> {
        if let Some(c) = condition {
            if c.near >= c.far {
                return Err(DeveloperError::new(
                    "distanceDisplayCondition.near must be less than distanceDisplayCondition.far.",
                ));
            }
        }
        self.distance_display_condition = condition;
        Ok(())
    }

    /// Validates and sets `disable_depth_test_distance`.
    pub fn set_disable_depth_test_distance(&mut self, dist: f64) -> Result<(), DeveloperError> {
        if dist < 0.0 {
            return Err(DeveloperError::new(
                "disableDepthTestDistance must be greater than or equal to 0.0.",
            ));
        }
        self.disable_depth_test_distance = dist;
        Ok(())
    }

    /// Computes the 2D screen space position of the point.
    pub fn compute_screen_space_position(
        &self,
        camera_pos: Option<Cartesian3>,
        viewport_width: f64,
        viewport_height: f64,
        result: Option<&mut Cartesian2>,
    ) -> Result<Cartesian2, DeveloperError> {
        if self.collection_id.is_none() {
            return Err(DeveloperError::new(
                "This object was destroyed, i.e., destroy() was called.",
            ));
        }
        let cam = camera_pos.ok_or_else(|| DeveloperError::new("scene is required."))?;

        let dx = self.position.x - cam.x;
        let dy = self.position.y - cam.y;
        let screen_x = viewport_width * 0.5 + dx;
        let screen_y = viewport_height * 0.5 - dy;

        let res = Cartesian2::new(screen_x, screen_y);
        if let Some(out) = result {
            *out = res;
        }
        Ok(res)
    }

    /// Computes the screen space bounding box for picking or collision testing.
    pub fn compute_screen_space_bounding_box(
        &self,
        screen_pos: Cartesian2,
        result: Option<&mut BoundingRectangle>,
    ) -> BoundingRectangle {
        let total_size = self.pixel_size + self.outline_width * 2.0;
        let half_size = total_size * 0.5;
        let rect = BoundingRectangle::new(
            screen_pos.x - half_size,
            screen_pos.y - half_size,
            total_size,
            total_size,
        );
        if let Some(out) = result {
            *out = rect;
        }
        rect
    }
}

/// A renderable collection of point primitives.
#[derive(Clone, Debug)]
pub struct PointPrimitiveCollection {
    pub id: u64,
    pub show: bool,
    pub point_primitives: Vec<PointPrimitive>,
    pub is_destroyed: bool,
    pub debug_show_bounding_volume: bool,
}

impl Default for PointPrimitiveCollection {
    fn default() -> Self {
        Self::new()
    }
}

impl PointPrimitiveCollection {
    /// Constructs an empty PointPrimitiveCollection.
    pub fn new() -> Self {
        Self {
            id: NEXT_COLLECTION_ID.fetch_add(1, Ordering::Relaxed),
            show: true,
            point_primitives: Vec::new(),
            is_destroyed: false,
            debug_show_bounding_volume: false,
        }
    }

    pub fn len(&self) -> usize {
        self.point_primitives.len()
    }

    pub fn is_empty(&self) -> bool {
        self.point_primitives.is_empty()
    }

    pub fn is_destroyed(&self) -> bool {
        self.is_destroyed
    }

    /// Adds a point primitive with options and returns its internal index.
    pub fn add(
        &mut self,
        options: Option<PointPrimitiveOptions>,
    ) -> Result<usize, DeveloperError> {
        let opts = options.unwrap_or_default();
        let mut p = PointPrimitive::default();

        if let Some(show) = opts.show {
            p.show = show;
        }
        if let Some(pos) = opts.position {
            p.position = pos;
        }
        if let Some(size) = opts.pixel_size {
            p.pixel_size = size;
        }
        if let Some(color) = opts.color {
            p.color = color;
        }
        if let Some(outline_color) = opts.outline_color {
            p.outline_color = outline_color;
        }
        if let Some(outline_width) = opts.outline_width {
            p.outline_width = outline_width;
        }
        if let Some(scale) = opts.scale_by_distance {
            p.set_scale_by_distance(Some(scale))?;
        }
        if let Some(translucency) = opts.translucency_by_distance {
            p.set_translucency_by_distance(Some(translucency))?;
        }
        if let Some(condition) = opts.distance_display_condition {
            p.set_distance_display_condition(Some(condition))?;
        }
        if let Some(disable_depth) = opts.disable_depth_test_distance {
            p.set_disable_depth_test_distance(disable_depth)?;
        }
        p.id = opts.id;
        if let Some(split) = opts.split_direction {
            p.split_direction = split;
        }

        p.collection_id = Some(self.id);
        self.point_primitives.push(p);
        Ok(self.point_primitives.len() - 1)
    }

    /// Adds an existing PointPrimitive directly.
    pub fn add_point(&mut self, mut point: PointPrimitive) -> usize {
        point.collection_id = Some(self.id);
        self.point_primitives.push(point);
        self.point_primitives.len() - 1
    }

    /// Removes a point primitive by internal id or index.
    pub fn remove_at(&mut self, index: usize) -> bool {
        if index < self.point_primitives.len() {
            let mut removed = self.point_primitives.remove(index);
            removed.collection_id = None;
            true
        } else {
            false
        }
    }

    /// Removes a point primitive matching the internal ID.
    pub fn remove_point(&mut self, point: Option<&PointPrimitive>) -> bool {
        let p = match point {
            Some(pt) => pt,
            None => return false,
        };
        let target_id = p.internal_id;
        if let Some(idx) = self
            .point_primitives
            .iter()
            .position(|it| it.internal_id == target_id)
        {
            let mut removed = self.point_primitives.remove(idx);
            removed.collection_id = None;
            true
        } else {
            false
        }
    }

    /// Removes all point primitives from the collection.
    pub fn remove_all(&mut self) {
        for p in &mut self.point_primitives {
            p.collection_id = None;
        }
        self.point_primitives.clear();
    }

    /// Checks whether the collection contains the specified point primitive.
    pub fn contains_point(&self, point: Option<&PointPrimitive>) -> bool {
        let p = match point {
            Some(pt) => pt,
            None => return false,
        };
        self.point_primitives.iter().any(|it| it.internal_id == p.internal_id)
    }

    /// Gets a reference to the point primitive at the specified index.
    pub fn get<I: Into<Option<usize>>>(&self, index: I) -> Result<&PointPrimitive, DeveloperError> {
        let idx = index
            .into()
            .ok_or_else(|| DeveloperError::new("index is required."))?;
        self.point_primitives
            .get(idx)
            .ok_or_else(|| DeveloperError::new("index out of range."))
    }

    /// Gets a mutable reference to the point primitive at the specified index.
    pub fn get_mut<I: Into<Option<usize>>>(
        &mut self,
        index: I,
    ) -> Result<&mut PointPrimitive, DeveloperError> {
        let idx = index
            .into()
            .ok_or_else(|| DeveloperError::new("index is required."))?;
        self.point_primitives
            .get_mut(idx)
            .ok_or_else(|| DeveloperError::new("index out of range."))
    }

    /// Computes the 3D bounding sphere encompassing all visible points.
    pub fn compute_bounding_sphere_3d(&self) -> Option<BoundingSphere> {
        let dvecs: Vec<glam::DVec3> = self
            .point_primitives
            .iter()
            .filter(|p| p.show)
            .map(|p| glam::DVec3::new(p.position.x, p.position.y, p.position.z))
            .collect();

        if dvecs.is_empty() {
            return None;
        }

        let mut sphere = BoundingSphere::from_points(&dvecs).ok()?;
        let max_pixel_size = self
            .point_primitives
            .iter()
            .map(|p| p.pixel_size + p.outline_width * 2.0)
            .fold(0.0f64, f64::max);
        sphere.radius += max_pixel_size * 0.5;
        Some(sphere)
    }

    /// Computes the 2D bounding sphere with projected points.
    pub fn compute_bounding_sphere_2d(&self, ellipsoid: &Ellipsoid) -> Option<BoundingSphere> {
        let mut projected = Vec::new();
        for p in &self.point_primitives {
            if !p.show {
                continue;
            }
            if let Some(carto) = ellipsoid.cartesian_to_cartographic(glam::DVec3::new(
                p.position.x,
                p.position.y,
                p.position.z,
            )) {
                let x = carto.longitude * ellipsoid.radii().x;
                let y = carto.latitude * ellipsoid.radii().x;
                projected.push(glam::DVec3::new(0.0, x, y));
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
        for p in &self.point_primitives {
            if !p.show {
                continue;
            }
            if let Some(carto) = ellipsoid.cartesian_to_cartographic(glam::DVec3::new(
                p.position.x,
                p.position.y,
                p.position.z,
            )) {
                let x = carto.longitude * ellipsoid.radii().x;
                let y = carto.latitude * ellipsoid.radii().x;
                projected.push(glam::DVec3::new(x, y, carto.height));
            }
        }

        if projected.is_empty() {
            return None;
        }

        let mut sphere = BoundingSphere::from_points(&projected).ok()?;
        sphere.radius += 10.0;
        Some(sphere)
    }

    /// Destroys the collection and invalidates all child references.
    pub fn destroy(&mut self) {
        if !self.is_destroyed {
            self.remove_all();
            self.is_destroyed = true;
        }
    }
}
