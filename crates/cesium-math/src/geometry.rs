//! Core geometry representations matching CesiumJS `Core/Geometry`, `Core/GeometryAttribute`, `Core/PrimitiveType`, and `Core/ComponentDatatype`.
//! Adapted from CesiumJS (Apache-2.0).

use crate::{BoundingSphere, DeveloperError, Matrix4};
use std::collections::HashMap;

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

impl PrimitiveType {
    pub const fn is_lines(&self) -> bool {
        matches!(self, Self::Lines | Self::LineLoop | Self::LineStrip)
    }

    pub const fn is_triangles(&self) -> bool {
        matches!(self, Self::Triangles | Self::TriangleStrip | Self::TriangleFan)
    }

    pub fn validate(value: Option<i32>) -> bool {
        match value {
            Some(v) => (0..=6).contains(&v),
            None => false,
        }
    }

    pub const fn from_i32(value: i32) -> Option<Self> {
        match value {
            0 => Some(Self::Points),
            1 => Some(Self::Lines),
            2 => Some(Self::LineLoop),
            3 => Some(Self::LineStrip),
            4 => Some(Self::Triangles),
            5 => Some(Self::TriangleStrip),
            6 => Some(Self::TriangleFan),
            _ => None,
        }
    }
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
    pub normalize: bool,
    pub values: Vec<f64>,
}

impl GeometryAttribute {
    pub fn new(
        component_datatype: ComponentDatatype,
        components_per_attribute: u8,
        values: Vec<f64>,
    ) -> Self {
        Self {
            component_datatype,
            components_per_attribute,
            normalize: false,
            values,
        }
    }

    pub fn with_normalize(
        component_datatype: ComponentDatatype,
        components_per_attribute: u8,
        normalize: bool,
        values: Vec<f64>,
    ) -> Result<Self, DeveloperError> {
        if !(1..=4).contains(&components_per_attribute) {
            return Err(DeveloperError::new(
                "componentsPerAttribute must be between 1 and 4.",
            ));
        }
        Ok(Self {
            component_datatype,
            components_per_attribute,
            normalize,
            values,
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct GeometryInstanceAttribute {
    pub component_datatype: ComponentDatatype,
    pub components_per_attribute: u8,
    pub normalize: bool,
    pub value: Vec<f64>,
}

impl GeometryInstanceAttribute {
    pub fn new(
        component_datatype: ComponentDatatype,
        components_per_attribute: u8,
        normalize: bool,
        value: Vec<f64>,
    ) -> Result<Self, DeveloperError> {
        if !(1..=4).contains(&components_per_attribute) {
            return Err(DeveloperError::new(
                "componentsPerAttribute must be between 1 and 4.",
            ));
        }
        Ok(Self {
            component_datatype,
            components_per_attribute,
            normalize,
            value,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum GeometryType {
    #[default]
    None = 0,
    Triangles = 1,
    Lines = 2,
    Polylines = 3,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ArcType {
    None = 0,
    #[default]
    Geodesic = 1,
    Rhumb = 2,
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
    pub prev_position: Option<GeometryAttribute>,
    pub next_position: Option<GeometryAttribute>,
    pub expand_and_width: Option<GeometryAttribute>,
    pub custom: HashMap<String, GeometryAttribute>,
}

impl GeometryAttributes {
    pub fn get(&self, name: &str) -> Option<&GeometryAttribute> {
        match name {
            "position" => self.position.as_ref(),
            "normal" => self.normal.as_ref(),
            "st" => self.st.as_ref(),
            "tangent" => self.tangent.as_ref(),
            "bitangent" => self.bitangent.as_ref(),
            "color" => self.color.as_ref(),
            "apply_offset" | "applyOffset" => self.apply_offset.as_ref(),
            "prev_position" | "prevPosition" => self.prev_position.as_ref(),
            "next_position" | "nextPosition" => self.next_position.as_ref(),
            "expand_and_width" | "expandAndWidth" => self.expand_and_width.as_ref(),
            _ => self.custom.get(name),
        }
    }

    pub fn get_mut(&mut self, name: &str) -> Option<&mut GeometryAttribute> {
        match name {
            "position" => self.position.as_mut(),
            "normal" => self.normal.as_mut(),
            "st" => self.st.as_mut(),
            "tangent" => self.tangent.as_mut(),
            "bitangent" => self.bitangent.as_mut(),
            "color" => self.color.as_mut(),
            "apply_offset" | "applyOffset" => self.apply_offset.as_mut(),
            "prev_position" | "prevPosition" => self.prev_position.as_mut(),
            "next_position" | "nextPosition" => self.next_position.as_mut(),
            "expand_and_width" | "expandAndWidth" => self.expand_and_width.as_mut(),
            _ => self.custom.get_mut(name),
        }
    }

    pub fn set(&mut self, name: &str, attr: GeometryAttribute) {
        match name {
            "position" => self.position = Some(attr),
            "normal" => self.normal = Some(attr),
            "st" => self.st = Some(attr),
            "tangent" => self.tangent = Some(attr),
            "bitangent" => self.bitangent = Some(attr),
            "color" => self.color = Some(attr),
            "apply_offset" | "applyOffset" => self.apply_offset = Some(attr),
            "prev_position" | "prevPosition" => self.prev_position = Some(attr),
            "next_position" | "nextPosition" => self.next_position = Some(attr),
            "expand_and_width" | "expandAndWidth" => self.expand_and_width = Some(attr),
            _ => {
                self.custom.insert(name.to_string(), attr);
            }
        }
    }

    pub fn remove(&mut self, name: &str) -> Option<GeometryAttribute> {
        match name {
            "position" => self.position.take(),
            "normal" => self.normal.take(),
            "st" => self.st.take(),
            "tangent" => self.tangent.take(),
            "bitangent" => self.bitangent.take(),
            "color" => self.color.take(),
            "apply_offset" | "applyOffset" => self.apply_offset.take(),
            "prev_position" | "prevPosition" => self.prev_position.take(),
            "next_position" | "nextPosition" => self.next_position.take(),
            "expand_and_width" | "expandAndWidth" => self.expand_and_width.take(),
            _ => self.custom.remove(name),
        }
    }

    pub fn contains_key(&self, name: &str) -> bool {
        self.get(name).is_some()
    }

    pub fn iter(&self) -> Vec<(&str, &GeometryAttribute)> {
        let mut list = Vec::new();
        if let Some(a) = &self.position {
            list.push(("position", a));
        }
        if let Some(a) = &self.normal {
            list.push(("normal", a));
        }
        if let Some(a) = &self.st {
            list.push(("st", a));
        }
        if let Some(a) = &self.tangent {
            list.push(("tangent", a));
        }
        if let Some(a) = &self.bitangent {
            list.push(("bitangent", a));
        }
        if let Some(a) = &self.color {
            list.push(("color", a));
        }
        if let Some(a) = &self.apply_offset {
            list.push(("applyOffset", a));
        }
        if let Some(a) = &self.prev_position {
            list.push(("prevPosition", a));
        }
        if let Some(a) = &self.next_position {
            list.push(("nextPosition", a));
        }
        if let Some(a) = &self.expand_and_width {
            list.push(("expandAndWidth", a));
        }
        for (k, v) in &self.custom {
            list.push((k.as_str(), v));
        }
        list
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Geometry {
    pub attributes: GeometryAttributes,
    pub indices: Vec<u32>,
    pub primitive_type: PrimitiveType,
    pub bounding_sphere: Option<BoundingSphere>,
}

impl Geometry {
    pub fn new(
        attributes: GeometryAttributes,
        indices: Vec<u32>,
        primitive_type: PrimitiveType,
        bounding_sphere: Option<BoundingSphere>,
    ) -> Self {
        Self {
            attributes,
            indices,
            primitive_type,
            bounding_sphere,
        }
    }

    pub fn compute_number_of_vertices(&self) -> Result<usize, DeveloperError> {
        let mut count: Option<usize> = None;
        for (_name, attr) in self.attributes.iter() {
            let components = attr.components_per_attribute as usize;
            if components == 0 {
                return Err(DeveloperError::new(
                    "componentsPerAttribute must be greater than zero.",
                ));
            }
            if !attr.values.len().is_multiple_of(components) {
                return Err(DeveloperError::new(
                    "Attribute values length is not a multiple of componentsPerAttribute.",
                ));
            }
            let num = attr.values.len() / components;
            if let Some(c) = count {
                if c != num {
                    return Err(DeveloperError::new(
                        "All attribute lists must have the same number of attributes.",
                    ));
                }
            } else {
                count = Some(num);
            }
        }
        Ok(count.unwrap_or(0))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct GeometryInstance {
    pub geometry: Geometry,
    pub model_matrix: Matrix4,
    pub id: Option<String>,
    pub attributes: HashMap<String, GeometryInstanceAttribute>,
}

impl GeometryInstance {
    pub fn new(
        geometry: Geometry,
        model_matrix: Option<Matrix4>,
        id: Option<String>,
        attributes: Option<HashMap<String, GeometryInstanceAttribute>>,
    ) -> Self {
        Self {
            geometry,
            model_matrix: model_matrix.unwrap_or(Matrix4::IDENTITY),
            id,
            attributes: attributes.unwrap_or_default(),
        }
    }
}
