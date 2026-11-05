//! BoxGraphics matching CesiumJS `DataSources/BoxGraphics.js` (Apache-2.0).

use std::rc::Rc;

use crate::{
    Cartesian3, Color, ColorMaterialProperty, ConstantProperty, DistanceDisplayCondition,
    DynProperty, Event, MaterialProperty, ShadowMode,
};

/// Initialization options for BoxGraphics using dynamic or constant properties.
#[derive(Default, Clone)]
pub struct BoxGraphicsOptions {
    pub material: Option<Rc<dyn MaterialProperty>>,
    pub show: Option<Rc<dyn DynProperty<bool>>>,
    pub fill: Option<Rc<dyn DynProperty<bool>>>,
    pub outline: Option<Rc<dyn DynProperty<bool>>>,
    pub outline_color: Option<Rc<dyn DynProperty<Color>>>,
    pub outline_width: Option<Rc<dyn DynProperty<f64>>>,
    pub dimensions: Option<Rc<dyn DynProperty<Cartesian3>>>,
    pub shadows: Option<Rc<dyn DynProperty<ShadowMode>>>,
    pub distance_display_condition: Option<Rc<dyn DynProperty<DistanceDisplayCondition>>>,
}

/// Raw literal initialization values for BoxGraphics (convenience constructor).
#[derive(Default, Clone, Debug)]
pub struct BoxGraphicsValues {
    pub material: Option<Color>,
    pub show: Option<bool>,
    pub fill: Option<bool>,
    pub outline: Option<bool>,
    pub outline_color: Option<Color>,
    pub outline_width: Option<f64>,
    pub dimensions: Option<Cartesian3>,
    pub shadows: Option<ShadowMode>,
    pub distance_display_condition: Option<DistanceDisplayCondition>,
}

/// Describes a box located at the position of the containing Entity.
#[derive(Clone, Default)]
pub struct BoxGraphics {
    pub material: Option<Rc<dyn MaterialProperty>>,
    pub show: Option<Rc<dyn DynProperty<bool>>>,
    pub fill: Option<Rc<dyn DynProperty<bool>>>,
    pub outline: Option<Rc<dyn DynProperty<bool>>>,
    pub outline_color: Option<Rc<dyn DynProperty<Color>>>,
    pub outline_width: Option<Rc<dyn DynProperty<f64>>>,
    pub dimensions: Option<Rc<dyn DynProperty<Cartesian3>>>,
    pub shadows: Option<Rc<dyn DynProperty<ShadowMode>>>,
    pub distance_display_condition: Option<Rc<dyn DynProperty<DistanceDisplayCondition>>>,
    definition_changed: Event<()>,
}

impl BoxGraphics {
    /// Creates a new BoxGraphics instance with undefined properties.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a BoxGraphics instance from property options.
    pub fn from_options(options: BoxGraphicsOptions) -> Self {
        Self {
            material: options.material,
            show: options.show,
            fill: options.fill,
            outline: options.outline,
            outline_color: options.outline_color,
            outline_width: options.outline_width,
            dimensions: options.dimensions,
            shadows: options.shadows,
            distance_display_condition: options.distance_display_condition,
            definition_changed: Event::new(),
        }
    }

    /// Creates a BoxGraphics instance from raw constant values.
    pub fn from_values(values: BoxGraphicsValues) -> Self {
        Self::from_options(BoxGraphicsOptions {
            material: values.material.map(|c| {
                Rc::new(ColorMaterialProperty::from_color(c)) as Rc<dyn MaterialProperty>
            }),
            show: values
                .show
                .map(|v| Rc::new(ConstantProperty::from_value(v)) as Rc<dyn DynProperty<bool>>),
            fill: values
                .fill
                .map(|v| Rc::new(ConstantProperty::from_value(v)) as Rc<dyn DynProperty<bool>>),
            outline: values
                .outline
                .map(|v| Rc::new(ConstantProperty::from_value(v)) as Rc<dyn DynProperty<bool>>),
            outline_color: values
                .outline_color
                .map(|v| Rc::new(ConstantProperty::from_value(v)) as Rc<dyn DynProperty<Color>>),
            outline_width: values
                .outline_width
                .map(|v| Rc::new(ConstantProperty::from_value(v)) as Rc<dyn DynProperty<f64>>),
            dimensions: values.dimensions.map(|v| {
                Rc::new(ConstantProperty::from_value(v)) as Rc<dyn DynProperty<Cartesian3>>
            }),
            shadows: values.shadows.map(|v| {
                Rc::new(ConstantProperty::from_value(v)) as Rc<dyn DynProperty<ShadowMode>>
            }),
            distance_display_condition: values.distance_display_condition.map(|v| {
                Rc::new(ConstantProperty::from_value(v))
                    as Rc<dyn DynProperty<DistanceDisplayCondition>>
            }),
        })
    }

    /// Gets the event that is raised whenever a property is changed or modified.
    pub fn definition_changed(&self) -> &Event<()> {
        &self.definition_changed
    }

    /// Assigns each unassigned property on this object to the value of the same property on the source.
    pub fn merge(&mut self, source: &BoxGraphics) {
        if self.material.is_none() {
            self.material = source.material.clone();
        }
        if self.show.is_none() {
            self.show = source.show.clone();
        }
        if self.fill.is_none() {
            self.fill = source.fill.clone();
        }
        if self.outline.is_none() {
            self.outline = source.outline.clone();
        }
        if self.outline_color.is_none() {
            self.outline_color = source.outline_color.clone();
        }
        if self.outline_width.is_none() {
            self.outline_width = source.outline_width.clone();
        }
        if self.dimensions.is_none() {
            self.dimensions = source.dimensions.clone();
        }
        if self.shadows.is_none() {
            self.shadows = source.shadows.clone();
        }
        if self.distance_display_condition.is_none() {
            self.distance_display_condition = source.distance_display_condition.clone();
        }
    }
}
