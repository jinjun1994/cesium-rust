use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use cesium_math::DeveloperError;
use serde_json::Value;

use crate::bounding_volume_semantics::MetadataClass;

/// A table of metadata properties with multiple rows (entities).
#[derive(Debug, Clone, Default)]
pub struct MetadataTable {
    pub class: MetadataClass,
    pub count: usize,
    pub properties: HashMap<String, Vec<Value>>,
}

impl MetadataTable {
    /// Creates a new `MetadataTable`.
    pub fn new(class: MetadataClass, count: usize, properties: HashMap<String, Vec<Value>>) -> Self {
        Self {
            class,
            count,
            properties,
        }
    }

    /// Whether this table contains property with `property_id`.
    pub fn has_property(&self, property_id: &str) -> bool {
        self.properties.contains_key(property_id)
    }

    /// Whether this table contains a property with semantic.
    pub fn has_property_by_semantic(&self, semantic: &str) -> bool {
        self.class
            .properties
            .iter()
            .any(|(k, prop)| prop.semantic.as_deref() == Some(semantic) && self.properties.contains_key(k))
    }

    /// Appends property IDs in this table to results and returns.
    pub fn get_property_ids(&self, results: &mut Vec<String>) -> Vec<String> {
        for key in self.properties.keys() {
            results.push(key.clone());
        }
        results.clone()
    }

    /// Gets property value for an entity.
    pub fn get_property(&self, entity_id: usize, property_id: &str) -> Option<Value> {
        self.properties.get(property_id).and_then(|col| col.get(entity_id).cloned())
    }

    /// Sets property value for an entity.
    pub fn set_property(&mut self, entity_id: usize, property_id: &str, value: Value) -> bool {
        if let Some(col) = self.properties.get_mut(property_id) {
            if entity_id < col.len() {
                col[entity_id] = value;
                return true;
            }
        }
        false
    }

    /// Finds property ID corresponding to semantic.
    pub fn find_property_id_by_semantic(&self, semantic: &str) -> Option<String> {
        self.class
            .properties
            .iter()
            .find(|(k, prop)| prop.semantic.as_deref() == Some(semantic) && self.properties.contains_key(*k))
            .map(|(k, _)| k.clone())
    }

    /// Gets property value by semantic for an entity.
    pub fn get_property_by_semantic(&self, entity_id: usize, semantic: &str) -> Option<Value> {
        let prop_id = self.find_property_id_by_semantic(semantic)?;
        self.get_property(entity_id, &prop_id)
    }

    /// Sets property value by semantic for an entity.
    pub fn set_property_by_semantic(&mut self, entity_id: usize, semantic: &str, value: Value) -> bool {
        if let Some(prop_id) = self.find_property_id_by_semantic(semantic) {
            self.set_property(entity_id, &prop_id, value)
        } else {
            false
        }
    }
}

/// Initialization options for an [`ImplicitMetadataView`].
#[derive(Clone, Default)]
pub struct ImplicitMetadataViewOptions {
    pub metadata_table: Option<Arc<RwLock<MetadataTable>>>,
    pub class: Option<MetadataClass>,
    pub entity_id: Option<usize>,
    pub property_table_json: Option<Value>,
}

/// A view representing a single entity within an implicit subtree metadata table.
#[derive(Clone)]
pub struct ImplicitMetadataView {
    metadata_table: Arc<RwLock<MetadataTable>>,
    class: MetadataClass,
    entity_id: usize,
    extras: Option<Value>,
    extensions: Option<Value>,
}

impl ImplicitMetadataView {
    /// Constructs an `ImplicitMetadataView` from options with validation.
    pub fn new(options: ImplicitMetadataViewOptions) -> Result<Self, DeveloperError> {
        let metadata_table = options
            .metadata_table
            .ok_or_else(|| DeveloperError::new("options.metadataTable is required."))?;
        let class = options
            .class
            .ok_or_else(|| DeveloperError::new("options.class is required."))?;
        let entity_id = options
            .entity_id
            .ok_or_else(|| DeveloperError::new("options.entityId is required."))?;
        let json = options
            .property_table_json
            .ok_or_else(|| DeveloperError::new("options.propertyTableJson is required."))?;

        let extras = json.get("extras").cloned();
        let extensions = json.get("extensions").cloned();

        Ok(Self {
            metadata_table,
            class,
            entity_id,
            extras,
            extensions,
        })
    }

    /// The class that properties conform to.
    pub fn class(&self) -> &MetadataClass {
        &self.class
    }

    /// Extra user-defined properties.
    pub fn extras(&self) -> Option<&Value> {
        self.extras.as_ref()
    }

    /// An object containing extensions.
    pub fn extensions(&self) -> Option<&Value> {
        self.extensions.as_ref()
    }

    /// Returns whether the metadata contains this property.
    pub fn has_property(&self, property_id: &str) -> bool {
        self.metadata_table.read().unwrap().has_property(property_id)
    }

    /// Returns whether the metadata contains a property with the given semantic.
    pub fn has_property_by_semantic(&self, semantic: &str) -> bool {
        self.metadata_table.read().unwrap().has_property_by_semantic(semantic)
    }

    /// Returns property IDs in the metadata table.
    pub fn get_property_ids(&self, results: &mut Vec<String>) -> Vec<String> {
        self.metadata_table.read().unwrap().get_property_ids(results)
    }

    /// Gets property value by property ID.
    pub fn get_property(&self, property_id: &str) -> Option<Value> {
        self.metadata_table.read().unwrap().get_property(self.entity_id, property_id)
    }

    /// Sets property value by property ID.
    pub fn set_property(&mut self, property_id: &str, value: Value) -> bool {
        self.metadata_table.write().unwrap().set_property(self.entity_id, property_id, value)
    }

    /// Gets property value by semantic.
    pub fn get_property_by_semantic(&self, semantic: &str) -> Option<Value> {
        self.metadata_table.read().unwrap().get_property_by_semantic(self.entity_id, semantic)
    }

    /// Sets property value by semantic.
    pub fn set_property_by_semantic(&mut self, semantic: &str, value: Value) -> bool {
        self.metadata_table.write().unwrap().set_property_by_semantic(self.entity_id, semantic, value)
    }
}
