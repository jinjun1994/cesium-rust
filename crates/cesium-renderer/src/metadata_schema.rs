//! MetadataSchema matching CesiumJS `Scene/MetadataSchema.js` (Apache-2.0).

use crate::metadata_class::MetadataClass;
use crate::metadata_enum::MetadataEnum;
use cesium_math::DeveloperError;
use serde_json::{json, Value};
use std::collections::HashMap;

/// A schema containing classes and enums.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct MetadataSchema {
    pub id: Option<String>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub version: Option<String>,
    pub classes: HashMap<String, MetadataClass>,
    pub enums: HashMap<String, MetadataEnum>,
    pub extras: Option<Value>,
    pub extensions: Option<Value>,
}

impl MetadataSchema {
    /// Creates a `MetadataSchema` from JSON options.
    pub fn from_json(val: Option<&Value>) -> Result<Self, DeveloperError> {
        let v = val.ok_or_else(|| DeveloperError::new("schema is required."))?;
        let obj = v
            .as_object()
            .ok_or_else(|| DeveloperError::new("schema must be an object."))?;

        let id = obj.get("id").and_then(|s| s.as_str()).map(|s| s.to_string());
        let name = obj.get("name").and_then(|s| s.as_str()).map(|s| s.to_string());
        let description = obj
            .get("description")
            .and_then(|s| s.as_str())
            .map(|s| s.to_string());
        let version = obj
            .get("version")
            .and_then(|s| s.as_str())
            .map(|s| s.to_string());
        let extras = obj.get("extras").cloned();
        let extensions = obj.get("extensions").cloned();

        let mut enums = HashMap::new();
        if let Some(enums_map) = obj.get("enums").and_then(|e| e.as_object()) {
            for (enum_id, enum_val) in enums_map {
                let options = json!({
                    "id": enum_id,
                    "enum": enum_val
                });
                let parsed_enum = MetadataEnum::from_json_options(&options)?;
                enums.insert(enum_id.clone(), parsed_enum);
            }
        }

        let mut classes = HashMap::new();
        if let Some(classes_map) = obj.get("classes").and_then(|c| c.as_object()) {
            for (class_id, class_val) in classes_map {
                let options = json!({
                    "id": class_id,
                    "class": class_val
                });
                let parsed_class = MetadataClass::from_json_with_enums(&options, Some(&enums))?;
                classes.insert(class_id.clone(), parsed_class);
            }
        }

        Ok(Self {
            id,
            name,
            description,
            version,
            classes,
            enums,
            extras,
            extensions,
        })
    }
}
