//! ModelFeature and ModelFeatureTable matching CesiumJS `Scene/Model/ModelFeature.js` and `Scene/Model/ModelFeatureTable.js` (Apache-2.0).

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use cesium_math::{Color, DeveloperError};
use serde_json::Value;

use crate::model_draw_command::StyleCommandsNeeded;
use crate::model_type::ModelType;

/// Metadata property definition in a model's property table.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ModelPropertyDefinition {
    pub semantic: Option<String>,
    pub property_type: Option<String>,
    pub component_type: Option<String>,
}

/// In-memory property table matching CesiumJS `PropertyTable`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ModelPropertyTable {
    pub count: usize,
    pub property_ids: Vec<String>,
    pub properties: HashMap<String, ModelPropertyDefinition>,
    pub values: HashMap<String, Vec<Value>>,
}

impl ModelPropertyTable {
    pub fn new(count: usize) -> Self {
        Self {
            count,
            property_ids: Vec::new(),
            properties: HashMap::new(),
            values: HashMap::new(),
        }
    }

    pub fn add_property(
        &mut self,
        name: impl Into<String>,
        definition: ModelPropertyDefinition,
        values: Vec<Value>,
    ) {
        let name_str = name.into();
        if !self.property_ids.contains(&name_str) {
            self.property_ids.push(name_str.clone());
        }
        self.properties.insert(name_str.clone(), definition);
        self.values.insert(name_str, values);
    }

    pub fn has_property(&self, feature_id: usize, name: &str) -> bool {
        feature_id < self.count && self.values.contains_key(name)
    }

    pub fn has_property_by_semantic(&self, feature_id: usize, semantic: &str) -> bool {
        if feature_id >= self.count {
            return false;
        }
        self.properties
            .values()
            .any(|def| def.semantic.as_deref() == Some(semantic))
    }

    pub fn get_property(&self, feature_id: usize, name: &str) -> Option<Value> {
        if feature_id >= self.count {
            return None;
        }
        self.values.get(name).and_then(|col| col.get(feature_id).cloned())
    }

    pub fn get_property_by_semantic(&self, feature_id: usize, semantic: &str) -> Option<Value> {
        if feature_id >= self.count {
            return None;
        }
        for (prop_name, def) in &self.properties {
            if def.semantic.as_deref() == Some(semantic) {
                return self.get_property(feature_id, prop_name);
            }
        }
        None
    }

    pub fn get_property_inherited(&self, feature_id: usize, name: &str) -> Option<Value> {
        if self.has_property_by_semantic(feature_id, name) {
            return self.get_property_by_semantic(feature_id, name);
        }
        self.get_property(feature_id, name)
    }

    pub fn get_property_ids(&self, results: &mut Vec<String>) -> Vec<String> {
        for id in &self.property_ids {
            results.push(id.clone());
        }
        results.clone()
    }

    pub fn set_property(&mut self, feature_id: usize, name: &str, value: Value) -> bool {
        if feature_id >= self.count {
            return false;
        }
        if let Some(col) = self.values.get_mut(name) {
            if feature_id < col.len() {
                col[feature_id] = value;
                return true;
            }
        }
        false
    }
}

/// Batch texture state matching CesiumJS `BatchTexture`.
#[derive(Clone, Debug, PartialEq)]
pub struct ModelFeatureBatchTexture {
    pub features_length: usize,
    pub show: Vec<bool>,
    pub color: Vec<Color>,
    pub translucent_features_length: usize,
    pub is_destroyed: bool,
    pub byte_length: usize,
}

impl ModelFeatureBatchTexture {
    pub const DEFAULT_COLOR_VALUE: Color = Color::WHITE;
    pub const DEFAULT_SHOW_VALUE: bool = true;

    pub fn new(features_length: usize) -> Self {
        Self {
            features_length,
            show: vec![Self::DEFAULT_SHOW_VALUE; features_length],
            color: vec![Self::DEFAULT_COLOR_VALUE; features_length],
            translucent_features_length: 0,
            is_destroyed: false,
            byte_length: features_length * 4,
        }
    }

    pub fn set_show(&mut self, feature_id: usize, show: bool) {
        if feature_id < self.show.len() {
            self.show[feature_id] = show;
        }
    }

    pub fn set_all_show(&mut self, show: bool) {
        self.show.fill(show);
    }

    pub fn get_show(&self, feature_id: usize) -> bool {
        self.show
            .get(feature_id)
            .copied()
            .unwrap_or(Self::DEFAULT_SHOW_VALUE)
    }

    pub fn set_color(&mut self, feature_id: usize, color: Color) {
        if feature_id < self.color.len() {
            self.color[feature_id] = color;
            self.update_translucent_count();
        }
    }

    pub fn set_all_color(&mut self, color: Color) {
        self.color.fill(color);
        self.update_translucent_count();
    }

    pub fn get_color(&self, feature_id: usize) -> Color {
        self.color
            .get(feature_id)
            .copied()
            .unwrap_or(Self::DEFAULT_COLOR_VALUE)
    }

    fn update_translucent_count(&mut self) {
        self.translucent_features_length = self
            .color
            .iter()
            .filter(|c| c.alpha > 0.0 && c.alpha < 1.0)
            .count();
    }

    pub fn is_destroyed(&self) -> bool {
        self.is_destroyed
    }

    pub fn destroy(&mut self) {
        self.is_destroyed = true;
    }
}

/// The model target that owns features.
#[derive(Clone, Debug, PartialEq)]
pub struct ModelRuntimeTarget {
    pub model_type: ModelType,
    pub has_content: bool,
}

impl Default for ModelRuntimeTarget {
    fn default() -> Self {
        Self {
            model_type: ModelType::Gltf,
            has_content: false,
        }
    }
}

/// Distinguishes between a regular ModelFeature and a 3D Tile feature.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModelFeatureKind {
    ModelFeature,
    Cesium3DTileFeature,
}

/// Internal state of a ModelFeatureTable.
#[derive(Debug)]
pub struct ModelFeatureTableInner {
    pub model: ModelRuntimeTarget,
    pub property_table: ModelPropertyTable,
    pub batch_texture: ModelFeatureBatchTexture,
    pub style_commands_needed: StyleCommandsNeeded,
    pub style_commands_needed_dirty: bool,
    pub is_destroyed: bool,
}

impl ModelFeatureTableInner {
    pub fn new(model: ModelRuntimeTarget, property_table: ModelPropertyTable) -> Self {
        let count = property_table.count;
        let batch_texture = ModelFeatureBatchTexture::new(count);
        Self {
            model,
            property_table,
            batch_texture,
            style_commands_needed: StyleCommandsNeeded::AllOpaque,
            style_commands_needed_dirty: false,
            is_destroyed: false,
        }
    }
}

/// A feature belonging to a Model or 3D Tile.
#[derive(Clone)]
pub struct ModelFeature {
    feature_id: usize,
    primitive: ModelRuntimeTarget,
    feature_table: Arc<RwLock<ModelFeatureTableInner>>,
    kind: ModelFeatureKind,
}

impl ModelFeature {
    pub fn new(
        primitive: ModelRuntimeTarget,
        feature_id: usize,
        feature_table: Arc<RwLock<ModelFeatureTableInner>>,
        kind: ModelFeatureKind,
    ) -> Self {
        Self {
            feature_id,
            primitive,
            feature_table,
            kind,
        }
    }

    pub fn primitive(&self) -> &ModelRuntimeTarget {
        &self.primitive
    }

    pub fn feature_table(&self) -> Arc<RwLock<ModelFeatureTableInner>> {
        Arc::clone(&self.feature_table)
    }

    pub fn feature_id(&self) -> usize {
        self.feature_id
    }

    pub fn kind(&self) -> ModelFeatureKind {
        self.kind
    }

    pub fn is_model_feature(&self) -> bool {
        self.kind == ModelFeatureKind::ModelFeature
    }

    pub fn is_cesium_3d_tile_feature(&self) -> bool {
        self.kind == ModelFeatureKind::Cesium3DTileFeature
    }

    pub fn show(&self) -> bool {
        self.feature_table
            .read()
            .unwrap()
            .batch_texture
            .get_show(self.feature_id)
    }

    pub fn set_show(&self, show: bool) {
        self.feature_table
            .write()
            .unwrap()
            .batch_texture
            .set_show(self.feature_id, show);
    }

    pub fn color(&self) -> Color {
        self.feature_table
            .read()
            .unwrap()
            .batch_texture
            .get_color(self.feature_id)
    }

    pub fn set_color(&self, color: Color) {
        self.feature_table
            .write()
            .unwrap()
            .batch_texture
            .set_color(self.feature_id, color);
    }

    pub fn has_property(&self, name: &str) -> bool {
        self.feature_table
            .read()
            .unwrap()
            .property_table
            .has_property(self.feature_id, name)
    }

    pub fn get_property(&self, name: &str) -> Option<Value> {
        self.feature_table
            .read()
            .unwrap()
            .property_table
            .get_property(self.feature_id, name)
    }

    pub fn get_property_inherited(&self, name: &str) -> Option<Value> {
        self.feature_table
            .read()
            .unwrap()
            .property_table
            .get_property_inherited(self.feature_id, name)
    }

    pub fn get_property_ids(&self, results: &mut Vec<String>) -> Vec<String> {
        self.feature_table
            .read()
            .unwrap()
            .property_table
            .get_property_ids(results)
    }

    pub fn set_property(&self, name: &str, value: impl Into<Value>) -> bool {
        self.feature_table
            .write()
            .unwrap()
            .property_table
            .set_property(self.feature_id, name, value.into())
    }
}

impl std::fmt::Debug for ModelFeature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ModelFeature")
            .field("feature_id", &self.feature_id)
            .field("primitive", &self.primitive)
            .field("kind", &self.kind)
            .finish()
    }
}

impl PartialEq for ModelFeature {
    fn eq(&self, other: &Self) -> bool {
        self.feature_id == other.feature_id
            && self.primitive == other.primitive
            && self.kind == other.kind
    }
}

/// Manages the ModelFeatures in a Model and extracts properties from a PropertyTable.
#[derive(Clone, Debug)]
pub struct ModelFeatureTable {
    inner: Arc<RwLock<ModelFeatureTableInner>>,
    features: Vec<ModelFeature>,
}

impl ModelFeatureTable {
    pub fn new(model: ModelRuntimeTarget, property_table: ModelPropertyTable) -> Self {
        let is_3d_tiles = ModelType::is_3d_tiles(model.model_type) || model.has_content;
        let kind = if is_3d_tiles {
            ModelFeatureKind::Cesium3DTileFeature
        } else {
            ModelFeatureKind::ModelFeature
        };

        let count = property_table.count;
        let inner = Arc::new(RwLock::new(ModelFeatureTableInner::new(
            model.clone(),
            property_table,
        )));

        let features = (0..count)
            .map(|i| ModelFeature::new(model.clone(), i, Arc::clone(&inner), kind))
            .collect();

        Self { inner, features }
    }

    pub fn features_length(&self) -> usize {
        self.features.len()
    }

    pub fn get_feature(&self, feature_id: usize) -> Option<ModelFeature> {
        self.features.get(feature_id).cloned()
    }

    pub fn features(&self) -> &[ModelFeature] {
        &self.features
    }

    pub fn has_property(&self, feature_id: usize, name: &str) -> bool {
        self.inner
            .read()
            .unwrap()
            .property_table
            .has_property(feature_id, name)
    }

    pub fn has_property_by_semantic(&self, feature_id: usize, semantic: &str) -> bool {
        self.inner
            .read()
            .unwrap()
            .property_table
            .has_property_by_semantic(feature_id, semantic)
    }

    pub fn get_property(&self, feature_id: usize, name: &str) -> Option<Value> {
        self.inner
            .read()
            .unwrap()
            .property_table
            .get_property(feature_id, name)
    }

    pub fn get_property_by_semantic(&self, feature_id: usize, semantic: &str) -> Option<Value> {
        self.inner
            .read()
            .unwrap()
            .property_table
            .get_property_by_semantic(feature_id, semantic)
    }

    pub fn get_property_inherited(&self, feature_id: usize, name: &str) -> Option<Value> {
        self.inner
            .read()
            .unwrap()
            .property_table
            .get_property_inherited(feature_id, name)
    }

    pub fn get_property_ids(&self, results: &mut Vec<String>) -> Vec<String> {
        self.inner
            .read()
            .unwrap()
            .property_table
            .get_property_ids(results)
    }

    pub fn set_property(&mut self, feature_id: usize, name: &str, value: impl Into<Value>) -> bool {
        self.inner
            .write()
            .unwrap()
            .property_table
            .set_property(feature_id, name, value.into())
    }

    pub fn get_show(&self, feature_id: usize) -> bool {
        self.inner
            .read()
            .unwrap()
            .batch_texture
            .get_show(feature_id)
    }

    pub fn set_show(&mut self, feature_id: usize, show: bool) {
        self.inner
            .write()
            .unwrap()
            .batch_texture
            .set_show(feature_id, show);
    }

    pub fn set_all_show(&mut self, show: bool) {
        self.inner.write().unwrap().batch_texture.set_all_show(show);
    }

    pub fn get_color(&self, feature_id: usize) -> Color {
        self.inner
            .read()
            .unwrap()
            .batch_texture
            .get_color(feature_id)
    }

    pub fn set_color(&mut self, feature_id: usize, color: Color) {
        self.inner
            .write()
            .unwrap()
            .batch_texture
            .set_color(feature_id, color);
    }

    pub fn set_all_color(&mut self, color: Color) {
        self.inner
            .write()
            .unwrap()
            .batch_texture
            .set_all_color(color);
    }

    pub fn batch_texture(&self) -> ModelFeatureBatchTexture {
        self.inner.read().unwrap().batch_texture.clone()
    }

    pub fn is_destroyed(&self) -> bool {
        self.inner.read().unwrap().is_destroyed
    }

    pub fn destroy(&mut self) -> Result<(), DeveloperError> {
        let mut inner = self.inner.write().unwrap();
        inner.batch_texture.destroy();
        inner.is_destroyed = true;
        Ok(())
    }
}
