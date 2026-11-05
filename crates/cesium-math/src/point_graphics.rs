//! PointGraphics matching CesiumJS `DataSources/PointGraphics.js` (Apache-2.0).

use std::rc::Rc;

use crate::{
    Color, ConstantProperty, DistanceDisplayCondition, DynProperty, Event, HeightReference,
    NearFarScalar, SplitDirection,
};

/// Initialization options for PointGraphics using dynamic or constant properties.
#[derive(Default, Clone)]
pub struct PointGraphicsOptions {
    pub show: Option<Rc<dyn DynProperty<bool>>>,
    pub pixel_size: Option<Rc<dyn DynProperty<f64>>>,
    pub height_reference: Option<Rc<dyn DynProperty<HeightReference>>>,
    pub color: Option<Rc<dyn DynProperty<Color>>>,
    pub outline_color: Option<Rc<dyn DynProperty<Color>>>,
    pub outline_width: Option<Rc<dyn DynProperty<f64>>>,
    pub scale_by_distance: Option<Rc<dyn DynProperty<NearFarScalar>>>,
    pub translucency_by_distance: Option<Rc<dyn DynProperty<NearFarScalar>>>,
    pub distance_display_condition: Option<Rc<dyn DynProperty<DistanceDisplayCondition>>>,
    pub disable_depth_test_distance: Option<Rc<dyn DynProperty<f64>>>,
    pub split_direction: Option<Rc<dyn DynProperty<SplitDirection>>>,
}

/// Raw literal initialization values for PointGraphics (convenience constructor).
#[derive(Default, Clone, Debug)]
pub struct PointGraphicsValues {
    pub show: Option<bool>,
    pub pixel_size: Option<f64>,
    pub height_reference: Option<HeightReference>,
    pub color: Option<Color>,
    pub outline_color: Option<Color>,
    pub outline_width: Option<f64>,
    pub scale_by_distance: Option<NearFarScalar>,
    pub translucency_by_distance: Option<NearFarScalar>,
    pub distance_display_condition: Option<DistanceDisplayCondition>,
    pub disable_depth_test_distance: Option<f64>,
    pub split_direction: Option<SplitDirection>,
}

/// Describes a graphical point located at the position of the containing Entity.
#[derive(Clone, Default)]
pub struct PointGraphics {
    pub show: Option<Rc<dyn DynProperty<bool>>>,
    pub pixel_size: Option<Rc<dyn DynProperty<f64>>>,
    pub height_reference: Option<Rc<dyn DynProperty<HeightReference>>>,
    pub color: Option<Rc<dyn DynProperty<Color>>>,
    pub outline_color: Option<Rc<dyn DynProperty<Color>>>,
    pub outline_width: Option<Rc<dyn DynProperty<f64>>>,
    pub scale_by_distance: Option<Rc<dyn DynProperty<NearFarScalar>>>,
    pub translucency_by_distance: Option<Rc<dyn DynProperty<NearFarScalar>>>,
    pub distance_display_condition: Option<Rc<dyn DynProperty<DistanceDisplayCondition>>>,
    pub disable_depth_test_distance: Option<Rc<dyn DynProperty<f64>>>,
    pub split_direction: Option<Rc<dyn DynProperty<SplitDirection>>>,
    definition_changed: Event<()>,
}

impl PointGraphics {
    /// Creates a new PointGraphics instance with undefined properties.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a PointGraphics instance from property options.
    pub fn from_options(options: PointGraphicsOptions) -> Self {
        Self {
            show: options.show,
            pixel_size: options.pixel_size,
            height_reference: options.height_reference,
            color: options.color,
            outline_color: options.outline_color,
            outline_width: options.outline_width,
            scale_by_distance: options.scale_by_distance,
            translucency_by_distance: options.translucency_by_distance,
            distance_display_condition: options.distance_display_condition,
            disable_depth_test_distance: options.disable_depth_test_distance,
            split_direction: options.split_direction,
            definition_changed: Event::new(),
        }
    }

    /// Creates a PointGraphics instance from raw constant values.
    pub fn from_values(values: PointGraphicsValues) -> Self {
        Self::from_options(PointGraphicsOptions {
            show: values
                .show
                .map(|v| Rc::new(ConstantProperty::from_value(v)) as Rc<dyn DynProperty<bool>>),
            pixel_size: values
                .pixel_size
                .map(|v| Rc::new(ConstantProperty::from_value(v)) as Rc<dyn DynProperty<f64>>),
            height_reference: values.height_reference.map(|v| {
                Rc::new(ConstantProperty::from_value(v)) as Rc<dyn DynProperty<HeightReference>>
            }),
            color: values
                .color
                .map(|v| Rc::new(ConstantProperty::from_value(v)) as Rc<dyn DynProperty<Color>>),
            outline_color: values
                .outline_color
                .map(|v| Rc::new(ConstantProperty::from_value(v)) as Rc<dyn DynProperty<Color>>),
            outline_width: values
                .outline_width
                .map(|v| Rc::new(ConstantProperty::from_value(v)) as Rc<dyn DynProperty<f64>>),
            scale_by_distance: values.scale_by_distance.map(|v| {
                Rc::new(ConstantProperty::from_value(v)) as Rc<dyn DynProperty<NearFarScalar>>
            }),
            translucency_by_distance: values.translucency_by_distance.map(|v| {
                Rc::new(ConstantProperty::from_value(v)) as Rc<dyn DynProperty<NearFarScalar>>
            }),
            distance_display_condition: values.distance_display_condition.map(|v| {
                Rc::new(ConstantProperty::from_value(v))
                    as Rc<dyn DynProperty<DistanceDisplayCondition>>
            }),
            disable_depth_test_distance: values
                .disable_depth_test_distance
                .map(|v| Rc::new(ConstantProperty::from_value(v)) as Rc<dyn DynProperty<f64>>),
            split_direction: values.split_direction.map(|v| {
                Rc::new(ConstantProperty::from_value(v)) as Rc<dyn DynProperty<SplitDirection>>
            }),
        })
    }

    /// Gets the event that is raised whenever a property is changed or modified.
    pub fn definition_changed(&self) -> &Event<()> {
        &self.definition_changed
    }

    /// Assigns each unassigned property on this object to the value of the same property on the source.
    pub fn merge(&mut self, source: &PointGraphics) {
        if self.show.is_none() {
            self.show = source.show.clone();
        }
        if self.pixel_size.is_none() {
            self.pixel_size = source.pixel_size.clone();
        }
        if self.height_reference.is_none() {
            self.height_reference = source.height_reference.clone();
        }
        if self.color.is_none() {
            self.color = source.color.clone();
        }
        if self.outline_color.is_none() {
            self.outline_color = source.outline_color.clone();
        }
        if self.outline_width.is_none() {
            self.outline_width = source.outline_width.clone();
        }
        if self.scale_by_distance.is_none() {
            self.scale_by_distance = source.scale_by_distance.clone();
        }
        if self.translucency_by_distance.is_none() {
            self.translucency_by_distance = source.translucency_by_distance.clone();
        }
        if self.distance_display_condition.is_none() {
            self.distance_display_condition = source.distance_display_condition.clone();
        }
        if self.disable_depth_test_distance.is_none() {
            self.disable_depth_test_distance = source.disable_depth_test_distance.clone();
        }
        if self.split_direction.is_none() {
            self.split_direction = source.split_direction.clone();
        }
    }
}
