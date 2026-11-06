//! Label and LabelCollection matching CesiumJS `Scene/LabelCollection.js` and `Scene/Label.js` (Apache-2.0).

use crate::{
    Cartesian2, Cartesian3, Color, DistanceDisplayCondition, HeightReference,
    HorizontalOrigin, LabelStyle, NearFarScalar, VerticalOrigin,
};

/// A single label representation managed by LabelCollection.
#[derive(Clone, Debug, PartialEq)]
pub struct Label {
    pub id: Option<String>,
    pub show: bool,
    pub position: Cartesian3,
    pub text: String,
    pub font: String,
    pub style: LabelStyle,
    pub fill_color: Color,
    pub outline_color: Color,
    pub outline_width: f64,
    pub show_background: bool,
    pub background_color: Color,
    pub background_padding: Cartesian2,
    pub scale: f64,
    pub horizontal_origin: HorizontalOrigin,
    pub vertical_origin: VerticalOrigin,
    pub eye_offset: Cartesian3,
    pub pixel_offset: Cartesian2,
    pub height_reference: HeightReference,
    pub translucency_by_distance: Option<NearFarScalar>,
    pub pixel_offset_scale_by_distance: Option<NearFarScalar>,
    pub scale_by_distance: Option<NearFarScalar>,
    pub distance_display_condition: Option<DistanceDisplayCondition>,
    pub disable_depth_test_distance: Option<f64>,
    pub clamped_position: Option<Cartesian3>,
}

impl Default for Label {
    fn default() -> Self {
        Self {
            id: None,
            show: true,
            position: Cartesian3::ZERO,
            text: String::new(),
            font: "30px sans-serif".to_string(),
            style: LabelStyle::Fill,
            fill_color: Color::WHITE,
            outline_color: Color::BLACK,
            outline_width: 1.0,
            show_background: false,
            background_color: Color::new(0.165, 0.165, 0.165, 0.8),
            background_padding: Cartesian2::new(7.0, 5.0),
            scale: 1.0,
            horizontal_origin: HorizontalOrigin::Center,
            vertical_origin: VerticalOrigin::Center,
            eye_offset: Cartesian3::ZERO,
            pixel_offset: Cartesian2::ZERO,
            height_reference: HeightReference::None,
            translucency_by_distance: None,
            pixel_offset_scale_by_distance: None,
            scale_by_distance: None,
            distance_display_condition: None,
            disable_depth_test_distance: None,
            clamped_position: None,
        }
    }
}

impl Label {
    /// Updates position clamping according to height reference.
    pub fn update_clamping(&mut self) {
        if self.height_reference != HeightReference::None {
            self.clamped_position = Some(self.position);
        } else {
            self.clamped_position = None;
        }
    }
}

/// A renderable collection of labels.
#[derive(Clone, Debug, Default)]
pub struct LabelCollection {
    pub labels: Vec<Label>,
}

impl LabelCollection {
    pub fn new() -> Self {
        Self {
            labels: Vec::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.labels.len()
    }

    pub fn is_empty(&self) -> bool {
        self.labels.is_empty()
    }

    pub fn get(&self, index: usize) -> Option<&Label> {
        self.labels.get(index)
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut Label> {
        self.labels.get_mut(index)
    }

    pub fn add(&mut self, label: Label) -> usize {
        self.labels.push(label);
        self.labels.len() - 1
    }

    pub fn remove(&mut self, index: usize) {
        if index < self.labels.len() {
            self.labels.remove(index);
        }
    }

    pub fn remove_all(&mut self) {
        self.labels.clear();
    }
}
