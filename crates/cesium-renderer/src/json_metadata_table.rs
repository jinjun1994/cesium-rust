use std::collections::HashMap;
use cesium_math::DeveloperError;

/// Free-form JSON metadata table, matching 3D Tiles legacy batch table behavior.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct JsonMetadataTable {
    count: usize,
    properties: HashMap<String, Vec<serde_json::Value>>,
}

impl JsonMetadataTable {
    /// Constructs a `JsonMetadataTable` with explicit count and properties.
    pub fn new(
        count: usize,
        properties: HashMap<String, Vec<serde_json::Value>>,
    ) -> Result<Self, DeveloperError> {
        if count == 0 {
            return Err(DeveloperError::new("count must be greater than 0"));
        }

        Ok(Self { count, properties })
    }

    /// Number of entities in this table.
    pub fn count(&self) -> usize {
        self.count
    }

    /// Returns whether the table contains the given property ID.
    pub fn has_property(&self, property_id: &str) -> bool {
        self.properties.contains_key(property_id)
    }

    /// Returns all property IDs in the table.
    pub fn get_property_ids(&self) -> Vec<String> {
        self.properties.keys().cloned().collect()
    }

    /// Gets a cloned property value for the entity at `index`.
    pub fn get_property(
        &self,
        index: usize,
        property_id: &str,
    ) -> Result<Option<serde_json::Value>, DeveloperError> {
        if index >= self.count {
            return Err(DeveloperError::new(format!(
                "index must be in the range [0, {})",
                self.count
            )));
        }

        Ok(self
            .properties
            .get(property_id)
            .and_then(|arr| arr.get(index).cloned())
            .filter(|v| !v.is_null()))
    }

    /// Sets the property value for entity at `index`. If property does not exist, it is created.
    pub fn set_property(
        &mut self,
        index: usize,
        property_id: &str,
        value: serde_json::Value,
    ) -> Result<(), DeveloperError> {
        if index >= self.count {
            return Err(DeveloperError::new(format!(
                "index must be in the range [0, {})",
                self.count
            )));
        }

        let entry = self
            .properties
            .entry(property_id.to_string())
            .or_insert_with(|| vec![serde_json::Value::Null; self.count]);

        if entry.len() < self.count {
            entry.resize(self.count, serde_json::Value::Null);
        }

        entry[index] = value;
        Ok(())
    }
}
