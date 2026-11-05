//! PolylineGraphics matching CesiumJS `DataSources/PolylineGraphics.js` (Apache-2.0).

use std::rc::Rc;

use crate::{
    ArcType, Cartesian3, ClassificationType, Color, ColorMaterialProperty, ConstantProperty,
    DistanceDisplayCondition, DynProperty, Event, MaterialProperty, ShadowMode,
};

/// Initialization options for PolylineGraphics using dynamic or constant properties.
#[derive(Default, Clone)]
pub struct PolylineGraphicsOptions {
    pub show: Option<Rc<dyn DynProperty<bool>>>,
    pub positions: Option<Rc<dyn DynProperty<Vec<Cartesian3>>>>,
    pub width: Option<Rc<dyn DynProperty<f64>>>,
    pub granularity: Option<Rc<dyn DynProperty<f64>>>,
    pub material: Option<Rc<dyn MaterialProperty>>,
    pub depth_fail_material: Option<Rc<dyn MaterialProperty>>,
    pub arc_type: Option<Rc<dyn DynProperty<ArcType>>>,
    pub clamp_to_ground: Option<Rc<dyn DynProperty<bool>>>,
    pub shadows: Option<Rc<dyn DynProperty<ShadowMode>>>,
    pub distance_display_condition: Option<Rc<dyn DynProperty<DistanceDisplayCondition>>>,
    pub classification_type: Option<Rc<dyn DynProperty<ClassificationType>>>,
    pub z_index: Option<Rc<dyn DynProperty<i32>>>,
}

/// Raw literal initialization values for PolylineGraphics (convenience constructor).
#[derive(Default, Clone, Debug)]
pub struct PolylineGraphicsValues {
    pub show: Option<bool>,
    pub positions: Option<Vec<Cartesian3>>,
    pub width: Option<f64>,
    pub granularity: Option<f64>,
    pub material: Option<Color>,
    pub depth_fail_material: Option<Color>,
    pub arc_type: Option<ArcType>,
    pub clamp_to_ground: Option<bool>,
    pub shadows: Option<ShadowMode>,
    pub distance_display_condition: Option<DistanceDisplayCondition>,
    pub classification_type: Option<ClassificationType>,
    pub z_index: Option<i32>,
}

/// Describes a polyline. The first two positions define a line segment,
/// and each additional position defines a line segment from the previous position.
#[derive(Clone, Default)]
pub struct PolylineGraphics {
    pub show: Option<Rc<dyn DynProperty<bool>>>,
    pub positions: Option<Rc<dyn DynProperty<Vec<Cartesian3>>>>,
    pub width: Option<Rc<dyn DynProperty<f64>>>,
    pub granularity: Option<Rc<dyn DynProperty<f64>>>,
    pub material: Option<Rc<dyn MaterialProperty>>,
    pub depth_fail_material: Option<Rc<dyn MaterialProperty>>,
    pub arc_type: Option<Rc<dyn DynProperty<ArcType>>>,
    pub clamp_to_ground: Option<Rc<dyn DynProperty<bool>>>,
    pub shadows: Option<Rc<dyn DynProperty<ShadowMode>>>,
    pub distance_display_condition: Option<Rc<dyn DynProperty<DistanceDisplayCondition>>>,
    pub classification_type: Option<Rc<dyn DynProperty<ClassificationType>>>,
    pub z_index: Option<Rc<dyn DynProperty<i32>>>,
    definition_changed: Event<()>,
}

impl PolylineGraphics {
    /// Creates a new PolylineGraphics instance with undefined properties.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a PolylineGraphics instance from property options.
    pub fn from_options(options: PolylineGraphicsOptions) -> Self {
        Self {
            show: options.show,
            positions: options.positions,
            width: options.width,
            granularity: options.granularity,
            material: options.material,
            depth_fail_material: options.depth_fail_material,
            arc_type: options.arc_type,
            clamp_to_ground: options.clamp_to_ground,
            shadows: options.shadows,
            distance_display_condition: options.distance_display_condition,
            classification_type: options.classification_type,
            z_index: options.z_index,
            definition_changed: Event::new(),
        }
    }

    /// Creates a PolylineGraphics instance from raw constant values.
    pub fn from_values(values: PolylineGraphicsValues) -> Self {
        Self::from_options(PolylineGraphicsOptions {
            show: values
                .show
                .map(|v| Rc::new(ConstantProperty::from_value(v)) as Rc<dyn DynProperty<bool>>),
            positions: values.positions.map(|v| {
                Rc::new(ConstantProperty::from_value(v)) as Rc<dyn DynProperty<Vec<Cartesian3>>>
            }),
            width: values
                .width
                .map(|v| Rc::new(ConstantProperty::from_value(v)) as Rc<dyn DynProperty<f64>>),
            granularity: values
                .granularity
                .map(|v| Rc::new(ConstantProperty::from_value(v)) as Rc<dyn DynProperty<f64>>),
            material: values.material.map(|c| {
                Rc::new(ColorMaterialProperty::from_color(c)) as Rc<dyn MaterialProperty>
            }),
            depth_fail_material: values.depth_fail_material.map(|c| {
                Rc::new(ColorMaterialProperty::from_color(c)) as Rc<dyn MaterialProperty>
            }),
            arc_type: values
                .arc_type
                .map(|v| Rc::new(ConstantProperty::from_value(v)) as Rc<dyn DynProperty<ArcType>>),
            clamp_to_ground: values
                .clamp_to_ground
                .map(|v| Rc::new(ConstantProperty::from_value(v)) as Rc<dyn DynProperty<bool>>),
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
    pub fn merge(&mut self, source: &PolylineGraphics) {
        if self.show.is_none() {
            self.show = source.show.clone();
        }
        if self.positions.is_none() {
            self.positions = source.positions.clone();
        }
        if self.width.is_none() {
            self.width = source.width.clone();
        }
        if self.granularity.is_none() {
            self.granularity = source.granularity.clone();
        }
        if self.material.is_none() {
            self.material = source.material.clone();
        }
        if self.depth_fail_material.is_none() {
            self.depth_fail_material = source.depth_fail_material.clone();
        }
        if self.arc_type.is_none() {
            self.arc_type = source.arc_type.clone();
        }
        if self.clamp_to_ground.is_none() {
            self.clamp_to_ground = source.clamp_to_ground.clone();
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
