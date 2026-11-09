use std::collections::HashMap;
use cesium_math::DeveloperError;
use serde_json::Value;

use crate::bounding_volume_semantics::MetadataClass;

/// Initialization options for [`ImplicitSubtreeMetadata`].
#[derive(Clone, Default)]
pub struct ImplicitSubtreeMetadataOptions {
    pub subtree_metadata: Option<Value>,
    pub class: Option<MetadataClass>,
}

/// Metadata describing an implicit subtree entity conforming to a metadata class.
#[derive(Debug, Clone)]
pub struct ImplicitSubtreeMetadata {
    class_def: MetadataClass,
    properties: HashMap<String, Value>,
    extras: Option<Value>,
    extensions: Option<Value>,
}

impl ImplicitSubtreeMetadata {
    /// Constructs an `ImplicitSubtreeMetadata` from options with validation.
    pub fn new(options: ImplicitSubtreeMetadataOptions) -> Result<Self, DeveloperError> {
        let subtree_json = options
            .subtree_metadata
            .ok_or_else(|| DeveloperError::new("options.subtreeMetadata is required."))?;
        let class_def = options
            .class
            .ok_or_else(|| DeveloperError::new("options.class is required."))?;

        let mut properties = HashMap::new();
        if let Some(props_obj) = subtree_json.get("properties").and_then(Value::as_object) {
            for (k, v) in props_obj {
                properties.insert(k.clone(), v.clone());
            }
        }

        let extras = subtree_json.get("extras").cloned();
        let extensions = subtree_json.get("extensions").cloned();

        Ok(Self {
            class_def,
            properties,
            extras,
            extensions,
        })
    }

    /// The class that metadata conforms to.
    pub fn class_def(&self) -> &MetadataClass {
        &self.class_def
    }

    /// Extra user-defined properties.
    pub fn extras(&self) -> Option<&Value> {
        self.extras.as_ref()
    }

    /// Extensions object.
    pub fn extensions(&self) -> Option<&Value> {
        self.extensions.as_ref()
    }

    /// Returns whether the subtree has this property (either in properties or via class default).
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

    /// Returns whether the subtree has a property with the given semantic.
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

    /// Returns an array of property IDs, optionally appending into `results`.
    pub fn get_property_ids(&self, results: Option<&mut Vec<String>>) -> Vec<String> {
        let mut list = Vec::new();
        for k in self.properties.keys() {
            if !list.contains(k) {
                list.push(k.clone());
            }
        }
        for (k, class_prop) in &self.class_def.properties {
            if class_prop.default.is_some() && !list.contains(k) {
                list.push(k.clone());
            }
        }

        if let Some(target) = results {
            target.extend(list.clone());
            target.clone()
        } else {
            list
        }
    }

    /// Returns a copy of the property value, throwing if property is not defined in class.
    pub fn get_property(&self, property_id: Option<&str>) -> Result<Option<Value>, DeveloperError> {
        let prop_id = property_id.ok_or_else(|| DeveloperError::new("propertyId is required."))?;

        if !self.class_def.properties.is_empty() && !self.class_def.properties.contains_key(prop_id) {
            return Err(DeveloperError::new(format!(
                "Property '{prop_id}' is not defined in class"
            )));
        }

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

    /// Sets property value.
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

    /// Returns property value by semantic.
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

    /// Sets property value by semantic.
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
