//! RectangleGraphics matching CesiumJS `DataSources/RectangleGraphics.js` (Apache-2.0).

use std::rc::Rc;

use crate::{
    ClassificationType, Color, ColorMaterialProperty, ConstantProperty, DistanceDisplayCondition,
    DynProperty, Event, GeoRectangle, HeightReference, MaterialProperty, ShadowMode,
};

/// Initialization options for RectangleGraphics using dynamic or constant properties.
#[derive(Default, Clone)]
pub struct RectangleGraphicsOptions {
    pub material: Option<Rc<dyn MaterialProperty>>,
    pub show: Option<Rc<dyn DynProperty<bool>>>,
    pub coordinates: Option<Rc<dyn DynProperty<GeoRectangle>>>,
    pub height: Option<Rc<dyn DynProperty<f64>>>,
    pub height_reference: Option<Rc<dyn DynProperty<HeightReference>>>,
    pub extruded_height: Option<Rc<dyn DynProperty<f64>>>,
    pub extruded_height_reference: Option<Rc<dyn DynProperty<HeightReference>>>,
    pub rotation: Option<Rc<dyn DynProperty<f64>>>,
    pub st_rotation: Option<Rc<dyn DynProperty<f64>>>,
    pub granularity: Option<Rc<dyn DynProperty<f64>>>,
    pub fill: Option<Rc<dyn DynProperty<bool>>>,
    pub outline: Option<Rc<dyn DynProperty<bool>>>,
    pub outline_color: Option<Rc<dyn DynProperty<Color>>>,
    pub outline_width: Option<Rc<dyn DynProperty<f64>>>,
    pub shadows: Option<Rc<dyn DynProperty<ShadowMode>>>,
    pub distance_display_condition: Option<Rc<dyn DynProperty<DistanceDisplayCondition>>>,
    pub classification_type: Option<Rc<dyn DynProperty<ClassificationType>>>,
    pub z_index: Option<Rc<dyn DynProperty<i32>>>,
}

/// Raw literal initialization values for RectangleGraphics (convenience constructor).
#[derive(Default, Clone, Debug)]
pub struct RectangleGraphicsValues {
    pub material: Option<Color>,
    pub show: Option<bool>,
    pub coordinates: Option<GeoRectangle>,
    pub height: Option<f64>,
    pub height_reference: Option<HeightReference>,
    pub extruded_height: Option<f64>,
    pub extruded_height_reference: Option<HeightReference>,
    pub rotation: Option<f64>,
    pub st_rotation: Option<f64>,
    pub granularity: Option<f64>,
    pub fill: Option<bool>,
    pub outline: Option<bool>,
    pub outline_color: Option<Color>,
    pub outline_width: Option<f64>,
    pub shadows: Option<ShadowMode>,
    pub distance_display_condition: Option<DistanceDisplayCondition>,
    pub classification_type: Option<ClassificationType>,
    pub z_index: Option<i32>,
}

/// Describes graphics for a Rectangle conforming to the curvature of the globe.
#[derive(Clone, Default)]
pub struct RectangleGraphics {
    pub material: Option<Rc<dyn MaterialProperty>>,
    pub show: Option<Rc<dyn DynProperty<bool>>>,
    pub coordinates: Option<Rc<dyn DynProperty<GeoRectangle>>>,
    pub height: Option<Rc<dyn DynProperty<f64>>>,
    pub height_reference: Option<Rc<dyn DynProperty<HeightReference>>>,
    pub extruded_height: Option<Rc<dyn DynProperty<f64>>>,
    pub extruded_height_reference: Option<Rc<dyn DynProperty<HeightReference>>>,
    pub rotation: Option<Rc<dyn DynProperty<f64>>>,
    pub st_rotation: Option<Rc<dyn DynProperty<f64>>>,
    pub granularity: Option<Rc<dyn DynProperty<f64>>>,
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

impl RectangleGraphics {
    /// Creates a new RectangleGraphics instance with undefined properties.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a RectangleGraphics instance from property options.
    pub fn from_options(options: RectangleGraphicsOptions) -> Self {
        Self {
            material: options.material,
            show: options.show,
            coordinates: options.coordinates,
            height: options.height,
            height_reference: options.height_reference,
            extruded_height: options.extruded_height,
            extruded_height_reference: options.extruded_height_reference,
            rotation: options.rotation,
            st_rotation: options.st_rotation,
            granularity: options.granularity,
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

    /// Creates a RectangleGraphics instance from raw constant values.
    pub fn from_values(values: RectangleGraphicsValues) -> Self {
        Self::from_options(RectangleGraphicsOptions {
            material: values.material.map(|c| {
                Rc::new(ColorMaterialProperty::from_color(c)) as Rc<dyn MaterialProperty>
            }),
            show: values
                .show
                .map(|v| Rc::new(ConstantProperty::from_value(v)) as Rc<dyn DynProperty<bool>>),
            coordinates: values.coordinates.map(|v| {
                Rc::new(ConstantProperty::from_value(v)) as Rc<dyn DynProperty<GeoRectangle>>
            }),
            height: values
                .height
                .map(|v| Rc::new(ConstantProperty::from_value(v)) as Rc<dyn DynProperty<f64>>),
            height_reference: values.height_reference.map(|v| {
                Rc::new(ConstantProperty::from_value(v)) as Rc<dyn DynProperty<HeightReference>>
            }),
            extruded_height: values
                .extruded_height
                .map(|v| Rc::new(ConstantProperty::from_value(v)) as Rc<dyn DynProperty<f64>>),
            extruded_height_reference: values.extruded_height_reference.map(|v| {
                Rc::new(ConstantProperty::from_value(v)) as Rc<dyn DynProperty<HeightReference>>
            }),
            rotation: values
                .rotation
                .map(|v| Rc::new(ConstantProperty::from_value(v)) as Rc<dyn DynProperty<f64>>),
            st_rotation: values
                .st_rotation
                .map(|v| Rc::new(ConstantProperty::from_value(v)) as Rc<dyn DynProperty<f64>>),
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
    pub fn merge(&mut self, source: &RectangleGraphics) {
        if self.material.is_none() {
            self.material = source.material.clone();
        }
        if self.show.is_none() {
            self.show = source.show.clone();
        }
        if self.coordinates.is_none() {
            self.coordinates = source.coordinates.clone();
        }
        if self.height.is_none() {
            self.height = source.height.clone();
        }
        if self.height_reference.is_none() {
            self.height_reference = source.height_reference.clone();
        }
        if self.extruded_height.is_none() {
            self.extruded_height = source.extruded_height.clone();
        }
        if self.extruded_height_reference.is_none() {
            self.extruded_height_reference = source.extruded_height_reference.clone();
        }
        if self.rotation.is_none() {
            self.rotation = source.rotation.clone();
        }
        if self.st_rotation.is_none() {
            self.st_rotation = source.st_rotation.clone();
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
