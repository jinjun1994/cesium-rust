//! MetadataTable matching CesiumJS `Scene/MetadataTable.js` (Apache-2.0).

use crate::metadata_class::MetadataClass;
use crate::metadata_table_property::{MetadataTableProperty, MetadataTablePropertyOptions};
use cesium_math::DeveloperError;
use serde_json::Value;
use std::collections::HashMap;

/// Options for creating a [`MetadataTable`].
#[derive(Clone, Debug, Default)]
pub struct MetadataTableOptions {
    pub count: usize,
    pub class_definition: Option<MetadataClass>,
    pub properties: Option<HashMap<String, Value>>,
    pub buffer_views: Option<HashMap<String, Vec<u8>>>,
}

/// A table containing binary metadata for a collection of entities.
#[derive(Clone, Debug, PartialEq)]
pub struct MetadataTable {
    count: usize,
    class_definition: MetadataClass,
    properties: HashMap<String, MetadataTableProperty>,
    byte_length: usize,
}

impl MetadataTable {
    /// Creates a new `MetadataTable`.
    pub fn new(options: MetadataTableOptions) -> Result<Self, DeveloperError> {
        let count = options.count;
        if count == 0 {
            return Err(DeveloperError::new("options.count must be greater than 0."));
        }

        let class_definition = options
            .class_definition
            .ok_or_else(|| DeveloperError::new("options.class is required."))?;

        let mut byte_length = 0;
        let mut properties = HashMap::new();

        let buffer_views = options.buffer_views.unwrap_or_default();

        if let Some(props_map) = options.properties {
            for (property_id, prop_val) in props_map {
                if let Some(class_prop) = class_definition.properties.get(&property_id) {
                    let prop_options = MetadataTablePropertyOptions {
                        count,
                        property: prop_val,
                        class_property: class_prop.clone(),
                        buffer_views: buffer_views.clone(),
                    };
                    let table_prop = MetadataTableProperty::new(prop_options)?;
                    byte_length += table_prop.byte_length();
                    properties.insert(property_id, table_prop);
                }
            }
        }

        Ok(Self {
            count,
            class_definition,
            properties,
            byte_length,
        })
    }

    /// Number of entities in the table.
    pub fn count(&self) -> usize {
        self.count
    }

    /// Class definition that properties conform to.
    pub fn class_definition(&self) -> &MetadataClass {
        &self.class_definition
    }

    /// Total byte length across all typed arrays in this table.
    pub fn byte_length(&self) -> usize {
        self.byte_length
    }

    /// Property table mappings.
    pub fn properties(&self) -> &HashMap<String, MetadataTableProperty> {
        &self.properties
    }

    /// Returns whether the table has the property with given ID or default class value.
    pub fn has_property(&self, property_id: &str) -> Result<bool, DeveloperError> {
        if property_id.is_empty() {
            return Err(DeveloperError::new("propertyId is required."));
        }

        if self.properties.contains_key(property_id) {
            return Ok(true);
        }

        if let Some(class_prop) = self.class_definition.properties.get(property_id) {
            if class_prop.default.is_some() {
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// Returns whether the table has a property with the given semantic or default class value.
    pub fn has_property_by_semantic(&self, semantic: &str) -> Result<bool, DeveloperError> {
        if semantic.is_empty() {
            return Err(DeveloperError::new("semantic is required."));
        }

        if let Some(class_prop) = self.class_definition.properties_by_semantic.get(semantic) {
            return self.has_property(&class_prop.id);
        }

        Ok(false)
    }

    /// Returns all property IDs in the table, including properties with default values.
    pub fn get_property_ids(&self) -> Vec<String> {
        let mut ids: Vec<String> = self.properties.keys().cloned().collect();
        for (class_prop_id, class_prop) in &self.class_definition.properties {
            if class_prop.default.is_some() && !self.properties.contains_key(class_prop_id) {
                ids.push(class_prop_id.clone());
            }
        }
        ids
    }

    /// Gets a cloned property value for the entity at `index`.
    pub fn get_property(
        &self,
        index: usize,
        property_id: &str,
    ) -> Result<Option<Value>, DeveloperError> {
        if property_id.is_empty() {
            return Err(DeveloperError::new("propertyId is required."));
        }
        if index >= self.count {
            return Err(DeveloperError::new(format!(
                "index must be in the range [0, {})",
                self.count
            )));
        }

        if let Some(prop) = self.properties.get(property_id) {
            return Ok(prop.get(index));
        }

        if let Some(class_prop) = self.class_definition.properties.get(property_id) {
            return Ok(class_prop.default.clone());
        }

        Ok(None)
    }

    /// Sets the property value for entity at `index`.
    pub fn set_property(
        &mut self,
        index: usize,
        property_id: &str,
        value: Value,
    ) -> Result<bool, DeveloperError> {
        if property_id.is_empty() {
            return Err(DeveloperError::new("propertyId is required."));
        }
        if index >= self.count {
            return Err(DeveloperError::new(format!(
                "index must be in the range [0, {})",
                self.count
            )));
        }

        if let Some(prop) = self.properties.get_mut(property_id) {
            prop.set(index, value)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Gets property value by semantic.
    pub fn get_property_by_semantic(
        &self,
        index: usize,
        semantic: &str,
    ) -> Result<Option<Value>, DeveloperError> {
        if semantic.is_empty() {
            return Err(DeveloperError::new("semantic is required."));
        }
        if index >= self.count {
            return Err(DeveloperError::new(format!(
                "index must be in the range [0, {})",
                self.count
            )));
        }

        if let Some(class_prop) = self.class_definition.properties_by_semantic.get(semantic) {
            self.get_property(index, &class_prop.id)
        } else {
            Ok(None)
        }
    }

    /// Sets property value by semantic.
    pub fn set_property_by_semantic(
        &mut self,
        index: usize,
        semantic: &str,
        value: Value,
    ) -> Result<bool, DeveloperError> {
        if semantic.is_empty() {
            return Err(DeveloperError::new("semantic is required."));
        }
        if index >= self.count {
            return Err(DeveloperError::new(format!(
                "index must be in the range [0, {})",
                self.count
            )));
        }

        if let Some(class_prop) = self.class_definition.properties_by_semantic.get(semantic).cloned() {
            self.set_property(index, &class_prop.id, value)
        } else {
            Ok(false)
        }
    }

    /// Returns underlying typed array buffer bytes for a given property ID.
    pub fn get_property_typed_array(&self, property_id: &str) -> Result<Option<Vec<u8>>, DeveloperError> {
        if property_id.is_empty() {
            return Err(DeveloperError::new("propertyId is required."));
        }

        Ok(self.properties.get(property_id).and_then(|p| p.get_typed_array()))
    }

    /// Returns underlying typed array buffer bytes for the property with given semantic.
    pub fn get_property_typed_array_by_semantic(
        &self,
        semantic: &str,
    ) -> Result<Option<Vec<u8>>, DeveloperError> {
        if semantic.is_empty() {
            return Err(DeveloperError::new("semantic is required."));
        }

        if let Some(class_prop) = self.class_definition.properties_by_semantic.get(semantic) {
            self.get_property_typed_array(&class_prop.id)
        } else {
            Ok(None)
        }
    }
}
