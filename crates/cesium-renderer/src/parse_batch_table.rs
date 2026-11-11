//! Parses 3D Tiles 1.0 batch tables and transcodes them into EXT_structural_metadata.
//! Adapted from CesiumJS `Scene/parseBatchTable.js` (Apache-2.0).

use std::collections::HashMap;

use cesium_math::DeveloperError;
use serde_json::{json, Value};

use crate::batch_table_hierarchy::BatchTableHierarchy;
use crate::json_metadata_table::JsonMetadataTable;
use crate::metadata_class::MetadataClass;
use crate::metadata_class_property::MetadataClassProperty;
use crate::metadata_schema::MetadataSchema;
use crate::metadata_table::{MetadataTable, MetadataTableOptions};
use crate::property_table::{PropertyTable, PropertyTableOptions};
use crate::structural_metadata::{StructuralMetadata, StructuralMetadataOptions};

/// Options for `parse_batch_table`.
#[derive(Clone, Debug, Default)]
pub struct ParseBatchTableOptions<'a> {
    pub count: Option<usize>,
    pub batch_table: Option<Value>,
    pub binary_body: Option<&'a [u8]>,
    pub parse_as_property_attributes: Option<bool>,
}

/// Transcodes a batch table component type string to an EXT_structural_metadata component type.
fn transcode_component_type(component_type: &str) -> &'static str {
    match component_type {
        "BYTE" => "INT8",
        "UNSIGNED_BYTE" => "UINT8",
        "SHORT" => "INT16",
        "UNSIGNED_SHORT" => "UINT16",
        "INT" => "INT32",
        "UNSIGNED_INT" => "UINT32",
        "FLOAT" => "FLOAT32",
        "DOUBLE" => "FLOAT64",
        _ => "FLOAT32",
    }
}

/// Parses a 3D Tiles 1.0 batch table and transcodes it to be compatible
/// with structural metadata from the `EXT_structural_metadata` glTF extension.
pub fn parse_batch_table(
    options: ParseBatchTableOptions,
) -> Result<StructuralMetadata, DeveloperError> {
    let count = options
        .count
        .ok_or_else(|| DeveloperError::new("options.count is required."))?;
    let batch_table = options
        .batch_table
        .ok_or_else(|| DeveloperError::new("options.batchTable is required."))?;

    let batch_table_obj = batch_table
        .as_object()
        .ok_or_else(|| DeveloperError::new("options.batchTable must be an object."))?;

    let binary_body = options.binary_body;

    // 1. Partition properties into binary, json, and hierarchy
    let extras = batch_table_obj.get("extras").cloned();
    let extensions = batch_table_obj.get("extensions").cloned();

    let hierarchy_extension = batch_table_obj
        .get("HIERARCHY")
        .cloned()
        .or_else(|| extensions.as_ref().and_then(|ext| ext.get("3DTILES_batch_table_hierarchy").cloned()));

    let mut json_properties = HashMap::new();
    let mut binary_properties = HashMap::new();

    for (prop_id, val) in batch_table_obj {
        if prop_id == "HIERARCHY" || prop_id == "extensions" || prop_id == "extras" {
            continue;
        }

        if let Some(arr) = val.as_array() {
            json_properties.insert(prop_id.clone(), arr.clone());
        } else if val.is_object() {
            binary_properties.insert(prop_id.clone(), val.clone());
        }
    }

    // 2. Build JsonMetadataTable if needed
    let json_metadata_table = if !json_properties.is_empty() && count > 0 {
        Some(JsonMetadataTable::new(count, json_properties)?)
    } else {
        None
    };

    // 3. Build BatchTableHierarchy if present
    let hierarchy = if let Some(ref h_json) = hierarchy_extension {
        Some(
            BatchTableHierarchy::new(Some(h_json), binary_body)
                .map_err(|e| DeveloperError::new(format!("BatchTableHierarchy error: {e:?}")))?,
        )
    } else {
        None
    };

    // 4. Transcode binary properties
    let class_name = MetadataClass::BATCH_TABLE_CLASS_NAME;
    let mut class_properties = HashMap::new();
    let mut table_properties = HashMap::new();
    let mut buffer_views = HashMap::new();

    for (buffer_view_count, (prop_id, prop_def)) in binary_properties.into_iter().enumerate() {
        let Some(body) = binary_body else {
            return Err(DeveloperError::new(format!(
                "Property {prop_id} requires a batch table binary."
            )));
        };

        let byte_offset = prop_def
            .get("byteOffset")
            .and_then(Value::as_u64)
            .unwrap_or(0) as usize;

        let comp_type_str = prop_def
            .get("componentType")
            .and_then(Value::as_str)
            .unwrap_or("FLOAT");
        let transcoded_comp = transcode_component_type(comp_type_str);

        let type_str = prop_def
            .get("type")
            .and_then(Value::as_str)
            .unwrap_or("SCALAR");

        let comp_size = match transcoded_comp {
            "INT8" | "UINT8" => 1,
            "INT16" | "UINT16" => 2,
            "INT32" | "UINT32" | "FLOAT32" => 4,
            "FLOAT64" => 8,
            _ => 4,
        };

        let elements_per_val = match type_str {
            "VEC2" => 2,
            "VEC3" => 3,
            "VEC4" | "MAT2" => 4,
            "MAT3" => 9,
            "MAT4" => 16,
            _ => 1,
        };

        let total_bytes = count * elements_per_val * comp_size;
        if byte_offset + total_bytes > body.len() {
            return Err(DeveloperError::new(format!(
                "Property {prop_id} binary range exceeds binary body length."
            )));
        }

        let slice = body[byte_offset..byte_offset + total_bytes].to_vec();
        let bv_id = buffer_view_count.to_string();
        buffer_views.insert(bv_id, slice);

        table_properties.insert(
            prop_id.clone(),
            json!({
                "values": buffer_view_count
            }),
        );

        let class_prop_json = json!({
            "type": type_str,
            "componentType": transcoded_comp
        });
        let class_property = MetadataClassProperty::from_json(prop_id.clone(), &class_prop_json, None)?;
        class_properties.insert(prop_id, class_property);
    }

    let class_def = MetadataClass {
        id: class_name.to_string(),
        name: Some("Batch Table".to_string()),
        description: None,
        properties: class_properties.clone(),
        properties_by_semantic: HashMap::new(),
        extras: None,
        extensions: None,
    };

    let mut schema = MetadataSchema::default();
    schema.classes.insert(class_name.to_string(), class_def.clone());

    let metadata_table = if !class_properties.is_empty() && count > 0 {
        Some(MetadataTable::new(MetadataTableOptions {
            count,
            class_definition: Some(class_def),
            properties: Some(table_properties),
            buffer_views: Some(buffer_views),
        })?)
    } else {
        None
    };

    let mut property_tables = Vec::new();
    if metadata_table.is_some() || json_metadata_table.is_some() || hierarchy.is_some() {
        let property_table = PropertyTable::new(PropertyTableOptions {
            name: Some("Batch Table".to_string()),
            id: Some(json!(0)),
            count,
            metadata_table,
            json_metadata_table,
            batch_table_hierarchy: hierarchy,
            texture: None,
            extras: None,
            extensions: None,
        });
        property_tables.push(property_table);
    }

    StructuralMetadata::new(StructuralMetadataOptions {
        schema,
        property_tables: Some(property_tables),
        property_textures: None,
        property_attributes: None,
        statistics: None,
        extras,
        extensions,
    })
}
