//! MetadataClass matching CesiumJS `Scene/MetadataClass.js` (Apache-2.0).

use crate::metadata_class_property::MetadataClassProperty;
use crate::metadata_enum::MetadataEnum;
use cesium_math::DeveloperError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A metadata class defining the schema for feature or tile metadata.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct MetadataClass {
    pub id: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub properties: HashMap<String, MetadataClassProperty>,
    pub properties_by_semantic: HashMap<String, MetadataClassProperty>,
    pub extras: Option<serde_json::Value>,
    pub extensions: Option<serde_json::Value>,
}

impl MetadataClass {
    /// Class name used for transcoding 3D Tiles 1.0 batch tables.
    pub const BATCH_TABLE_CLASS_NAME: &'static str = "_batchTable";

    /// Creates a `MetadataClass` from JSON options (throws DeveloperError if id or class missing).
    pub fn from_json(options: &serde_json::Value) -> Result<Self, DeveloperError> {
        Self::from_json_with_enums(options, None)
    }

    /// Creates a `MetadataClass` from JSON options and optional enum mappings.
    pub fn from_json_with_enums(
        options: &serde_json::Value,
        enums: Option<&HashMap<String, MetadataEnum>>,
    ) -> Result<Self, DeveloperError> {
        let id = options
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| DeveloperError::new("options.id is required."))?
            .to_string();

        let class_obj = options
            .get("class")
            .and_then(|c| c.as_object())
            .ok_or_else(|| DeveloperError::new("options.class is required."))?;

        let name = class_obj
            .get("name")
            .and_then(|n| n.as_str())
            .map(|s| s.to_string());

        let description = class_obj
            .get("description")
            .and_then(|d| d.as_str())
            .map(|s| s.to_string());

        let extras = class_obj.get("extras").cloned();
        let extensions = class_obj.get("extensions").cloned();

        let mut properties = HashMap::new();
        let mut properties_by_semantic = HashMap::new();

        if let Some(props_map) = class_obj
            .get("properties")
            .and_then(|p| p.as_object())
        {
            for (prop_id, prop_val) in props_map {
                let class_prop =
                    MetadataClassProperty::from_json(prop_id.clone(), prop_val, enums)?;
                if let Some(ref sem) = class_prop.semantic {
                    properties_by_semantic.insert(sem.clone(), class_prop.clone());
                }
                properties.insert(prop_id.clone(), class_prop);
            }
        }

        Ok(Self {
            id,
            name,
            description,
            properties,
            properties_by_semantic,
            extras,
            extensions,
        })
    }

    /// Convenience parser matching legacy signature for single JSON definition.
    pub fn from_value(json: &serde_json::Value) -> Self {
        Self::from_json(json).unwrap_or_default()
    }
}
