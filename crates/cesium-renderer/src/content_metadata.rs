use crate::bounding_volume_semantics::MetadataClass;
use cesium_math::DeveloperError;
use serde_json::Value;
use std::collections::HashMap;

/// Options for creating a `ContentMetadata` instance.
#[derive(Debug, Clone, Default)]
pub struct ContentMetadataOptions {
    /// The raw content JSON or extension JSON.
    pub content: Option<Value>,
    /// The metadata class defining schemas and semantics.
    pub class_def: Option<MetadataClass>,
}

/// Metadata about the content of a 3D Tile (3D Tiles 1.1 or 3DTILES_metadata extension).
#[derive(Debug, Clone)]
pub struct ContentMetadata {
    /// The class that properties conform to.
    pub class_def: MetadataClass,
    properties: HashMap<String, Value>,
    /// Extras JSON.
    pub extras: Option<Value>,
    /// Extensions JSON.
    pub extensions: Option<Value>,
}

impl ContentMetadata {
    /// Creates a new `ContentMetadata` instance.
    pub fn new(options: ContentMetadataOptions) -> Result<Self, DeveloperError> {
        let content = options
            .content
            .ok_or_else(|| DeveloperError::new("options.content is required."))?;
        let class_def = options
            .class_def
            .ok_or_else(|| DeveloperError::new("options.class is required."))?;

        let mut properties = HashMap::new();
        if let Some(props_obj) = content.get("properties").and_then(Value::as_object) {
            for (k, v) in props_obj {
                properties.insert(k.clone(), v.clone());
            }
        }

        let extras = content.get("extras").cloned();
        let extensions = content.get("extensions").cloned();

        Ok(Self {
            class_def,
            properties,
            extras,
            extensions,
        })
    }

    /// Returns whether the content has this property.
    pub fn has_property(&self, property_id: &str) -> bool {
        self.properties.contains_key(property_id)
    }

    /// Returns whether the content has a property with the given semantic.
    pub fn has_property_by_semantic(&self, semantic: &str) -> bool {
        for (prop_id, class_prop) in &self.class_def.properties {
            if class_prop.semantic.as_deref() == Some(semantic) && self.properties.contains_key(prop_id) {
                return true;
            }
        }
        false
    }

    /// Returns an array of property IDs.
    pub fn get_property_ids(&self, results: &mut Vec<String>) -> Vec<String> {
        for k in self.properties.keys() {
            results.push(k.clone());
        }
        results.clone()
    }

    /// Returns a copy of the value of the property with the given ID.
    pub fn get_property(&self, property_id: &str) -> Result<Value, DeveloperError> {
        if !self.class_def.properties.contains_key(property_id) {
            return Err(DeveloperError::new(format!(
                "Class definition missing property {property_id}"
            )));
        }

        self.properties
            .get(property_id)
            .cloned()
            .ok_or_else(|| DeveloperError::new(format!("Property {property_id} not defined")))
    }

    /// Sets the value of the property with the given ID.
    pub fn set_property(&mut self, property_id: &str, value: Value) -> bool {
        if !self.properties.contains_key(property_id) {
            return false;
        }

        self.properties.insert(property_id.to_string(), value);
        true
    }

    /// Returns a copy of the value of the property with the given semantic.
    pub fn get_property_by_semantic(&self, semantic: &str) -> Option<Value> {
        for (prop_id, class_prop) in &self.class_def.properties {
            if class_prop.semantic.as_deref() == Some(semantic) {
                return self.properties.get(prop_id).cloned();
            }
        }
        None
    }

    /// Sets the value of the property with the given semantic.
    pub fn set_property_by_semantic(&mut self, semantic: &str, value: Value) -> bool {
        let mut target_prop_id = None;
        for (prop_id, class_prop) in &self.class_def.properties {
            if class_prop.semantic.as_deref() == Some(semantic) {
                target_prop_id = Some(prop_id.clone());
                break;
            }
        }

        if let Some(prop_id) = target_prop_id {
            self.set_property(&prop_id, value)
        } else {
            false
        }
    }
}
