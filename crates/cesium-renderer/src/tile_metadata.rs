//! TileMetadata matching CesiumJS `Scene/TileMetadata.js` (Apache-2.0).

use std::collections::HashMap;
use cesium_math::DeveloperError;
use serde_json::Value;

use crate::bounding_volume_semantics::PropertyBySemantic;
use crate::metadata_class::MetadataClass;
use crate::metadata_entity::{MetadataEntity, MetadataEntityTrait};

/// Options for creating a `TileMetadata` instance.
#[derive(Debug, Clone, Default)]
pub struct TileMetadataOptions {
    pub tile: Option<Value>,
    pub class_def: Option<MetadataClass>,
}

/// Metadata about a 3D Tile (3DTILES_metadata extension).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TileMetadata {
    pub class_def: Option<MetadataClass>,
    pub properties: HashMap<String, Value>,
    pub extras: Option<Value>,
    pub extensions: Option<Value>,
}

impl TileMetadata {
    /// Creates a new `TileMetadata` directly from optional class and properties.
    pub fn new(class: Option<MetadataClass>, properties: HashMap<String, Value>) -> Self {
        Self {
            class_def: class,
            properties,
            extras: None,
            extensions: None,
        }
    }

    /// Constructs a `TileMetadata` from options, validating that both `tile` and `class` are provided.
    pub fn from_options(options: TileMetadataOptions) -> Result<Self, DeveloperError> {
        let tile = options
            .tile
            .ok_or_else(|| DeveloperError::new("options.tile is required."))?;
        let class_def = options
            .class_def
            .ok_or_else(|| DeveloperError::new("options.class is required."))?;

        let mut properties = HashMap::new();
        if let Some(props_obj) = tile.get("properties").and_then(Value::as_object) {
            for (k, v) in props_obj {
                properties.insert(k.clone(), v.clone());
            }
        }

        let extras = tile.get("extras").cloned();
        let extensions = tile.get("extensions").cloned();

        Ok(Self {
            class_def: Some(class_def),
            properties,
            extras,
            extensions,
        })
    }

    pub fn class_definition(&self) -> Option<&MetadataClass> {
        self.class_def.as_ref()
    }

    pub fn extras(&self) -> Option<&Value> {
        self.extras.as_ref()
    }

    pub fn extensions(&self) -> Option<&Value> {
        self.extensions.as_ref()
    }

    pub fn has_property(&self, property_id: &str) -> bool {
        if let Some(ref class) = self.class_def {
            MetadataEntity::has_property(property_id, &self.properties, class)
        } else {
            self.properties.contains_key(property_id)
        }
    }

    pub fn has_property_by_semantic(&self, semantic: &str) -> bool {
        if let Some(ref class) = self.class_def {
            MetadataEntity::has_property_by_semantic(semantic, &self.properties, class)
        } else {
            false
        }
    }

    pub fn get_property_ids(&self, results: Option<&mut Vec<String>>) -> Vec<String> {
        let ids = if let Some(ref class) = self.class_def {
            MetadataEntity::get_property_ids(&self.properties, class)
        } else {
            let mut keys: Vec<String> = self.properties.keys().cloned().collect();
            keys.sort();
            keys
        };
        if let Some(res) = results {
            res.extend(ids.clone());
        }
        ids
    }

    pub fn get_property(&self, property_id: &str) -> Result<Value, DeveloperError> {
        let class = self
            .class_def
            .as_ref()
            .ok_or_else(|| DeveloperError::new("Metadata class is undefined"))?;
        let val_opt = MetadataEntity::get_property(property_id, &self.properties, class)?;
        val_opt.ok_or_else(|| DeveloperError::new(format!("Property '{property_id}' does not exist")))
    }

    pub fn set_property(&mut self, property_id: &str, value: Value) -> bool {
        if let Some(ref class) = self.class_def {
            MetadataEntity::set_property(property_id, value, &mut self.properties, class)
                .unwrap_or(false)
        } else if self.properties.contains_key(property_id) {
            self.properties.insert(property_id.to_string(), value);
            true
        } else {
            false
        }
    }

    pub fn get_property_by_semantic(&self, semantic: &str) -> Option<Value> {
        let class = self.class_def.as_ref()?;
        MetadataEntity::get_property_by_semantic(semantic, &self.properties, class).ok().flatten()
    }

    pub fn set_property_by_semantic(&mut self, semantic: &str, value: Value) -> bool {
        if let Some(ref class) = self.class_def {
            MetadataEntity::set_property_by_semantic(semantic, value, &mut self.properties, class)
                .unwrap_or(false)
        } else {
            false
        }
    }
}

impl PropertyBySemantic for TileMetadata {
    fn get_property_by_semantic(&self, semantic: &str) -> Option<Value> {
        TileMetadata::get_property_by_semantic(self, semantic)
    }
}

impl MetadataEntityTrait for TileMetadata {
    fn class_definition(&self) -> Result<&MetadataClass, DeveloperError> {
        self.class_def
            .as_ref()
            .ok_or_else(|| DeveloperError::new("Metadata class is undefined"))
    }

    fn has_property(&self, property_id: &str) -> Result<bool, DeveloperError> {
        Ok(TileMetadata::has_property(self, property_id))
    }

    fn has_property_by_semantic(&self, semantic: &str) -> Result<bool, DeveloperError> {
        Ok(TileMetadata::has_property_by_semantic(self, semantic))
    }

    fn get_property_ids(&self) -> Result<Vec<String>, DeveloperError> {
        Ok(TileMetadata::get_property_ids(self, None))
    }

    fn get_property(&self, property_id: &str) -> Result<Option<Value>, DeveloperError> {
        let class = self
            .class_def
            .as_ref()
            .ok_or_else(|| DeveloperError::new("Metadata class is undefined"))?;
        MetadataEntity::get_property(property_id, &self.properties, class)
    }

    fn set_property(&mut self, property_id: &str, value: Value) -> Result<bool, DeveloperError> {
        Ok(TileMetadata::set_property(self, property_id, value))
    }

    fn get_property_by_semantic(&self, semantic: &str) -> Result<Option<Value>, DeveloperError> {
        Ok(TileMetadata::get_property_by_semantic(self, semantic))
    }

    fn set_property_by_semantic(&mut self, semantic: &str, value: Value) -> Result<bool, DeveloperError> {
        Ok(TileMetadata::set_property_by_semantic(self, semantic, value))
    }
}
