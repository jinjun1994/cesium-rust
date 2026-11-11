//! PropertyTable matching CesiumJS `Scene/PropertyTable.js` (Apache-2.0).

use std::collections::HashSet;
use std::rc::Rc;

use cesium_math::DeveloperError;
use serde_json::Value;

use crate::batch_table_hierarchy::BatchTableHierarchy;
use crate::json_metadata_table::JsonMetadataTable;
use crate::metadata_class::MetadataClass;
use crate::metadata_table::MetadataTable;
use crate::metadata_table_property::MetadataTableProperty;
use crate::texture::Texture;

/// Options for constructing a `PropertyTable`.
#[derive(Default)]
pub struct PropertyTableOptions {
    pub name: Option<String>,
    pub id: Option<Value>,
    pub count: usize,
    pub metadata_table: Option<MetadataTable>,
    pub json_metadata_table: Option<JsonMetadataTable>,
    pub batch_table_hierarchy: Option<BatchTableHierarchy>,
    pub texture: Option<Rc<Texture>>,
    pub extras: Option<Value>,
    pub extensions: Option<Value>,
}

/// A property table for use with `EXT_structural_metadata` or 3D Tiles 1.0 batch tables.
pub struct PropertyTable {
    name: Option<String>,
    id: Option<Value>,
    count: usize,
    metadata_table: Option<MetadataTable>,
    json_metadata_table: Option<JsonMetadataTable>,
    batch_table_hierarchy: Option<BatchTableHierarchy>,
    texture: Option<Rc<Texture>>,
    extras: Option<Value>,
    extensions: Option<Value>,
}

impl PropertyTable {
    /// Creates a new `PropertyTable`.
    pub fn new(options: PropertyTableOptions) -> Self {
        Self {
            name: options.name,
            id: options.id,
            count: options.count,
            metadata_table: options.metadata_table,
            json_metadata_table: options.json_metadata_table,
            batch_table_hierarchy: options.batch_table_hierarchy,
            texture: options.texture,
            extras: options.extras,
            extensions: options.extensions,
        }
    }

    /// The human-readable name of the table.
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// The unique id of the table.
    pub fn id(&self) -> Option<&Value> {
        self.id.as_ref()
    }

    /// The number of features in the table.
    pub fn count(&self) -> usize {
        self.count
    }

    /// The class that properties conform to (if binary metadata table is present).
    pub fn class(&self) -> Option<&MetadataClass> {
        self.metadata_table.as_ref().map(|t| t.class_definition())
    }

    /// The properties stored in this table (if binary metadata table is present).
    pub fn properties(&self) -> Option<&std::collections::HashMap<String, MetadataTableProperty>> {
        self.metadata_table.as_ref().map(|t| t.properties())
    }

    /// Extra user-defined properties.
    pub fn extras(&self) -> Option<&Value> {
        self.extras.as_ref()
    }

    /// Extension dictionary.
    pub fn extensions(&self) -> Option<&Value> {
        self.extensions.as_ref()
    }

    /// Total amount of binary metadata stored in memory in bytes.
    pub fn byte_length(&self) -> usize {
        let mut total = 0;
        if let Some(ref mt) = self.metadata_table {
            total += mt.byte_length();
        }
        if let Some(ref bth) = self.batch_table_hierarchy {
            total += bth.byte_length();
        }
        total
    }

    /// GPU texture containing packed property table data, if any.
    pub fn texture(&self) -> Option<&Rc<Texture>> {
        self.texture.as_ref()
    }

    /// Internal json metadata table reference (for inspection/testing).
    pub fn json_metadata_table(&self) -> Option<&JsonMetadataTable> {
        self.json_metadata_table.as_ref()
    }

    /// Returns whether the feature has this property.
    pub fn has_property(&self, index: usize, property_id: &str) -> Result<bool, DeveloperError> {
        if property_id.is_empty() {
            return Err(DeveloperError::new("propertyId is required."));
        }
        if let Some(ref mt) = self.metadata_table {
            if mt.has_property(property_id)? {
                return Ok(true);
            }
        }
        if let Some(ref bth) = self.batch_table_hierarchy {
            if bth.has_property(index, property_id) {
                return Ok(true);
            }
        }
        if let Some(ref jmt) = self.json_metadata_table {
            if jmt.has_property(property_id) {
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// Returns whether the feature has a property with the given semantic.
    pub fn has_property_by_semantic(
        &self,
        _index: usize,
        semantic: &str,
    ) -> Result<bool, DeveloperError> {
        if semantic.is_empty() {
            return Err(DeveloperError::new("semantic is required."));
        }
        if let Some(ref mt) = self.metadata_table {
            return mt.has_property_by_semantic(semantic);
        }
        Ok(false)
    }

    /// Returns whether any feature has this property.
    pub fn property_exists(&self, property_id: &str) -> Result<bool, DeveloperError> {
        if property_id.is_empty() {
            return Err(DeveloperError::new("propertyId is required."));
        }
        if let Some(ref mt) = self.metadata_table {
            if mt.has_property(property_id)? {
                return Ok(true);
            }
        }
        if let Some(ref bth) = self.batch_table_hierarchy {
            if bth.property_exists(property_id) {
                return Ok(true);
            }
        }
        if let Some(ref jmt) = self.json_metadata_table {
            if jmt.has_property(property_id) {
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// Returns whether any feature has a property with the given semantic.
    pub fn property_exists_by_semantic(&self, semantic: &str) -> Result<bool, DeveloperError> {
        if semantic.is_empty() {
            return Err(DeveloperError::new("semantic is required."));
        }
        if let Some(ref mt) = self.metadata_table {
            return mt.has_property_by_semantic(semantic);
        }
        Ok(false)
    }

    /// Returns array of property IDs for a specific feature (or all property IDs).
    pub fn get_property_ids(&self, index: Option<usize>) -> Vec<String> {
        let mut results = Vec::new();
        let mut seen = HashSet::new();

        if let Some(ref mt) = self.metadata_table {
            for id in mt.get_property_ids() {
                if seen.insert(id.clone()) {
                    results.push(id);
                }
            }
        }

        if let Some(ref bth) = self.batch_table_hierarchy {
            let idx = index.unwrap_or(0);
            for id in bth.get_property_ids(idx) {
                if seen.insert(id.clone()) {
                    results.push(id);
                }
            }
        }

        if let Some(ref jmt) = self.json_metadata_table {
            for id in jmt.get_property_ids() {
                if seen.insert(id.clone()) {
                    results.push(id);
                }
            }
        }

        results
    }

    /// Returns the property value for a given feature index and property ID.
    pub fn get_property(&self, index: usize, property_id: &str) -> Option<Value> {
        if let Some(ref mt) = self.metadata_table {
            if let Ok(Some(val)) = mt.get_property(index, property_id) {
                return Some(val);
            }
        }
        if let Some(ref bth) = self.batch_table_hierarchy {
            if let Some(val) = bth.get_property(index, property_id) {
                return Some(val);
            }
        }
        if let Some(ref jmt) = self.json_metadata_table {
            if let Ok(Some(val)) = jmt.get_property(index, property_id) {
                return Some(val);
            }
        }
        None
    }

    /// Sets the value of the property for a given feature index.
    pub fn set_property(
        &mut self,
        index: usize,
        property_id: &str,
        value: Value,
    ) -> Result<(), DeveloperError> {
        if let Some(ref mut mt) = self.metadata_table {
            if mt.has_property(property_id)? {
                mt.set_property(index, property_id, value)?;
                return Ok(());
            }
        }

        if let Some(ref mut bth) = self.batch_table_hierarchy {
            if bth.has_property(index, property_id) {
                bth.set_property(index, property_id, value)
                    .map_err(|e| DeveloperError::new(e.to_string()))?;
                return Ok(());
            }
        }

        if self.json_metadata_table.is_none() {
            self.json_metadata_table = Some(JsonMetadataTable::new(
                self.count,
                std::collections::HashMap::new(),
            )?);
        }

        if let Some(ref mut jmt) = self.json_metadata_table {
            jmt.set_property(index, property_id, value)?;
        }

        Ok(())
    }

    /// Returns the property value for a given feature index and semantic.
    pub fn get_property_by_semantic(&self, index: usize, semantic: &str) -> Option<Value> {
        if let Some(ref mt) = self.metadata_table {
            if let Ok(Some(val)) = mt.get_property_by_semantic(index, semantic) {
                return Some(val);
            }
        }
        None
    }

    /// Sets the property value for a given semantic.
    pub fn set_property_by_semantic(
        &mut self,
        index: usize,
        semantic: &str,
        value: Value,
    ) -> Result<bool, DeveloperError> {
        if let Some(ref mut mt) = self.metadata_table {
            return mt.set_property_by_semantic(index, semantic, value);
        }
        Ok(false)
    }

    /// Returns a typed array containing the property values for a given propertyId.
    pub fn get_property_typed_array(&self, property_id: &str) -> Option<Vec<u8>> {
        if let Some(ref mt) = self.metadata_table {
            if let Ok(Some(bytes)) = mt.get_property_typed_array(property_id) {
                return Some(bytes);
            }
        }
        None
    }

    /// Returns a typed array containing property values for a given semantic.
    pub fn get_property_typed_array_by_semantic(&self, semantic: &str) -> Option<Vec<u8>> {
        if let Some(ref mt) = self.metadata_table {
            if let Ok(Some(bytes)) = mt.get_property_typed_array_by_semantic(semantic) {
                return Some(bytes);
            }
        }
        None
    }

    /// Checks if a feature belongs to a given class in hierarchy.
    pub fn is_class(&self, feature_id: usize, class_name: &str) -> Result<bool, DeveloperError> {
        self.check_feature_id(feature_id)?;
        if let Some(ref bth) = self.batch_table_hierarchy {
            return Ok(bth.is_class(feature_id, class_name));
        }
        Ok(false)
    }

    /// Checks if a feature's exact class matches class_name.
    pub fn is_exact_class(
        &self,
        feature_id: usize,
        class_name: &str,
    ) -> Result<bool, DeveloperError> {
        self.check_feature_id(feature_id)?;
        Ok(self.get_exact_class_name(feature_id)? == Some(class_name.to_string()))
    }

    /// Gets the exact class name for a feature.
    pub fn get_exact_class_name(
        &self,
        feature_id: usize,
    ) -> Result<Option<String>, DeveloperError> {
        self.check_feature_id(feature_id)?;
        if let Some(ref bth) = self.batch_table_hierarchy {
            return Ok(bth.get_class_name(feature_id).map(|s| s.to_string()));
        }
        Ok(None)
    }

    fn check_feature_id(&self, feature_id: usize) -> Result<(), DeveloperError> {
        if feature_id >= self.count {
            return Err(DeveloperError::new(format!(
                "featureId is required and must be between zero and featuresLength - 1 ({}).",
                self.count
            )));
        }
        Ok(())
    }
}
