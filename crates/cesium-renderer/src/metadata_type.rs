//! MetadataType matching CesiumJS `Scene/MetadataType.js` (Apache-2.0).

use cesium_math::DeveloperError;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// Mathematical object type corresponding to a metadata vector or matrix type.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MetadataMathType {
    Cartesian2,
    Cartesian3,
    Cartesian4,
    Matrix2,
    Matrix3,
    Matrix4,
}

/// Metadata types describing single values, vectors, matrices, or enums.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum MetadataType {
    /// A single scalar component.
    #[serde(rename = "SCALAR")]
    Scalar,
    /// A 2-component vector.
    #[serde(rename = "VEC2")]
    Vec2,
    /// A 3-component vector.
    #[serde(rename = "VEC3")]
    Vec3,
    /// A 4-component vector.
    #[serde(rename = "VEC4")]
    Vec4,
    /// A 2x2 matrix.
    #[serde(rename = "MAT2")]
    Mat2,
    /// A 3x3 matrix.
    #[serde(rename = "MAT3")]
    Mat3,
    /// A 4x4 matrix.
    #[serde(rename = "MAT4")]
    Mat4,
    /// A boolean (true/false) value.
    #[serde(rename = "BOOLEAN")]
    Boolean,
    /// A UTF-8 encoded string.
    #[serde(rename = "STRING")]
    String,
    /// An enumerated value.
    #[serde(rename = "ENUM")]
    Enum,
}

impl MetadataType {
    /// Returns true if this type is a vector (`VEC2`, `VEC3`, or `VEC4`).
    pub const fn is_vector_type(&self) -> bool {
        matches!(self, Self::Vec2 | Self::Vec3 | Self::Vec4)
    }

    /// Returns true if this type is a matrix (`MAT2`, `MAT3`, or `MAT4`).
    pub const fn is_matrix_type(&self) -> bool {
        matches!(self, Self::Mat2 | Self::Mat3 | Self::Mat4)
    }

    /// Returns the number of components for this type.
    pub const fn get_component_count(&self) -> usize {
        match self {
            Self::Scalar | Self::Boolean | Self::String | Self::Enum => 1,
            Self::Vec2 => 2,
            Self::Vec3 => 3,
            Self::Vec4 | Self::Mat2 => 4,
            Self::Mat3 => 9,
            Self::Mat4 => 16,
        }
    }

    /// Returns the mathematical class type associated with vector/matrix types.
    pub const fn get_math_type(&self) -> Option<MetadataMathType> {
        match self {
            Self::Vec2 => Some(MetadataMathType::Cartesian2),
            Self::Vec3 => Some(MetadataMathType::Cartesian3),
            Self::Vec4 => Some(MetadataMathType::Cartesian4),
            Self::Mat2 => Some(MetadataMathType::Matrix2),
            Self::Mat3 => Some(MetadataMathType::Matrix3),
            Self::Mat4 => Some(MetadataMathType::Matrix4),
            _ => None,
        }
    }

    /// Returns the number of components from a type name string, or DeveloperError if invalid.
    pub fn get_component_count_from_str(s: &str) -> Result<usize, DeveloperError> {
        let t: Self = s.parse()?;
        Ok(t.get_component_count())
    }

    /// Returns the string representation.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Scalar => "SCALAR",
            Self::Vec2 => "VEC2",
            Self::Vec3 => "VEC3",
            Self::Vec4 => "VEC4",
            Self::Mat2 => "MAT2",
            Self::Mat3 => "MAT3",
            Self::Mat4 => "MAT4",
            Self::Boolean => "BOOLEAN",
            Self::String => "STRING",
            Self::Enum => "ENUM",
        }
    }
}

impl FromStr for MetadataType {
    type Err = DeveloperError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "SCALAR" => Ok(Self::Scalar),
            "VEC2" => Ok(Self::Vec2),
            "VEC3" => Ok(Self::Vec3),
            "VEC4" => Ok(Self::Vec4),
            "MAT2" => Ok(Self::Mat2),
            "MAT3" => Ok(Self::Mat3),
            "MAT4" => Ok(Self::Mat4),
            "BOOLEAN" => Ok(Self::Boolean),
            "STRING" => Ok(Self::String),
            "ENUM" => Ok(Self::Enum),
            _ => Err(DeveloperError::new(format!("Invalid metadata type {s}"))),
        }
    }
}
