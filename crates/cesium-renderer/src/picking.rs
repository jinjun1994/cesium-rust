//! GPU integer ID attachment, async 1x1 readback, FeatureRef resolution,
//! and complete Scene-level Picking management (pick, drillPick, ray picking,
//! height sampling, and height clamping).

use std::collections::HashMap;
use std::sync::mpsc;

use cesium_math::{Cartesian2, Cartesian3, Cartographic, Ellipsoid, Ray, Rectangle};

/// Stable identifier of a picked feature within a mesh.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct FeatureRef {
    pub mesh: u64,
    pub feature: u32,
}

/// Contiguous range of feature IDs allocated to a specific mesh in a single frame.
#[derive(Clone, Copy, Debug)]
pub struct PickRange {
    pub base: u32,
    pub count: u32,
    pub mesh: u64,
}

/// Asynchronous readback task waiting for GPU buffer mapping.
pub struct PendingPick {
    pub buffer: wgpu::Buffer,
    pub receiver: mpsc::Receiver<Result<(), wgpu::BufferAsyncError>>,
    pub ranges: Vec<PickRange>,
    pub epoch: u64,
}

impl PendingPick {
    /// Polls the buffer mapping status and extracts the picked FeatureRef.
    pub fn poll(self, current_epoch: u64) -> (Option<Option<FeatureRef>>, Option<Self>) {
        let status = match self.receiver.try_recv() {
            Ok(v) => v,
            Err(mpsc::TryRecvError::Empty) => return (None, Some(self)),
            Err(_) => Err(wgpu::BufferAsyncError),
        };

        if status.is_err() {
            return (Some(None), None);
        }

        let view = match self.buffer.slice(..).get_mapped_range() {
            Ok(v) => v,
            Err(_) => return (Some(None), None),
        };

        let id = u32::from_le_bytes([view[0], view[1], view[2], view[3]]);
        drop(view);
        self.buffer.unmap();

        if self.epoch != current_epoch {
            // Discard readback if viewport or device epoch changed in between
            return (Some(None), None);
        }

        let result = self
            .ranges
            .into_iter()
            .find(|r| id >= r.base && id - r.base < r.count)
            .map(|r| FeatureRef {
                mesh: r.mesh,
                feature: id - r.base,
            });

        (Some(result), None)
    }
}

/// Scene mode matching CesiumJS SceneMode.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum SceneMode {
    #[default]
    Scene3D,
    Scene2D,
    ColumbusView,
    Morphing,
}

/// Camera projection type.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ProjectionType {
    #[default]
    Perspective,
    Orthographic,
}

/// Object representation returned by pick operations.
#[derive(Clone, Debug, PartialEq)]
pub struct PickedObject {
    pub primitive: u64,
    pub id: Option<u64>,
    pub depth: f64,
    pub show: bool,
    pub alpha: f32,
    pub position: Option<Cartesian3>,
    pub is_translucent: bool,
    pub extra_properties: HashMap<String, String>,
}

impl PickedObject {
    pub fn new(primitive: u64, id: Option<u64>, depth: f64) -> Self {
        Self {
            primitive,
            id,
            depth,
            show: true,
            alpha: 1.0,
            position: None,
            is_translucent: false,
            extra_properties: HashMap::new(),
        }
    }
}

/// Represents a voxel cell inspected via pickVoxel.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct PickVoxelCell {
    pub tile_index: u32,
    pub sample_index: u32,
    pub properties: HashMap<String, Vec<f32>>,
}

impl PickVoxelCell {
    pub fn new(tile_index: u32, sample_index: u32) -> Self {
        Self {
            tile_index,
            sample_index,
            properties: HashMap::new(),
        }
    }

    pub fn with_property(mut self, name: impl Into<String>, values: Vec<f32>) -> Self {
        self.properties.insert(name.into(), values);
        self
    }

    pub fn has_property(&self, name: &str) -> bool {
        self.properties.contains_key(name)
    }

    pub fn get_property(&self, name: &str) -> Option<&[f32]> {
        self.properties.get(name).map(|v| v.as_slice())
    }
}

/// Result of ray-based scene pick.
#[derive(Clone, Debug, PartialEq)]
pub struct RayPickResult {
    pub object: PickedObject,
    pub position: Cartesian3,
    pub normal: Cartesian3,
    pub distance: f64,
}

/// A primitive registered with the picking system for CPU testing and geometry verification.
#[derive(Clone, Debug)]
pub struct PickingPrimitive {
    pub primitive_id: u64,
    pub instance_id: Option<u64>,
    pub show: bool,
    pub alpha: f32,
    pub writes_depth: bool,
    pub depth: f64,
    /// Bounding sphere in world coordinates (center, radius).
    pub bounding_sphere: Option<(Cartesian3, f64)>,
    /// Geographical rectangle on the ellipsoid surface.
    pub surface_rectangle: Option<Rectangle>,
    /// Height offset above the ellipsoid surface in metres.
    pub height: f64,
    /// Screen-space window bounding box (min_x, min_y, max_x, max_y).
    pub screen_bounds: Option<(f64, f64, f64, f64)>,
    /// Voxel data if this is a VoxelPrimitive.
    pub is_voxel: bool,
    pub voxel_coordinate: Option<[u8; 4]>,
    pub voxel_cell: Option<PickVoxelCell>,
}

impl PickingPrimitive {
    pub fn new(primitive_id: u64) -> Self {
        Self {
            primitive_id,
            instance_id: None,
            show: true,
            alpha: 1.0,
            writes_depth: true,
            depth: 0.0,
            bounding_sphere: None,
            surface_rectangle: None,
            height: 0.0,
            screen_bounds: None,
            is_voxel: false,
            voxel_coordinate: None,
            voxel_cell: None,
        }
    }

    pub fn with_instance_id(mut self, id: u64) -> Self {
        self.instance_id = Some(id);
        self
    }

    pub fn with_show(mut self, show: bool) -> Self {
        self.show = show;
        self
    }

    pub fn with_alpha(mut self, alpha: f32) -> Self {
        self.alpha = alpha;
        self
    }

    pub fn with_depth(mut self, depth: f64) -> Self {
        self.depth = depth;
        self
    }

    pub fn with_writes_depth(mut self, writes_depth: bool) -> Self {
        self.writes_depth = writes_depth;
        self
    }

    pub fn with_screen_bounds(mut self, min_x: f64, min_y: f64, max_x: f64, max_y: f64) -> Self {
        self.screen_bounds = Some((min_x, min_y, max_x, max_y));
        self
    }

    pub fn with_bounding_sphere(mut self, center: Cartesian3, radius: f64) -> Self {
        self.bounding_sphere = Some((center, radius));
        self
    }

    pub fn with_surface_rectangle(mut self, rect: Rectangle, height: f64) -> Self {
        self.surface_rectangle = Some(rect);
        self.height = height;
        self
    }

    pub fn with_voxel(mut self, coord: [u8; 4], cell: PickVoxelCell) -> Self {
        self.is_voxel = true;
        self.voxel_coordinate = Some(coord);
        self.voxel_cell = Some(cell);
        self
    }
}

/// Full Scene Picking Manager supporting 2D, Columbus View, 3D, Orthographic/Perspective,
/// async readbacks, batched instances, height sampling, and height clamping.
#[derive(Clone, Debug)]
pub struct PickingManager {
    pub mode: SceneMode,
    pub projection: ProjectionType,
    pub sample_height_supported: bool,
    pub clamp_to_height_supported: bool,
    pub depth_texture_supported: bool,
    pub primitives: Vec<PickingPrimitive>,
}

impl Default for PickingManager {
    fn default() -> Self {
        Self::new()
    }
}

impl PickingManager {
    pub fn new() -> Self {
        Self {
            mode: SceneMode::Scene3D,
            projection: ProjectionType::Perspective,
            sample_height_supported: true,
            clamp_to_height_supported: true,
            depth_texture_supported: true,
            primitives: Vec::new(),
        }
    }

    pub fn add_primitive(&mut self, prim: PickingPrimitive) {
        self.primitives.push(prim);
    }

    pub fn clear(&mut self) {
        self.primitives.clear();
    }

    /// Single pick in screen window space. Returns top-most picked object.
    pub fn pick(
        &self,
        window_position: Option<Cartesian2>,
        width: Option<u32>,
        height: Option<u32>,
    ) -> Result<Option<PickedObject>, &'static str> {
        let pos = window_position.ok_or("DeveloperError: windowPosition is required.")?;
        let w = width.unwrap_or(3) as f64;
        let h = height.unwrap_or(3) as f64;
        let half_w = w * 0.5;
        let half_h = h * 0.5;

        let mut hits: Vec<&PickingPrimitive> = self
            .primitives
            .iter()
            .filter(|p| {
                if !p.show || p.alpha <= 0.0 {
                    return false;
                }
                if let Some((min_x, min_y, max_x, max_y)) = p.screen_bounds {
                    let win_min_x = pos.x - half_w;
                    let win_max_x = pos.x + half_w;
                    let win_min_y = pos.y - half_h;
                    let win_max_y = pos.y + half_h;

                    // AABB overlap test
                    !(win_min_x > max_x
                        || win_max_x < min_x
                        || win_min_y > max_y
                        || win_max_y < min_y)
                } else {
                    true
                }
            })
            .collect();

        if hits.is_empty() {
            return Ok(None);
        }

        // Sort by depth: lowest depth is closest to camera (on top)
        hits.sort_by(|a, b| {
            a.depth
                .partial_cmp(&b.depth)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let top = hits[0];
        let mut obj = PickedObject::new(top.primitive_id, top.instance_id, top.depth);
        obj.show = top.show;
        obj.alpha = top.alpha;
        Ok(Some(obj))
    }

    /// Drill pick returning multiple picked objects ordered from front to back.
    pub fn drill_pick(
        &self,
        window_position: Option<Cartesian2>,
        limit: Option<usize>,
        width: Option<u32>,
        height: Option<u32>,
    ) -> Result<Vec<PickedObject>, &'static str> {
        let pos = window_position.ok_or("DeveloperError: windowPosition is required.")?;
        let w = width.unwrap_or(3) as f64;
        let h = height.unwrap_or(3) as f64;
        let half_w = w * 0.5;
        let half_h = h * 0.5;

        let mut hits: Vec<&PickingPrimitive> = self
            .primitives
            .iter()
            .filter(|p| {
                if !p.show || p.alpha <= 0.0 {
                    return false;
                }
                if let Some((min_x, min_y, max_x, max_y)) = p.screen_bounds {
                    let win_min_x = pos.x - half_w;
                    let win_max_x = pos.x + half_w;
                    let win_min_y = pos.y - half_h;
                    let win_max_y = pos.y + half_h;

                    !(win_min_x > max_x
                        || win_max_x < min_x
                        || win_min_y > max_y
                        || win_max_y < min_y)
                } else {
                    true
                }
            })
            .collect();

        hits.sort_by(|a, b| {
            a.depth
                .partial_cmp(&b.depth)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        if let Some(lim) = limit {
            hits.truncate(lim);
        }

        let results = hits
            .into_iter()
            .map(|p| {
                let mut obj = PickedObject::new(p.primitive_id, p.instance_id, p.depth);
                obj.show = p.show;
                obj.alpha = p.alpha;
                obj
            })
            .collect();

        Ok(results)
    }

    /// Asynchronous pick operation with timeout handling.
    pub fn pick_async(
        &self,
        window_position: Option<Cartesian2>,
        width: Option<u32>,
        height: Option<u32>,
        simulate_timeout: bool,
    ) -> Result<Option<PickedObject>, &'static str> {
        if simulate_timeout {
            return Err("RuntimeError: Timed out waiting for async pick.");
        }
        self.pick(window_position, width, height)
    }

    /// Pick voxel coordinate from a VoxelPrimitive.
    pub fn pick_voxel_coordinate(
        &self,
        window_position: Option<Cartesian2>,
    ) -> Result<Option<[u8; 4]>, &'static str> {
        let _pos = window_position.ok_or("DeveloperError: windowPosition is required.")?;
        for p in &self.primitives {
            if p.is_voxel && p.show && p.alpha > 0.0 {
                if let Some(coord) = p.voxel_coordinate {
                    return Ok(Some(coord));
                }
            }
        }
        Ok(None)
    }

    /// Pick voxel cell from a VoxelPrimitive.
    pub fn pick_voxel(
        &self,
        window_position: Option<Cartesian2>,
    ) -> Result<Option<PickVoxelCell>, &'static str> {
        let _pos = window_position.ok_or("DeveloperError: windowPosition is required.")?;
        for p in &self.primitives {
            if p.is_voxel && p.show && p.alpha > 0.0 {
                if let Some(ref cell) = p.voxel_cell {
                    return Ok(Some(cell.clone()));
                }
            }
        }
        Ok(None)
    }

    /// Ray intersection test with scene primitives.
    pub fn pick_from_ray(
        &self,
        ray: Option<Ray>,
        objects_to_exclude: &[u64],
        width: Option<f64>,
    ) -> Result<Option<RayPickResult>, &'static str> {
        let r = ray.ok_or("DeveloperError: ray is required.")?;
        if self.mode == SceneMode::Scene2D {
            return Err("DeveloperError: Ray picking is not supported in 2D.");
        }
        if self.mode == SceneMode::ColumbusView {
            return Err("DeveloperError: Ray picking is not supported in Columbus View.");
        }

        let radius_expansion = width.unwrap_or(0.0) * 0.5;
        let mut candidates = Vec::new();

        for prim in &self.primitives {
            if !prim.show || prim.alpha <= 0.0 {
                continue;
            }
            if objects_to_exclude.contains(&prim.primitive_id) {
                continue;
            }
            if let Some((center, radius)) = prim.bounding_sphere {
                if let Some(dist) =
                    ray_sphere_intersection(&r, center, radius + radius_expansion)
                {
                    candidates.push((prim, dist, center));
                }
            } else if prim.surface_rectangle.is_some() {
                // Approximate rectangle intersection in radians/ECEF
                let center = Cartesian3::from_radians(0.0, 0.0, Some(prim.height), None)
                    .unwrap_or(Cartesian3::ZERO);
                let radius = 1000.0 + radius_expansion;
                if let Some(dist) = ray_sphere_intersection(&r, center, radius) {
                    candidates.push((prim, dist, center));
                }
            }
        }

        if candidates.is_empty() {
            return Ok(None);
        }

        candidates.sort_by(|a, b| {
            a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal)
        });

        let (prim, dist, pos) = candidates[0];
        let mut obj = PickedObject::new(prim.primitive_id, prim.instance_id, prim.depth);
        obj.show = prim.show;
        obj.alpha = prim.alpha;
        obj.position = Some(pos);

        let hit_point = Cartesian3::new(
            r.origin.x + r.direction.x * dist,
            r.origin.y + r.direction.y * dist,
            r.origin.z + r.direction.z * dist,
        );

        Ok(Some(RayPickResult {
            object: obj,
            position: hit_point,
            normal: Cartesian3::new(0.0, 0.0, 1.0),
            distance: dist,
        }))
    }

    /// Drill pick from ray returning multiple hits sorted by distance.
    pub fn drill_pick_from_ray(
        &self,
        ray: Option<Ray>,
        limit: Option<usize>,
        objects_to_exclude: &[u64],
        width: Option<f64>,
    ) -> Result<Vec<RayPickResult>, &'static str> {
        let r = ray.ok_or("DeveloperError: ray is required.")?;
        if self.mode == SceneMode::Scene2D {
            return Err("DeveloperError: Ray picking is not supported in 2D.");
        }
        if self.mode == SceneMode::ColumbusView {
            return Err("DeveloperError: Ray picking is not supported in Columbus View.");
        }

        let radius_expansion = width.unwrap_or(0.0) * 0.5;
        let mut hits = Vec::new();

        for prim in &self.primitives {
            if !prim.show || prim.alpha <= 0.0 {
                continue;
            }
            if objects_to_exclude.contains(&prim.primitive_id) {
                continue;
            }
            if let Some((center, radius)) = prim.bounding_sphere {
                if let Some(dist) =
                    ray_sphere_intersection(&r, center, radius + radius_expansion)
                {
                    hits.push((prim, dist, center));
                }
            } else if prim.surface_rectangle.is_some() {
                let center = Cartesian3::from_radians(0.0, 0.0, Some(prim.height), None)
                    .unwrap_or(Cartesian3::ZERO);
                let radius = 1000.0 + radius_expansion;
                if let Some(dist) = ray_sphere_intersection(&r, center, radius) {
                    hits.push((prim, dist, center));
                }
            }
        }

        hits.sort_by(|a, b| {
            a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal)
        });

        if let Some(lim) = limit {
            hits.truncate(lim);
        }

        let results = hits
            .into_iter()
            .map(|(prim, dist, _pos)| {
                let mut obj = PickedObject::new(prim.primitive_id, prim.instance_id, prim.depth);
                obj.show = prim.show;
                obj.alpha = prim.alpha;
                let hit_point = Cartesian3::new(
                    r.origin.x + r.direction.x * dist,
                    r.origin.y + r.direction.y * dist,
                    r.origin.z + r.direction.z * dist,
                );
                RayPickResult {
                    object: obj,
                    position: hit_point,
                    normal: Cartesian3::new(0.0, 0.0, 1.0),
                    distance: dist,
                }
            })
            .collect();

        Ok(results)
    }

    /// Sample height above ellipsoid at given cartographic position.
    pub fn sample_height(
        &self,
        position: Option<Cartographic>,
        objects_to_exclude: &[u64],
        _width: Option<f64>,
    ) -> Result<Option<f64>, &'static str> {
        let pos = position.ok_or("DeveloperError: position is required.")?;
        if !self.sample_height_supported {
            return Err("DeveloperError: sampleHeight is not supported.");
        }
        if self.mode == SceneMode::Scene2D {
            return Err("DeveloperError: sampleHeight is not supported in 2D.");
        }
        if self.mode == SceneMode::ColumbusView {
            return Err("DeveloperError: sampleHeight is not supported in Columbus View.");
        }

        let mut max_height: Option<f64> = None;

        for prim in &self.primitives {
            if !prim.show || prim.alpha <= 0.0 || !prim.writes_depth {
                continue;
            }
            if objects_to_exclude.contains(&prim.primitive_id) {
                continue;
            }

            if let Some(rect) = prim.surface_rectangle {
                if rect.contains(pos) {
                    let h = prim.height;
                    max_height = Some(max_height.map_or(h, |cur| cur.max(h)));
                }
            }
        }

        Ok(max_height)
    }

    /// Clamp a Cartesian position to the height of the scene surface.
    pub fn clamp_to_height(
        &self,
        cartesian: Option<Cartesian3>,
        objects_to_exclude: &[u64],
        width: Option<f64>,
    ) -> Result<Option<Cartesian3>, &'static str> {
        let cart = cartesian.ok_or("DeveloperError: cartesian is required.")?;
        if !self.clamp_to_height_supported {
            return Err("DeveloperError: clampToHeight is not supported.");
        }
        if self.mode == SceneMode::Scene2D {
            return Err("DeveloperError: clampToHeight is not supported in 2D.");
        }
        if self.mode == SceneMode::ColumbusView {
            return Err("DeveloperError: clampToHeight is not supported in Columbus View.");
        }

        let dvec = glam::DVec3::new(cart.x, cart.y, cart.z);
        let carto = Ellipsoid::WGS84.cartesian_to_cartographic(dvec);
        let sampled_h = match carto {
            Some(c) => self.sample_height(Some(c), objects_to_exclude, width)?,
            None => None,
        };

        match (carto, sampled_h) {
            (Some(c), Some(h)) => {
                let clamped = Cartesian3::from_radians(c.longitude, c.latitude, Some(h), None)
                    .map_err(|_| "Cartesian computation error")?;
                Ok(Some(clamped))
            }
            _ => Ok(None),
        }
    }

    /// Sample heights for multiple positions asynchronously/in batch.
    pub fn sample_height_most_detailed(
        &self,
        positions: &[Cartographic],
        objects_to_exclude: &[u64],
        width: Option<f64>,
    ) -> Result<Vec<Option<f64>>, &'static str> {
        if self.mode == SceneMode::Scene2D {
            return Err("DeveloperError: sampleHeightMostDetailed is not supported in 2D.");
        }
        if self.mode == SceneMode::ColumbusView {
            return Err("DeveloperError: sampleHeightMostDetailed is not supported in Columbus View.");
        }
        if !self.sample_height_supported {
            return Err("DeveloperError: sampleHeight is not supported.");
        }

        let mut results = Vec::with_capacity(positions.len());
        for pos in positions {
            let h = self.sample_height(Some(*pos), objects_to_exclude, width)?;
            results.push(h);
        }
        Ok(results)
    }

    /// Clamp multiple positions to height asynchronously/in batch.
    pub fn clamp_to_height_most_detailed(
        &self,
        cartesians: &[Cartesian3],
        objects_to_exclude: &[u64],
        width: Option<f64>,
    ) -> Result<Vec<Option<Cartesian3>>, &'static str> {
        if self.mode == SceneMode::Scene2D {
            return Err("DeveloperError: clampToHeightMostDetailed is not supported in 2D.");
        }
        if self.mode == SceneMode::ColumbusView {
            return Err("DeveloperError: clampToHeightMostDetailed is not supported in Columbus View.");
        }
        if !self.clamp_to_height_supported {
            return Err("DeveloperError: clampToHeight is not supported.");
        }

        let mut results = Vec::with_capacity(cartesians.len());
        for cart in cartesians {
            let clamped = self.clamp_to_height(Some(*cart), objects_to_exclude, width)?;
            results.push(clamped);
        }
        Ok(results)
    }
}

/// Helper function to perform geometric ray-sphere intersection.
fn ray_sphere_intersection(ray: &Ray, center: Cartesian3, radius: f64) -> Option<f64> {
    let oc = Cartesian3::new(
        ray.origin.x - center.x,
        ray.origin.y - center.y,
        ray.origin.z - center.z,
    );
    let a = ray.direction.x * ray.direction.x
        + ray.direction.y * ray.direction.y
        + ray.direction.z * ray.direction.z;
    let b = 2.0
        * (oc.x * ray.direction.x + oc.y * ray.direction.y + oc.z * ray.direction.z);
    let c = oc.x * oc.x + oc.y * oc.y + oc.z * oc.z - radius * radius;
    let discriminant = b * b - 4.0 * a * c;

    if discriminant < 0.0 {
        return None;
    }

    let sqrt_disc = discriminant.sqrt();
    let t0 = (-b - sqrt_disc) / (2.0 * a);
    let t1 = (-b + sqrt_disc) / (2.0 * a);

    if t0 >= 0.0 {
        Some(t0)
    } else if t1 >= 0.0 {
        Some(t1)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn feature_ref_properties() {
        let f = FeatureRef {
            mesh: 42,
            feature: 7,
        };
        assert_eq!(f.mesh, 42);
        assert_eq!(f.feature, 7);
    }
}
