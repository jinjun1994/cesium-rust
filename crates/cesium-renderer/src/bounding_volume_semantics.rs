use cesium_math::DeveloperError;
use serde_json::Value;
use std::collections::HashMap;

/// Trait for objects that can look up metadata values by semantic name.
pub trait PropertyBySemantic {
    /// Returns the property value matching the given semantic, or `None`.
    fn get_property_by_semantic(&self, semantic: &str) -> Option<Value>;
}

/// A parsed bounding volume from 3D Tiles 1.1 metadata.
#[derive(Debug, Clone, PartialEq)]
pub enum BoundingVolumeSemanticValue {
    Box(Vec<f64>),
    Region(Vec<f64>),
    Sphere(Vec<f64>),
}

/// The result of parsing all bounding volume semantics from metadata.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct BoundingVolumeSemanticsResult {
    pub bounding_volume: Option<BoundingVolumeSemanticValue>,
    pub minimum_height: Option<f64>,
    pub maximum_height: Option<f64>,
}

/// Metadata property definition in a [`MetadataClass`].
#[derive(Debug, Clone, Default)]
pub struct MetadataClassProperty {
    pub semantic: Option<String>,
    pub property_type: Option<String>,
    pub component_type: Option<String>,
    pub array: Option<bool>,
    pub count: Option<usize>,
    pub default: Option<Value>,
}

/// A metadata class describing property schemas and semantics.
#[derive(Debug, Clone, Default)]
pub struct MetadataClass {
    pub id: String,
    pub properties: HashMap<String, MetadataClassProperty>,
}

impl MetadataClass {
    /// Constructs a `MetadataClass` from JSON definition.
    pub fn from_json(json: &Value) -> Self {
        let id = json.get("id").and_then(Value::as_str).unwrap_or("").to_string();
        let mut properties = HashMap::new();

        if let Some(props_obj) = json
            .get("class")
            .and_then(|c| c.get("properties"))
            .and_then(Value::as_object)
        {
            for (prop_id, prop_val) in props_obj {
                let semantic = prop_val.get("semantic").and_then(Value::as_str).map(String::from);
                let property_type = prop_val.get("type").and_then(Value::as_str).map(String::from);
                let component_type = prop_val.get("componentType").and_then(Value::as_str).map(String::from);
                let array = prop_val.get("array").and_then(Value::as_bool);
                let count = prop_val.get("count").and_then(Value::as_u64).map(|n| n as usize);
                let default = prop_val.get("default").cloned();

                properties.insert(
                    prop_id.clone(),
                    MetadataClassProperty {
                        semantic,
                        property_type,
                        component_type,
                        array,
                        count,
                        default,
                    },
                );
            }
        }

        Self { id, properties }
    }
}

/// Represents tile metadata containing properties and optional class schema.
#[derive(Debug, Clone, Default)]
pub struct TileMetadata {
    pub class: Option<MetadataClass>,
    pub properties: HashMap<String, Value>,
}

impl TileMetadata {
    /// Creates a new `TileMetadata` from class and properties.
    pub fn new(class: Option<MetadataClass>, properties: HashMap<String, Value>) -> Self {
        Self { class, properties }
    }
}

impl PropertyBySemantic for TileMetadata {
    fn get_property_by_semantic(&self, semantic: &str) -> Option<Value> {
        let class = self.class.as_ref()?;
        for (prop_id, prop_def) in &class.properties {
            if prop_def.semantic.as_deref() == Some(semantic) {
                if let Some(val) = self.properties.get(prop_id) {
                    return Some(val.clone());
                }
            }
        }
        None
    }
}

/// Utilities for parsing bounding volume semantics from 3D Tiles 1.1 metadata.
pub struct BoundingVolumeSemantics;

impl BoundingVolumeSemantics {
    fn check_prefix(prefix: Option<&str>) -> Result<&str, DeveloperError> {
        let p = prefix.ok_or_else(|| DeveloperError::new("prefix is required"))?;
        if p != "TILE" && p != "CONTENT" {
            return Err(DeveloperError::new(
                "prefix must be either 'TILE' or 'CONTENT'",
            ));
        }
        Ok(p)
    }

    /// Parse all bounding volume-related semantics (`{prefix}_BOUNDING_BOX`,
    /// `{prefix}_BOUNDING_REGION`, `{prefix}_BOUNDING_SPHERE`, `{prefix}_MINIMUM_HEIGHT`,
    /// `{prefix}_MAXIMUM_HEIGHT`).
    pub fn parse_all_bounding_volume_semantics(
        prefix: Option<&str>,
        metadata: Option<&dyn PropertyBySemantic>,
    ) -> Result<BoundingVolumeSemanticsResult, DeveloperError> {
        let p = Self::check_prefix(prefix)?;
        let meta = metadata.ok_or_else(|| DeveloperError::new("metadata is required"))?;

        let bounding_volume = Self::parse_bounding_volume_semantic_internal(p, meta);
        let minimum_height = Self::parse_minimum_height_internal(p, meta);
        let maximum_height = Self::parse_maximum_height_internal(p, meta);

        Ok(BoundingVolumeSemanticsResult {
            bounding_volume,
            minimum_height,
            maximum_height,
        })
    }

    /// Parse bounding volume from tile or content metadata with precedence:
    /// Box, then Region, then Sphere.
    pub fn parse_bounding_volume_semantic(
        prefix: Option<&str>,
        metadata: Option<&dyn PropertyBySemantic>,
    ) -> Result<Option<BoundingVolumeSemanticValue>, DeveloperError> {
        let p = Self::check_prefix(prefix)?;
        let meta = metadata.ok_or_else(|| DeveloperError::new("metadata is required"))?;
        Ok(Self::parse_bounding_volume_semantic_internal(p, meta))
    }

    fn parse_bounding_volume_semantic_internal(
        prefix: &str,
        metadata: &dyn PropertyBySemantic,
    ) -> Option<BoundingVolumeSemanticValue> {
        // 1. Box
        let box_semantic = format!("{prefix}_BOUNDING_BOX");
        if let Some(val) = metadata.get_property_by_semantic(&box_semantic) {
            if let Some(arr) = Self::to_f64_vec(&val) {
                return Some(BoundingVolumeSemanticValue::Box(arr));
            }
        }

        // 2. Region
        let region_semantic = format!("{prefix}_BOUNDING_REGION");
        if let Some(val) = metadata.get_property_by_semantic(&region_semantic) {
            if let Some(arr) = Self::to_f64_vec(&val) {
                return Some(BoundingVolumeSemanticValue::Region(arr));
            }
        }

        // 3. Sphere
        let sphere_semantic = format!("{prefix}_BOUNDING_SPHERE");
        if let Some(val) = metadata.get_property_by_semantic(&sphere_semantic) {
            if let Some(arr) = Self::to_f64_vec(&val) {
                return Some(BoundingVolumeSemanticValue::Sphere(arr));
            }
        }

        None
    }

    /// Parse minimum height from tile or content metadata.
    pub fn parse_minimum_height(
        prefix: Option<&str>,
        metadata: Option<&dyn PropertyBySemantic>,
    ) -> Result<Option<f64>, DeveloperError> {
        let p = Self::check_prefix(prefix)?;
        let meta = metadata.ok_or_else(|| DeveloperError::new("metadata is required"))?;
        Ok(Self::parse_minimum_height_internal(p, meta))
    }

    fn parse_minimum_height_internal(
        prefix: &str,
        metadata: &dyn PropertyBySemantic,
    ) -> Option<f64> {
        let semantic = format!("{prefix}_MINIMUM_HEIGHT");
        metadata.get_property_by_semantic(&semantic)?.as_f64()
    }

    /// Parse maximum height from tile or content metadata.
    pub fn parse_maximum_height(
        prefix: Option<&str>,
        metadata: Option<&dyn PropertyBySemantic>,
    ) -> Result<Option<f64>, DeveloperError> {
        let p = Self::check_prefix(prefix)?;
        let meta = metadata.ok_or_else(|| DeveloperError::new("metadata is required"))?;
        Ok(Self::parse_maximum_height_internal(p, meta))
    }

    fn parse_maximum_height_internal(
        prefix: &str,
        metadata: &dyn PropertyBySemantic,
    ) -> Option<f64> {
        let semantic = format!("{prefix}_MAXIMUM_HEIGHT");
        metadata.get_property_by_semantic(&semantic)?.as_f64()
    }

    fn to_f64_vec(value: &Value) -> Option<Vec<f64>> {
        if let Some(arr) = value.as_array() {
            let vec: Option<Vec<f64>> = arr.iter().map(Value::as_f64).collect();
            vec
        } else {
            None
        }
    }
}
