//! Box voxel shape matching CesiumJS `Scene/VoxelBoxShape.js` (Apache-2.0).

use cesium_math::{BoundingSphere, Cartesian3, DeveloperError, Matrix4};
use glam::{DMat3, DVec3};

use crate::spatial_node::SpatialNode;
use crate::voxel_shape_type::VoxelShapeType;

/// Trait implemented by voxel shapes capable of calculating tile bounding volumes.
pub trait VoxelShape {
    /// Returns the shape type enum for this voxel shape.
    fn shape_type(&self) -> VoxelShapeType;

    /// Computes the oriented bounding box containing the specified tile coordinate.
    fn compute_oriented_bounding_box_for_tile(
        &self,
        tile_level: u32,
        tile_x: u32,
        tile_y: u32,
        tile_z: u32,
    ) -> cesium_math::OrientedBoundingBox;

    /// Computes the oriented bounding box containing the specified sample within a tile.
    fn compute_oriented_bounding_box_for_sample(
        &self,
        spatial_node: &SpatialNode,
        tile_dimensions: Cartesian3,
        tile_uv: Cartesian3,
    ) -> cesium_math::OrientedBoundingBox;
}

/// A box-shaped voxel region with configurable bounds and transform.
#[derive(Debug, Clone, PartialEq)]
pub struct VoxelBoxShape {
    pub min_bounds: Cartesian3,
    pub max_bounds: Cartesian3,
    pub render_min_bounds: Cartesian3,
    pub render_max_bounds: Cartesian3,
    pub shape_transform: Matrix4,
    pub bound_transform: Matrix4,
    pub oriented_bounding_box: cesium_math::OrientedBoundingBox,
    pub bounding_sphere: BoundingSphere,
    pub shader_maximum_intersections_length: usize,
    pub local_to_shape_uv_translate: Cartesian3,
    pub local_to_shape_uv_scale: Cartesian3,
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
            render_min_bounds: Self::DEFAULT_MIN_BOUNDS,
            render_max_bounds: Self::DEFAULT_MAX_BOUNDS,
            shape_transform: Matrix4::IDENTITY,
            bound_transform: Matrix4::IDENTITY,
            oriented_bounding_box: cesium_math::OrientedBoundingBox::default(),
            bounding_sphere: BoundingSphere::default(),
            shader_maximum_intersections_length: 0,
            local_to_shape_uv_translate: Cartesian3::ZERO,
            local_to_shape_uv_scale: Cartesian3::new(1.0, 1.0, 1.0),
        }
    }

    /// Updates the shape transform and bounding intervals. Returns true if visible.
    pub fn update(
        &mut self,
        model_matrix: Option<Matrix4>,
        min_bounds: Option<Cartesian3>,
        max_bounds: Option<Cartesian3>,
        clip_min_bounds: Option<Cartesian3>,
        clip_max_bounds: Option<Cartesian3>,
    ) -> Result<bool, DeveloperError> {
        let model_matrix =
            model_matrix.ok_or_else(|| DeveloperError::new("modelMatrix is required."))?;
        let min_bounds =
            min_bounds.ok_or_else(|| DeveloperError::new("minBounds is required."))?;
        let max_bounds =
            max_bounds.ok_or_else(|| DeveloperError::new("maxBounds is required."))?;

        let clip_min = clip_min_bounds.unwrap_or(min_bounds);
        let clip_max = clip_max_bounds.unwrap_or(max_bounds);

        self.min_bounds = min_bounds;
        self.max_bounds = max_bounds;

        let render_min_bounds = Cartesian3::new(
            min_bounds.x.clamp(clip_min.x, clip_max.x),
            min_bounds.y.clamp(clip_min.y, clip_max.y),
            min_bounds.z.clamp(clip_min.z, clip_max.z),
        );
        let render_max_bounds = Cartesian3::new(
            max_bounds.x.clamp(clip_min.x, clip_max.x),
            max_bounds.y.clamp(clip_min.y, clip_max.y),
            max_bounds.z.clamp(clip_min.z, clip_max.z),
        );

        self.render_min_bounds = render_min_bounds;
        self.render_max_bounds = render_max_bounds;

        let scale = model_matrix.get_scale();

        let zero_bound_count = (if render_min_bounds.x == render_max_bounds.x { 1 } else { 0 })
            + (if render_min_bounds.y == render_max_bounds.y { 1 } else { 0 })
            + (if render_min_bounds.z == render_max_bounds.z { 1 } else { 0 });

        if render_min_bounds.x > render_max_bounds.x
            || render_min_bounds.y > render_max_bounds.y
            || render_min_bounds.z > render_max_bounds.z
            || zero_bound_count >= 2
            || scale.x == 0.0
            || scale.y == 0.0
            || scale.z == 0.0
        {
            return Ok(false);
        }

        self.shape_transform = model_matrix;
        self.oriented_bounding_box =
            get_box_chunk_obb(render_min_bounds, render_max_bounds, &self.shape_transform);

        let half_axes = self.oriented_bounding_box.half_axes;
        let center = self.oriented_bounding_box.center;

        self.bound_transform = Matrix4::from_column_major_array([
            half_axes.x_axis.x,
            half_axes.x_axis.y,
            half_axes.x_axis.z,
            0.0,
            half_axes.y_axis.x,
            half_axes.y_axis.y,
            half_axes.y_axis.z,
            0.0,
            half_axes.z_axis.x,
            half_axes.z_axis.y,
            half_axes.z_axis.z,
            0.0,
            center.x,
            center.y,
            center.z,
            1.0,
        ]);

        self.bounding_sphere =
            BoundingSphere::from_oriented_bounding_box(&self.oriented_bounding_box)
                .map_err(|e| DeveloperError::new(format!("{e:?}")))?;

        self.shader_maximum_intersections_length = 1;

        let uv_scale = Cartesian3::new(
            bound_scale(min_bounds.x, max_bounds.x),
            bound_scale(min_bounds.y, max_bounds.y),
            bound_scale(min_bounds.z, max_bounds.z),
        );
        self.local_to_shape_uv_scale = uv_scale;
        self.local_to_shape_uv_translate = Cartesian3::new(
            -uv_scale.x * min_bounds.x,
            -uv_scale.y * min_bounds.y,
            -uv_scale.z * min_bounds.z,
        );

        Ok(true)
    }

    /// Computes an oriented bounding box for a specified tile with validation.
    pub fn compute_oriented_bounding_box_for_tile_checked(
        &self,
        tile_level: Option<u32>,
        tile_x: Option<u32>,
        tile_y: Option<u32>,
        tile_z: Option<u32>,
    ) -> Result<cesium_math::OrientedBoundingBox, DeveloperError> {
        let level = tile_level.ok_or_else(|| DeveloperError::new("tileLevel is required."))?;
        let x = tile_x.ok_or_else(|| DeveloperError::new("tileX is required."))?;
        let y = tile_y.ok_or_else(|| DeveloperError::new("tileY is required."))?;
        let z = tile_z.ok_or_else(|| DeveloperError::new("tileZ is required."))?;

        Ok(self.compute_oriented_bounding_box_for_tile(level, x, y, z))
    }

    /// Computes an oriented bounding box for a sample with validation.
    pub fn compute_oriented_bounding_box_for_sample_checked(
        &self,
        spatial_node: Option<&SpatialNode>,
        tile_dimensions: Option<Cartesian3>,
        tile_uv: Option<Cartesian3>,
    ) -> Result<cesium_math::OrientedBoundingBox, DeveloperError> {
        let node = spatial_node.ok_or_else(|| DeveloperError::new("spatialNode is required."))?;
        let dims =
            tile_dimensions.ok_or_else(|| DeveloperError::new("tileDimensions is required."))?;
        let uv = tile_uv.ok_or_else(|| DeveloperError::new("tileUv is required."))?;

        Ok(self.compute_oriented_bounding_box_for_sample(node, dims, uv))
    }
}

fn bound_scale(min_bound: f64, max_bound: f64) -> f64 {
    let diff = max_bound - min_bound;
    if diff.abs() <= 1e-7 {
        1.0
    } else {
        1.0 / diff
    }
}

impl VoxelShape for VoxelBoxShape {
    fn shape_type(&self) -> VoxelShapeType {
        VoxelShapeType::Box
    }

    fn compute_oriented_bounding_box_for_tile(
        &self,
        tile_level: u32,
        tile_x: u32,
        tile_y: u32,
        tile_z: u32,
    ) -> cesium_math::OrientedBoundingBox {
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

        get_box_chunk_obb(tile_min, tile_max, &self.shape_transform)
    }

    fn compute_oriented_bounding_box_for_sample(
        &self,
        spatial_node: &SpatialNode,
        tile_dimensions: Cartesian3,
        tile_uv: Cartesian3,
    ) -> cesium_math::OrientedBoundingBox {
        let tile_size_at_level = 1.0 / (1u64 << spatial_node.level) as f64;
        let sample_size_x = 1.0 / tile_dimensions.x * tile_size_at_level;
        let sample_size_y = 1.0 / tile_dimensions.y * tile_size_at_level;
        let sample_size_z = 1.0 / tile_dimensions.z * tile_size_at_level;

        let min_lerp_x = (spatial_node.x as f64 + tile_uv.x) * tile_size_at_level;
        let min_lerp_y = (spatial_node.y as f64 + tile_uv.y) * tile_size_at_level;
        let min_lerp_z = (spatial_node.z as f64 + tile_uv.z) * tile_size_at_level;

        let max_lerp_x = min_lerp_x + sample_size_x;
        let max_lerp_y = min_lerp_y + sample_size_y;
        let max_lerp_z = min_lerp_z + sample_size_z;

        let sample_min = Cartesian3::new(
            self.min_bounds.x + (self.max_bounds.x - self.min_bounds.x) * min_lerp_x,
            self.min_bounds.y + (self.max_bounds.y - self.min_bounds.y) * min_lerp_y,
            self.min_bounds.z + (self.max_bounds.z - self.min_bounds.z) * min_lerp_z,
        );
        let sample_max = Cartesian3::new(
            self.min_bounds.x + (self.max_bounds.x - self.min_bounds.x) * max_lerp_x,
            self.min_bounds.y + (self.max_bounds.y - self.min_bounds.y) * max_lerp_y,
            self.min_bounds.z + (self.max_bounds.z - self.min_bounds.z) * max_lerp_z,
        );

        get_box_chunk_obb(sample_min, sample_max, &self.shape_transform)
    }
}

fn get_box_chunk_obb(
    minimum_bounds: Cartesian3,
    maximum_bounds: Cartesian3,
    matrix: &Matrix4,
) -> cesium_math::OrientedBoundingBox {
    let is_default_bounds = minimum_bounds == VoxelBoxShape::DEFAULT_MIN_BOUNDS
        && maximum_bounds == VoxelBoxShape::DEFAULT_MAX_BOUNDS;

    if is_default_bounds {
        let trans = matrix.get_translation();
        let center = DVec3::new(trans.x, trans.y, trans.z);
        let mat3 = matrix.get_matrix3();
        let half_axes = DMat3::from_cols_array(&mat3.values);
        cesium_math::OrientedBoundingBox::new(center, half_axes)
    } else {
        let local_center = (minimum_bounds + maximum_bounds) * 0.5;
        let center = matrix.multiply_by_point(local_center);

        let scale = matrix.get_scale();
        let sub_scale = Cartesian3::new(
            scale.x * 0.5 * (maximum_bounds.x - minimum_bounds.x),
            scale.y * 0.5 * (maximum_bounds.y - minimum_bounds.y),
            scale.z * 0.5 * (maximum_bounds.z - minimum_bounds.z),
        );

        let col0 = Cartesian3::new(matrix.values[0], matrix.values[1], matrix.values[2]);
        let col1 = Cartesian3::new(matrix.values[4], matrix.values[5], matrix.values[6]);
        let col2 = Cartesian3::new(matrix.values[8], matrix.values[9], matrix.values[10]);

        let rot_col0 = if scale.x != 0.0 { col0 / scale.x } else { col0 };
        let rot_col1 = if scale.y != 0.0 { col1 / scale.y } else { col1 };
        let rot_col2 = if scale.z != 0.0 { col2 / scale.z } else { col2 };

        let half_x = rot_col0 * sub_scale.x;
        let half_y = rot_col1 * sub_scale.y;
        let half_z = rot_col2 * sub_scale.z;

        let half_axes = DMat3::from_cols(
            DVec3::new(half_x.x, half_x.y, half_x.z),
            DVec3::new(half_y.x, half_y.y, half_y.z),
            DVec3::new(half_z.x, half_z.y, half_z.z),
        );

        cesium_math::OrientedBoundingBox::new(DVec3::new(center.x, center.y, center.z), half_axes)
    }
}
