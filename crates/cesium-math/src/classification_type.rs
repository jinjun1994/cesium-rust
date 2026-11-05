//! ClassificationType matching CesiumJS `Scene/ClassificationType.js` (Apache-2.0).

/// Whether a classification affects terrain, 3D Tiles or both.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ClassificationType {
    Terrain = 0,
    Cesium3DTile = 1,
    #[default]
    Both = 2,
}
