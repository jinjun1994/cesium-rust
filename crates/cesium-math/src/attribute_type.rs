//! Attribute type representations matching CesiumJS `Scene/AttributeType`.
//! Adapted from CesiumJS (Apache-2.0).

use crate::errors::DeveloperError;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AttributeType {
    Scalar,
    Vec2,
    Vec3,
    Vec4,
    Mat2,
    Mat3,
    Mat4,
}

impl AttributeType {
    pub const SCALAR: Self = Self::Scalar;
    pub const VEC2: Self = Self::Vec2;
    pub const VEC3: Self = Self::Vec3;
    pub const VEC4: Self = Self::Vec4;
    pub const MAT2: Self = Self::Mat2;
    pub const MAT3: Self = Self::Mat3;
    pub const MAT4: Self = Self::Mat4;

    pub fn from_name(name: &str) -> Result<Self, DeveloperError> {
        match name {
            "SCALAR" => Ok(Self::Scalar),
            "VEC2" => Ok(Self::Vec2),
            "VEC3" => Ok(Self::Vec3),
            "VEC4" => Ok(Self::Vec4),
            "MAT2" => Ok(Self::Mat2),
            "MAT3" => Ok(Self::Mat3),
            "MAT4" => Ok(Self::Mat4),
            _ => Err(DeveloperError::new("attributeType is not a valid value.")),
        }
    }

    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Scalar => "SCALAR",
            Self::Vec2 => "VEC2",
            Self::Vec3 => "VEC3",
            Self::Vec4 => "VEC4",
            Self::Mat2 => "MAT2",
            Self::Mat3 => "MAT3",
            Self::Mat4 => "MAT4",
        }
    }

    pub const fn get_glsl_type(&self) -> &'static str {
        match self {
            Self::Scalar => "float",
            Self::Vec2 => "vec2",
            Self::Vec3 => "vec3",
            Self::Vec4 => "vec4",
            Self::Mat2 => "mat2",
            Self::Mat3 => "mat3",
            Self::Mat4 => "mat4",
        }
    }

    pub const fn get_number_of_components(&self) -> usize {
        match self {
            Self::Scalar => 1,
            Self::Vec2 => 2,
            Self::Vec3 => 3,
            Self::Vec4 | Self::Mat2 => 4,
            Self::Mat3 => 9,
            Self::Mat4 => 16,
        }
    }

    pub const fn get_attribute_location_count(&self) -> usize {
        match self {
            Self::Scalar | Self::Vec2 | Self::Vec3 | Self::Vec4 => 1,
            Self::Mat2 => 2,
            Self::Mat3 => 3,
            Self::Mat4 => 4,
        }
    }

    pub const fn get_math_type(&self) -> &'static str {
        match self {
            Self::Scalar => "Number",
            Self::Vec2 => "Cartesian2",
            Self::Vec3 => "Cartesian3",
            Self::Vec4 => "Cartesian4",
            Self::Mat2 => "Matrix2",
            Self::Mat3 => "Matrix3",
            Self::Mat4 => "Matrix4",
        }
    }
}
