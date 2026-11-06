//! PointPrimitive and PointPrimitiveCollection matching CesiumJS `Scene/PointPrimitiveCollection.js` and `Scene/PointPrimitive.js` (Apache-2.0).

use crate::{
    Cartesian3, Color, DistanceDisplayCondition, NearFarScalar, SplitDirection,
};

/// A single point primitive representation.
#[derive(Clone, Debug, PartialEq)]
pub struct PointPrimitive {
    pub id: Option<String>,
    pub show: bool,
    pub position: Cartesian3,
    pub color: Color,
    pub outline_color: Color,
    pub outline_width: f64,
    pub pixel_size: f64,
    pub scale_by_distance: Option<NearFarScalar>,
    pub translucency_by_distance: Option<NearFarScalar>,
    pub distance_display_condition: Option<DistanceDisplayCondition>,
    pub disable_depth_test_distance: f64,
    pub split_direction: SplitDirection,
}

impl Default for PointPrimitive {
    fn default() -> Self {
        Self {
            id: None,
            show: true,
            position: Cartesian3::ZERO,
            color: Color::WHITE,
            outline_color: Color::BLACK,
            outline_width: 0.0,
            pixel_size: 1.0,
            scale_by_distance: None,
            translucency_by_distance: None,
            distance_display_condition: None,
            disable_depth_test_distance: 0.0,
            split_direction: SplitDirection::None,
        }
    }
}

/// A renderable collection of point primitives.
#[derive(Clone, Debug, Default)]
pub struct PointPrimitiveCollection {
    pub point_primitives: Vec<PointPrimitive>,
}

impl PointPrimitiveCollection {
    pub fn new() -> Self {
        Self {
            point_primitives: Vec::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.point_primitives.len()
    }

    pub fn is_empty(&self) -> bool {
        self.point_primitives.is_empty()
    }

    pub fn get(&self, index: usize) -> Option<&PointPrimitive> {
        self.point_primitives.get(index)
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut PointPrimitive> {
        self.point_primitives.get_mut(index)
    }

    pub fn add(&mut self, primitive: PointPrimitive) -> usize {
        self.point_primitives.push(primitive);
        self.point_primitives.len() - 1
    }

    pub fn remove(&mut self, index: usize) {
        if index < self.point_primitives.len() {
            self.point_primitives.remove(index);
        }
    }

    pub fn remove_all(&mut self) {
        self.point_primitives.clear();
    }
}
