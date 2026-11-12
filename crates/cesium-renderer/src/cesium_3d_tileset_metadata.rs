//! 3D Tileset metadata container matching CesiumJS `Scene/Cesium3DTilesetMetadata.js`.
use std::collections::HashMap;

use cesium_math::DeveloperError;
use serde_json::Value;

use crate::group_metadata::{GroupMetadata, GroupMetadataOptions};
use crate::metadata_schema::MetadataSchema;
use crate::tileset_metadata::TilesetMetadata;

/// Options for constructing [`Cesium3DTilesetMetadata`].
#[derive(Clone, Debug, Default)]
pub struct Cesium3DTilesetMetadataOptions {
    pub metadata_json: Option<Value>,
    pub schema: Option<MetadataSchema>,
}

/// Metadata about a 3D Tileset (3D Tiles 1.1 or `3DTILES_metadata` extension).
#[derive(Clone, Debug)]
pub struct Cesium3DTilesetMetadata {
    pub schema: MetadataSchema,
    pub groups: Vec<GroupMetadata>,
    pub group_ids: Vec<String>,
    pub tileset: Option<TilesetMetadata>,
    pub statistics: Option<Value>,
    pub extras: Option<Value>,
    pub extensions: Option<Value>,
}

impl Cesium3DTilesetMetadata {
    /// Creates a new [`Cesium3DTilesetMetadata`], validating `metadata_json` and `schema`.
    pub fn new(options: Cesium3DTilesetMetadataOptions) -> Result<Self, DeveloperError> {
        let metadata_json = options
            .metadata_json
            .ok_or_else(|| DeveloperError::new("options.metadataJson is required."))?;
        let schema = options
            .schema
            .ok_or_else(|| DeveloperError::new("options.schema is required."))?;

        let metadata_obj = metadata_json
            .get("metadata")
            .or_else(|| metadata_json.get("tileset"));

        let tileset = if let Some(meta) = metadata_obj {
            let class_name = meta.get("class").and_then(Value::as_str);
            let class_def = class_name.and_then(|name| schema.classes.get(name).cloned());
            let mut props = HashMap::new();
            if let Some(obj) = meta.get("properties").and_then(Value::as_object) {
                for (k, v) in obj {
                    props.insert(k.clone(), v.clone());
                }
            }
            let mut tm = TilesetMetadata::new(class_def, props);
            tm.extras = meta.get("extras").cloned();
            tm.extensions = meta.get("extensions").cloned();
            Some(tm)
        } else {
            None
        };

        let mut group_ids = Vec::new();
        let mut groups = Vec::new();
        if let Some(groups_val) = metadata_json.get("groups") {
            if let Some(arr) = groups_val.as_array() {
                for group_val in arr {
                    let class_name = group_val.get("class").and_then(Value::as_str);
                    let class_def = class_name.and_then(|name| schema.classes.get(name).cloned());
                    groups.push(GroupMetadata::new(GroupMetadataOptions {
                        id: None,
                        group: Some(group_val.clone()),
                        class_def,
                    })?);
                }
            } else if let Some(obj) = groups_val.as_object() {
                let mut keys: Vec<String> = obj.keys().cloned().collect();
                keys.sort();
                group_ids = keys.clone();
                for group_id in keys {
                    if let Some(group_val) = obj.get(&group_id) {
                        let class_name = group_val.get("class").and_then(Value::as_str);
                        let class_def =
                            class_name.and_then(|name| schema.classes.get(name).cloned());
                        groups.push(GroupMetadata::new(GroupMetadataOptions {
                            id: Some(group_id),
                            group: Some(group_val.clone()),
                            class_def,
                        })?);
                    }
                }
            }
        }

        Ok(Self {
            schema,
            groups,
            group_ids,
            tileset,
            statistics: metadata_json.get("statistics").cloned(),
            extras: metadata_json.get("extras").cloned(),
            extensions: metadata_json.get("extensions").cloned(),
        })
    }
}
