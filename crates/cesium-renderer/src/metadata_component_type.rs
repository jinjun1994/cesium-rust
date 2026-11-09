//! MetadataComponentType matching CesiumJS `Scene/MetadataComponentType.js` (Apache-2.0).

use cesium_math::DeveloperError;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// Metadata component types.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum MetadataComponentType {
    /// An 8-bit signed integer.
    #[serde(rename = "INT8")]
    Int8,
    /// An 8-bit unsigned integer.
    #[serde(rename = "UINT8")]
    Uint8,
    /// A 16-bit signed integer.
    #[serde(rename = "INT16")]
    Int16,
    /// A 16-bit unsigned integer.
    #[serde(rename = "UINT16")]
    Uint16,
    /// A 32-bit signed integer.
    #[serde(rename = "INT32")]
    Int32,
    /// A 32-bit unsigned integer.
    #[serde(rename = "UINT32")]
    Uint32,
    /// A 64-bit signed integer.
    #[serde(rename = "INT64")]
    Int64,
    /// A 64-bit unsigned integer.
    #[serde(rename = "UINT64")]
    Uint64,
    /// A 32-bit floating point number.
    #[serde(rename = "FLOAT32")]
    Float32,
    /// A 64-bit floating point number.
    #[serde(rename = "FLOAT64")]
    Float64,
}

impl MetadataComponentType {
    /// Returns the size in bytes for this component type.
    pub const fn get_size_in_bytes(component_type: Self) -> usize {
        match component_type {
            Self::Int8 | Self::Uint8 => 1,
            Self::Int16 | Self::Uint16 => 2,
            Self::Int32 | Self::Uint32 | Self::Float32 => 4,
            Self::Int64 | Self::Uint64 | Self::Float64 => 8,
        }
    }

    /// Returns the minimum value as an f64.
    pub fn get_minimum(component_type: Self) -> f64 {
        match component_type {
            Self::Int8 => i8::MIN as f64,
            Self::Uint8 => 0.0,
            Self::Int16 => i16::MIN as f64,
            Self::Uint16 => 0.0,
            Self::Int32 => i32::MIN as f64,
            Self::Uint32 => 0.0,
            Self::Int64 => -9223372036854775808.0,
            Self::Uint64 => 0.0,
            Self::Float32 => -340282346638528859811704183484516925440.0,
            Self::Float64 => -f64::MAX,
        }
    }

    /// Returns the maximum value as an f64.
    pub fn get_maximum(component_type: Self) -> f64 {
        match component_type {
            Self::Int8 => i8::MAX as f64,
            Self::Uint8 => u8::MAX as f64,
            Self::Int16 => i16::MAX as f64,
            Self::Uint16 => u16::MAX as f64,
            Self::Int32 => i32::MAX as f64,
            Self::Uint32 => u32::MAX as f64,
            Self::Int64 => 9223372036854775807.0,
            Self::Uint64 => 18446744073709551615.0,
            Self::Float32 => 340282346638528859811704183484516925440.0,
            Self::Float64 => f64::MAX,
        }
    }

    /// Returns the string representation.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Int8 => "INT8",
            Self::Uint8 => "UINT8",
            Self::Int16 => "INT16",
            Self::Uint16 => "UINT16",
            Self::Int32 => "INT32",
            Self::Uint32 => "UINT32",
            Self::Int64 => "INT64",
            Self::Uint64 => "UINT64",
            Self::Float32 => "FLOAT32",
            Self::Float64 => "FLOAT64",
        }
    }

    /// Returns whether this component type is an integer type.
    pub const fn is_integer_type(&self) -> bool {
        matches!(
            self,
            Self::Int8
                | Self::Uint8
                | Self::Int16
                | Self::Uint16
                | Self::Int32
                | Self::Uint32
                | Self::Int64
                | Self::Uint64
        )
    }

    /// Returns whether this component type is an unsigned integer type.
    pub const fn is_unsigned_integer_type(&self) -> bool {
        matches!(
            self,
            Self::Uint8 | Self::Uint16 | Self::Uint32 | Self::Uint64
        )
    }

    /// Returns whether this component type can be used as vector components.
    pub const fn is_vector_compatible(&self) -> bool {
        !matches!(self, Self::Int64 | Self::Uint64)
    }

    /// Returns the category ("int", "uint", or "float").
    pub const fn category(&self) -> &'static str {
        match self {
            Self::Int8 | Self::Int16 | Self::Int32 | Self::Int64 => "int",
            Self::Uint8 | Self::Uint16 | Self::Uint32 | Self::Uint64 => "uint",
            Self::Float32 | Self::Float64 => "float",
        }
    }

    /// Returns the GPU component type taking downcasting into account.
    pub const fn gpu_component_type(&self) -> Self {
        match self {
            Self::Int64 => Self::Int32,
            Self::Uint64 => Self::Uint32,
            Self::Float64 => Self::Float32,
            other => *other,
        }
    }

    /// Normalizes signed integers to [-1.0, 1.0] and unsigned integers to [0.0, 1.0].
    pub fn normalize(value: f64, component_type: Self) -> Result<f64, DeveloperError> {
        if !component_type.is_integer_type() {
            return Err(DeveloperError::new("type must be an integer type"));
        }
        let max = Self::get_maximum(component_type);
        Ok((value / max).max(-1.0))
    }

    /// Unnormalizes numbers into integer values.
    pub fn unnormalize(value: f64, component_type: Self) -> Result<f64, DeveloperError> {
        if !component_type.is_integer_type() {
            return Err(DeveloperError::new("type must be an integer type"));
        }
        let max = Self::get_maximum(component_type);
        let min = if component_type.is_unsigned_integer_type() {
            0.0
        } else {
            -max
        };

        let rounded = value.signum() * (value.abs() * max).round();
        Ok(rounded.clamp(min, max))
    }

    /// Applies offset and scale transform: scale * value + offset.
    pub fn apply_value_transform(value: f64, offset: f64, scale: f64) -> f64 {
        scale * value + offset
    }

    /// Inverts value transform: (value - offset) / scale.
    pub fn unapply_value_transform(value: f64, offset: f64, scale: f64) -> f64 {
        if scale == 0.0 {
            0.0
        } else {
            (value - offset) / scale
        }
    }

    /// Clamps value to component type limits.
    pub fn clamp_to_limits(value: f64, component_type: Self) -> f64 {
        value.clamp(
            Self::get_minimum(component_type),
            Self::get_maximum(component_type),
        )
    }
}

impl FromStr for MetadataComponentType {
    type Err = DeveloperError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "INT8" => Ok(Self::Int8),
            "UINT8" => Ok(Self::Uint8),
            "INT16" => Ok(Self::Int16),
            "UINT16" => Ok(Self::Uint16),
            "INT32" => Ok(Self::Int32),
            "UINT32" => Ok(Self::Uint32),
            "INT64" => Ok(Self::Int64),
            "UINT64" => Ok(Self::Uint64),
            "FLOAT32" => Ok(Self::Float32),
            "FLOAT64" => Ok(Self::Float64),
            _ => Err(DeveloperError::new(format!(
                "Invalid MetadataComponentType: {s}"
            ))),
        }
    }
}
