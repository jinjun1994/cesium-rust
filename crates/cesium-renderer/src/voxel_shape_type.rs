//! VoxelShapeType enum matching CesiumJS `Scene/VoxelShapeType.js` (Apache-2.0).

use std::str::FromStr;

use cesium_math::{Cartesian3, DeveloperError};

use crate::voxel_box_shape::{VoxelBoxShape, VoxelShape};
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
    /// The string identifier for Box shape matching CesiumJS.
    pub const BOX: &'static str = "BOX";
    /// The string identifier for Ellipsoid shape matching CesiumJS.
    pub const ELLIPSOID: &'static str = "ELLIPSOID";
    /// The string identifier for Cylinder shape matching CesiumJS.
    pub const CYLINDER: &'static str = "CYLINDER";

    /// Gets the default minimum bounds for this shape type.
    pub fn get_min_bounds(shape_type: Self) -> Cartesian3 {
        match shape_type {
            Self::Box => VoxelBoxShape::DEFAULT_MIN_BOUNDS,
            Self::Ellipsoid => VoxelEllipsoidShape::DEFAULT_MIN_BOUNDS,
            Self::Cylinder => VoxelCylinderShape::DEFAULT_MIN_BOUNDS,
        }
    }

    /// Gets the default minimum bounds from a shape type string, returning DeveloperError if invalid.
    pub fn get_min_bounds_from_str(shape_type: &str) -> Result<Cartesian3, DeveloperError> {
        let shape = Self::from_str(shape_type)?;
        Ok(Self::get_min_bounds(shape))
    }

    /// Gets the default maximum bounds for this shape type.
    pub fn get_max_bounds(shape_type: Self) -> Cartesian3 {
        match shape_type {
            Self::Box => VoxelBoxShape::DEFAULT_MAX_BOUNDS,
            Self::Ellipsoid => VoxelEllipsoidShape::DEFAULT_MAX_BOUNDS,
            Self::Cylinder => VoxelCylinderShape::DEFAULT_MAX_BOUNDS,
        }
    }

    /// Gets the default maximum bounds from a shape type string, returning DeveloperError if invalid.
    pub fn get_max_bounds_from_str(shape_type: &str) -> Result<Cartesian3, DeveloperError> {
        let shape = Self::from_str(shape_type)?;
        Ok(Self::get_max_bounds(shape))
    }

    /// Constructs a shape object for this shape type.
    pub fn create_shape(shape_type: Self) -> Box<dyn VoxelShape> {
        match shape_type {
            Self::Box => Box::new(VoxelBoxShape::default()),
            Self::Ellipsoid => Box::new(VoxelEllipsoidShape::default()),
            Self::Cylinder => Box::new(VoxelCylinderShape::default()),
        }
    }

    /// Constructs a shape object from a shape type string, returning DeveloperError if invalid.
    pub fn create_shape_from_str(shape_type: &str) -> Result<Box<dyn VoxelShape>, DeveloperError> {
        let shape = Self::from_str(shape_type)?;
        Ok(Self::create_shape(shape))
    }
}

impl FromStr for VoxelShapeType {
    type Err = DeveloperError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "BOX" | "Box" | "box" => Ok(Self::Box),
            "ELLIPSOID" | "Ellipsoid" | "ellipsoid" => Ok(Self::Ellipsoid),
            "CYLINDER" | "Cylinder" | "cylinder" => Ok(Self::Cylinder),
            _ => Err(DeveloperError::new(format!("Invalid shape type {s}"))),
        }
    }
}

impl std::fmt::Display for VoxelShapeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Box => write!(f, "BOX"),
            Self::Ellipsoid => write!(f, "ELLIPSOID"),
            Self::Cylinder => write!(f, "CYLINDER"),
        }
    }
}
