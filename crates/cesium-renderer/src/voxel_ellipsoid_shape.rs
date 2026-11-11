//! Ellipsoid voxel shape matching CesiumJS `Scene/VoxelEllipsoidShape.js` (Apache-2.0).

use std::f64::consts::{FRAC_PI_2, PI};

use cesium_math::{
    BoundingSphere, Cartesian3, DeveloperError, Ellipsoid, GeoRectangle, Matrix3, Matrix4,
    OrientedBoundingBox,
};
use glam::{DMat3, DVec3};

use crate::spatial_node::SpatialNode;
use crate::voxel_box_shape::VoxelShape;
use crate::voxel_shape_type::VoxelShapeType;

/// An ellipsoid-shaped voxel region with configurable bounds and transform.
#[derive(Debug, Clone, PartialEq)]
pub struct VoxelEllipsoidShape {
    pub min_bounds: Cartesian3,
    pub max_bounds: Cartesian3,
    pub rectangle: GeoRectangle,
    pub minimum_height: f64,
    pub maximum_height: f64,
    pub ellipsoid: Ellipsoid,
    pub translation: Cartesian3,
    pub rotation: Matrix3,
    pub shape_transform: Matrix4,
    pub bound_transform: Matrix4,
    pub oriented_bounding_box: OrientedBoundingBox,
    pub bounding_sphere: BoundingSphere,
    pub shader_maximum_intersections_length: usize,
}

impl Default for VoxelEllipsoidShape {
    fn default() -> Self {
        Self::new()
    }
}

impl VoxelEllipsoidShape {
    pub const DEFAULT_MIN_BOUNDS: Cartesian3 = Cartesian3::new(-PI, -FRAC_PI_2, -6356752.314245179);
    pub const DEFAULT_MAX_BOUNDS: Cartesian3 = Cartesian3::new(PI, FRAC_PI_2, 10000000.0);

    /// Constructs a new VoxelEllipsoidShape with default bounds and identity transform.
    pub fn new() -> Self {
        Self {
            min_bounds: Self::DEFAULT_MIN_BOUNDS,
            max_bounds: Self::DEFAULT_MAX_BOUNDS,
            rectangle: GeoRectangle::MAXIMUM,
            minimum_height: Self::DEFAULT_MIN_BOUNDS.z,
            maximum_height: Self::DEFAULT_MAX_BOUNDS.z,
            ellipsoid: Ellipsoid::WGS84,
            translation: Cartesian3::ZERO,
            rotation: Matrix3::IDENTITY,
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
        let min_bounds =
            min_bounds.ok_or_else(|| DeveloperError::new("minBounds is required."))?;
        let max_bounds =
            max_bounds.ok_or_else(|| DeveloperError::new("maxBounds is required."))?;

        let clip_min = clip_min_bounds.unwrap_or(Self::DEFAULT_MIN_BOUNDS);
        let clip_max = clip_max_bounds.unwrap_or(Self::DEFAULT_MAX_BOUNDS);

        let radii = model_matrix.get_scale();
        let min_radius = radii.x.min(radii.y).min(radii.z);
        let actual_min_bounds = Cartesian3::new(-PI, -FRAC_PI_2, -min_radius);

        let shape_min = Cartesian3::new(
            min_bounds.x.clamp(actual_min_bounds.x, Self::DEFAULT_MAX_BOUNDS.x),
            min_bounds.y.clamp(actual_min_bounds.y, Self::DEFAULT_MAX_BOUNDS.y),
            min_bounds.z.clamp(actual_min_bounds.z, Self::DEFAULT_MAX_BOUNDS.z),
        );
        let shape_max = Cartesian3::new(
            max_bounds.x.clamp(actual_min_bounds.x, Self::DEFAULT_MAX_BOUNDS.x),
            max_bounds.y.clamp(actual_min_bounds.y, Self::DEFAULT_MAX_BOUNDS.y),
            max_bounds.z.clamp(actual_min_bounds.z, Self::DEFAULT_MAX_BOUNDS.z),
        );

        let clamped_clip_min = Cartesian3::new(
            clip_min.x.clamp(actual_min_bounds.x, Self::DEFAULT_MAX_BOUNDS.x),
            clip_min.y.clamp(actual_min_bounds.y, Self::DEFAULT_MAX_BOUNDS.y),
            clip_min.z.clamp(actual_min_bounds.z, Self::DEFAULT_MAX_BOUNDS.z),
        );
        let clamped_clip_max = Cartesian3::new(
            clip_max.x.clamp(actual_min_bounds.x, Self::DEFAULT_MAX_BOUNDS.x),
            clip_max.y.clamp(actual_min_bounds.y, Self::DEFAULT_MAX_BOUNDS.y),
            clip_max.z.clamp(actual_min_bounds.z, Self::DEFAULT_MAX_BOUNDS.z),
        );

        let render_min = Cartesian3::new(
            shape_min.x.max(clamped_clip_min.x),
            shape_min.y.max(clamped_clip_min.y),
            shape_min.z.max(clamped_clip_min.z),
        );
        let render_max = Cartesian3::new(
            shape_max.x.min(clamped_clip_max.x),
            shape_max.y.min(clamped_clip_max.y),
            shape_max.z.min(clamped_clip_max.z),
        );

        if render_min.y > render_max.y
            || render_min.y == Self::DEFAULT_MAX_BOUNDS.y
            || render_max.y == Self::DEFAULT_MIN_BOUNDS.y
            || render_min.z > render_max.z
            || radii.x == 0.0
            || radii.y == 0.0
            || radii.z == 0.0
        {
            return Ok(false);
        }

        self.min_bounds = shape_min;
        self.max_bounds = shape_max;

        self.rectangle = GeoRectangle {
            west: shape_min.x,
            south: shape_min.y,
            east: shape_max.x,
            north: shape_max.y,
        };
        self.translation = model_matrix.get_translation();

        // Rotation from model_matrix
        let col0 = Cartesian3::new(model_matrix.values[0], model_matrix.values[1], model_matrix.values[2]);
        let col1 = Cartesian3::new(model_matrix.values[4], model_matrix.values[5], model_matrix.values[6]);
        let col2 = Cartesian3::new(model_matrix.values[8], model_matrix.values[9], model_matrix.values[10]);

        let rot_col0 = if radii.x != 0.0 { col0 / radii.x } else { col0 };
        let rot_col1 = if radii.y != 0.0 { col1 / radii.y } else { col1 };
        let rot_col2 = if radii.z != 0.0 { col2 / radii.z } else { col2 };

        self.rotation = Matrix3::from_column_major_array([
            rot_col0.x, rot_col0.y, rot_col0.z,
            rot_col1.x, rot_col1.y, rot_col1.z,
            rot_col2.x, rot_col2.y, rot_col2.z,
        ]);

        self.ellipsoid = Ellipsoid::new(DVec3::new(radii.x, radii.y, radii.z))
            .map_err(|e| DeveloperError::new(format!("{e:?}")))?;
        self.minimum_height = shape_min.z;
        self.maximum_height = shape_max.z;

        let render_rect = GeoRectangle {
            west: render_min.x,
            south: render_min.y,
            east: render_max.x,
            north: render_max.y,
        };

        self.oriented_bounding_box = get_ellipsoid_chunk_obb(
            render_rect,
            render_min.z,
            render_max.z,
            self.ellipsoid,
            self.translation,
            self.rotation,
        );

        self.shape_transform = Matrix4::from_rotation_translation(&self.rotation, self.translation);

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

impl VoxelShape for VoxelEllipsoidShape {
    fn shape_type(&self) -> VoxelShapeType {
        VoxelShapeType::Ellipsoid
    }

    fn compute_oriented_bounding_box_for_tile(
        &self,
        tile_level: u32,
        tile_x: u32,
        tile_y: u32,
        tile_z: u32,
    ) -> OrientedBoundingBox {
        let size_at_level = 1.0 / (1u64 << tile_level) as f64;
        let min_lon_lerp = tile_x as f64 * size_at_level;
        let max_lon_lerp = (tile_x + 1) as f64 * size_at_level;
        let min_lat_lerp = tile_y as f64 * size_at_level;
        let max_lat_lerp = (tile_y + 1) as f64 * size_at_level;
        let min_h_lerp = tile_z as f64 * size_at_level;
        let max_h_lerp = (tile_z + 1) as f64 * size_at_level;

        let west = lerp(self.rectangle.west, self.rectangle.east, min_lon_lerp);
        let east = lerp(self.rectangle.west, self.rectangle.east, max_lon_lerp);
        let south = lerp(self.rectangle.south, self.rectangle.north, min_lat_lerp);
        let north = lerp(self.rectangle.south, self.rectangle.north, max_lat_lerp);

        let sub_rect = GeoRectangle { west, south, east, north };
        let min_h = lerp(self.minimum_height, self.maximum_height, min_h_lerp);
        let max_h = lerp(self.minimum_height, self.maximum_height, max_h_lerp);

        get_ellipsoid_chunk_obb(
            sub_rect,
            min_h,
            max_h,
            self.ellipsoid,
            self.translation,
            self.rotation,
        )
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

        let west = lerp(self.rectangle.west, self.rectangle.east, min_lerp_x);
        let east = lerp(self.rectangle.west, self.rectangle.east, max_lerp_x);
        let south = lerp(self.rectangle.south, self.rectangle.north, min_lerp_y);
        let north = lerp(self.rectangle.south, self.rectangle.north, max_lerp_y);

        let sub_rect = GeoRectangle { west, south, east, north };
        let min_h = lerp(self.minimum_height, self.maximum_height, min_lerp_z);
        let max_h = lerp(self.minimum_height, self.maximum_height, max_lerp_z);

        get_ellipsoid_chunk_obb(
            sub_rect,
            min_h,
            max_h,
            self.ellipsoid,
            self.translation,
            self.rotation,
        )
    }
}

fn lerp(p: f64, q: f64, time: f64) -> f64 {
    (1.0 - time) * p + time * q
}

fn get_ellipsoid_chunk_obb(
    rectangle: GeoRectangle,
    min_height: f64,
    max_height: f64,
    ellipsoid: Ellipsoid,
    translation: Cartesian3,
    rotation: Matrix3,
) -> OrientedBoundingBox {
    let mut obb = OrientedBoundingBox::from_rectangle(
        rectangle,
        min_height,
        max_height,
        ellipsoid,
    )
    .unwrap_or_default();

    obb.center += DVec3::new(translation.x, translation.y, translation.z);

    let rot = DMat3::from_cols_array(&rotation.values);
    obb.half_axes *= rot;

    obb
}
