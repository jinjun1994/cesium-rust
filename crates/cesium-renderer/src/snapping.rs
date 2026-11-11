//! Snapping matching CesiumJS `Scene/Snapping.js` (Apache-2.0).

use cesium_math::{Cartesian2, Cartesian3, DeveloperError};

/// Radius around crosshair, in pixels, used to sample the nearest surface (the occluder under cursor).
pub const SNAP_OCCLUDER_RADIUS_PIXELS: f64 = 3.0;

/// An edge deeper than this fraction beyond the occluder is treated as occluded.
pub const SNAP_OCCLUSION_TOLERANCE: f64 = 0.1;

/// A candidate hit from a snapping pass (either surface or silhouette edge).
#[derive(Clone, Debug, PartialEq)]
pub struct SnapHit {
    pub x: f64,
    pub y: f64,
    pub depth: f64,
    pub is_edge: bool,
    pub object_id: Option<u64>,
}

impl SnapHit {
    pub fn new(x: f64, y: f64, depth: f64, is_edge: bool, object_id: Option<u64>) -> Self {
        Self {
            x,
            y,
            depth,
            is_edge,
            object_id,
        }
    }

    pub fn surface(x: f64, y: f64, depth: f64, object_id: Option<u64>) -> Self {
        Self::new(x, y, depth, false, object_id)
    }

    pub fn edge(x: f64, y: f64, depth: f64, object_id: Option<u64>) -> Self {
        Self::new(x, y, depth, true, object_id)
    }

    pub fn cursor_distance(&self) -> f64 {
        (self.x * self.x + self.y * self.y).sqrt()
    }
}

/// Final result returned by snapping query.
#[derive(Clone, Debug, PartialEq)]
pub struct SceneSnapResult {
    pub position: Cartesian3,
    pub surface_position: Option<Cartesian3>,
    pub screen_position: Cartesian2,
    pub is_edge: bool,
    pub object_id: Option<u64>,
}

/// Snapping logic implementation matching CesiumJS `Snapping`.
pub struct Snapping;

impl Snapping {
    /// Selects the best candidate hit from a screen-space aperture.
    pub fn select_best_hit(hits: &[SnapHit]) -> Option<&SnapHit> {
        if hits.is_empty() {
            return None;
        }

        let mut occluder_depth = f64::INFINITY;
        for hit in hits {
            if !hit.is_edge && hit.cursor_distance() <= SNAP_OCCLUDER_RADIUS_PIXELS {
                occluder_depth = occluder_depth.min(hit.depth);
            }
        }
        let max_edge_depth = occluder_depth * (1.0 + SNAP_OCCLUSION_TOLERANCE);

        let visible: Vec<&SnapHit> = hits
            .iter()
            .filter(|hit| !hit.is_edge || hit.depth <= max_edge_depth)
            .collect();

        if visible.is_empty() {
            return None;
        }

        let want_edge = visible.iter().any(|hit| hit.is_edge);
        let group: Vec<&SnapHit> = visible
            .into_iter()
            .filter(|hit| hit.is_edge == want_edge)
            .collect();

        group.into_iter().min_by(|a, b| {
            a.cursor_distance()
                .partial_cmp(&b.cursor_distance())
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    }

    /// Finds the surface hit belonging to the same object that lies closest to the region center.
    pub fn nearest_surface_hit<'a>(hits: &'a [SnapHit], object_id: u64) -> Option<&'a SnapHit> {
        let mut best: Option<&'a SnapHit> = None;
        let mut best_dist = f64::INFINITY;

        for hit in hits {
            if hit.is_edge || hit.object_id != Some(object_id) {
                continue;
            }
            let dist = hit.x * hit.x + hit.y * hit.y;
            if dist < best_dist {
                best = Some(hit);
                best_dist = dist;
            }
        }
        best
    }

    /// Unprojects a snap hit's eye-space depth along a camera pick ray into a world position.
    pub fn snap_hit_to_world(
        camera_direction_wc: Cartesian3,
        ray_origin: Cartesian3,
        ray_direction: Option<Cartesian3>,
        hit: &SnapHit,
    ) -> Option<Cartesian3> {
        let dir = ray_direction?;
        let cos = dir.dot(&camera_direction_wc);
        if cos <= 0.0 {
            return None;
        }
        let t = hit.depth / cos;
        Some(ray_origin + dir * t)
    }

    /// Validates window position parameter for `snap`.
    pub fn validate_window_position(
        window_position: Option<Cartesian2>,
    ) -> Result<Cartesian2, DeveloperError> {
        window_position
            .ok_or_else(|| DeveloperError::new("windowPosition is required."))
    }
}
