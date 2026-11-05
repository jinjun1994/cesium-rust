//! EllipsoidGraphics matching CesiumJS `DataSources/EllipsoidGraphics.js` (Apache-2.0).

use std::rc::Rc;

use crate::{
    Cartesian3, Color, ColorMaterialProperty, ConstantProperty, DistanceDisplayCondition,
    DynProperty, Event, HeightReference, MaterialProperty, ShadowMode,
};

/// Initialization options for EllipsoidGraphics using dynamic or constant properties.
#[derive(Default, Clone)]
pub struct EllipsoidGraphicsOptions {
    pub radii: Option<Rc<dyn DynProperty<Cartesian3>>>,
    pub inner_radii: Option<Rc<dyn DynProperty<Cartesian3>>>,
    pub minimum_clock: Option<Rc<dyn DynProperty<f64>>>,
    pub maximum_clock: Option<Rc<dyn DynProperty<f64>>>,
    pub minimum_cone: Option<Rc<dyn DynProperty<f64>>>,
    pub maximum_cone: Option<Rc<dyn DynProperty<f64>>>,
    pub height_reference: Option<Rc<dyn DynProperty<HeightReference>>>,
    pub fill: Option<Rc<dyn DynProperty<bool>>>,
    pub material: Option<Rc<dyn MaterialProperty>>,
    pub outline: Option<Rc<dyn DynProperty<bool>>>,
    pub outline_color: Option<Rc<dyn DynProperty<Color>>>,
    pub outline_width: Option<Rc<dyn DynProperty<f64>>>,
    pub stack_partitions: Option<Rc<dyn DynProperty<i32>>>,
    pub slice_partitions: Option<Rc<dyn DynProperty<i32>>>,
    pub subdivisions: Option<Rc<dyn DynProperty<i32>>>,
    pub show: Option<Rc<dyn DynProperty<bool>>>,
    pub shadows: Option<Rc<dyn DynProperty<ShadowMode>>>,
    pub distance_display_condition: Option<Rc<dyn DynProperty<DistanceDisplayCondition>>>,
}

/// Raw literal initialization values for EllipsoidGraphics (convenience constructor).
#[derive(Default, Clone, Debug)]
pub struct EllipsoidGraphicsValues {
    pub radii: Option<Cartesian3>,
    pub inner_radii: Option<Cartesian3>,
    pub minimum_clock: Option<f64>,
    pub maximum_clock: Option<f64>,
    pub minimum_cone: Option<f64>,
    pub maximum_cone: Option<f64>,
    pub height_reference: Option<HeightReference>,
    pub fill: Option<bool>,
    pub material: Option<Color>,
    pub outline: Option<bool>,
    pub outline_color: Option<Color>,
    pub outline_width: Option<f64>,
    pub stack_partitions: Option<i32>,
    pub slice_partitions: Option<i32>,
    pub subdivisions: Option<i32>,
    pub show: Option<bool>,
    pub shadows: Option<ShadowMode>,
    pub distance_display_condition: Option<DistanceDisplayCondition>,
}

/// Describes an ellipsoid or sphere.
#[derive(Clone, Default)]
pub struct EllipsoidGraphics {
    pub radii: Option<Rc<dyn DynProperty<Cartesian3>>>,
    pub inner_radii: Option<Rc<dyn DynProperty<Cartesian3>>>,
    pub minimum_clock: Option<Rc<dyn DynProperty<f64>>>,
    pub maximum_clock: Option<Rc<dyn DynProperty<f64>>>,
    pub minimum_cone: Option<Rc<dyn DynProperty<f64>>>,
    pub maximum_cone: Option<Rc<dyn DynProperty<f64>>>,
    pub height_reference: Option<Rc<dyn DynProperty<HeightReference>>>,
    pub fill: Option<Rc<dyn DynProperty<bool>>>,
    pub material: Option<Rc<dyn MaterialProperty>>,
    pub outline: Option<Rc<dyn DynProperty<bool>>>,
    pub outline_color: Option<Rc<dyn DynProperty<Color>>>,
    pub outline_width: Option<Rc<dyn DynProperty<f64>>>,
    pub stack_partitions: Option<Rc<dyn DynProperty<i32>>>,
    pub slice_partitions: Option<Rc<dyn DynProperty<i32>>>,
    pub subdivisions: Option<Rc<dyn DynProperty<i32>>>,
    pub show: Option<Rc<dyn DynProperty<bool>>>,
    pub shadows: Option<Rc<dyn DynProperty<ShadowMode>>>,
    pub distance_display_condition: Option<Rc<dyn DynProperty<DistanceDisplayCondition>>>,
    definition_changed: Event<()>,
}

impl EllipsoidGraphics {
    /// Creates a new EllipsoidGraphics instance with undefined properties.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates an EllipsoidGraphics instance from property options.
    pub fn from_options(options: EllipsoidGraphicsOptions) -> Self {
        Self {
            radii: options.radii,
            inner_radii: options.inner_radii,
            minimum_clock: options.minimum_clock,
            maximum_clock: options.maximum_clock,
            minimum_cone: options.minimum_cone,
            maximum_cone: options.maximum_cone,
            height_reference: options.height_reference,
            fill: options.fill,
            material: options.material,
            outline: options.outline,
            outline_color: options.outline_color,
            outline_width: options.outline_width,
            stack_partitions: options.stack_partitions,
            slice_partitions: options.slice_partitions,
            subdivisions: options.subdivisions,
            show: options.show,
            shadows: options.shadows,
            distance_display_condition: options.distance_display_condition,
            definition_changed: Event::new(),
        }
    }

    /// Creates an EllipsoidGraphics instance from raw constant values.
    pub fn from_values(values: EllipsoidGraphicsValues) -> Self {
        Self::from_options(EllipsoidGraphicsOptions {
            radii: values
                .radii
                .map(|v| Rc::new(ConstantProperty::from_value(v)) as Rc<dyn DynProperty<Cartesian3>>),
            inner_radii: values
                .inner_radii
                .map(|v| Rc::new(ConstantProperty::from_value(v)) as Rc<dyn DynProperty<Cartesian3>>),
            minimum_clock: values
                .minimum_clock
                .map(|v| Rc::new(ConstantProperty::from_value(v)) as Rc<dyn DynProperty<f64>>),
            maximum_clock: values
                .maximum_clock
                .map(|v| Rc::new(ConstantProperty::from_value(v)) as Rc<dyn DynProperty<f64>>),
            minimum_cone: values
                .minimum_cone
                .map(|v| Rc::new(ConstantProperty::from_value(v)) as Rc<dyn DynProperty<f64>>),
            maximum_cone: values
                .maximum_cone
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
            stack_partitions: values
                .stack_partitions
                .map(|v| Rc::new(ConstantProperty::from_value(v)) as Rc<dyn DynProperty<i32>>),
            slice_partitions: values
                .slice_partitions
                .map(|v| Rc::new(ConstantProperty::from_value(v)) as Rc<dyn DynProperty<i32>>),
            subdivisions: values
                .subdivisions
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
    pub fn merge(&mut self, source: &EllipsoidGraphics) {
        if self.radii.is_none() {
            self.radii = source.radii.clone();
        }
        if self.inner_radii.is_none() {
            self.inner_radii = source.inner_radii.clone();
        }
        if self.minimum_clock.is_none() {
            self.minimum_clock = source.minimum_clock.clone();
        }
        if self.maximum_clock.is_none() {
            self.maximum_clock = source.maximum_clock.clone();
        }
        if self.minimum_cone.is_none() {
            self.minimum_cone = source.minimum_cone.clone();
        }
        if self.maximum_cone.is_none() {
            self.maximum_cone = source.maximum_cone.clone();
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
        if self.stack_partitions.is_none() {
            self.stack_partitions = source.stack_partitions.clone();
        }
        if self.slice_partitions.is_none() {
            self.slice_partitions = source.slice_partitions.clone();
        }
        if self.subdivisions.is_none() {
            self.subdivisions = source.subdivisions.clone();
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
