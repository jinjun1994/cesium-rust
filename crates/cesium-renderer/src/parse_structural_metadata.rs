//! Parses EXT_structural_metadata glTF extension to create a StructuralMetadata object.
//! Adapted from CesiumJS `Scene/parseStructuralMetadata.js` (Apache-2.0).

use std::collections::HashMap;
use std::rc::Rc;

use cesium_math::DeveloperError;
use serde_json::{json, Value};

use crate::metadata_schema::MetadataSchema;
use crate::metadata_table::{MetadataTable, MetadataTableOptions};
use crate::property_attribute::PropertyAttribute;
use crate::property_table::{PropertyTable, PropertyTableOptions};
use crate::property_texture::PropertyTexture;
use crate::structural_metadata::{StructuralMetadata, StructuralMetadataOptions};
use crate::texture::Texture;

/// Options for configuring `parse_structural_metadata`.
#[derive(Clone, Default)]
pub struct ParseStructuralMetadataOptions<'a> {
    pub extension: Option<Value>,
    pub schema: Option<MetadataSchema>,
    pub buffer_views: Option<HashMap<String, Vec<u8>>>,
    pub textures: Option<&'a HashMap<usize, Rc<Texture>>>,
}

/// Parses the `EXT_structural_metadata` glTF extension to create a [`StructuralMetadata`] object.
pub fn parse_structural_metadata(
    options: ParseStructuralMetadataOptions,
) -> Result<StructuralMetadata, DeveloperError> {
    let extension = options
        .extension
        .ok_or_else(|| DeveloperError::new("options.extension is required."))?;
    let schema = options
        .schema
        .ok_or_else(|| DeveloperError::new("options.schema is required."))?;

    let mut property_tables = Vec::new();

    if let Some(pt_list) = extension.get("propertyTables").and_then(Value::as_array) {
        for (i, property_table) in pt_list.iter().enumerate() {
            let class_name = property_table
                .get("class")
                .and_then(Value::as_str)
                .unwrap_or_default();
            let count = property_table
                .get("count")
                .and_then(Value::as_u64)
                .unwrap_or(0) as usize;

            let class_def = schema.classes.get(class_name).cloned();

            let props_map = property_table
                .get("properties")
                .and_then(Value::as_object)
                .map(|p| p.iter().map(|(k, v)| (k.clone(), v.clone())).collect());

            let metadata_table = if count > 0 && class_def.is_some() {
                Some(MetadataTable::new(MetadataTableOptions {
                    count,
                    class_definition: class_def,
                    properties: props_map,
                    buffer_views: options.buffer_views.clone(),
                })?)
            } else {
                None
            };

            let name = property_table
                .get("name")
                .and_then(Value::as_str)
                .map(|s| s.to_string());

            property_tables.push(PropertyTable::new(PropertyTableOptions {
                id: Some(json!(i)),
                name,
                count,
                metadata_table,
                extras: property_table.get("extras").cloned(),
                extensions: property_table.get("extensions").cloned(),
                ..Default::default()
            }));
        }
    }

    let mut property_textures = Vec::new();

    if let Some(tex_list) = extension.get("propertyTextures").and_then(Value::as_array) {
        for (i, property_texture) in tex_list.iter().enumerate() {
            let class_name = property_texture
                .get("class")
                .and_then(Value::as_str)
                .unwrap_or_default();
            let class_def = schema.classes.get(class_name);

            let name = property_texture
                .get("name")
                .and_then(Value::as_str)
                .map(|s| s.to_string());

            if let Some(textures_map) = options.textures {
                let pt = PropertyTexture::new(
                    Some(property_texture),
                    class_def,
                    Some(textures_map),
                    name,
                    Some(json!(i)),
                )?;
                property_textures.push(pt);
            }
        }
    }

    let mut property_attributes = Vec::new();

    if let Some(attr_list) = extension.get("propertyAttributes").and_then(Value::as_array) {
        for (i, property_attribute) in attr_list.iter().enumerate() {
            let class_name = property_attribute
                .get("class")
                .and_then(Value::as_str)
                .unwrap_or_default();
            let class_def = schema.classes.get(class_name);

            let name = property_attribute
                .get("name")
                .and_then(Value::as_str)
                .map(|s| s.to_string());

            let pa = PropertyAttribute::new(class_def, Some(property_attribute))?
                .with_id(Some(json!(i)))
                .with_name(name);

            property_attributes.push(pa);
        }
    }

    StructuralMetadata::new(StructuralMetadataOptions {
        schema,
        property_tables: Some(property_tables),
        property_textures: Some(property_textures),
        property_attributes: Some(property_attributes),
        statistics: extension.get("statistics").cloned(),
        extras: extension.get("extras").cloned(),
        extensions: extension.get("extensions").cloned(),
    })
}
