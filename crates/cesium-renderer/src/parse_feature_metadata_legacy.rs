//! Parses EXT_feature_metadata glTF extension to create a StructuralMetadata object.
//! Adapted from CesiumJS `Scene/parseFeatureMetadataLegacy.js` (Apache-2.0).

use std::collections::HashMap;
use std::rc::Rc;

use cesium_math::DeveloperError;
use serde_json::{json, Value};

use crate::metadata_schema::MetadataSchema;
use crate::metadata_table::{MetadataTable, MetadataTableOptions};
use crate::property_table::{PropertyTable, PropertyTableOptions};
use crate::property_texture::PropertyTexture;
use crate::structural_metadata::{StructuralMetadata, StructuralMetadataOptions};
use crate::texture::Texture;

/// Options for configuring `parse_feature_metadata_legacy`.
#[derive(Clone, Default)]
pub struct ParseFeatureMetadataLegacyOptions<'a> {
    pub extension: Option<Value>,
    pub schema: Option<MetadataSchema>,
    pub buffer_views: Option<HashMap<String, Vec<u8>>>,
    pub textures: Option<&'a HashMap<usize, Rc<Texture>>>,
}

fn reformat_channels(channels_str: &str) -> Vec<usize> {
    channels_str
        .chars()
        .filter_map(|c| match c {
            'r' | 'R' => Some(0),
            'g' | 'G' => Some(1),
            'b' | 'B' => Some(2),
            'a' | 'A' => Some(3),
            _ => None,
        })
        .collect()
}

fn transcode_to_property_texture(feature_texture: &Value) -> Value {
    let mut properties = json!({});

    if let Some(props_obj) = feature_texture.get("properties").and_then(Value::as_object) {
        let mut props_map = serde_json::Map::new();
        for (prop_id, old_prop) in props_obj {
            let mut prop = serde_json::Map::new();

            if let Some(channels_str) = old_prop.get("channels").and_then(Value::as_str) {
                let channels = reformat_channels(channels_str);
                prop.insert("channels".to_string(), json!(channels));
            }

            if let Some(extras) = old_prop.get("extras") {
                prop.insert("extras".to_string(), extras.clone());
            }
            if let Some(extensions) = old_prop.get("extensions") {
                prop.insert("extensions".to_string(), extensions.clone());
            }

            if let Some(tex_info) = old_prop.get("texture").and_then(Value::as_object) {
                for (k, v) in tex_info {
                    prop.insert(k.clone(), v.clone());
                }
            }

            props_map.insert(prop_id.clone(), Value::Object(prop));
        }
        properties = Value::Object(props_map);
    }

    json!({
        "class": feature_texture.get("class").unwrap_or(&Value::Null),
        "properties": properties,
        "extras": feature_texture.get("extras").unwrap_or(&Value::Null),
        "extensions": feature_texture.get("extensions").unwrap_or(&Value::Null),
    })
}

/// Parses the `EXT_feature_metadata` glTF extension to create a [`StructuralMetadata`] object.
pub fn parse_feature_metadata_legacy(
    options: ParseFeatureMetadataLegacyOptions,
) -> Result<StructuralMetadata, DeveloperError> {
    let extension = options
        .extension
        .ok_or_else(|| DeveloperError::new("options.extension is required."))?;
    let schema = options
        .schema
        .ok_or_else(|| DeveloperError::new("options.schema is required."))?;

    let mut property_tables = Vec::new();

    if let Some(ft_map) = extension.get("featureTables").and_then(Value::as_object) {
        let mut sorted_ids: Vec<&String> = ft_map.keys().collect();
        sorted_ids.sort();

        for feature_table_id in sorted_ids {
            let feature_table = &ft_map[feature_table_id];
            let class_name = feature_table
                .get("class")
                .and_then(Value::as_str)
                .unwrap_or_default();
            let count = feature_table
                .get("count")
                .and_then(Value::as_u64)
                .unwrap_or(0) as usize;

            let class_def = schema.classes.get(class_name).cloned();

            let props_map = feature_table
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

            property_tables.push(PropertyTable::new(PropertyTableOptions {
                id: Some(json!(feature_table_id)),
                count,
                metadata_table,
                extras: feature_table.get("extras").cloned(),
                extensions: feature_table.get("extensions").cloned(),
                ..Default::default()
            }));
        }
    }

    let mut property_textures = Vec::new();

    if let Some(tex_map) = extension.get("featureTextures").and_then(Value::as_object) {
        let mut sorted_ids: Vec<&String> = tex_map.keys().collect();
        sorted_ids.sort();

        for feature_texture_id in sorted_ids {
            let feature_texture = &tex_map[feature_texture_id];
            let class_name = feature_texture
                .get("class")
                .and_then(Value::as_str)
                .unwrap_or_default();
            let class_def = schema.classes.get(class_name);

            let transcoded_texture = transcode_to_property_texture(feature_texture);

            if let Some(textures_map) = options.textures {
                let pt = PropertyTexture::new(
                    Some(&transcoded_texture),
                    class_def,
                    Some(textures_map),
                    None,
                    Some(json!(feature_texture_id)),
                )?;
                property_textures.push(pt);
            }
        }
    }

    StructuralMetadata::new(StructuralMetadataOptions {
        schema,
        property_tables: Some(property_tables),
        property_textures: Some(property_textures),
        property_attributes: None,
        statistics: extension.get("statistics").cloned(),
        extras: extension.get("extras").cloned(),
        extensions: extension.get("extensions").cloned(),
    })
}
