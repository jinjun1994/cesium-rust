//! Vector feature abstraction for 3D Tiles matching CesiumJS `Scene/Cesium3DTileVectorFeature.js`.
use std::collections::HashMap;

use cesium_math::Color;
use serde_json::Value;

use crate::create_vector_tile_buffers::{BufferCollection, VectorGltf3DTileContent};

/// A vector feature of a 3D Tileset, matching CesiumJS `Cesium3DTileVectorFeature`.
#[derive(Clone, Debug)]
pub struct Cesium3DTileVectorFeature {
    batch_id: usize,
    batch_table_id: usize,
    primitives_by_collection: HashMap<usize, Vec<usize>>,
}

impl Cesium3DTileVectorFeature {
    /// Creates a new `Cesium3DTileVectorFeature`.
    pub fn new(batch_id: usize, batch_table_id: usize) -> Self {
        Self {
            batch_id,
            batch_table_id,
            primitives_by_collection: HashMap::new(),
        }
    }

    /// Returns the feature ID (batch ID).
    pub fn feature_id(&self) -> usize {
        self.batch_id
    }

    /// Returns the batch table ID.
    pub fn batch_table_id(&self) -> usize {
        self.batch_table_id
    }

    /// Associates a primitive index in `collection_index` with this feature.
    pub fn add_primitive_by_collection(&mut self, collection_index: usize, primitive_index: usize) {
        self.primitives_by_collection
            .entry(collection_index)
            .or_default()
            .push(primitive_index);
    }

    /// Returns all pick IDs associated with this feature's primitives.
    pub fn pick_ids(&self, content: &VectorGltf3DTileContent) -> Vec<u32> {
        let mut ids = Vec::new();
        let mut col_indices: Vec<usize> = self.primitives_by_collection.keys().copied().collect();
        col_indices.sort_unstable();
        for col_idx in col_indices {
            if let Some(prim_indices) = self.primitives_by_collection.get(&col_idx) {
                for &prim_idx in prim_indices {
                    if let Some(&pick_id) = content.pick_ids.get(&(col_idx, prim_idx)) {
                        ids.push(pick_id);
                    }
                }
            }
        }
        ids
    }

    /// Returns `true` if any primitive belonging to this feature is shown.
    pub fn show(&self, content: &VectorGltf3DTileContent) -> bool {
        for (&col_idx, prim_indices) in &self.primitives_by_collection {
            let Some(collection) = content.collections.get(col_idx) else {
                continue;
            };
            let shows = match collection {
                BufferCollection::Point(p) => &p.shows,
                BufferCollection::Polyline(l) => &l.shows,
                BufferCollection::Polygon(poly) => &poly.shows,
            };
            for &prim_idx in prim_indices {
                if shows.get(prim_idx).copied().unwrap_or(false) {
                    return true;
                }
            }
        }
        false
    }

    /// Sets `show` on all primitives belonging to this feature.
    pub fn set_show(&self, content: &mut VectorGltf3DTileContent, value: bool) {
        for (&col_idx, prim_indices) in &self.primitives_by_collection {
            let Some(collection) = content.collections.get_mut(col_idx) else {
                continue;
            };
            let shows = match collection {
                BufferCollection::Point(p) => &mut p.shows,
                BufferCollection::Polyline(l) => &mut l.shows,
                BufferCollection::Polygon(poly) => &mut poly.shows,
            };
            for &prim_idx in prim_indices {
                if let Some(slot) = shows.get_mut(prim_idx) {
                    *slot = value;
                }
            }
        }
    }

    /// Returns the color of the first primitive belonging to this feature.
    pub fn color(&self, content: &VectorGltf3DTileContent) -> Color {
        let mut col_indices: Vec<usize> = self.primitives_by_collection.keys().copied().collect();
        col_indices.sort_unstable();
        for col_idx in col_indices {
            let Some(collection) = content.collections.get(col_idx) else {
                continue;
            };
            let Some(prim_indices) = self.primitives_by_collection.get(&col_idx) else {
                continue;
            };
            for &prim_idx in prim_indices {
                let c = match collection {
                    BufferCollection::Point(p) => p.materials.get(prim_idx).map(|m| m.color),
                    BufferCollection::Polyline(l) => l.materials.get(prim_idx).map(|m| m.color),
                    BufferCollection::Polygon(poly) => {
                        poly.materials.get(prim_idx).map(|m| m.color)
                    }
                };
                if let Some(color) = c {
                    return color;
                }
            }
        }
        Color::WHITE
    }

    /// Sets `color` on all primitives belonging to this feature.
    pub fn set_color(&self, content: &mut VectorGltf3DTileContent, color: Color) {
        for (&col_idx, prim_indices) in &self.primitives_by_collection {
            let Some(collection) = content.collections.get_mut(col_idx) else {
                continue;
            };
            match collection {
                BufferCollection::Point(p) => {
                    for &prim_idx in prim_indices {
                        if let Some(mat) = p.materials.get_mut(prim_idx) {
                            mat.color = color;
                        }
                    }
                }
                BufferCollection::Polyline(l) => {
                    for &prim_idx in prim_indices {
                        if let Some(mat) = l.materials.get_mut(prim_idx) {
                            mat.color = color;
                        }
                    }
                }
                BufferCollection::Polygon(poly) => {
                    for &prim_idx in prim_indices {
                        if let Some(mat) = poly.materials.get_mut(prim_idx) {
                            mat.color = color;
                        }
                    }
                }
            }
        }
    }

    fn resolve_feature<'a>(
        &self,
        content: &'a VectorGltf3DTileContent,
    ) -> Option<&'a crate::create_vector_tile_buffers::VectorGltfFeature> {
        content
            .get_feature(self.batch_id, Some(self.batch_table_id))
            .or_else(|| {
                if self.batch_table_id == 0 {
                    content.get_feature(self.batch_id, None)
                } else {
                    None
                }
            })
    }

    /// Returns whether the feature has a property named `name`.
    pub fn has_property(&self, content: &VectorGltf3DTileContent, name: &str) -> bool {
        self.resolve_feature(content)
            .is_some_and(|f| f.has_property(name))
    }

    /// Returns the property value for `name`.
    pub fn get_property(&self, content: &VectorGltf3DTileContent, name: &str) -> Option<Value> {
        self.resolve_feature(content)
            .and_then(|f| f.properties.get(name).cloned())
    }

    /// Returns all property IDs for this feature.
    pub fn get_property_ids(&self, content: &VectorGltf3DTileContent) -> Vec<String> {
        let mut ids: Vec<String> = self
            .resolve_feature(content)
            .map(|f| f.properties.keys().cloned().collect())
            .unwrap_or_default();
        ids.sort();
        ids
    }
}
