//! Cylinder voxel shape matching CesiumJS `Scene/VoxelCylinderShape.js` (Apache-2.0).

use std::f64::consts::{PI, TAU};

use cesium_math::{BoundingSphere, Cartesian3, DeveloperError, Matrix4, OrientedBoundingBox};
use glam::{DMat3, DMat4, DVec3};

use crate::spatial_node::SpatialNode;
use crate::voxel_box_shape::VoxelShape;

/// A cylinder-shaped voxel region with configurable bounds and transform.
#[derive(Debug, Clone, PartialEq)]
pub struct VoxelCylinderShape {
    pub min_bounds: Cartesian3,
    pub max_bounds: Cartesian3,
    pub shape_transform: Matrix4,
    pub bound_transform: Matrix4,
    pub oriented_bounding_box: OrientedBoundingBox,
    pub bounding_sphere: BoundingSphere,
    pub shader_maximum_intersections_length: usize,
}

impl Default for VoxelCylinderShape {
    fn default() -> Self {
        Self::new()
    }
}

impl VoxelCylinderShape {
    pub const DEFAULT_MIN_BOUNDS: Cartesian3 = Cartesian3::new(0.0, -PI, -1.0);
    pub const DEFAULT_MAX_BOUNDS: Cartesian3 = Cartesian3::new(1.0, PI, 1.0);

    /// Constructs a new VoxelCylinderShape with default bounds and identity transform.
    pub fn new() -> Self {
        Self {
            min_bounds: Self::DEFAULT_MIN_BOUNDS,
            max_bounds: Self::DEFAULT_MAX_BOUNDS,
            shape_transform: Matrix4::IDENTITY,
            bound_transform: Matrix4::IDENTITY,
            oriented_bounding_box: OrientedBoundingBox::default(),
            bounding_sphere: BoundingSphere::default(),
            shader_maximum_intersections_length: 0,
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
        let mut min_bounds =
            min_bounds.ok_or_else(|| DeveloperError::new("minBounds is required."))?;
        let mut max_bounds =
            max_bounds.ok_or_else(|| DeveloperError::new("maxBounds is required."))?;

        let mut clip_min = clip_min_bounds.unwrap_or(min_bounds);
        let mut clip_max = clip_max_bounds.unwrap_or(max_bounds);

        // Clamp radius to non-negative and normalize angle to [-PI, PI]
        min_bounds.x = min_bounds.x.max(0.0);
        max_bounds.x = max_bounds.x.max(0.0);
        min_bounds.y = negative_pi_to_pi(min_bounds.y);
        max_bounds.y = negative_pi_to_pi(max_bounds.y);

        clip_min.y = negative_pi_to_pi(clip_min.y);
        clip_max.y = negative_pi_to_pi(clip_max.y);

        self.min_bounds = min_bounds;
        self.max_bounds = max_bounds;

        let render_min_bounds = Cartesian3::new(
            min_bounds.x.max(clip_min.x),
            min_bounds.y.max(clip_min.y),
            min_bounds.z.max(clip_min.z),
        );
        let render_max_bounds = Cartesian3::new(
            max_bounds.x.min(clip_max.x),
            max_bounds.y.min(clip_max.y),
            max_bounds.z.min(clip_max.z),
        );

        let scale = model_matrix.get_scale();

        if render_max_bounds.x == 0.0
            || render_min_bounds.x > render_max_bounds.x
            || render_min_bounds.z > render_max_bounds.z
            || scale.x.abs() <= 1e-10
            || scale.y.abs() <= 1e-10
            || scale.z.abs() <= 1e-10
        {
            return Ok(false);
        }

        self.shape_transform = model_matrix;
        self.oriented_bounding_box = get_cylinder_chunk_obb(
            render_min_bounds,
            render_max_bounds,
            &self.shape_transform,
        );

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

        self.shader_maximum_intersections_length = 2;

        Ok(true)
    }

    /// Computes an oriented bounding box for a specified tile with validation.
    pub fn compute_oriented_bounding_box_for_tile_checked(
        &self,
        tile_level: Option<u32>,
        tile_x: Option<u32>,
        tile_y: Option<u32>,
        tile_z: Option<u32>,
    ) -> Result<OrientedBoundingBox, DeveloperError> {
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
    ) -> Result<OrientedBoundingBox, DeveloperError> {
        let node = spatial_node.ok_or_else(|| DeveloperError::new("spatialNode is required."))?;
        let dims =
            tile_dimensions.ok_or_else(|| DeveloperError::new("tileDimensions is required."))?;
        let uv = tile_uv.ok_or_else(|| DeveloperError::new("tileUv is required."))?;

        Ok(self.compute_oriented_bounding_box_for_sample(node, dims, uv))
    }
}

fn negative_pi_to_pi(angle: f64) -> f64 {
    if (-PI..=PI).contains(&angle) {
        return angle;
    }
    let a = (angle + PI).rem_euclid(TAU);
    a - PI
}

impl VoxelShape for VoxelCylinderShape {
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

        get_cylinder_chunk_obb(tile_min, tile_max, &self.shape_transform)
    }

    fn compute_oriented_bounding_box_for_sample(
        &self,
        spatial_node: &SpatialNode,
        tile_dimensions: Cartesian3,
        tile_uv: Cartesian3,
    ) -> OrientedBoundingBox {
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

        get_cylinder_chunk_obb(sample_min, sample_max, &self.shape_transform)
    }
}

fn get_cylinder_chunk_obb(
    chunk_min_bounds: Cartesian3,
    chunk_max_bounds: Cartesian3,
    matrix: &Matrix4,
) -> OrientedBoundingBox {
    let radius_start = chunk_min_bounds.x;
    let radius_end = chunk_max_bounds.x;
    let angle_start = chunk_min_bounds.y;
    let angle_end = if chunk_max_bounds.y < angle_start {
        chunk_max_bounds.y + TAU
    } else {
        chunk_max_bounds.y
    };
    let height_start = chunk_min_bounds.z;
    let height_end = chunk_max_bounds.z;

    let angle_range = angle_end - angle_start;
    let angle_mid = angle_start + angle_range * 0.5;

    let mut test_angles = [0.0; 5];
    let mut test_angle_count = 0;

    test_angles[test_angle_count] = angle_start;
    test_angle_count += 1;
    test_angles[test_angle_count] = angle_end;
    test_angle_count += 1;
    test_angles[test_angle_count] = angle_mid;
    test_angle_count += 1;

    if angle_range > PI {
        test_angles[test_angle_count] = angle_mid - std::f64::consts::FRAC_PI_2;
        test_angle_count += 1;
        test_angles[test_angle_count] = angle_mid + std::f64::consts::FRAC_PI_2;
        test_angle_count += 1;
    }

    let mut min_x = f64::INFINITY;
    let mut min_y = f64::INFINITY;
    let mut max_x = f64::NEG_INFINITY;
    let mut max_y = f64::NEG_INFINITY;

    for &test_angle in test_angles.iter().take(test_angle_count) {
        let angle = test_angle - angle_mid;
        let cos_angle = angle.cos();
        let sin_angle = angle.sin();
        let x1 = cos_angle * radius_start;
        let y1 = sin_angle * radius_start;
        let x2 = cos_angle * radius_end;
        let y2 = sin_angle * radius_end;

        min_x = min_x.min(x1).min(x2);
        min_y = min_y.min(y1).min(y2);
        max_x = max_x.max(x1).max(x2);
        max_y = max_y.max(y1).max(y2);
    }

    let extent_x = max_x - min_x;
    let extent_y = max_y - min_y;
    let extent_z = height_end - height_start;

    let center_x = (min_x + max_x) * 0.5;
    let center_y = (min_y + max_y) * 0.5;
    let center_z = (height_start + height_end) * 0.5;

    let rot_z = DMat3::from_rotation_z(angle_mid);
    let scale_mat = DMat4::from_scale(DVec3::new(extent_x, extent_y, extent_z));
    let rot_mat = DMat4::from_mat3(rot_z);
    let trans_mat = DMat4::from_translation(DVec3::new(center_x, center_y, center_z));

    let local_mat = rot_mat * trans_mat * scale_mat;
    let mat = DMat4::from_cols_array(&matrix.values);
    let global_mat = mat * local_mat;

    OrientedBoundingBox::from_transformation(global_mat).unwrap()
}
