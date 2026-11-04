//! Core geometry representations matching CesiumJS `Core/Geometry`, `Core/GeometryAttribute`, `Core/PrimitiveType`, and `Core/ComponentDatatype`.
//! Adapted from CesiumJS (Apache-2.0).

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

#[derive(Clone, Debug, Default, PartialEq)]
pub struct GeometryAttributes {
    pub position: Option<GeometryAttribute>,
    pub normal: Option<GeometryAttribute>,
    pub st: Option<GeometryAttribute>,
    pub tangent: Option<GeometryAttribute>,
    pub bitangent: Option<GeometryAttribute>,
    pub color: Option<GeometryAttribute>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Geometry {
    pub attributes: GeometryAttributes,
    pub indices: Vec<u32>,
    pub primitive_type: PrimitiveType,
}
