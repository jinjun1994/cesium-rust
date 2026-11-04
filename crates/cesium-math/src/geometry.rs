//! Core geometry representations matching CesiumJS `Core/Geometry`, `Core/GeometryAttribute`, `Core/PrimitiveType`, and `Core/ComponentDatatype`.
//! Adapted from CesiumJS (Apache-2.0).

use crate::BoundingSphere;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PrimitiveType {
    Points = 0,
    Lines = 1,
    LineLoop = 2,
    LineStrip = 3,
    Triangles = 4,
    TriangleStrip = 5,
    TriangleFan = 6,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ComponentDatatype {
    Byte = 5120,
    UnsignedByte = 5121,
    Short = 5122,
    UnsignedShort = 5123,
    Int = 5124,
    UnsignedInt = 5125,
    Float = 5126,
    Double = 5130,
}

impl ComponentDatatype {
    pub const fn size_in_bytes(&self) -> usize {
        match self {
            Self::Byte | Self::UnsignedByte => 1,
            Self::Short | Self::UnsignedShort => 2,
            Self::Int | Self::UnsignedInt | Self::Float => 4,
            Self::Double => 8,
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "BYTE" => Some(Self::Byte),
            "UNSIGNED_BYTE" => Some(Self::UnsignedByte),
            "SHORT" => Some(Self::Short),
            "UNSIGNED_SHORT" => Some(Self::UnsignedShort),
            "INT" => Some(Self::Int),
            "UNSIGNED_INT" => Some(Self::UnsignedInt),
            "FLOAT" => Some(Self::Float),
            "DOUBLE" => Some(Self::Double),
            _ => None,
        }
    }

    pub fn from_value(value: u32) -> Option<Self> {
        match value {
            5120 => Some(Self::Byte),
            5121 => Some(Self::UnsignedByte),
            5122 => Some(Self::Short),
            5123 => Some(Self::UnsignedShort),
            5124 => Some(Self::Int),
            5125 => Some(Self::UnsignedInt),
            5126 => Some(Self::Float),
            5130 => Some(Self::Double),
            _ => None,
        }
    }

    pub fn validate(value: u32) -> bool {
        Self::from_value(value).is_some()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct GeometryAttribute {
    pub component_datatype: ComponentDatatype,
    pub components_per_attribute: u8,
    pub values: Vec<f64>,
}

impl GeometryAttribute {
    pub fn new(component_datatype: ComponentDatatype, components_per_attribute: u8, values: Vec<f64>) -> Self {
        Self {
            component_datatype,
            components_per_attribute,
            values,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum CornerType {
    #[default]
    Rounded = 0,
    Mitered = 1,
    Beveled = 2,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum GeometryOffsetAttribute {
    #[default]
    None = 0,
    Top = 1,
    All = 2,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct GeometryAttributes {
    pub position: Option<GeometryAttribute>,
    pub normal: Option<GeometryAttribute>,
    pub st: Option<GeometryAttribute>,
    pub tangent: Option<GeometryAttribute>,
    pub bitangent: Option<GeometryAttribute>,
    pub color: Option<GeometryAttribute>,
    pub apply_offset: Option<GeometryAttribute>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Geometry {
    pub attributes: GeometryAttributes,
    pub indices: Vec<u32>,
    pub primitive_type: PrimitiveType,
    pub bounding_sphere: Option<BoundingSphere>,
}
