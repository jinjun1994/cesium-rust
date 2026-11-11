//! Primitive culling logic matching CesiumJS `Scene/PrimitiveCulling.js` (Apache-2.0).

use cesium_math::{BoundingSphere, CullingVolume, Intersect, Occluder};

/// Utility for frustum culling and horizon occlusion culling of primitives.
pub struct PrimitiveCuller;

impl PrimitiveCuller {
    /// Determines whether a bounding sphere is visible within the culling volume and not occluded.
    pub fn is_visible(
        bounding_sphere: &BoundingSphere,
        culling_volume: &CullingVolume,
        occluder: Option<&Occluder>,
    ) -> bool {
        // Frustum culling
        let visibility = culling_volume.compute_visibility(bounding_sphere);
        if visibility == Intersect::Outside {
            return false;
        }

        // Horizon occlusion culling
        if let Some(occ) = occluder {
            if !occ.is_bounding_sphere_visible(*bounding_sphere) {
                return false;
            }
        }

        true
    }
}
