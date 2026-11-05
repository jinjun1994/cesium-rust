//! LabelGraphics matching CesiumJS `DataSources/LabelGraphics.js` (Apache-2.0).

use std::rc::Rc;

use crate::{
    Cartesian2, Cartesian3, Color, ConstantProperty, DistanceDisplayCondition, DynProperty, Event,
    HeightReference, HorizontalOrigin, LabelStyle, NearFarScalar, VerticalOrigin,
};

/// Initialization options for LabelGraphics using dynamic or constant properties.
#[derive(Default, Clone)]
pub struct LabelGraphicsOptions {
    pub show: Option<Rc<dyn DynProperty<bool>>>,
    pub text: Option<Rc<dyn DynProperty<String>>>,
    pub font: Option<Rc<dyn DynProperty<String>>>,
    pub style: Option<Rc<dyn DynProperty<LabelStyle>>>,
    pub scale: Option<Rc<dyn DynProperty<f64>>>,
    pub show_background: Option<Rc<dyn DynProperty<bool>>>,
    pub background_color: Option<Rc<dyn DynProperty<Color>>>,
    pub background_padding: Option<Rc<dyn DynProperty<Cartesian2>>>,
    pub pixel_offset: Option<Rc<dyn DynProperty<Cartesian2>>>,
    pub eye_offset: Option<Rc<dyn DynProperty<Cartesian3>>>,
    pub horizontal_origin: Option<Rc<dyn DynProperty<HorizontalOrigin>>>,
    pub vertical_origin: Option<Rc<dyn DynProperty<VerticalOrigin>>>,
    pub height_reference: Option<Rc<dyn DynProperty<HeightReference>>>,
    pub fill_color: Option<Rc<dyn DynProperty<Color>>>,
    pub outline_color: Option<Rc<dyn DynProperty<Color>>>,
    pub outline_width: Option<Rc<dyn DynProperty<f64>>>,
    pub translucency_by_distance: Option<Rc<dyn DynProperty<NearFarScalar>>>,
    pub pixel_offset_scale_by_distance: Option<Rc<dyn DynProperty<NearFarScalar>>>,
    pub scale_by_distance: Option<Rc<dyn DynProperty<NearFarScalar>>>,
    pub distance_display_condition: Option<Rc<dyn DynProperty<DistanceDisplayCondition>>>,
    pub disable_depth_test_distance: Option<Rc<dyn DynProperty<f64>>>,
}

/// Raw literal initialization values for LabelGraphics (convenience constructor).
#[derive(Default, Clone, Debug)]
pub struct LabelGraphicsValues {
    pub show: Option<bool>,
    pub text: Option<String>,
    pub font: Option<String>,
    pub style: Option<LabelStyle>,
    pub scale: Option<f64>,
    pub show_background: Option<bool>,
    pub background_color: Option<Color>,
    pub background_padding: Option<Cartesian2>,
    pub pixel_offset: Option<Cartesian2>,
    pub eye_offset: Option<Cartesian3>,
    pub horizontal_origin: Option<HorizontalOrigin>,
    pub vertical_origin: Option<VerticalOrigin>,
    pub height_reference: Option<HeightReference>,
    pub fill_color: Option<Color>,
    pub outline_color: Option<Color>,
    pub outline_width: Option<f64>,
    pub translucency_by_distance: Option<NearFarScalar>,
    pub pixel_offset_scale_by_distance: Option<NearFarScalar>,
    pub scale_by_distance: Option<NearFarScalar>,
    pub distance_display_condition: Option<DistanceDisplayCondition>,
    pub disable_depth_test_distance: Option<f64>,
}

/// Describes a two dimensional label located at the position of the containing Entity.
#[derive(Clone, Default)]
pub struct LabelGraphics {
    pub show: Option<Rc<dyn DynProperty<bool>>>,
    pub text: Option<Rc<dyn DynProperty<String>>>,
    pub font: Option<Rc<dyn DynProperty<String>>>,
    pub style: Option<Rc<dyn DynProperty<LabelStyle>>>,
    pub scale: Option<Rc<dyn DynProperty<f64>>>,
    pub show_background: Option<Rc<dyn DynProperty<bool>>>,
    pub background_color: Option<Rc<dyn DynProperty<Color>>>,
    pub background_padding: Option<Rc<dyn DynProperty<Cartesian2>>>,
    pub pixel_offset: Option<Rc<dyn DynProperty<Cartesian2>>>,
    pub eye_offset: Option<Rc<dyn DynProperty<Cartesian3>>>,
    pub horizontal_origin: Option<Rc<dyn DynProperty<HorizontalOrigin>>>,
    pub vertical_origin: Option<Rc<dyn DynProperty<VerticalOrigin>>>,
    pub height_reference: Option<Rc<dyn DynProperty<HeightReference>>>,
    pub fill_color: Option<Rc<dyn DynProperty<Color>>>,
    pub outline_color: Option<Rc<dyn DynProperty<Color>>>,
    pub outline_width: Option<Rc<dyn DynProperty<f64>>>,
    pub translucency_by_distance: Option<Rc<dyn DynProperty<NearFarScalar>>>,
    pub pixel_offset_scale_by_distance: Option<Rc<dyn DynProperty<NearFarScalar>>>,
    pub scale_by_distance: Option<Rc<dyn DynProperty<NearFarScalar>>>,
    pub distance_display_condition: Option<Rc<dyn DynProperty<DistanceDisplayCondition>>>,
    pub disable_depth_test_distance: Option<Rc<dyn DynProperty<f64>>>,
    definition_changed: Event<()>,
}

impl LabelGraphics {
    /// Creates a new LabelGraphics instance with undefined properties.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a LabelGraphics instance from property options.
    pub fn from_options(options: LabelGraphicsOptions) -> Self {
        Self {
            show: options.show,
            text: options.text,
            font: options.font,
            style: options.style,
            scale: options.scale,
            show_background: options.show_background,
            background_color: options.background_color,
            background_padding: options.background_padding,
            pixel_offset: options.pixel_offset,
            eye_offset: options.eye_offset,
            horizontal_origin: options.horizontal_origin,
            vertical_origin: options.vertical_origin,
            height_reference: options.height_reference,
            fill_color: options.fill_color,
            outline_color: options.outline_color,
            outline_width: options.outline_width,
            translucency_by_distance: options.translucency_by_distance,
            pixel_offset_scale_by_distance: options.pixel_offset_scale_by_distance,
            scale_by_distance: options.scale_by_distance,
            distance_display_condition: options.distance_display_condition,
            disable_depth_test_distance: options.disable_depth_test_distance,
            definition_changed: Event::new(),
        }
    }

    /// Creates a LabelGraphics instance from raw constant values.
    pub fn from_values(values: LabelGraphicsValues) -> Self {
        Self::from_options(LabelGraphicsOptions {
            show: values
                .show
                .map(|v| Rc::new(ConstantProperty::from_value(v)) as Rc<dyn DynProperty<bool>>),
            text: values
                .text
                .map(|v| Rc::new(ConstantProperty::from_value(v)) as Rc<dyn DynProperty<String>>),
            font: values
                .font
                .map(|v| Rc::new(ConstantProperty::from_value(v)) as Rc<dyn DynProperty<String>>),
            style: values.style.map(|v| {
                Rc::new(ConstantProperty::from_value(v)) as Rc<dyn DynProperty<LabelStyle>>
            }),
            scale: values
                .scale
                .map(|v| Rc::new(ConstantProperty::from_value(v)) as Rc<dyn DynProperty<f64>>),
            show_background: values
                .show_background
                .map(|v| Rc::new(ConstantProperty::from_value(v)) as Rc<dyn DynProperty<bool>>),
            background_color: values
                .background_color
                .map(|v| Rc::new(ConstantProperty::from_value(v)) as Rc<dyn DynProperty<Color>>),
            background_padding: values.background_padding.map(|v| {
                Rc::new(ConstantProperty::from_value(v)) as Rc<dyn DynProperty<Cartesian2>>
            }),
            pixel_offset: values
                .pixel_offset
                .map(|v| Rc::new(ConstantProperty::from_value(v)) as Rc<dyn DynProperty<Cartesian2>>),
            eye_offset: values
                .eye_offset
                .map(|v| Rc::new(ConstantProperty::from_value(v)) as Rc<dyn DynProperty<Cartesian3>>),
            horizontal_origin: values.horizontal_origin.map(|v| {
                Rc::new(ConstantProperty::from_value(v)) as Rc<dyn DynProperty<HorizontalOrigin>>
            }),
            vertical_origin: values.vertical_origin.map(|v| {
                Rc::new(ConstantProperty::from_value(v)) as Rc<dyn DynProperty<VerticalOrigin>>
            }),
            height_reference: values.height_reference.map(|v| {
                Rc::new(ConstantProperty::from_value(v)) as Rc<dyn DynProperty<HeightReference>>
            }),
            fill_color: values
                .fill_color
                .map(|v| Rc::new(ConstantProperty::from_value(v)) as Rc<dyn DynProperty<Color>>),
            outline_color: values
                .outline_color
                .map(|v| Rc::new(ConstantProperty::from_value(v)) as Rc<dyn DynProperty<Color>>),
            outline_width: values
                .outline_width
                .map(|v| Rc::new(ConstantProperty::from_value(v)) as Rc<dyn DynProperty<f64>>),
            translucency_by_distance: values.translucency_by_distance.map(|v| {
                Rc::new(ConstantProperty::from_value(v)) as Rc<dyn DynProperty<NearFarScalar>>
            }),
            pixel_offset_scale_by_distance: values.pixel_offset_scale_by_distance.map(|v| {
                Rc::new(ConstantProperty::from_value(v)) as Rc<dyn DynProperty<NearFarScalar>>
            }),
            scale_by_distance: values.scale_by_distance.map(|v| {
                Rc::new(ConstantProperty::from_value(v)) as Rc<dyn DynProperty<NearFarScalar>>
            }),
            distance_display_condition: values.distance_display_condition.map(|v| {
                Rc::new(ConstantProperty::from_value(v))
                    as Rc<dyn DynProperty<DistanceDisplayCondition>>
            }),
            disable_depth_test_distance: values
                .disable_depth_test_distance
                .map(|v| Rc::new(ConstantProperty::from_value(v)) as Rc<dyn DynProperty<f64>>),
        })
    }

    /// Gets the event that is raised whenever a property is changed or modified.
    pub fn definition_changed(&self) -> &Event<()> {
        &self.definition_changed
    }

    /// Assigns each unassigned property on this object to the value of the same property on the source.
    pub fn merge(&mut self, source: &LabelGraphics) {
        if self.show.is_none() {
            self.show = source.show.clone();
        }
        if self.text.is_none() {
            self.text = source.text.clone();
        }
        if self.font.is_none() {
            self.font = source.font.clone();
        }
        if self.style.is_none() {
            self.style = source.style.clone();
        }
        if self.scale.is_none() {
            self.scale = source.scale.clone();
        }
        if self.show_background.is_none() {
            self.show_background = source.show_background.clone();
        }
        if self.background_color.is_none() {
            self.background_color = source.background_color.clone();
        }
        if self.background_padding.is_none() {
            self.background_padding = source.background_padding.clone();
        }
        if self.pixel_offset.is_none() {
            self.pixel_offset = source.pixel_offset.clone();
        }
        if self.eye_offset.is_none() {
            self.eye_offset = source.eye_offset.clone();
        }
        if self.horizontal_origin.is_none() {
            self.horizontal_origin = source.horizontal_origin.clone();
        }
        if self.vertical_origin.is_none() {
            self.vertical_origin = source.vertical_origin.clone();
        }
        if self.height_reference.is_none() {
            self.height_reference = source.height_reference.clone();
        }
        if self.fill_color.is_none() {
            self.fill_color = source.fill_color.clone();
        }
        if self.outline_color.is_none() {
            self.outline_color = source.outline_color.clone();
        }
        if self.outline_width.is_none() {
            self.outline_width = source.outline_width.clone();
        }
        if self.translucency_by_distance.is_none() {
            self.translucency_by_distance = source.translucency_by_distance.clone();
        }
        if self.pixel_offset_scale_by_distance.is_none() {
            self.pixel_offset_scale_by_distance = source.pixel_offset_scale_by_distance.clone();
        }
        if self.scale_by_distance.is_none() {
            self.scale_by_distance = source.scale_by_distance.clone();
        }
        if self.distance_display_condition.is_none() {
            self.distance_display_condition = source.distance_display_condition.clone();
        }
        if self.disable_depth_test_distance.is_none() {
            self.disable_depth_test_distance = source.disable_depth_test_distance.clone();
        }
    }
}
