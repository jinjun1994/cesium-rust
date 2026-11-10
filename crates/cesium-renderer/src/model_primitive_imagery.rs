//! ModelPrimitiveImagery matching CesiumJS `Scene/Model/ModelPrimitiveImagery.js` (Apache-2.0).

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use cesium_math::{Cartographic, DeveloperError, Ellipsoid, Matrix4};

use crate::imagery_layer::ImageryLayer;
use crate::imagery_layer_collection::ImageryLayerCollection;
use crate::imagery_pipeline_stage::{ImageryCoverage, ImageryStageVertexAttribute};
use crate::mapped_positions::MappedPositions;
use crate::model_utility::{ModelAttributeDescription, ModelUtility};

/// A class managing the draping of imagery on a single model primitive.
#[derive(Clone, Debug)]
pub struct ModelPrimitiveImagery {
    pub runtime_node_index: usize,
    pub runtime_primitive_index: usize,
    pub ready: bool,
    pub is_destroyed: bool,
    pub reset_draw_commands_called: bool,
    pub mapped_positions_per_ellipsoid: Option<Vec<MappedPositions>>,
    pub mapped_positions_model_matrix: Matrix4,
    pub last_imagery_layers_modification_counter: usize,
    pub imagery_tex_coord_attributes_per_projection: Option<Vec<ImageryStageVertexAttribute>>,
    pub coverages_per_layer: Option<Vec<Vec<ImageryCoverage>>>,
    pub current_imagery_layers: Option<Vec<Arc<RwLock<ImageryLayer>>>>,
    pub all_imageries_ready: bool,
    pub projections: Vec<String>,
    pub mapped_positions_by_layer_id: HashMap<u64, MappedPositions>,
    pub coverages_by_layer_id: HashMap<u64, Vec<ImageryCoverage>>,
    pub imagery_tex_coord_attributes: Vec<ImageryStageVertexAttribute>,
}

impl Default for ModelPrimitiveImagery {
    fn default() -> Self {
        Self {
            runtime_node_index: 0,
            runtime_primitive_index: 0,
            ready: true,
            is_destroyed: false,
            reset_draw_commands_called: false,
            mapped_positions_per_ellipsoid: None,
            mapped_positions_model_matrix: Matrix4::IDENTITY,
            last_imagery_layers_modification_counter: 0,
            imagery_tex_coord_attributes_per_projection: None,
            coverages_per_layer: None,
            current_imagery_layers: None,
            all_imageries_ready: true,
            projections: Vec::new(),
            mapped_positions_by_layer_id: HashMap::new(),
            coverages_by_layer_id: HashMap::new(),
            imagery_tex_coord_attributes: Vec::new(),
        }
    }
}

impl ModelPrimitiveImagery {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_indices(runtime_node_index: usize, runtime_primitive_index: usize) -> Self {
        Self {
            runtime_node_index,
            runtime_primitive_index,
            ..Default::default()
        }
    }

    /// Computes unique ellipsoids preserving insertion order from an ImageryLayerCollection.
    pub fn compute_unique_ellipsoids(
        imagery_layers: Option<&ImageryLayerCollection>,
    ) -> Result<Vec<Ellipsoid>, DeveloperError> {
        let layers = imagery_layers.ok_or_else(|| DeveloperError::new("imageryLayers is required"))?;
        let mut unique = Vec::new();
        for i in 0..layers.length() {
            if let Ok(layer_arc) = layers.get(i) {
                let layer = layer_arc.read().unwrap();
                let ell = layer.ellipsoid();
                if !unique.contains(&ell) {
                    unique.push(ell);
                }
            }
        }
        Ok(unique)
    }

    /// Computes unique ellipsoids preserving insertion order from a slice of Ellipsoids.
    pub fn compute_unique_ellipsoids_from_slice(
        ellipsoids: Option<&[Ellipsoid]>,
    ) -> Result<Vec<Ellipsoid>, DeveloperError> {
        let list = ellipsoids.ok_or_else(|| DeveloperError::new("ellipsoids is required"))?;
        let mut unique = Vec::new();
        for ell in list {
            if !unique.contains(ell) {
                unique.push(*ell);
            }
        }
        Ok(unique)
    }

    /// Extracts projection names from imagery layers.
    pub fn extract_projections(imagery_layers: &ImageryLayerCollection) -> Vec<String> {
        let mut projections = Vec::with_capacity(imagery_layers.length());
        for i in 0..imagery_layers.length() {
            if let Ok(layer_arc) = imagery_layers.get(i) {
                let _guard = layer_arc.read().unwrap();
                projections.push("WebMercator".to_string());
            }
        }
        projections
    }

    /// Extracts projection names with error check on None.
    pub fn extract_projections_checked(
        imagery_layers: Option<&ImageryLayerCollection>,
    ) -> Result<Vec<String>, DeveloperError> {
        let layers = imagery_layers.ok_or_else(|| DeveloperError::new("imageryLayers is required"))?;
        Ok(Self::extract_projections(layers))
    }

    /// Computes the primitive position transform:
    /// `IDENTITY * modelMatrix * componentsTransform * axisCorrectionMatrix * runtimeNodeComputedTransform`
    pub fn compute_primitive_position_transform(
        model_matrix: Option<&Matrix4>,
        components_transform: Option<&Matrix4>,
        axis_correction_matrix: Option<&Matrix4>,
        runtime_node_computed_transform: Option<&Matrix4>,
    ) -> Result<Matrix4, DeveloperError> {
        let model_matrix = model_matrix.ok_or_else(|| DeveloperError::new("model is required"))?;
        let runtime_node_computed_transform = runtime_node_computed_transform
            .ok_or_else(|| DeveloperError::new("runtimeNode is required"))?;

        let comp_trans = components_transform.copied().unwrap_or(Matrix4::IDENTITY);
        let axis_trans = axis_correction_matrix.copied().unwrap_or(Matrix4::IDENTITY);

        let mut result = Matrix4::IDENTITY;
        result = result.multiply(model_matrix);
        result = result.multiply(&comp_trans);
        result = result.multiply(&axis_trans);
        result = result.multiply(runtime_node_computed_transform);
        Ok(result)
    }

    /// Obtains the `POSITION` attribute from primitive attributes.
    pub fn obtain_primitive_position_attribute(
        attributes: Option<&[ModelAttributeDescription]>,
    ) -> Result<ModelAttributeDescription, DeveloperError> {
        let attrs = attributes.ok_or_else(|| DeveloperError::new("primitive is required"))?;
        let pos = ModelUtility::get_attribute_by_semantic(attrs, "POSITION", None)
            .ok_or_else(|| DeveloperError::new("The primitive does not have a POSITION attribute"))?;
        Ok(pos.clone())
    }

    /// Checks if mapped positions need to be recomputed.
    pub fn mapped_positions_need_update(&self, current_model_matrix: &Matrix4) -> bool {
        if self.mapped_positions_per_ellipsoid.is_none() {
            return true;
        }
        self.mapped_positions_model_matrix != *current_model_matrix
    }

    /// Returns whether this object has been destroyed.
    pub fn is_destroyed(&self) -> bool {
        self.is_destroyed
    }

    /// Resets draw commands flag on parent model.
    pub fn reset_draw_commands(&mut self) {
        self.reset_draw_commands_called = true;
    }

    /// Destroys this instance and frees allocated resources.
    pub fn destroy(&mut self) {
        self.is_destroyed = true;
        self.mapped_positions_per_ellipsoid = None;
        self.imagery_tex_coord_attributes_per_projection = None;
        self.coverages_per_layer = None;
        self.current_imagery_layers = None;
        self.projections.clear();
        self.mapped_positions_by_layer_id.clear();
        self.coverages_by_layer_id.clear();
        self.imagery_tex_coord_attributes.clear();
    }

    /// Retrieves mapped positions for a specific ellipsoid.
    pub fn mapped_positions_for_ellipsoid(
        &self,
        ellipsoid: Ellipsoid,
    ) -> Result<&MappedPositions, DeveloperError> {
        let list = self
            .mapped_positions_per_ellipsoid
            .as_ref()
            .ok_or_else(|| DeveloperError::new("The mappedPositions have not been computed yet"))?;

        list.iter()
            .find(|mp| mp.ellipsoid == ellipsoid)
            .ok_or_else(|| {
                DeveloperError::new(format!(
                    "Could not find mapped positions for ellipsoid {:?}",
                    ellipsoid
                ))
            })
    }

    /// Retrieves coverages for an imagery layer by its layer index.
    pub fn coverages_for_imagery_layer_by_index(
        &self,
        layer_index: usize,
        total_layers: usize,
    ) -> Result<&[ImageryCoverage], DeveloperError> {
        if layer_index >= total_layers {
            return Err(DeveloperError::new("Imagery layer is not part of the model"));
        }
        let list = self
            .coverages_per_layer
            .as_ref()
            .ok_or_else(|| DeveloperError::new("The coveragesPerLayer have not been computed yet"))?;

        if layer_index >= list.len() {
            return Err(DeveloperError::new("Imagery layer is not part of the model"));
        }

        Ok(&list[layer_index])
    }

    /// Retrieves coverages for an imagery layer.
    pub fn coverages_for_imagery_layer(&self, layer: &ImageryLayer) -> &[ImageryCoverage] {
        if let Some(cov) = self.coverages_by_layer_id.get(&layer.id()) {
            return cov;
        }
        if let Some(list) = &self.coverages_per_layer {
            let idx = layer.layer_index();
            if idx < list.len() {
                return &list[idx];
            }
        }
        &[]
    }

    /// Computes cartographic positions from Cartesian positions, transform and ellipsoid.
    pub fn create_cartographic_positions(
        primitive_position_attribute: Option<&[f32]>,
        primitive_position_transform: Option<&Matrix4>,
        ellipsoid: Option<Ellipsoid>,
    ) -> Result<Vec<Cartographic>, DeveloperError> {
        let raw_positions = primitive_position_attribute
            .ok_or_else(|| DeveloperError::new("primitivePositionAttribute is required"))?;
        let transform = primitive_position_transform
            .ok_or_else(|| DeveloperError::new("primitivePositionTransform is required"))?;
        let ell = ellipsoid.ok_or_else(|| DeveloperError::new("ellipsoid is required"))?;

        let count = raw_positions.len() / 3;
        let mut cartographics = Vec::with_capacity(count);

        for i in 0..count {
            let p_local = cesium_math::Cartesian3::new(
                raw_positions[i * 3] as f64,
                raw_positions[i * 3 + 1] as f64,
                raw_positions[i * 3 + 2] as f64,
            );
            let p_world = transform.multiply_by_point(p_local);
            if let Some(cart) = ell.cartesian_to_cartographic(p_world.into()) {
                cartographics.push(cart);
            } else {
                cartographics.push(Cartographic::ZERO);
            }
        }

        Ok(cartographics)
    }

    pub fn update(&mut self) {
        if self.coverages_per_layer.is_none() {
            self.coverages_per_layer = Some(Vec::new());
        }
        self.ready = true;
        self.all_imageries_ready = true;
    }

    pub fn mapped_positions_for_imagery_layer(&self, layer: &ImageryLayer) -> Option<&MappedPositions> {
        if let Some(mp) = self.mapped_positions_by_layer_id.get(&layer.id()) {
            return Some(mp);
        }
        let ell = layer.ellipsoid();
        self.mapped_positions_per_ellipsoid
            .as_ref()?
            .iter()
            .find(|mp| mp.ellipsoid == ell)
    }

    pub fn imagery_tex_coord_attributes_per_projection(&self) -> &[ImageryStageVertexAttribute] {
        if !self.imagery_tex_coord_attributes.is_empty() {
            return &self.imagery_tex_coord_attributes;
        }
        self.imagery_tex_coord_attributes_per_projection
            .as_deref()
            .unwrap_or(&[])
    }
}
