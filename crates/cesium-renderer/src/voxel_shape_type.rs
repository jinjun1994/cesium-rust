//! VoxelShapeType enum matching CesiumJS `Scene/VoxelShapeType.js` (Apache-2.0).

use cesium_math::Cartesian3;

use crate::voxel_box_shape::VoxelBoxShape;
use crate::voxel_cylinder_shape::VoxelCylinderShape;
use crate::voxel_ellipsoid_shape::VoxelEllipsoidShape;

/// An enum of voxel shapes. The shape controls how the voxel grid is mapped to 3D space.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum VoxelShapeType {
    /// A box shape.
    #[default]
    Box,
    /// An ellipsoid shape.
    Ellipsoid,
    /// A cylinder shape.
    Cylinder,
}

impl VoxelShapeType {
    /// Gets the default minimum bounds for this shape type.
    pub fn get_min_bounds(shape_type: Self) -> Cartesian3 {
        match shape_type {
            Self::Box => VoxelBoxShape::DEFAULT_MIN_BOUNDS,
            Self::Ellipsoid => VoxelEllipsoidShape::DEFAULT_MIN_BOUNDS,
            Self::Cylinder => VoxelCylinderShape::DEFAULT_MIN_BOUNDS,
        }
    }

    /// Gets the default maximum bounds for this shape type.
    pub fn get_max_bounds(shape_type: Self) -> Cartesian3 {
        match shape_type {
            Self::Box => VoxelBoxShape::DEFAULT_MAX_BOUNDS,
            Self::Ellipsoid => VoxelEllipsoidShape::DEFAULT_MAX_BOUNDS,
            Self::Cylinder => VoxelCylinderShape::DEFAULT_MAX_BOUNDS,
        }
    }
}
