//! Color geometry instance attribute matching CesiumJS `Core/ColorGeometryInstanceAttribute`.
//! Adapted from CesiumJS (Apache-2.0).

use crate::{color::Color, geometry::ComponentDatatype};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ColorGeometryInstanceAttribute {
    pub component_datatype: ComponentDatatype,
    pub components_per_attribute: u8,
    pub normalize: bool,
    pub value: [u8; 4],
}

impl Default for ColorGeometryInstanceAttribute {
    fn default() -> Self {
        Self::new(1.0, 1.0, 1.0, 1.0)
    }
}

impl ColorGeometryInstanceAttribute {
    pub fn new(red: f32, green: f32, blue: f32, alpha: f32) -> Self {
        let color = Color::new(red, green, blue, alpha);
        Self {
            component_datatype: ComponentDatatype::UnsignedByte,
            components_per_attribute: 4,
            normalize: true,
            value: Self::to_value(&color),
        }
    }

    pub fn from_color(color: Color) -> Self {
        Self {
            component_datatype: ComponentDatatype::UnsignedByte,
            components_per_attribute: 4,
            normalize: true,
            value: Self::to_value(&color),
        }
    }

    pub fn to_value(color: &Color) -> [u8; 4] {
        color.to_bytes()
    }

    pub fn equals(&self, other: &Self) -> bool {
        self == other
    }
}
