//! ModelGraphics matching CesiumJS `DataSources/ModelGraphics.js` (Apache-2.0).

use std::rc::Rc;

use crate::{
    Cartesian2, ClippingPlaneCollection, Color, ColorBlendMode, ConstantProperty, CustomShader,
    DistanceDisplayCondition, DynProperty, Event, HeightReference, PropertyBag, ShadowMode,
    TranslationRotationScale,
};

/// Initialization options for ModelGraphics using dynamic or constant properties.
#[derive(Default, Clone)]
pub struct ModelGraphicsOptions {
    pub uri: Option<Rc<dyn DynProperty<String>>>,
    pub scale: Option<Rc<dyn DynProperty<f64>>>,
    pub show: Option<Rc<dyn DynProperty<bool>>>,
    pub minimum_pixel_size: Option<Rc<dyn DynProperty<f64>>>,
    pub maximum_scale: Option<Rc<dyn DynProperty<f64>>>,
    pub incrementally_load_textures: Option<Rc<dyn DynProperty<bool>>>,
    pub run_animations: Option<Rc<dyn DynProperty<bool>>>,
    pub clamp_animations: Option<Rc<dyn DynProperty<bool>>>,
    pub shadows: Option<Rc<dyn DynProperty<ShadowMode>>>,
    pub height_reference: Option<Rc<dyn DynProperty<HeightReference>>>,
    pub distance_display_condition: Option<Rc<dyn DynProperty<DistanceDisplayCondition>>>,
    pub silhouette_color: Option<Rc<dyn DynProperty<Color>>>,
    pub silhouette_size: Option<Rc<dyn DynProperty<f64>>>,
    pub color: Option<Rc<dyn DynProperty<Color>>>,
    pub color_blend_mode: Option<Rc<dyn DynProperty<ColorBlendMode>>>,
    pub color_blend_amount: Option<Rc<dyn DynProperty<f64>>>,
    pub clipping_planes: Option<Rc<dyn DynProperty<ClippingPlaneCollection>>>,
    pub custom_shader: Option<Rc<dyn DynProperty<CustomShader>>>,
    pub image_based_lighting_factor: Option<Rc<dyn DynProperty<Cartesian2>>>,
    pub light_color: Option<Rc<dyn DynProperty<Color>>>,
    pub node_transformations: Option<PropertyBag<TranslationRotationScale>>,
    pub articulations: Option<PropertyBag<f64>>,
}

/// Raw literal initialization values for ModelGraphics (convenience constructor).
#[derive(Default, Clone, Debug)]
pub struct ModelGraphicsValues {
    pub uri: Option<String>,
    pub scale: Option<f64>,
    pub show: Option<bool>,
    pub minimum_pixel_size: Option<f64>,
    pub maximum_scale: Option<f64>,
    pub incrementally_load_textures: Option<bool>,
    pub run_animations: Option<bool>,
    pub clamp_animations: Option<bool>,
    pub shadows: Option<ShadowMode>,
    pub height_reference: Option<HeightReference>,
    pub distance_display_condition: Option<DistanceDisplayCondition>,
    pub silhouette_color: Option<Color>,
    pub silhouette_size: Option<f64>,
    pub color: Option<Color>,
    pub color_blend_mode: Option<ColorBlendMode>,
    pub color_blend_amount: Option<f64>,
    pub clipping_planes: Option<ClippingPlaneCollection>,
    pub custom_shader: Option<CustomShader>,
    pub image_based_lighting_factor: Option<Cartesian2>,
    pub light_color: Option<Color>,
}

/// Describes a 3D model defined as a glTF asset.
#[derive(Clone, Default)]
pub struct ModelGraphics {
    pub uri: Option<Rc<dyn DynProperty<String>>>,
    pub scale: Option<Rc<dyn DynProperty<f64>>>,
    pub show: Option<Rc<dyn DynProperty<bool>>>,
    pub minimum_pixel_size: Option<Rc<dyn DynProperty<f64>>>,
    pub maximum_scale: Option<Rc<dyn DynProperty<f64>>>,
    pub incrementally_load_textures: Option<Rc<dyn DynProperty<bool>>>,
    pub run_animations: Option<Rc<dyn DynProperty<bool>>>,
    pub clamp_animations: Option<Rc<dyn DynProperty<bool>>>,
    pub shadows: Option<Rc<dyn DynProperty<ShadowMode>>>,
    pub height_reference: Option<Rc<dyn DynProperty<HeightReference>>>,
    pub distance_display_condition: Option<Rc<dyn DynProperty<DistanceDisplayCondition>>>,
    pub silhouette_color: Option<Rc<dyn DynProperty<Color>>>,
    pub silhouette_size: Option<Rc<dyn DynProperty<f64>>>,
    pub color: Option<Rc<dyn DynProperty<Color>>>,
    pub color_blend_mode: Option<Rc<dyn DynProperty<ColorBlendMode>>>,
    pub color_blend_amount: Option<Rc<dyn DynProperty<f64>>>,
    pub clipping_planes: Option<Rc<dyn DynProperty<ClippingPlaneCollection>>>,
    pub custom_shader: Option<Rc<dyn DynProperty<CustomShader>>>,
    pub image_based_lighting_factor: Option<Rc<dyn DynProperty<Cartesian2>>>,
    pub light_color: Option<Rc<dyn DynProperty<Color>>>,
    pub node_transformations: PropertyBag<TranslationRotationScale>,
    pub articulations: PropertyBag<f64>,
    definition_changed: Event<()>,
}

impl ModelGraphics {
    /// Creates a new ModelGraphics instance with undefined properties.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a ModelGraphics instance from property options.
    pub fn from_options(options: ModelGraphicsOptions) -> Self {
        Self {
            uri: options.uri,
            scale: options.scale,
            show: options.show,
            minimum_pixel_size: options.minimum_pixel_size,
            maximum_scale: options.maximum_scale,
            incrementally_load_textures: options.incrementally_load_textures,
            run_animations: options.run_animations,
            clamp_animations: options.clamp_animations,
            shadows: options.shadows,
            height_reference: options.height_reference,
            distance_display_condition: options.distance_display_condition,
            silhouette_color: options.silhouette_color,
            silhouette_size: options.silhouette_size,
            color: options.color,
            color_blend_mode: options.color_blend_mode,
            color_blend_amount: options.color_blend_amount,
            clipping_planes: options.clipping_planes,
            custom_shader: options.custom_shader,
            image_based_lighting_factor: options.image_based_lighting_factor,
            light_color: options.light_color,
            node_transformations: options.node_transformations.unwrap_or_default(),
            articulations: options.articulations.unwrap_or_default(),
            definition_changed: Event::new(),
        }
    }

    /// Creates a ModelGraphics instance from raw constant values.
    pub fn from_values(values: ModelGraphicsValues) -> Self {
        Self::from_options(ModelGraphicsOptions {
            uri: values
                .uri
                .map(|v| Rc::new(ConstantProperty::from_value(v)) as Rc<dyn DynProperty<String>>),
            scale: values
                .scale
                .map(|v| Rc::new(ConstantProperty::from_value(v)) as Rc<dyn DynProperty<f64>>),
            show: values
                .show
                .map(|v| Rc::new(ConstantProperty::from_value(v)) as Rc<dyn DynProperty<bool>>),
            minimum_pixel_size: values
                .minimum_pixel_size
                .map(|v| Rc::new(ConstantProperty::from_value(v)) as Rc<dyn DynProperty<f64>>),
            maximum_scale: values
                .maximum_scale
                .map(|v| Rc::new(ConstantProperty::from_value(v)) as Rc<dyn DynProperty<f64>>),
            incrementally_load_textures: values
                .incrementally_load_textures
                .map(|v| Rc::new(ConstantProperty::from_value(v)) as Rc<dyn DynProperty<bool>>),
            run_animations: values
                .run_animations
                .map(|v| Rc::new(ConstantProperty::from_value(v)) as Rc<dyn DynProperty<bool>>),
            clamp_animations: values
                .clamp_animations
                .map(|v| Rc::new(ConstantProperty::from_value(v)) as Rc<dyn DynProperty<bool>>),
            shadows: values.shadows.map(|v| {
                Rc::new(ConstantProperty::from_value(v)) as Rc<dyn DynProperty<ShadowMode>>
            }),
            height_reference: values.height_reference.map(|v| {
                Rc::new(ConstantProperty::from_value(v)) as Rc<dyn DynProperty<HeightReference>>
            }),
            distance_display_condition: values.distance_display_condition.map(|v| {
                Rc::new(ConstantProperty::from_value(v))
                    as Rc<dyn DynProperty<DistanceDisplayCondition>>
            }),
            silhouette_color: values
                .silhouette_color
                .map(|v| Rc::new(ConstantProperty::from_value(v)) as Rc<dyn DynProperty<Color>>),
            silhouette_size: values
                .silhouette_size
                .map(|v| Rc::new(ConstantProperty::from_value(v)) as Rc<dyn DynProperty<f64>>),
            color: values
                .color
                .map(|v| Rc::new(ConstantProperty::from_value(v)) as Rc<dyn DynProperty<Color>>),
            color_blend_mode: values.color_blend_mode.map(|v| {
                Rc::new(ConstantProperty::from_value(v)) as Rc<dyn DynProperty<ColorBlendMode>>
            }),
            color_blend_amount: values
                .color_blend_amount
                .map(|v| Rc::new(ConstantProperty::from_value(v)) as Rc<dyn DynProperty<f64>>),
            clipping_planes: values.clipping_planes.map(|v| {
                Rc::new(ConstantProperty::from_value(v))
                    as Rc<dyn DynProperty<ClippingPlaneCollection>>
            }),
            custom_shader: values.custom_shader.map(|v| {
                Rc::new(ConstantProperty::from_value(v)) as Rc<dyn DynProperty<CustomShader>>
            }),
            image_based_lighting_factor: values.image_based_lighting_factor.map(|v| {
                Rc::new(ConstantProperty::from_value(v)) as Rc<dyn DynProperty<Cartesian2>>
            }),
            light_color: values
                .light_color
                .map(|v| Rc::new(ConstantProperty::from_value(v)) as Rc<dyn DynProperty<Color>>),
            node_transformations: None,
            articulations: None,
        })
    }

    /// Gets the event that is raised whenever a property is changed or modified.
    pub fn definition_changed(&self) -> &Event<()> {
        &self.definition_changed
    }

    /// Assigns each unassigned property on this object to the value of the same property on the source.
    pub fn merge(&mut self, source: &ModelGraphics) {
        if self.uri.is_none() {
            self.uri = source.uri.clone();
        }
        if self.scale.is_none() {
            self.scale = source.scale.clone();
        }
        if self.show.is_none() {
            self.show = source.show.clone();
        }
        if self.minimum_pixel_size.is_none() {
            self.minimum_pixel_size = source.minimum_pixel_size.clone();
        }
        if self.maximum_scale.is_none() {
            self.maximum_scale = source.maximum_scale.clone();
        }
        if self.incrementally_load_textures.is_none() {
            self.incrementally_load_textures = source.incrementally_load_textures.clone();
        }
        if self.run_animations.is_none() {
            self.run_animations = source.run_animations.clone();
        }
        if self.clamp_animations.is_none() {
            self.clamp_animations = source.clamp_animations.clone();
        }
        if self.shadows.is_none() {
            self.shadows = source.shadows.clone();
        }
        if self.height_reference.is_none() {
            self.height_reference = source.height_reference.clone();
        }
        if self.distance_display_condition.is_none() {
            self.distance_display_condition = source.distance_display_condition.clone();
        }
        if self.silhouette_color.is_none() {
            self.silhouette_color = source.silhouette_color.clone();
        }
        if self.silhouette_size.is_none() {
            self.silhouette_size = source.silhouette_size.clone();
        }
        if self.color.is_none() {
            self.color = source.color.clone();
        }
        if self.color_blend_mode.is_none() {
            self.color_blend_mode = source.color_blend_mode.clone();
        }
        if self.color_blend_amount.is_none() {
            self.color_blend_amount = source.color_blend_amount.clone();
        }
        if self.clipping_planes.is_none() {
            self.clipping_planes = source.clipping_planes.clone();
        }
        if self.custom_shader.is_none() {
            self.custom_shader = source.custom_shader.clone();
        }
        if self.image_based_lighting_factor.is_none() {
            self.image_based_lighting_factor = source.image_based_lighting_factor.clone();
        }
        if self.light_color.is_none() {
            self.light_color = source.light_color.clone();
        }
        self.node_transformations.merge(&source.node_transformations);
        self.articulations.merge(&source.articulations);
    }
}
