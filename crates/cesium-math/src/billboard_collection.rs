//! Billboard and BillboardCollection matching CesiumJS `Scene/BillboardCollection.js` and `Scene/Billboard.js` (Apache-2.0).

use crate::{
    BoundingRectangle, Cartesian2, Cartesian3, Color, DeveloperError, DistanceDisplayCondition,
    HeightReference, HorizontalOrigin, Matrix4, NearFarScalar, SplitDirection, VerticalOrigin,
};

/// A single billboard representation managed by BillboardCollection.
#[derive(Clone, Debug, PartialEq)]
pub struct Billboard {
    pub id: Option<String>,
    pub show: bool,
    pub position: Cartesian3,
    pub color: Color,
    pub eye_offset: Cartesian3,
    pub pixel_offset: Cartesian2,
    pub scale: f64,
    pub rotation: f64,
    pub aligned_axis: Cartesian3,
    pub horizontal_origin: HorizontalOrigin,
    pub vertical_origin: VerticalOrigin,
    pub height_reference: HeightReference,
    pub width: Option<f64>,
    pub height: Option<f64>,
    pub scale_by_distance: Option<NearFarScalar>,
    pub translucency_by_distance: Option<NearFarScalar>,
    pub pixel_offset_scale_by_distance: Option<NearFarScalar>,
    pub size_in_meters: bool,
    pub distance_display_condition: Option<DistanceDisplayCondition>,
    pub disable_depth_test_distance: Option<f64>,
    pub split_direction: SplitDirection,
    pub image: Option<String>,
    pub image_sub_region: Option<BoundingRectangle>,
    pub ready: bool,
}

impl Default for Billboard {
    fn default() -> Self {
        Self {
            id: None,
            show: true,
            position: Cartesian3::ZERO,
            color: Color::WHITE,
            eye_offset: Cartesian3::ZERO,
            pixel_offset: Cartesian2::ZERO,
            scale: 1.0,
            rotation: 0.0,
            aligned_axis: Cartesian3::ZERO,
            horizontal_origin: HorizontalOrigin::Center,
            vertical_origin: VerticalOrigin::Center,
            height_reference: HeightReference::None,
            width: None,
            height: None,
            scale_by_distance: None,
            translucency_by_distance: None,
            pixel_offset_scale_by_distance: None,
            size_in_meters: false,
            distance_display_condition: None,
            disable_depth_test_distance: None,
            split_direction: SplitDirection::None,
            image: None,
            image_sub_region: None,
            ready: true,
        }
    }
}

impl Billboard {
    /// Validates `NearFarScalar`, `DistanceDisplayCondition`, and `disable_depth_test_distance` invariants.
    pub fn validate(&self) -> Result<(), DeveloperError> {
        if let Some(ref s) = self.scale_by_distance {
            if s.near >= s.far {
                return Err(DeveloperError::new(
                    "scaleByDistance.farDistance must be greater than scaleByDistance.nearDistance.",
                ));
            }
        }
        if let Some(ref t) = self.translucency_by_distance {
            if t.near >= t.far {
                return Err(DeveloperError::new(
                    "translucencyByDistance.farDistance must be greater than translucencyByDistance.nearDistance.",
                ));
            }
        }
        if let Some(ref p) = self.pixel_offset_scale_by_distance {
            if p.near >= p.far {
                return Err(DeveloperError::new(
                    "pixelOffsetScaleByDistance.farDistance must be greater than pixelOffsetScaleByDistance.nearDistance.",
                ));
            }
        }
        if let Some(ref d) = self.distance_display_condition {
            if d.near >= d.far {
                return Err(DeveloperError::new(
                    "distanceDisplayCondition.far must be greater than distanceDisplayCondition.near.",
                ));
            }
        }
        if let Some(dist) = self.disable_depth_test_distance {
            if dist < 0.0 {
                return Err(DeveloperError::new(
                    "disableDepthTestDistance must be greater than or equal to 0.0.",
                ));
            }
        }
        Ok(())
    }
}

/// A renderable collection of billboards.
#[derive(Clone, Debug)]
pub struct BillboardCollection {
    pub billboards: Vec<Billboard>,
    pub model_matrix: Matrix4,
    pub show: bool,
    pub debug_show_bounding_volume: bool,
    pub destroy_texture_atlas: bool,
    destroyed: bool,
}

impl Default for BillboardCollection {
    fn default() -> Self {
        Self::new()
    }
}

impl BillboardCollection {
    pub fn new() -> Self {
        Self {
            billboards: Vec::new(),
            model_matrix: Matrix4::IDENTITY,
            show: true,
            debug_show_bounding_volume: false,
            destroy_texture_atlas: true,
            destroyed: false,
        }
    }

    pub fn len(&self) -> usize {
        self.billboards.len()
    }

    pub fn is_empty(&self) -> bool {
        self.billboards.is_empty()
    }

    pub fn get(&self, index: usize) -> Option<&Billboard> {
        self.billboards.get(index)
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut Billboard> {
        self.billboards.get_mut(index)
    }

    pub fn add(&mut self, billboard: Billboard) -> usize {
        self.billboards.push(billboard);
        self.billboards.len() - 1
    }

    pub fn add_checked(&mut self, billboard: Billboard) -> Result<usize, DeveloperError> {
        billboard.validate()?;
        Ok(self.add(billboard))
    }

    pub fn contains(&self, billboard: &Billboard) -> bool {
        self.billboards.contains(billboard)
    }

    pub fn remove(&mut self, index: usize) {
        if index < self.billboards.len() {
            self.billboards.remove(index);
        }
    }

    pub fn remove_billboard(&mut self, billboard: &Billboard) -> bool {
        if let Some(pos) = self.billboards.iter().position(|b| b == billboard) {
            self.billboards.remove(pos);
            true
        } else {
            false
        }
    }

    pub fn remove_all(&mut self) {
        self.billboards.clear();
    }

    pub fn is_destroyed(&self) -> bool {
        self.destroyed
    }

    pub fn destroy(&mut self) {
        self.billboards.clear();
        self.destroyed = true;
    }
}
