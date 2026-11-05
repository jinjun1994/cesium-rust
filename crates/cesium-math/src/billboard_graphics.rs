//! BillboardGraphics matching CesiumJS `DataSources/BillboardGraphics.js` (Apache-2.0).

use std::rc::Rc;

use crate::{
    BoundingRectangle, Cartesian2, Cartesian3, Color, ConstantProperty, DistanceDisplayCondition,
    DynProperty, Event, HeightReference, HorizontalOrigin, NearFarScalar, SplitDirection,
    VerticalOrigin,
};

/// Initialization options for BillboardGraphics using dynamic or constant properties.
#[derive(Default, Clone)]
pub struct BillboardGraphicsOptions {
    pub image: Option<Rc<dyn DynProperty<String>>>,
    pub image_sub_region: Option<Rc<dyn DynProperty<BoundingRectangle>>>,
    pub scale: Option<Rc<dyn DynProperty<f64>>>,
    pub rotation: Option<Rc<dyn DynProperty<f64>>>,
    pub aligned_axis: Option<Rc<dyn DynProperty<Cartesian3>>>,
    pub color: Option<Rc<dyn DynProperty<Color>>>,
    pub height_reference: Option<Rc<dyn DynProperty<HeightReference>>>,
    pub horizontal_origin: Option<Rc<dyn DynProperty<HorizontalOrigin>>>,
    pub vertical_origin: Option<Rc<dyn DynProperty<VerticalOrigin>>>,
    pub eye_offset: Option<Rc<dyn DynProperty<Cartesian3>>>,
    pub pixel_offset: Option<Rc<dyn DynProperty<Cartesian2>>>,
    pub show: Option<Rc<dyn DynProperty<bool>>>,
    pub width: Option<Rc<dyn DynProperty<f64>>>,
    pub height: Option<Rc<dyn DynProperty<f64>>>,
    pub scale_by_distance: Option<Rc<dyn DynProperty<NearFarScalar>>>,
    pub translucency_by_distance: Option<Rc<dyn DynProperty<NearFarScalar>>>,
    pub pixel_offset_scale_by_distance: Option<Rc<dyn DynProperty<NearFarScalar>>>,
    pub size_in_meters: Option<Rc<dyn DynProperty<bool>>>,
    pub distance_display_condition: Option<Rc<dyn DynProperty<DistanceDisplayCondition>>>,
    pub disable_depth_test_distance: Option<Rc<dyn DynProperty<f64>>>,
    pub split_direction: Option<Rc<dyn DynProperty<SplitDirection>>>,
}

/// Raw literal initialization values for BillboardGraphics (convenience constructor).
#[derive(Default, Clone, Debug)]
pub struct BillboardGraphicsValues {
    pub image: Option<String>,
    pub image_sub_region: Option<BoundingRectangle>,
    pub scale: Option<f64>,
    pub rotation: Option<f64>,
    pub aligned_axis: Option<Cartesian3>,
    pub color: Option<Color>,
    pub height_reference: Option<HeightReference>,
    pub horizontal_origin: Option<HorizontalOrigin>,
    pub vertical_origin: Option<VerticalOrigin>,
    pub eye_offset: Option<Cartesian3>,
    pub pixel_offset: Option<Cartesian2>,
    pub show: Option<bool>,
    pub width: Option<f64>,
    pub height: Option<f64>,
    pub scale_by_distance: Option<NearFarScalar>,
    pub translucency_by_distance: Option<NearFarScalar>,
    pub pixel_offset_scale_by_distance: Option<NearFarScalar>,
    pub size_in_meters: Option<bool>,
    pub distance_display_condition: Option<DistanceDisplayCondition>,
    pub disable_depth_test_distance: Option<f64>,
    pub split_direction: Option<SplitDirection>,
}

/// Describes a two-dimensional icon, marker, or point of interest located at the position of the containing Entity.
#[derive(Clone, Default)]
pub struct BillboardGraphics {
    pub image: Option<Rc<dyn DynProperty<String>>>,
    pub image_sub_region: Option<Rc<dyn DynProperty<BoundingRectangle>>>,
    pub scale: Option<Rc<dyn DynProperty<f64>>>,
    pub rotation: Option<Rc<dyn DynProperty<f64>>>,
    pub aligned_axis: Option<Rc<dyn DynProperty<Cartesian3>>>,
    pub color: Option<Rc<dyn DynProperty<Color>>>,
    pub height_reference: Option<Rc<dyn DynProperty<HeightReference>>>,
    pub horizontal_origin: Option<Rc<dyn DynProperty<HorizontalOrigin>>>,
    pub vertical_origin: Option<Rc<dyn DynProperty<VerticalOrigin>>>,
    pub eye_offset: Option<Rc<dyn DynProperty<Cartesian3>>>,
    pub pixel_offset: Option<Rc<dyn DynProperty<Cartesian2>>>,
    pub show: Option<Rc<dyn DynProperty<bool>>>,
    pub width: Option<Rc<dyn DynProperty<f64>>>,
    pub height: Option<Rc<dyn DynProperty<f64>>>,
    pub scale_by_distance: Option<Rc<dyn DynProperty<NearFarScalar>>>,
    pub translucency_by_distance: Option<Rc<dyn DynProperty<NearFarScalar>>>,
    pub pixel_offset_scale_by_distance: Option<Rc<dyn DynProperty<NearFarScalar>>>,
    pub size_in_meters: Option<Rc<dyn DynProperty<bool>>>,
    pub distance_display_condition: Option<Rc<dyn DynProperty<DistanceDisplayCondition>>>,
    pub disable_depth_test_distance: Option<Rc<dyn DynProperty<f64>>>,
    pub split_direction: Option<Rc<dyn DynProperty<SplitDirection>>>,
    definition_changed: Event<()>,
}

impl BillboardGraphics {
    /// Creates a new BillboardGraphics instance with undefined properties.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a BillboardGraphics instance from property options.
    pub fn from_options(options: BillboardGraphicsOptions) -> Self {
        Self {
            image: options.image,
            image_sub_region: options.image_sub_region,
            scale: options.scale,
            rotation: options.rotation,
            aligned_axis: options.aligned_axis,
            color: options.color,
            height_reference: options.height_reference,
            horizontal_origin: options.horizontal_origin,
            vertical_origin: options.vertical_origin,
            eye_offset: options.eye_offset,
            pixel_offset: options.pixel_offset,
            show: options.show,
            width: options.width,
            height: options.height,
            scale_by_distance: options.scale_by_distance,
            translucency_by_distance: options.translucency_by_distance,
            pixel_offset_scale_by_distance: options.pixel_offset_scale_by_distance,
            size_in_meters: options.size_in_meters,
            distance_display_condition: options.distance_display_condition,
            disable_depth_test_distance: options.disable_depth_test_distance,
            split_direction: options.split_direction,
            definition_changed: Event::new(),
        }
    }

    /// Creates a BillboardGraphics instance from raw constant values.
    pub fn from_values(values: BillboardGraphicsValues) -> Self {
        Self::from_options(BillboardGraphicsOptions {
            image: values
                .image
                .map(|v| Rc::new(ConstantProperty::from_value(v)) as Rc<dyn DynProperty<String>>),
            image_sub_region: values.image_sub_region.map(|v| {
                Rc::new(ConstantProperty::from_value(v)) as Rc<dyn DynProperty<BoundingRectangle>>
            }),
            scale: values
                .scale
                .map(|v| Rc::new(ConstantProperty::from_value(v)) as Rc<dyn DynProperty<f64>>),
            rotation: values
                .rotation
                .map(|v| Rc::new(ConstantProperty::from_value(v)) as Rc<dyn DynProperty<f64>>),
            aligned_axis: values.aligned_axis.map(|v| {
                Rc::new(ConstantProperty::from_value(v)) as Rc<dyn DynProperty<Cartesian3>>
            }),
            color: values
                .color
                .map(|v| Rc::new(ConstantProperty::from_value(v)) as Rc<dyn DynProperty<Color>>),
            height_reference: values.height_reference.map(|v| {
                Rc::new(ConstantProperty::from_value(v)) as Rc<dyn DynProperty<HeightReference>>
            }),
            horizontal_origin: values.horizontal_origin.map(|v| {
                Rc::new(ConstantProperty::from_value(v)) as Rc<dyn DynProperty<HorizontalOrigin>>
            }),
            vertical_origin: values.vertical_origin.map(|v| {
                Rc::new(ConstantProperty::from_value(v)) as Rc<dyn DynProperty<VerticalOrigin>>
            }),
            eye_offset: values.eye_offset.map(|v| {
                Rc::new(ConstantProperty::from_value(v)) as Rc<dyn DynProperty<Cartesian3>>
            }),
            pixel_offset: values.pixel_offset.map(|v| {
                Rc::new(ConstantProperty::from_value(v)) as Rc<dyn DynProperty<Cartesian2>>
            }),
            show: values
                .show
                .map(|v| Rc::new(ConstantProperty::from_value(v)) as Rc<dyn DynProperty<bool>>),
            width: values
                .width
                .map(|v| Rc::new(ConstantProperty::from_value(v)) as Rc<dyn DynProperty<f64>>),
            height: values
                .height
                .map(|v| Rc::new(ConstantProperty::from_value(v)) as Rc<dyn DynProperty<f64>>),
            scale_by_distance: values.scale_by_distance.map(|v| {
                Rc::new(ConstantProperty::from_value(v)) as Rc<dyn DynProperty<NearFarScalar>>
            }),
            translucency_by_distance: values.translucency_by_distance.map(|v| {
                Rc::new(ConstantProperty::from_value(v)) as Rc<dyn DynProperty<NearFarScalar>>
            }),
            pixel_offset_scale_by_distance: values.pixel_offset_scale_by_distance.map(|v| {
                Rc::new(ConstantProperty::from_value(v)) as Rc<dyn DynProperty<NearFarScalar>>
            }),
            size_in_meters: values
                .size_in_meters
                .map(|v| Rc::new(ConstantProperty::from_value(v)) as Rc<dyn DynProperty<bool>>),
            distance_display_condition: values.distance_display_condition.map(|v| {
                Rc::new(ConstantProperty::from_value(v))
                    as Rc<dyn DynProperty<DistanceDisplayCondition>>
            }),
            disable_depth_test_distance: values
                .disable_depth_test_distance
                .map(|v| Rc::new(ConstantProperty::from_value(v)) as Rc<dyn DynProperty<f64>>),
            split_direction: values.split_direction.map(|v| {
                Rc::new(ConstantProperty::from_value(v)) as Rc<dyn DynProperty<SplitDirection>>
            }),
        })
    }

    /// Gets the event that is raised whenever a property is changed or modified.
    pub fn definition_changed(&self) -> &Event<()> {
        &self.definition_changed
    }

    /// Assigns each unassigned property on this object to the value of the same property on the source.
    pub fn merge(&mut self, source: &BillboardGraphics) {
        if self.image.is_none() {
            self.image = source.image.clone();
        }
        if self.image_sub_region.is_none() {
            self.image_sub_region = source.image_sub_region.clone();
        }
        if self.scale.is_none() {
            self.scale = source.scale.clone();
        }
        if self.rotation.is_none() {
            self.rotation = source.rotation.clone();
        }
        if self.aligned_axis.is_none() {
            self.aligned_axis = source.aligned_axis.clone();
        }
        if self.color.is_none() {
            self.color = source.color.clone();
        }
        if self.height_reference.is_none() {
            self.height_reference = source.height_reference.clone();
        }
        if self.horizontal_origin.is_none() {
            self.horizontal_origin = source.horizontal_origin.clone();
        }
        if self.vertical_origin.is_none() {
            self.vertical_origin = source.vertical_origin.clone();
        }
        if self.eye_offset.is_none() {
            self.eye_offset = source.eye_offset.clone();
        }
        if self.pixel_offset.is_none() {
            self.pixel_offset = source.pixel_offset.clone();
        }
        if self.show.is_none() {
            self.show = source.show.clone();
        }
        if self.width.is_none() {
            self.width = source.width.clone();
        }
        if self.height.is_none() {
            self.height = source.height.clone();
        }
        if self.scale_by_distance.is_none() {
            self.scale_by_distance = source.scale_by_distance.clone();
        }
        if self.translucency_by_distance.is_none() {
            self.translucency_by_distance = source.translucency_by_distance.clone();
        }
        if self.pixel_offset_scale_by_distance.is_none() {
            self.pixel_offset_scale_by_distance = source.pixel_offset_scale_by_distance.clone();
        }
        if self.size_in_meters.is_none() {
            self.size_in_meters = source.size_in_meters.clone();
        }
        if self.distance_display_condition.is_none() {
            self.distance_display_condition = source.distance_display_condition.clone();
        }
        if self.disable_depth_test_distance.is_none() {
            self.disable_depth_test_distance = source.disable_depth_test_distance.clone();
        }
        if self.split_direction.is_none() {
            self.split_direction = source.split_direction.clone();
        }
    }
}
