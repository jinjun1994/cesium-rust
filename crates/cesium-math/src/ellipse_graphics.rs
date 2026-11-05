//! EllipseGraphics matching CesiumJS `DataSources/EllipseGraphics.js` (Apache-2.0).

use std::rc::Rc;

use crate::{
    ClassificationType, Color, ColorMaterialProperty, ConstantProperty, DistanceDisplayCondition,
    DynProperty, Event, MaterialProperty, ShadowMode,
};

/// Initialization options for EllipseGraphics using dynamic or constant properties.
#[derive(Default, Clone)]
pub struct EllipseGraphicsOptions {
    pub material: Option<Rc<dyn MaterialProperty>>,
    pub show: Option<Rc<dyn DynProperty<bool>>>,
    pub semi_minor_axis: Option<Rc<dyn DynProperty<f64>>>,
    pub semi_major_axis: Option<Rc<dyn DynProperty<f64>>>,
    pub height: Option<Rc<dyn DynProperty<f64>>>,
    pub extruded_height: Option<Rc<dyn DynProperty<f64>>>,
    pub granularity: Option<Rc<dyn DynProperty<f64>>>,
    pub rotation: Option<Rc<dyn DynProperty<f64>>>,
    pub st_rotation: Option<Rc<dyn DynProperty<f64>>>,
    pub number_of_vertical_lines: Option<Rc<dyn DynProperty<i32>>>,
    pub fill: Option<Rc<dyn DynProperty<bool>>>,
    pub outline: Option<Rc<dyn DynProperty<bool>>>,
    pub outline_color: Option<Rc<dyn DynProperty<Color>>>,
    pub outline_width: Option<Rc<dyn DynProperty<f64>>>,
    pub shadows: Option<Rc<dyn DynProperty<ShadowMode>>>,
    pub distance_display_condition: Option<Rc<dyn DynProperty<DistanceDisplayCondition>>>,
    pub classification_type: Option<Rc<dyn DynProperty<ClassificationType>>>,
    pub z_index: Option<Rc<dyn DynProperty<i32>>>,
}

/// Raw literal initialization values for EllipseGraphics (convenience constructor).
#[derive(Default, Clone, Debug)]
pub struct EllipseGraphicsValues {
    pub material: Option<Color>,
    pub show: Option<bool>,
    pub semi_minor_axis: Option<f64>,
    pub semi_major_axis: Option<f64>,
    pub height: Option<f64>,
    pub extruded_height: Option<f64>,
    pub granularity: Option<f64>,
    pub rotation: Option<f64>,
    pub st_rotation: Option<f64>,
    pub number_of_vertical_lines: Option<i32>,
    pub fill: Option<bool>,
    pub outline: Option<bool>,
    pub outline_color: Option<Color>,
    pub outline_width: Option<f64>,
    pub shadows: Option<ShadowMode>,
    pub distance_display_condition: Option<DistanceDisplayCondition>,
    pub classification_type: Option<ClassificationType>,
    pub z_index: Option<i32>,
}

/// Describes an ellipse defined by a center point and semi-major and semi-minor axes.
#[derive(Clone, Default)]
pub struct EllipseGraphics {
    pub material: Option<Rc<dyn MaterialProperty>>,
    pub show: Option<Rc<dyn DynProperty<bool>>>,
    pub semi_minor_axis: Option<Rc<dyn DynProperty<f64>>>,
    pub semi_major_axis: Option<Rc<dyn DynProperty<f64>>>,
    pub height: Option<Rc<dyn DynProperty<f64>>>,
    pub extruded_height: Option<Rc<dyn DynProperty<f64>>>,
    pub granularity: Option<Rc<dyn DynProperty<f64>>>,
    pub rotation: Option<Rc<dyn DynProperty<f64>>>,
    pub st_rotation: Option<Rc<dyn DynProperty<f64>>>,
    pub number_of_vertical_lines: Option<Rc<dyn DynProperty<i32>>>,
    pub fill: Option<Rc<dyn DynProperty<bool>>>,
    pub outline: Option<Rc<dyn DynProperty<bool>>>,
    pub outline_color: Option<Rc<dyn DynProperty<Color>>>,
    pub outline_width: Option<Rc<dyn DynProperty<f64>>>,
    pub shadows: Option<Rc<dyn DynProperty<ShadowMode>>>,
    pub distance_display_condition: Option<Rc<dyn DynProperty<DistanceDisplayCondition>>>,
    pub classification_type: Option<Rc<dyn DynProperty<ClassificationType>>>,
    pub z_index: Option<Rc<dyn DynProperty<i32>>>,
    definition_changed: Event<()>,
}

impl EllipseGraphics {
    /// Creates a new EllipseGraphics instance with undefined properties.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates an EllipseGraphics instance from property options.
    pub fn from_options(options: EllipseGraphicsOptions) -> Self {
        Self {
            material: options.material,
            show: options.show,
            semi_minor_axis: options.semi_minor_axis,
            semi_major_axis: options.semi_major_axis,
            height: options.height,
            extruded_height: options.extruded_height,
            granularity: options.granularity,
            rotation: options.rotation,
            st_rotation: options.st_rotation,
            number_of_vertical_lines: options.number_of_vertical_lines,
            fill: options.fill,
            outline: options.outline,
            outline_color: options.outline_color,
            outline_width: options.outline_width,
            shadows: options.shadows,
            distance_display_condition: options.distance_display_condition,
            classification_type: options.classification_type,
            z_index: options.z_index,
            definition_changed: Event::new(),
        }
    }

    /// Creates an EllipseGraphics instance from raw constant values.
    pub fn from_values(values: EllipseGraphicsValues) -> Self {
        Self::from_options(EllipseGraphicsOptions {
            material: values.material.map(|c| {
                Rc::new(ColorMaterialProperty::from_color(c)) as Rc<dyn MaterialProperty>
            }),
            show: values
                .show
                .map(|v| Rc::new(ConstantProperty::from_value(v)) as Rc<dyn DynProperty<bool>>),
            semi_minor_axis: values
                .semi_minor_axis
                .map(|v| Rc::new(ConstantProperty::from_value(v)) as Rc<dyn DynProperty<f64>>),
            semi_major_axis: values
                .semi_major_axis
                .map(|v| Rc::new(ConstantProperty::from_value(v)) as Rc<dyn DynProperty<f64>>),
            height: values
                .height
                .map(|v| Rc::new(ConstantProperty::from_value(v)) as Rc<dyn DynProperty<f64>>),
            extruded_height: values
                .extruded_height
                .map(|v| Rc::new(ConstantProperty::from_value(v)) as Rc<dyn DynProperty<f64>>),
            granularity: values
                .granularity
                .map(|v| Rc::new(ConstantProperty::from_value(v)) as Rc<dyn DynProperty<f64>>),
            rotation: values
                .rotation
                .map(|v| Rc::new(ConstantProperty::from_value(v)) as Rc<dyn DynProperty<f64>>),
            st_rotation: values
                .st_rotation
                .map(|v| Rc::new(ConstantProperty::from_value(v)) as Rc<dyn DynProperty<f64>>),
            number_of_vertical_lines: values.number_of_vertical_lines.map(|v| {
                Rc::new(ConstantProperty::from_value(v)) as Rc<dyn DynProperty<i32>>
            }),
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
            shadows: values.shadows.map(|v| {
                Rc::new(ConstantProperty::from_value(v)) as Rc<dyn DynProperty<ShadowMode>>
            }),
            distance_display_condition: values.distance_display_condition.map(|v| {
                Rc::new(ConstantProperty::from_value(v))
                    as Rc<dyn DynProperty<DistanceDisplayCondition>>
            }),
            classification_type: values.classification_type.map(|v| {
                Rc::new(ConstantProperty::from_value(v)) as Rc<dyn DynProperty<ClassificationType>>
            }),
            z_index: values
                .z_index
                .map(|v| Rc::new(ConstantProperty::from_value(v)) as Rc<dyn DynProperty<i32>>),
        })
    }

    /// Gets the event that is raised whenever a property is changed or modified.
    pub fn definition_changed(&self) -> &Event<()> {
        &self.definition_changed
    }

    /// Assigns each unassigned property on this object to the value of the same property on the source.
    pub fn merge(&mut self, source: &EllipseGraphics) {
        if self.material.is_none() {
            self.material = source.material.clone();
        }
        if self.show.is_none() {
            self.show = source.show.clone();
        }
        if self.semi_minor_axis.is_none() {
            self.semi_minor_axis = source.semi_minor_axis.clone();
        }
        if self.semi_major_axis.is_none() {
            self.semi_major_axis = source.semi_major_axis.clone();
        }
        if self.height.is_none() {
            self.height = source.height.clone();
        }
        if self.extruded_height.is_none() {
            self.extruded_height = source.extruded_height.clone();
        }
        if self.granularity.is_none() {
            self.granularity = source.granularity.clone();
        }
        if self.rotation.is_none() {
            self.rotation = source.rotation.clone();
        }
        if self.st_rotation.is_none() {
            self.st_rotation = source.st_rotation.clone();
        }
        if self.number_of_vertical_lines.is_none() {
            self.number_of_vertical_lines = source.number_of_vertical_lines.clone();
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
        if self.shadows.is_none() {
            self.shadows = source.shadows.clone();
        }
        if self.distance_display_condition.is_none() {
            self.distance_display_condition = source.distance_display_condition.clone();
        }
        if self.classification_type.is_none() {
            self.classification_type = source.classification_type.clone();
        }
        if self.z_index.is_none() {
            self.z_index = source.z_index.clone();
        }
    }
}
