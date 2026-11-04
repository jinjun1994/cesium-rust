//! ShowGeometryInstanceAttribute matching CesiumJS `Core/ShowGeometryInstanceAttribute`.
//! Adapted from CesiumJS (Apache-2.0).

use crate::geometry::ComponentDatatype;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ShowGeometryInstanceAttribute {
    pub component_datatype: ComponentDatatype,
    pub components_per_attribute: u8,
    pub normalize: bool,
    pub value: [u8; 1],
}

impl Default for ShowGeometryInstanceAttribute {
    fn default() -> Self {
        Self::new(true)
    }
}

impl ShowGeometryInstanceAttribute {
    pub fn new(show: bool) -> Self {
        Self {
            component_datatype: ComponentDatatype::UnsignedByte,
            components_per_attribute: 1,
            normalize: false,
            value: Self::to_value(show),
        }
    }

    pub fn to_value(show: bool) -> [u8; 1] {
        [if show { 1 } else { 0 }]
    }
}
