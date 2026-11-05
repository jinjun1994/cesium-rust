//! PolylineVolumeGraphics matching CesiumJS `DataSources/PolylineVolumeGraphics.js` (Apache-2.0).

use std::rc::Rc;

use crate::{
    Cartesian2, Cartesian3, Color, ColorMaterialProperty, ConstantProperty, CornerType,
    DistanceDisplayCondition, DynProperty, Event, MaterialProperty, ShadowMode,
};

/// Initialization options for PolylineVolumeGraphics using dynamic or constant properties.
#[derive(Default, Clone)]
pub struct PolylineVolumeGraphicsOptions {
    pub material: Option<Rc<dyn MaterialProperty>>,
    pub positions: Option<Rc<dyn DynProperty<Vec<Cartesian3>>>>,
    pub shape: Option<Rc<dyn DynProperty<Vec<Cartesian2>>>>,
    pub show: Option<Rc<dyn DynProperty<bool>>>,
    pub granularity: Option<Rc<dyn DynProperty<f64>>>,
    pub fill: Option<Rc<dyn DynProperty<bool>>>,
    pub outline: Option<Rc<dyn DynProperty<bool>>>,
    pub outline_color: Option<Rc<dyn DynProperty<Color>>>,
    pub outline_width: Option<Rc<dyn DynProperty<f64>>>,
    pub corner_type: Option<Rc<dyn DynProperty<CornerType>>>,
    pub shadows: Option<Rc<dyn DynProperty<ShadowMode>>>,
    pub distance_display_condition: Option<Rc<dyn DynProperty<DistanceDisplayCondition>>>,
}

/// Raw literal initialization values for PolylineVolumeGraphics (convenience constructor).
#[derive(Default, Clone, Debug)]
pub struct PolylineVolumeGraphicsValues {
    pub material: Option<Color>,
    pub positions: Option<Vec<Cartesian3>>,
    pub shape: Option<Vec<Cartesian2>>,
    pub show: Option<bool>,
    pub granularity: Option<f64>,
    pub fill: Option<bool>,
    pub outline: Option<bool>,
    pub outline_color: Option<Color>,
    pub outline_width: Option<f64>,
    pub corner_type: Option<CornerType>,
    pub shadows: Option<ShadowMode>,
    pub distance_display_condition: Option<DistanceDisplayCondition>,
}

/// Describes a polyline volume defined as a line strip and corresponding two-dimensional shape.
#[derive(Clone, Default)]
pub struct PolylineVolumeGraphics {
    pub material: Option<Rc<dyn MaterialProperty>>,
    pub positions: Option<Rc<dyn DynProperty<Vec<Cartesian3>>>>,
    pub shape: Option<Rc<dyn DynProperty<Vec<Cartesian2>>>>,
    pub show: Option<Rc<dyn DynProperty<bool>>>,
    pub granularity: Option<Rc<dyn DynProperty<f64>>>,
    pub fill: Option<Rc<dyn DynProperty<bool>>>,
    pub outline: Option<Rc<dyn DynProperty<bool>>>,
    pub outline_color: Option<Rc<dyn DynProperty<Color>>>,
    pub outline_width: Option<Rc<dyn DynProperty<f64>>>,
    pub corner_type: Option<Rc<dyn DynProperty<CornerType>>>,
    pub shadows: Option<Rc<dyn DynProperty<ShadowMode>>>,
    pub distance_display_condition: Option<Rc<dyn DynProperty<DistanceDisplayCondition>>>,
    definition_changed: Event<()>,
}

impl PolylineVolumeGraphics {
    /// Creates a new PolylineVolumeGraphics instance with undefined properties.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a PolylineVolumeGraphics instance from property options.
    pub fn from_options(options: PolylineVolumeGraphicsOptions) -> Self {
        Self {
            material: options.material,
            positions: options.positions,
            shape: options.shape,
            show: options.show,
            granularity: options.granularity,
            fill: options.fill,
            outline: options.outline,
            outline_color: options.outline_color,
            outline_width: options.outline_width,
            corner_type: options.corner_type,
            shadows: options.shadows,
            distance_display_condition: options.distance_display_condition,
            definition_changed: Event::new(),
        }
    }

    /// Creates a PolylineVolumeGraphics instance from raw constant values.
    pub fn from_values(values: PolylineVolumeGraphicsValues) -> Self {
        Self::from_options(PolylineVolumeGraphicsOptions {
            material: values.material.map(|c| {
                Rc::new(ColorMaterialProperty::from_color(c)) as Rc<dyn MaterialProperty>
            }),
            positions: values.positions.map(|v| {
                Rc::new(ConstantProperty::from_value(v)) as Rc<dyn DynProperty<Vec<Cartesian3>>>
            }),
            shape: values.shape.map(|v| {
                Rc::new(ConstantProperty::from_value(v)) as Rc<dyn DynProperty<Vec<Cartesian2>>>
            }),
            show: values
                .show
                .map(|v| Rc::new(ConstantProperty::from_value(v)) as Rc<dyn DynProperty<bool>>),
            granularity: values
                .granularity
                .map(|v| Rc::new(ConstantProperty::from_value(v)) as Rc<dyn DynProperty<f64>>),
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
            corner_type: values.corner_type.map(|v| {
                Rc::new(ConstantProperty::from_value(v)) as Rc<dyn DynProperty<CornerType>>
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
    pub fn merge(&mut self, source: &PolylineVolumeGraphics) {
        if self.material.is_none() {
            self.material = source.material.clone();
        }
        if self.positions.is_none() {
            self.positions = source.positions.clone();
        }
        if self.shape.is_none() {
            self.shape = source.shape.clone();
        }
        if self.show.is_none() {
            self.show = source.show.clone();
        }
        if self.granularity.is_none() {
            self.granularity = source.granularity.clone();
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
        if self.corner_type.is_none() {
            self.corner_type = source.corner_type.clone();
        }
        if self.shadows.is_none() {
            self.shadows = source.shadows.clone();
        }
        if self.distance_display_condition.is_none() {
            self.distance_display_condition = source.distance_display_condition.clone();
        }
    }
}
