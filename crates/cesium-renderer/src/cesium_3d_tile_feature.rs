//! Feature abstraction for 3D Tiles matching CesiumJS `Scene/Cesium3DTileFeature.js`.
use std::collections::HashMap;

use cesium_math::Color;
use serde_json::Value;

use crate::content_metadata::ContentMetadata;
use crate::group_metadata::GroupMetadata;
use crate::implicit_subtree_metadata::ImplicitSubtreeMetadata;
use crate::model_feature_table::ModelFeatureTable;
use crate::tile_metadata::TileMetadata;
use crate::tileset_metadata::TilesetMetadata;

/// Content context backing a [`Cesium3DTileFeature`].
#[derive(Clone, Debug, Default)]
pub struct Cesium3DTileFeatureContent {
    pub feature_table: Option<ModelFeatureTable>,
    pub content_metadata: Option<ContentMetadata>,
    pub tile_metadata: Option<TileMetadata>,
    pub subtree_metadata: Option<ImplicitSubtreeMetadata>,
    pub group_metadata: Option<GroupMetadata>,
    pub tileset_metadata: Option<TilesetMetadata>,
    pub polyline_positions: HashMap<usize, Vec<f64>>,
    pub show: HashMap<usize, bool>,
    pub color: HashMap<usize, Color>,
    pub extra_properties: HashMap<(usize, String), Value>,
    pub feature_properties_dirty: bool,
}

impl Cesium3DTileFeatureContent {
    /// Looks up a property across all metadata granularities from most specific to most general.
    pub fn get_property_inherited(&self, batch_id: usize, name: &str) -> Option<Value> {
        if let Some(ref ft) = self.feature_table {
            if ft.has_property_by_semantic(batch_id, name) {
                return ft.get_property_by_semantic(batch_id, name);
            }
            if ft.has_property(batch_id, name) {
                return ft.get_property(batch_id, name);
            }
        }

        if let Some(ref cm) = self.content_metadata {
            if cm.has_property_by_semantic(name) {
                return cm.get_property_by_semantic(name);
            }
            if cm.has_property(name) {
                return cm.get_property(name).ok();
            }
        }

        if let Some(ref tm) = self.tile_metadata {
            if tm.has_property_by_semantic(name) {
                return tm.get_property_by_semantic(name);
            }
            if tm.has_property(name) {
                return tm.get_property(name).ok();
            }
        }

        if let Some(ref sm) = self.subtree_metadata {
            if sm.has_property_by_semantic(Some(name)).unwrap_or(false) {
                return sm.get_property_by_semantic(Some(name)).ok().flatten();
            }
            if sm.has_property(Some(name)).unwrap_or(false) {
                return sm.get_property(Some(name)).ok().flatten();
            }
        }

        if let Some(ref gm) = self.group_metadata {
            if gm.has_property_by_semantic(Some(name)).unwrap_or(false) {
                return gm.get_property_by_semantic(Some(name)).ok().flatten();
            }
            if gm.has_property(Some(name)).unwrap_or(false) {
                return gm.get_property(Some(name)).ok().flatten();
            }
        }

        if let Some(ref tsm) = self.tileset_metadata {
            if tsm.has_property_by_semantic(name) {
                return tsm.get_property_by_semantic(name);
            }
            if tsm.has_property(name) {
                return tsm.get_property(name).ok();
            }
        }

        None
    }
}

/// A feature of a 3D Tileset, matching CesiumJS `Cesium3DTileFeature`.
#[derive(Clone, Debug)]
pub struct Cesium3DTileFeature {
    batch_id: usize,
}

impl Cesium3DTileFeature {
    /// Creates a feature handle for `batch_id`.
    pub fn new(batch_id: usize) -> Self {
        Self { batch_id }
    }

    /// Returns the batch/feature ID associated with this feature.
    pub fn feature_id(&self) -> usize {
        self.batch_id
    }

    /// Returns whether this feature is shown (defaults to `true`).
    pub fn show(&self, content: &Cesium3DTileFeatureContent) -> bool {
        content.show.get(&self.batch_id).copied().unwrap_or(true)
    }

    /// Sets whether this feature is shown.
    pub fn set_show(&self, content: &mut Cesium3DTileFeatureContent, value: bool) {
        content.show.insert(self.batch_id, value);
    }

    /// Returns the highlight color for this feature (defaults to `Color::WHITE`).
    pub fn color(&self, content: &Cesium3DTileFeatureContent) -> Color {
        content
            .color
            .get(&self.batch_id)
            .copied()
            .unwrap_or(Color::WHITE)
    }

    /// Sets the highlight color for this feature.
    pub fn set_color(&self, content: &mut Cesium3DTileFeatureContent, value: Color) {
        content.color.insert(self.batch_id, value);
    }

    /// Returns decoded ECEF polyline positions if available.
    pub fn polyline_positions<'a>(
        &self,
        content: &'a Cesium3DTileFeatureContent,
    ) -> Option<&'a [f64]> {
        content
            .polyline_positions
            .get(&self.batch_id)
            .map(|v| v.as_slice())
    }

    /// Returns whether the feature has a property with `name` in its feature/batch table.
    pub fn has_property(&self, content: &Cesium3DTileFeatureContent, name: &str) -> bool {
        content
            .extra_properties
            .contains_key(&(self.batch_id, name.to_string()))
            || content
                .feature_table
                .as_ref()
                .is_some_and(|ft| ft.has_property(self.batch_id, name))
    }

    /// Returns all property IDs for this feature.
    pub fn get_property_ids(&self, content: &Cesium3DTileFeatureContent) -> Vec<String> {
        let mut results = Vec::new();
        if let Some(ref ft) = content.feature_table {
            results = ft.get_property_ids(&mut results);
        }
        for (id, key) in content.extra_properties.keys() {
            if *id == self.batch_id && !results.contains(key) {
                results.push(key.clone());
            }
        }
        results
    }

    /// Returns the property value for `name` from the feature/batch table.
    pub fn get_property(
        &self,
        content: &Cesium3DTileFeatureContent,
        name: &str,
    ) -> Option<Value> {
        if let Some(val) = content
            .extra_properties
            .get(&(self.batch_id, name.to_string()))
        {
            return Some(val.clone());
        }
        content
            .feature_table
            .as_ref()
            .and_then(|ft| ft.get_property(self.batch_id, name))
    }

    /// Sets the property value for `name` on the feature/batch table and marks properties dirty.
    pub fn set_property(
        &self,
        content: &mut Cesium3DTileFeatureContent,
        name: &str,
        value: Value,
    ) {
        let existed = content
            .feature_table
            .as_ref()
            .is_some_and(|ft| ft.has_property(self.batch_id, name));
        if existed {
            if let Some(ref mut ft) = content.feature_table {
                ft.set_property(self.batch_id, name, value);
            }
        } else {
            content
                .extra_properties
                .insert((self.batch_id, name.to_string()), value);
        }
        content.feature_properties_dirty = true;
    }

    /// Looks up a property across feature, content, tile, subtree, group, and tileset metadata.
    pub fn get_property_inherited(
        &self,
        content: &Cesium3DTileFeatureContent,
        name: &str,
    ) -> Option<Value> {
        content.get_property_inherited(self.batch_id, name)
    }
}
