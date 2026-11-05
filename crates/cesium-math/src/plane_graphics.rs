//! PlaneGraphics matching CesiumJS `DataSources/PlaneGraphics.js` (Apache-2.0).

use std::rc::Rc;

use crate::{
    Cartesian2, Color, ColorMaterialProperty, ConstantProperty, DistanceDisplayCondition,
    DynProperty, Event, MaterialProperty, Plane, ShadowMode,
};

/// Initialization options for PlaneGraphics using dynamic or constant properties.
#[derive(Default, Clone)]
pub struct PlaneGraphicsOptions {
    pub plane: Option<Rc<dyn DynProperty<Plane>>>,
    pub dimensions: Option<Rc<dyn DynProperty<Cartesian2>>>,
    pub show: Option<Rc<dyn DynProperty<bool>>>,
    pub fill: Option<Rc<dyn DynProperty<bool>>>,
    pub material: Option<Rc<dyn MaterialProperty>>,
    pub outline: Option<Rc<dyn DynProperty<bool>>>,
    pub outline_color: Option<Rc<dyn DynProperty<Color>>>,
    pub outline_width: Option<Rc<dyn DynProperty<f64>>>,
    pub shadows: Option<Rc<dyn DynProperty<ShadowMode>>>,
    pub distance_display_condition: Option<Rc<dyn DynProperty<DistanceDisplayCondition>>>,
}

/// Raw literal initialization values for PlaneGraphics (convenience constructor).
#[derive(Default, Clone, Debug)]
pub struct PlaneGraphicsValues {
    pub plane: Option<Plane>,
    pub dimensions: Option<Cartesian2>,
    pub show: Option<bool>,
    pub fill: Option<bool>,
    pub material: Option<Color>,
    pub outline: Option<bool>,
    pub outline_color: Option<Color>,
    pub outline_width: Option<f64>,
    pub shadows: Option<ShadowMode>,
    pub distance_display_condition: Option<DistanceDisplayCondition>,
}

/// Describes a plane defined by normal, distance, and 2D dimensions.
#[derive(Clone, Default)]
pub struct PlaneGraphics {
    pub plane: Option<Rc<dyn DynProperty<Plane>>>,
    pub dimensions: Option<Rc<dyn DynProperty<Cartesian2>>>,
    pub show: Option<Rc<dyn DynProperty<bool>>>,
    pub fill: Option<Rc<dyn DynProperty<bool>>>,
    pub material: Option<Rc<dyn MaterialProperty>>,
    pub outline: Option<Rc<dyn DynProperty<bool>>>,
    pub outline_color: Option<Rc<dyn DynProperty<Color>>>,
    pub outline_width: Option<Rc<dyn DynProperty<f64>>>,
    pub shadows: Option<Rc<dyn DynProperty<ShadowMode>>>,
    pub distance_display_condition: Option<Rc<dyn DynProperty<DistanceDisplayCondition>>>,
    definition_changed: Event<()>,
}

impl PlaneGraphics {
    /// Creates a new PlaneGraphics instance with undefined properties.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a PlaneGraphics instance from property options.
    pub fn from_options(options: PlaneGraphicsOptions) -> Self {
        Self {
            plane: options.plane,
            dimensions: options.dimensions,
            show: options.show,
            fill: options.fill,
            material: options.material,
            outline: options.outline,
            outline_color: options.outline_color,
            outline_width: options.outline_width,
            shadows: options.shadows,
            distance_display_condition: options.distance_display_condition,
            definition_changed: Event::new(),
        }
    }

    /// Creates a PlaneGraphics instance from raw constant values.
    pub fn from_values(values: PlaneGraphicsValues) -> Self {
        Self::from_options(PlaneGraphicsOptions {
            plane: values
                .plane
                .map(|v| Rc::new(ConstantProperty::from_value(v)) as Rc<dyn DynProperty<Plane>>),
            dimensions: values
                .dimensions
                .map(|v| Rc::new(ConstantProperty::from_value(v)) as Rc<dyn DynProperty<Cartesian2>>),
            show: values
                .show
                .map(|v| Rc::new(ConstantProperty::from_value(v)) as Rc<dyn DynProperty<bool>>),
            fill: values
                .fill
                .map(|v| Rc::new(ConstantProperty::from_value(v)) as Rc<dyn DynProperty<bool>>),
            material: values.material.map(|c| {
                Rc::new(ColorMaterialProperty::from_color(c)) as Rc<dyn MaterialProperty>
            }),
            outline: values
                .outline
                .map(|v| Rc::new(ConstantProperty::from_value(v)) as Rc<dyn DynProperty<bool>>),
            outline_color: values
                .outline_color
                .map(|v| Rc::new(ConstantProperty::from_value(v)) as Rc<dyn DynProperty<Color>>),
            outline_width: values
                .outline_width
                .map(|v| Rc::new(ConstantProperty::from_value(v)) as Rc<dyn DynProperty<f64>>),
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
    pub fn merge(&mut self, source: &PlaneGraphics) {
        if self.plane.is_none() {
            self.plane = source.plane.clone();
        }
        if self.dimensions.is_none() {
            self.dimensions = source.dimensions.clone();
        }
        if self.show.is_none() {
            self.show = source.show.clone();
        }
        if self.fill.is_none() {
            self.fill = source.fill.clone();
        }
        if self.material.is_none() {
            self.material = source.material.clone();
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
        if self.shadows.is_none() {
            self.shadows = source.shadows.clone();
        }
        if self.distance_display_condition.is_none() {
            self.distance_display_condition = source.distance_display_condition.clone();
        }
    }
}
