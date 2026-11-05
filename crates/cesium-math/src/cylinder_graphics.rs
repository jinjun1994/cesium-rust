//! CylinderGraphics matching CesiumJS `DataSources/CylinderGraphics.js` (Apache-2.0).

use std::rc::Rc;

use crate::{
    Color, ColorMaterialProperty, ConstantProperty, DistanceDisplayCondition, DynProperty, Event,
    HeightReference, MaterialProperty, ShadowMode,
};

/// Initialization options for CylinderGraphics using dynamic or constant properties.
#[derive(Default, Clone)]
pub struct CylinderGraphicsOptions {
    pub length: Option<Rc<dyn DynProperty<f64>>>,
    pub top_radius: Option<Rc<dyn DynProperty<f64>>>,
    pub bottom_radius: Option<Rc<dyn DynProperty<f64>>>,
    pub height_reference: Option<Rc<dyn DynProperty<HeightReference>>>,
    pub fill: Option<Rc<dyn DynProperty<bool>>>,
    pub material: Option<Rc<dyn MaterialProperty>>,
    pub outline: Option<Rc<dyn DynProperty<bool>>>,
    pub outline_color: Option<Rc<dyn DynProperty<Color>>>,
    pub outline_width: Option<Rc<dyn DynProperty<f64>>>,
    pub number_of_vertical_lines: Option<Rc<dyn DynProperty<i32>>>,
    pub slices: Option<Rc<dyn DynProperty<i32>>>,
    pub show: Option<Rc<dyn DynProperty<bool>>>,
    pub shadows: Option<Rc<dyn DynProperty<ShadowMode>>>,
    pub distance_display_condition: Option<Rc<dyn DynProperty<DistanceDisplayCondition>>>,
}

/// Raw literal initialization values for CylinderGraphics (convenience constructor).
#[derive(Default, Clone, Debug)]
pub struct CylinderGraphicsValues {
    pub length: Option<f64>,
    pub top_radius: Option<f64>,
    pub bottom_radius: Option<f64>,
    pub height_reference: Option<HeightReference>,
    pub fill: Option<bool>,
    pub material: Option<Color>,
    pub outline: Option<bool>,
    pub outline_color: Option<Color>,
    pub outline_width: Option<f64>,
    pub number_of_vertical_lines: Option<i32>,
    pub slices: Option<i32>,
    pub show: Option<bool>,
    pub shadows: Option<ShadowMode>,
    pub distance_display_condition: Option<DistanceDisplayCondition>,
}

/// Describes a cylinder, truncated cone, or cone defined by a length, top radius, and bottom radius.
#[derive(Clone, Default)]
pub struct CylinderGraphics {
    pub length: Option<Rc<dyn DynProperty<f64>>>,
    pub top_radius: Option<Rc<dyn DynProperty<f64>>>,
    pub bottom_radius: Option<Rc<dyn DynProperty<f64>>>,
    pub height_reference: Option<Rc<dyn DynProperty<HeightReference>>>,
    pub fill: Option<Rc<dyn DynProperty<bool>>>,
    pub material: Option<Rc<dyn MaterialProperty>>,
    pub outline: Option<Rc<dyn DynProperty<bool>>>,
    pub outline_color: Option<Rc<dyn DynProperty<Color>>>,
    pub outline_width: Option<Rc<dyn DynProperty<f64>>>,
    pub number_of_vertical_lines: Option<Rc<dyn DynProperty<i32>>>,
    pub slices: Option<Rc<dyn DynProperty<i32>>>,
    pub show: Option<Rc<dyn DynProperty<bool>>>,
    pub shadows: Option<Rc<dyn DynProperty<ShadowMode>>>,
    pub distance_display_condition: Option<Rc<dyn DynProperty<DistanceDisplayCondition>>>,
    definition_changed: Event<()>,
}

impl CylinderGraphics {
    /// Creates a new CylinderGraphics instance with undefined properties.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a CylinderGraphics instance from property options.
    pub fn from_options(options: CylinderGraphicsOptions) -> Self {
        Self {
            length: options.length,
            top_radius: options.top_radius,
            bottom_radius: options.bottom_radius,
            height_reference: options.height_reference,
            fill: options.fill,
            material: options.material,
            outline: options.outline,
            outline_color: options.outline_color,
            outline_width: options.outline_width,
            number_of_vertical_lines: options.number_of_vertical_lines,
            slices: options.slices,
            show: options.show,
            shadows: options.shadows,
            distance_display_condition: options.distance_display_condition,
            definition_changed: Event::new(),
        }
    }

    /// Creates a CylinderGraphics instance from raw constant values.
    pub fn from_values(values: CylinderGraphicsValues) -> Self {
        Self::from_options(CylinderGraphicsOptions {
            length: values
                .length
                .map(|v| Rc::new(ConstantProperty::from_value(v)) as Rc<dyn DynProperty<f64>>),
            top_radius: values
                .top_radius
                .map(|v| Rc::new(ConstantProperty::from_value(v)) as Rc<dyn DynProperty<f64>>),
            bottom_radius: values
                .bottom_radius
                .map(|v| Rc::new(ConstantProperty::from_value(v)) as Rc<dyn DynProperty<f64>>),
            height_reference: values.height_reference.map(|v| {
                Rc::new(ConstantProperty::from_value(v)) as Rc<dyn DynProperty<HeightReference>>
            }),
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
            number_of_vertical_lines: values.number_of_vertical_lines.map(|v| {
                Rc::new(ConstantProperty::from_value(v)) as Rc<dyn DynProperty<i32>>
            }),
            slices: values
                .slices
                .map(|v| Rc::new(ConstantProperty::from_value(v)) as Rc<dyn DynProperty<i32>>),
            show: values
                .show
                .map(|v| Rc::new(ConstantProperty::from_value(v)) as Rc<dyn DynProperty<bool>>),
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
    pub fn merge(&mut self, source: &CylinderGraphics) {
        if self.length.is_none() {
            self.length = source.length.clone();
        }
        if self.top_radius.is_none() {
            self.top_radius = source.top_radius.clone();
        }
        if self.bottom_radius.is_none() {
            self.bottom_radius = source.bottom_radius.clone();
        }
        if self.height_reference.is_none() {
            self.height_reference = source.height_reference.clone();
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
        if self.number_of_vertical_lines.is_none() {
            self.number_of_vertical_lines = source.number_of_vertical_lines.clone();
        }
        if self.slices.is_none() {
            self.slices = source.slices.clone();
        }
        if self.show.is_none() {
            self.show = source.show.clone();
        }
        if self.shadows.is_none() {
            self.shadows = source.shadows.clone();
        }
        if self.distance_display_condition.is_none() {
            self.distance_display_condition = source.distance_display_condition.clone();
        }
    }
}
