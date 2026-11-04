//! DistanceDisplayConditionGeometryInstanceAttribute matching CesiumJS `Core/DistanceDisplayConditionGeometryInstanceAttribute`.
//! Adapted from CesiumJS (Apache-2.0).

use crate::{distance_display_condition::DistanceDisplayCondition, geometry::ComponentDatatype};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DistanceDisplayConditionGeometryInstanceAttribute {
    pub component_datatype: ComponentDatatype,
    pub components_per_attribute: u8,
    pub normalize: bool,
    pub value: [f32; 2],
}

impl DistanceDisplayConditionGeometryInstanceAttribute {
    pub fn new(near: f32, far: f32) -> Option<Self> {
        if far <= near {
            return None;
        }
        Some(Self {
            component_datatype: ComponentDatatype::Float,
            components_per_attribute: 2,
            normalize: false,
            value: [near, far],
        })
    }

    pub fn from_distance_display_condition(dc: &DistanceDisplayCondition) -> Option<Self> {
        if dc.far <= dc.near {
            return None;
        }
        Some(Self {
            component_datatype: ComponentDatatype::Float,
            components_per_attribute: 2,
            normalize: false,
            value: Self::to_value(dc),
        })
    }

    pub fn to_value(dc: &DistanceDisplayCondition) -> [f32; 2] {
        [dc.near as f32, dc.far as f32]
    }
}
