//! MetadataEnum matching CesiumJS `Scene/MetadataEnum.js` and `Scene/MetadataEnumValue.js` (Apache-2.0).

use crate::metadata_component_type::MetadataComponentType;
use cesium_math::DeveloperError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A single enum value.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MetadataEnumValue {
    pub name: String,
    pub value: i64,
    pub description: Option<String>,
    pub extras: Option<serde_json::Value>,
    pub extensions: Option<serde_json::Value>,
}

impl MetadataEnumValue {
    /// Deserializes a `MetadataEnumValue` from a JSON value.
    pub fn from_json(val: Option<&serde_json::Value>) -> Result<Self, DeveloperError> {
        let v = val.ok_or_else(|| DeveloperError::new("options.value is required."))?;
        if v.is_null() {
            return Err(DeveloperError::new("options.value is required."));
        }
        let obj = v
            .as_object()
            .ok_or_else(|| DeveloperError::new("value must be an object"))?;

        let name = obj
            .get("name")
            .and_then(|n| n.as_str())
            .ok_or_else(|| DeveloperError::new("name is required"))?
            .to_string();

        let value = obj
            .get("value")
            .and_then(|val| val.as_i64())
            .ok_or_else(|| DeveloperError::new("value integer is required"))?;

        let description = obj
            .get("description")
            .and_then(|d| d.as_str())
            .map(|s| s.to_string());

        let extras = obj.get("extras").cloned();
        let extensions = obj.get("extensions").cloned();

        Ok(Self {
            name,
            value,
            description,
            extras,
            extensions,
        })
    }
}

/// Options for deserializing a `MetadataEnum`.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct MetadataEnumJson {
    pub name: Option<String>,
    pub description: Option<String>,
    #[serde(rename = "valueType")]
    pub value_type: Option<String>,
    pub values: Vec<MetadataEnumValue>,
    pub extras: Option<serde_json::Value>,
    pub extensions: Option<serde_json::Value>,
}

/// An enumerated type definition in 3D metadata.
#[derive(Clone, Debug, PartialEq)]
pub struct MetadataEnum {
    pub id: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub value_type: MetadataComponentType,
    pub values: Vec<MetadataEnumValue>,
    pub values_by_name: HashMap<String, i64>,
    pub values_by_number: HashMap<i64, String>,
    pub extras: Option<serde_json::Value>,
    pub extensions: Option<serde_json::Value>,
}

impl MetadataEnum {
    /// Creates a `MetadataEnum` from JSON options containing `id` and `enum`.
    pub fn from_json_options(options: &serde_json::Value) -> Result<Self, DeveloperError> {
        let id = options
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| DeveloperError::new("options.id is required."))?
            .to_string();

        let enum_val = options
            .get("enum")
            .ok_or_else(|| DeveloperError::new("options.enum is required."))?;

        let enum_json: MetadataEnumJson = serde_json::from_value(enum_val.clone()).map_err(|e| {
            DeveloperError::new(format!("Failed to parse enum JSON: {e}"))
        })?;

        Self::from_json(id, enum_json)
    }

    /// Creates a `MetadataEnum` from an explicit id and `MetadataEnumJson`.
    pub fn from_json(id: String, enum_json: MetadataEnumJson) -> Result<Self, DeveloperError> {
        let value_type = if let Some(ref vt) = enum_json.value_type {
            vt.parse::<MetadataComponentType>()?
        } else {
            MetadataComponentType::Uint16
        };

        let mut values_by_name = HashMap::new();
        let mut values_by_number = HashMap::new();

        for val in &enum_json.values {
            values_by_name.insert(val.name.clone(), val.value);
            values_by_number.insert(val.value, val.name.clone());
        }

        Ok(Self {
            id,
            name: enum_json.name,
            description: enum_json.description,
            value_type,
            values: enum_json.values,
            values_by_name,
            values_by_number,
            extras: enum_json.extras,
            extensions: enum_json.extensions,
        })
    }
}
