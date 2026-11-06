//! Billboard and BillboardCollection matching CesiumJS `Scene/BillboardCollection.js` and `Scene/Billboard.js` (Apache-2.0).

use crate::{
    BoundingRectangle, Cartesian2, Cartesian3, Color, DistanceDisplayCondition, HeightReference,
    HorizontalOrigin, NearFarScalar, SplitDirection, VerticalOrigin,
};

/// A single billboard representation managed by BillboardCollection.
#[derive(Clone, Debug, PartialEq)]
pub struct Billboard {
    pub id: Option<String>,
    pub show: bool,
    pub position: Cartesian3,
    pub color: Color,
    pub eye_offset: Cartesian3,
    pub pixel_offset: Cartesian2,
    pub scale: f64,
    pub rotation: f64,
    pub aligned_axis: Cartesian3,
    pub horizontal_origin: HorizontalOrigin,
    pub vertical_origin: VerticalOrigin,
    pub height_reference: HeightReference,
    pub width: Option<f64>,
    pub height: Option<f64>,
    pub scale_by_distance: Option<NearFarScalar>,
    pub translucency_by_distance: Option<NearFarScalar>,
    pub pixel_offset_scale_by_distance: Option<NearFarScalar>,
    pub size_in_meters: bool,
    pub distance_display_condition: Option<DistanceDisplayCondition>,
    pub disable_depth_test_distance: Option<f64>,
    pub split_direction: SplitDirection,
    pub image: Option<String>,
    pub image_sub_region: Option<BoundingRectangle>,
    pub ready: bool,
}

impl Default for Billboard {
    fn default() -> Self {
        Self {
            id: None,
            show: true,
            position: Cartesian3::ZERO,
            color: Color::WHITE,
            eye_offset: Cartesian3::ZERO,
            pixel_offset: Cartesian2::ZERO,
            scale: 1.0,
            rotation: 0.0,
            aligned_axis: Cartesian3::ZERO,
            horizontal_origin: HorizontalOrigin::Center,
            vertical_origin: VerticalOrigin::Center,
            height_reference: HeightReference::None,
            width: None,
            height: None,
            scale_by_distance: None,
            translucency_by_distance: None,
            pixel_offset_scale_by_distance: None,
            size_in_meters: false,
            distance_display_condition: None,
            disable_depth_test_distance: None,
            split_direction: SplitDirection::None,
            image: None,
            image_sub_region: None,
            ready: true,
        }
    }
}

/// A renderable collection of billboards.
#[derive(Clone, Debug, Default)]
pub struct BillboardCollection {
    pub billboards: Vec<Billboard>,
}

impl BillboardCollection {
    pub fn new() -> Self {
        Self {
            billboards: Vec::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.billboards.len()
    }

    pub fn is_empty(&self) -> bool {
        self.billboards.is_empty()
    }

    pub fn get(&self, index: usize) -> Option<&Billboard> {
        self.billboards.get(index)
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut Billboard> {
        self.billboards.get_mut(index)
    }

    pub fn add(&mut self, billboard: Billboard) -> usize {
        self.billboards.push(billboard);
        self.billboards.len() - 1
    }

    pub fn remove(&mut self, index: usize) {
        if index < self.billboards.len() {
            self.billboards.remove(index);
        }
    }

    pub fn remove_all(&mut self) {
        self.billboards.clear();
    }
}
