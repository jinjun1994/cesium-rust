//! PolygonGraphics matching CesiumJS `DataSources/PolygonGraphics.js` (Apache-2.0).

use std::rc::Rc;

use crate::{
    ArcType, ClassificationType, Color, ColorMaterialProperty, ConstantProperty,
    DistanceDisplayCondition, DynProperty, Event, HeightReference, MaterialProperty,
    PolygonHierarchy, ShadowMode,
};

/// Initialization options for PolygonGraphics using dynamic or constant properties.
#[derive(Default, Clone)]
pub struct PolygonGraphicsOptions {
    pub show: Option<Rc<dyn DynProperty<bool>>>,
    pub hierarchy: Option<Rc<dyn DynProperty<PolygonHierarchy>>>,
    pub height: Option<Rc<dyn DynProperty<f64>>>,
    pub height_reference: Option<Rc<dyn DynProperty<HeightReference>>>,
    pub extruded_height: Option<Rc<dyn DynProperty<f64>>>,
    pub extruded_height_reference: Option<Rc<dyn DynProperty<HeightReference>>>,
    pub st_rotation: Option<Rc<dyn DynProperty<f64>>>,
    pub granularity: Option<Rc<dyn DynProperty<f64>>>,
    pub fill: Option<Rc<dyn DynProperty<bool>>>,
    pub material: Option<Rc<dyn MaterialProperty>>,
    pub outline: Option<Rc<dyn DynProperty<bool>>>,
    pub outline_color: Option<Rc<dyn DynProperty<Color>>>,
    pub outline_width: Option<Rc<dyn DynProperty<f64>>>,
    pub per_position_height: Option<Rc<dyn DynProperty<bool>>>,
    pub close_top: Option<Rc<dyn DynProperty<bool>>>,
    pub close_bottom: Option<Rc<dyn DynProperty<bool>>>,
    pub arc_type: Option<Rc<dyn DynProperty<ArcType>>>,
    pub shadows: Option<Rc<dyn DynProperty<ShadowMode>>>,
    pub distance_display_condition: Option<Rc<dyn DynProperty<DistanceDisplayCondition>>>,
    pub classification_type: Option<Rc<dyn DynProperty<ClassificationType>>>,
    pub z_index: Option<Rc<dyn DynProperty<i32>>>,
    pub texture_coordinates: Option<Rc<dyn DynProperty<PolygonHierarchy>>>,
}

/// Raw literal initialization values for PolygonGraphics (convenience constructor).
#[derive(Default, Clone, Debug)]
pub struct PolygonGraphicsValues {
    pub show: Option<bool>,
    pub hierarchy: Option<PolygonHierarchy>,
    pub height: Option<f64>,
    pub height_reference: Option<HeightReference>,
    pub extruded_height: Option<f64>,
    pub extruded_height_reference: Option<HeightReference>,
    pub st_rotation: Option<f64>,
    pub granularity: Option<f64>,
    pub fill: Option<bool>,
    pub material: Option<Color>,
    pub outline: Option<bool>,
    pub outline_color: Option<Color>,
    pub outline_width: Option<f64>,
    pub per_position_height: Option<bool>,
    pub close_top: Option<bool>,
    pub close_bottom: Option<bool>,
    pub arc_type: Option<ArcType>,
    pub shadows: Option<ShadowMode>,
    pub distance_display_condition: Option<DistanceDisplayCondition>,
    pub classification_type: Option<ClassificationType>,
    pub z_index: Option<i32>,
    pub texture_coordinates: Option<PolygonHierarchy>,
}

/// Describes a polygon defined by an hierarchy of linear rings.
#[derive(Clone, Default)]
pub struct PolygonGraphics {
    pub show: Option<Rc<dyn DynProperty<bool>>>,
    pub hierarchy: Option<Rc<dyn DynProperty<PolygonHierarchy>>>,
    pub height: Option<Rc<dyn DynProperty<f64>>>,
    pub height_reference: Option<Rc<dyn DynProperty<HeightReference>>>,
    pub extruded_height: Option<Rc<dyn DynProperty<f64>>>,
    pub extruded_height_reference: Option<Rc<dyn DynProperty<HeightReference>>>,
    pub st_rotation: Option<Rc<dyn DynProperty<f64>>>,
    pub granularity: Option<Rc<dyn DynProperty<f64>>>,
    pub fill: Option<Rc<dyn DynProperty<bool>>>,
    pub material: Option<Rc<dyn MaterialProperty>>,
    pub outline: Option<Rc<dyn DynProperty<bool>>>,
    pub outline_color: Option<Rc<dyn DynProperty<Color>>>,
    pub outline_width: Option<Rc<dyn DynProperty<f64>>>,
    pub per_position_height: Option<Rc<dyn DynProperty<bool>>>,
    pub close_top: Option<Rc<dyn DynProperty<bool>>>,
    pub close_bottom: Option<Rc<dyn DynProperty<bool>>>,
    pub arc_type: Option<Rc<dyn DynProperty<ArcType>>>,
    pub shadows: Option<Rc<dyn DynProperty<ShadowMode>>>,
    pub distance_display_condition: Option<Rc<dyn DynProperty<DistanceDisplayCondition>>>,
    pub classification_type: Option<Rc<dyn DynProperty<ClassificationType>>>,
    pub z_index: Option<Rc<dyn DynProperty<i32>>>,
    pub texture_coordinates: Option<Rc<dyn DynProperty<PolygonHierarchy>>>,
    definition_changed: Event<()>,
}

impl PolygonGraphics {
    /// Creates a new PolygonGraphics instance with undefined properties.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a PolygonGraphics instance from property options.
    pub fn from_options(options: PolygonGraphicsOptions) -> Self {
        Self {
            show: options.show,
            hierarchy: options.hierarchy,
            height: options.height,
            height_reference: options.height_reference,
            extruded_height: options.extruded_height,
            extruded_height_reference: options.extruded_height_reference,
            st_rotation: options.st_rotation,
            granularity: options.granularity,
            fill: options.fill,
            material: options.material,
            outline: options.outline,
            outline_color: options.outline_color,
            outline_width: options.outline_width,
            per_position_height: options.per_position_height,
            close_top: options.close_top,
            close_bottom: options.close_bottom,
            arc_type: options.arc_type,
            shadows: options.shadows,
            distance_display_condition: options.distance_display_condition,
            classification_type: options.classification_type,
            z_index: options.z_index,
            texture_coordinates: options.texture_coordinates,
            definition_changed: Event::new(),
        }
    }

    /// Creates a PolygonGraphics instance from raw constant values.
    pub fn from_values(values: PolygonGraphicsValues) -> Self {
        Self::from_options(PolygonGraphicsOptions {
            show: values
                .show
                .map(|v| Rc::new(ConstantProperty::from_value(v)) as Rc<dyn DynProperty<bool>>),
            hierarchy: values.hierarchy.map(|v| {
                Rc::new(ConstantProperty::from_value(v)) as Rc<dyn DynProperty<PolygonHierarchy>>
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
            st_rotation: values
                .st_rotation
                .map(|v| Rc::new(ConstantProperty::from_value(v)) as Rc<dyn DynProperty<f64>>),
            granularity: values
                .granularity
                .map(|v| Rc::new(ConstantProperty::from_value(v)) as Rc<dyn DynProperty<f64>>),
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
            per_position_height: values
                .per_position_height
                .map(|v| Rc::new(ConstantProperty::from_value(v)) as Rc<dyn DynProperty<bool>>),
            close_top: values
                .close_top
                .map(|v| Rc::new(ConstantProperty::from_value(v)) as Rc<dyn DynProperty<bool>>),
            close_bottom: values
                .close_bottom
                .map(|v| Rc::new(ConstantProperty::from_value(v)) as Rc<dyn DynProperty<bool>>),
            arc_type: values
                .arc_type
                .map(|v| Rc::new(ConstantProperty::from_value(v)) as Rc<dyn DynProperty<ArcType>>),
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
            texture_coordinates: values.texture_coordinates.map(|v| {
                Rc::new(ConstantProperty::from_value(v)) as Rc<dyn DynProperty<PolygonHierarchy>>
            }),
        })
    }

    /// Gets the event that is raised whenever a property is changed or modified.
    pub fn definition_changed(&self) -> &Event<()> {
        &self.definition_changed
    }

    /// Assigns each unassigned property on this object to the value of the same property on the source.
    pub fn merge(&mut self, source: &PolygonGraphics) {
        if self.show.is_none() {
            self.show = source.show.clone();
        }
        if self.hierarchy.is_none() {
            self.hierarchy = source.hierarchy.clone();
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
        if self.st_rotation.is_none() {
            self.st_rotation = source.st_rotation.clone();
        }
        if self.granularity.is_none() {
            self.granularity = source.granularity.clone();
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
        if self.per_position_height.is_none() {
            self.per_position_height = source.per_position_height.clone();
        }
        if self.close_top.is_none() {
            self.close_top = source.close_top.clone();
        }
        if self.close_bottom.is_none() {
            self.close_bottom = source.close_bottom.clone();
        }
        if self.arc_type.is_none() {
            self.arc_type = source.arc_type.clone();
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
        if self.texture_coordinates.is_none() {
            self.texture_coordinates = source.texture_coordinates.clone();
        }
    }
}
