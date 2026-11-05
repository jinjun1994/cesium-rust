//! PathGraphics matching CesiumJS `DataSources/PathGraphics.js` (Apache-2.0).

use std::rc::Rc;

use crate::{
    Color, ColorMaterialProperty, ConstantProperty, DistanceDisplayCondition, DynProperty, Event,
    MaterialProperty, PathMode,
};

/// Initialization options for PathGraphics using dynamic or constant properties.
#[derive(Default, Clone)]
pub struct PathGraphicsOptions {
    pub material: Option<Rc<dyn MaterialProperty>>,
    pub width: Option<Rc<dyn DynProperty<f64>>>,
    pub show: Option<Rc<dyn DynProperty<bool>>>,
    pub lead_time: Option<Rc<dyn DynProperty<f64>>>,
    pub trail_time: Option<Rc<dyn DynProperty<f64>>>,
    pub resolution: Option<Rc<dyn DynProperty<f64>>>,
    pub distance_display_condition: Option<Rc<dyn DynProperty<DistanceDisplayCondition>>>,
    pub relative_to: Option<Rc<dyn DynProperty<String>>>,
    pub material_mode: Option<Rc<dyn DynProperty<PathMode>>>,
}

/// Raw literal initialization values for PathGraphics (convenience constructor).
#[derive(Default, Clone, Debug)]
pub struct PathGraphicsValues {
    pub material: Option<Color>,
    pub width: Option<f64>,
    pub show: Option<bool>,
    pub lead_time: Option<f64>,
    pub trail_time: Option<f64>,
    pub resolution: Option<f64>,
    pub distance_display_condition: Option<DistanceDisplayCondition>,
    pub relative_to: Option<String>,
    pub material_mode: Option<PathMode>,
}

/// Describes a polyline defined as the path made by an Entity as it moves over time.
#[derive(Clone, Default)]
pub struct PathGraphics {
    pub material: Option<Rc<dyn MaterialProperty>>,
    pub width: Option<Rc<dyn DynProperty<f64>>>,
    pub show: Option<Rc<dyn DynProperty<bool>>>,
    pub lead_time: Option<Rc<dyn DynProperty<f64>>>,
    pub trail_time: Option<Rc<dyn DynProperty<f64>>>,
    pub resolution: Option<Rc<dyn DynProperty<f64>>>,
    pub distance_display_condition: Option<Rc<dyn DynProperty<DistanceDisplayCondition>>>,
    pub relative_to: Option<Rc<dyn DynProperty<String>>>,
    pub material_mode: Option<Rc<dyn DynProperty<PathMode>>>,
    definition_changed: Event<()>,
}

impl PathGraphics {
    /// Creates a new PathGraphics instance with undefined properties.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a PathGraphics instance from property options.
    pub fn from_options(options: PathGraphicsOptions) -> Self {
        Self {
            material: options.material,
            width: options.width,
            show: options.show,
            lead_time: options.lead_time,
            trail_time: options.trail_time,
            resolution: options.resolution,
            distance_display_condition: options.distance_display_condition,
            relative_to: options.relative_to,
            material_mode: options.material_mode,
            definition_changed: Event::new(),
        }
    }

    /// Creates a PathGraphics instance from raw constant values.
    pub fn from_values(values: PathGraphicsValues) -> Self {
        Self::from_options(PathGraphicsOptions {
            material: values.material.map(|c| {
                Rc::new(ColorMaterialProperty::from_color(c)) as Rc<dyn MaterialProperty>
            }),
            width: values
                .width
                .map(|v| Rc::new(ConstantProperty::from_value(v)) as Rc<dyn DynProperty<f64>>),
            show: values
                .show
                .map(|v| Rc::new(ConstantProperty::from_value(v)) as Rc<dyn DynProperty<bool>>),
            lead_time: values
                .lead_time
                .map(|v| Rc::new(ConstantProperty::from_value(v)) as Rc<dyn DynProperty<f64>>),
            trail_time: values
                .trail_time
                .map(|v| Rc::new(ConstantProperty::from_value(v)) as Rc<dyn DynProperty<f64>>),
            resolution: values
                .resolution
                .map(|v| Rc::new(ConstantProperty::from_value(v)) as Rc<dyn DynProperty<f64>>),
            distance_display_condition: values.distance_display_condition.map(|v| {
                Rc::new(ConstantProperty::from_value(v))
                    as Rc<dyn DynProperty<DistanceDisplayCondition>>
            }),
            relative_to: values
                .relative_to
                .map(|v| Rc::new(ConstantProperty::from_value(v)) as Rc<dyn DynProperty<String>>),
            material_mode: values.material_mode.map(|v| {
                Rc::new(ConstantProperty::from_value(v)) as Rc<dyn DynProperty<PathMode>>
            }),
        })
    }

    /// Gets the event that is raised whenever a property is changed or modified.
    pub fn definition_changed(&self) -> &Event<()> {
        &self.definition_changed
    }

    /// Assigns each unassigned property on this object to the value of the same property on the source.
    pub fn merge(&mut self, source: &PathGraphics) {
        if self.material.is_none() {
            self.material = source.material.clone();
        }
        if self.width.is_none() {
            self.width = source.width.clone();
        }
        if self.show.is_none() {
            self.show = source.show.clone();
        }
        if self.lead_time.is_none() {
            self.lead_time = source.lead_time.clone();
        }
        if self.trail_time.is_none() {
            self.trail_time = source.trail_time.clone();
        }
        if self.resolution.is_none() {
            self.resolution = source.resolution.clone();
        }
        if self.distance_display_condition.is_none() {
            self.distance_display_condition = source.distance_display_condition.clone();
        }
        if self.relative_to.is_none() {
            self.relative_to = source.relative_to.clone();
        }
        if self.material_mode.is_none() {
            self.material_mode = source.material_mode.clone();
        }
    }
}
