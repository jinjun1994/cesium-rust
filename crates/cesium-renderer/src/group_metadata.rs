//! GroupMetadata matching CesiumJS `Scene/GroupMetadata.js` (Apache-2.0).

use crate::bounding_volume_semantics::MetadataClass;
use cesium_math::DeveloperError;
use serde_json::Value;
use std::collections::HashMap;

/// Options for creating a `GroupMetadata` instance.
#[derive(Debug, Clone, Default)]
pub struct GroupMetadataOptions {
    /// The ID of the group.
    pub id: Option<String>,
    /// The group JSON object.
    pub group: Option<Value>,
    /// The class that group metadata conforms to.
    pub class_def: Option<MetadataClass>,
}

/// Metadata about a group of 3D Tile contents (3DTILES_metadata extension).
#[derive(Debug, Clone)]
pub struct GroupMetadata {
    id: Option<String>,
    class_def: MetadataClass,
    properties: HashMap<String, Value>,
    extras: Option<Value>,
    extensions: Option<Value>,
}

impl GroupMetadata {
    /// Creates a new `GroupMetadata` instance.
    pub fn new(options: GroupMetadataOptions) -> Result<Self, DeveloperError> {
        let group = options
            .group
            .ok_or_else(|| DeveloperError::new("options.group is required."))?;
        let class_def = options
            .class_def
            .ok_or_else(|| DeveloperError::new("options.class is required."))?;

        let mut properties = HashMap::new();
        if let Some(props_obj) = group.get("properties").and_then(Value::as_object) {
            for (k, v) in props_obj {
                properties.insert(k.clone(), v.clone());
            }
        }

        let extras = group.get("extras").cloned();
        let extensions = group.get("extensions").cloned();

        Ok(Self {
            id: options.id,
            class_def,
            properties,
            extras,
            extensions,
        })
    }

    /// The ID of the group.
    pub fn id(&self) -> Option<&str> {
        self.id.as_deref()
    }

    /// The class that properties conform to.
    pub fn class_def(&self) -> &MetadataClass {
        &self.class_def
    }

    /// Extra user-defined properties.
    pub fn extras(&self) -> Option<&Value> {
        self.extras.as_ref()
    }

    /// An object containing extensions.
    pub fn extensions(&self) -> Option<&Value> {
        self.extensions.as_ref()
    }

    /// Returns whether the group has this property (either explicit or default).
    pub fn has_property(&self, property_id: Option<&str>) -> Result<bool, DeveloperError> {
        let prop_id = property_id.ok_or_else(|| DeveloperError::new("propertyId is required."))?;
        if self.properties.contains_key(prop_id) {
            return Ok(true);
        }
        if let Some(class_prop) = self.class_def.properties.get(prop_id) {
            if class_prop.default.is_some() {
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// Returns whether the group has a property with the given semantic.
    pub fn has_property_by_semantic(&self, semantic: Option<&str>) -> Result<bool, DeveloperError> {
        let sem = semantic.ok_or_else(|| DeveloperError::new("semantic is required."))?;
        for (prop_id, class_prop) in &self.class_def.properties {
            if class_prop.semantic.as_deref() == Some(sem)
                && (self.properties.contains_key(prop_id) || class_prop.default.is_some())
            {
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// Returns an array of property IDs.
    pub fn get_property_ids(&self, results: &mut Vec<String>) -> Vec<String> {
        for prop_id in self.properties.keys() {
            if !results.contains(prop_id) {
                results.push(prop_id.clone());
            }
        }
        for (prop_id, class_prop) in &self.class_def.properties {
            if class_prop.default.is_some() && !results.contains(prop_id) {
                results.push(prop_id.clone());
            }
        }
        results.clone()
    }

    /// Returns a copy of the value of the property with the given ID.
    pub fn get_property(&self, property_id: Option<&str>) -> Result<Option<Value>, DeveloperError> {
        let prop_id = property_id.ok_or_else(|| DeveloperError::new("propertyId is required."))?;
        if let Some(val) = self.properties.get(prop_id) {
            return Ok(Some(val.clone()));
        }
        if let Some(class_prop) = self.class_def.properties.get(prop_id) {
            if let Some(default_val) = &class_prop.default {
                return Ok(Some(default_val.clone()));
            }
        }
        Ok(None)
    }

    /// Sets the value of the property with the given ID.
    pub fn set_property(
        &mut self,
        property_id: Option<&str>,
        value: Option<Value>,
    ) -> Result<bool, DeveloperError> {
        let prop_id = property_id.ok_or_else(|| DeveloperError::new("propertyId is required."))?;
        let val = value.ok_or_else(|| DeveloperError::new("value is required."))?;

        if !self.class_def.properties.contains_key(prop_id) {
            return Ok(false);
        }

        self.properties.insert(prop_id.to_string(), val);
        Ok(true)
    }

    /// Returns a copy of the value of the property with the given semantic.
    pub fn get_property_by_semantic(
        &self,
        semantic: Option<&str>,
    ) -> Result<Option<Value>, DeveloperError> {
        let sem = semantic.ok_or_else(|| DeveloperError::new("semantic is required."))?;
        for (prop_id, class_prop) in &self.class_def.properties {
            if class_prop.semantic.as_deref() == Some(sem) {
                return self.get_property(Some(prop_id));
            }
        }
        Ok(None)
    }

    /// Sets the value of the property with the given semantic.
    pub fn set_property_by_semantic(
        &mut self,
        semantic: Option<&str>,
        value: Option<Value>,
    ) -> Result<bool, DeveloperError> {
        let sem = semantic.ok_or_else(|| DeveloperError::new("semantic is required."))?;
        let val = value.ok_or_else(|| DeveloperError::new("value is required."))?;

        let mut target_prop_id = None;
        for (prop_id, class_prop) in &self.class_def.properties {
            if class_prop.semantic.as_deref() == Some(sem) {
                target_prop_id = Some(prop_id.clone());
                break;
            }
        }

        if let Some(prop_id) = target_prop_id {
            self.set_property(Some(&prop_id), Some(val))
        } else {
            Ok(false)
        }
    }
}
