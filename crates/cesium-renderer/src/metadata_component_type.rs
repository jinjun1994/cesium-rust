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
            Self::Uint8 => u8::MIN as f64,
            Self::Int16 => i16::MIN as f64,
            Self::Uint16 => u16::MIN as f64,
            Self::Int32 => i32::MIN as f64,
            Self::Uint32 => u32::MIN as f64,
            Self::Int64 => i64::MIN as f64,
            Self::Uint64 => u64::MIN as f64,
            Self::Float32 => f32::MIN as f64,
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
            Self::Int64 => i64::MAX as f64,
            Self::Uint64 => u64::MAX as f64,
            Self::Float32 => f32::MAX as f64,
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
