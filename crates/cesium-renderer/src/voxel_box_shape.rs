use cesium_math::{Cartesian3, Matrix4, OrientedBoundingBox};
use glam::{DMat3, DVec3};

/// Trait implemented by voxel shapes capable of calculating tile bounding volumes.
pub trait VoxelShape {
    /// Computes the oriented bounding box containing the specified tile coordinate.
    fn compute_oriented_bounding_box_for_tile(
        &self,
        tile_level: u32,
        tile_x: u32,
        tile_y: u32,
        tile_z: u32,
    ) -> OrientedBoundingBox;
}

/// A box-shaped voxel region with configurable bounds and transform.
#[derive(Debug, Clone, PartialEq)]
pub struct VoxelBoxShape {
    pub min_bounds: Cartesian3,
    pub max_bounds: Cartesian3,
    pub model_matrix: Matrix4,
}

impl Default for VoxelBoxShape {
    fn default() -> Self {
        Self::new()
    }
}

impl VoxelBoxShape {
    pub const DEFAULT_MIN_BOUNDS: Cartesian3 = Cartesian3::new(-1.0, -1.0, -1.0);
    pub const DEFAULT_MAX_BOUNDS: Cartesian3 = Cartesian3::new(1.0, 1.0, 1.0);

    /// Constructs a new VoxelBoxShape with default bounds and identity transform.
    pub fn new() -> Self {
        Self {
            min_bounds: Self::DEFAULT_MIN_BOUNDS,
            max_bounds: Self::DEFAULT_MAX_BOUNDS,
            model_matrix: Matrix4::IDENTITY,
        }
    }

    /// Updates the shape transform and bounding intervals.
    pub fn update(
        &mut self,
        model_matrix: Matrix4,
        min_bounds: Cartesian3,
        max_bounds: Cartesian3,
    ) {
        self.model_matrix = model_matrix;
        self.min_bounds = min_bounds;
        self.max_bounds = max_bounds;
    }
}

impl VoxelShape for VoxelBoxShape {
    fn compute_oriented_bounding_box_for_tile(
        &self,
        tile_level: u32,
        tile_x: u32,
        tile_y: u32,
        tile_z: u32,
    ) -> OrientedBoundingBox {
        let size_at_level = 1.0 / (1u64 << tile_level) as f64;

        let tile_min_x = self.min_bounds.x
            + (self.max_bounds.x - self.min_bounds.x) * (size_at_level * tile_x as f64);
        let tile_min_y = self.min_bounds.y
            + (self.max_bounds.y - self.min_bounds.y) * (size_at_level * tile_y as f64);
        let tile_min_z = self.min_bounds.z
            + (self.max_bounds.z - self.min_bounds.z) * (size_at_level * tile_z as f64);

        let tile_max_x = self.min_bounds.x
            + (self.max_bounds.x - self.min_bounds.x) * (size_at_level * (tile_x + 1) as f64);
        let tile_max_y = self.min_bounds.y
            + (self.max_bounds.y - self.min_bounds.y) * (size_at_level * (tile_y + 1) as f64);
        let tile_max_z = self.min_bounds.z
            + (self.max_bounds.z - self.min_bounds.z) * (size_at_level * (tile_z + 1) as f64);

        let tile_min = Cartesian3::new(tile_min_x, tile_min_y, tile_min_z);
        let tile_max = Cartesian3::new(tile_max_x, tile_max_y, tile_max_z);

        get_box_chunk_obb(tile_min, tile_max, &self.model_matrix)
    }
}

fn get_box_chunk_obb(
    min_bounds: Cartesian3,
    max_bounds: Cartesian3,
    matrix: &Matrix4,
) -> OrientedBoundingBox {
    let local_center = (min_bounds + max_bounds) * 0.5;
    let center = matrix.multiply_by_point(local_center);

    let local_half_extent = (max_bounds - min_bounds) * 0.5;
    let rot_scale = matrix.get_matrix3().multiply_by_scale(local_half_extent);
    let half_axes = DMat3::from_cols_array(&rot_scale.values);

    OrientedBoundingBox::new(DVec3::new(center.x, center.y, center.z), half_axes)
}
