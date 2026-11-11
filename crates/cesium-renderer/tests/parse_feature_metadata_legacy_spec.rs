//! Spec matching CesiumJS `Specs/Scene/parseFeatureMetadataLegacySpec.js` (Apache-2.0).

use std::collections::HashMap;

use cesium_renderer::{
    parse_feature_metadata_legacy, MetadataSchema,
    ParseFeatureMetadataLegacyOptions,
};
use serde_json::json;

fn create_feature_tables_schema() -> MetadataSchema {
    let schema_json = json!({
        "classes": {
            "building": {
                "properties": {
                    "name": {
                        "type": "STRING"
                    },
                    "height": {
                        "type": "FLOAT64"
                    }
                }
            },
            "tree": {
                "properties": {
                    "species": {
                        "type": "STRING"
                    }
                }
            }
        }
    });
    MetadataSchema::from_json(Some(&schema_json)).expect("Schema creation")
}

#[test]
fn test_throws_without_extension() {
    let schema = create_feature_tables_schema();
    let res = parse_feature_metadata_legacy(ParseFeatureMetadataLegacyOptions {
        extension: None,
        schema: Some(schema),
        buffer_views: None,
        textures: None,
    });
    assert!(res.is_err());
    assert_eq!(res.unwrap_err().message, "options.extension is required.");
}

#[test]
fn test_throws_without_schema() {
    let res = parse_feature_metadata_legacy(ParseFeatureMetadataLegacyOptions {
        extension: Some(json!({})),
        schema: None,
        buffer_views: None,
        textures: None,
    });
    assert!(res.is_err());
    assert_eq!(res.unwrap_err().message, "options.schema is required.");
}

#[test]
fn test_parses_extension_with_default_values() {
    let schema = create_feature_tables_schema();
    let metadata = parse_feature_metadata_legacy(ParseFeatureMetadataLegacyOptions {
        extension: Some(json!({})),
        schema: Some(schema),
        buffer_views: None,
        textures: None,
    })
    .expect("Should parse default values");

    assert!(metadata.statistics.is_none());
    assert!(metadata.extras.is_none());
    assert!(metadata.extensions.is_none());
    assert!(metadata.property_tables.is_empty());
}

#[test]
fn test_parses_extension_with_feature_tables() {
    let schema = create_feature_tables_schema();

    // Binary string table buffer helper
    let names = ["Building A", "Building B", "Building C"];
    let mut names_bytes = Vec::new();
    let mut names_offsets = vec![0usize];
    for name in names {
        names_bytes.extend_from_slice(name.as_bytes());
        names_offsets.push(names_bytes.len());
    }

    let heights: [f64; 3] = [10.0, 20.0, 30.0];
    let mut heights_bytes = Vec::new();
    for h in heights {
        heights_bytes.extend_from_slice(&h.to_le_bytes());
    }

    let mut string_offsets_bytes = Vec::new();
    for off in names_offsets {
        string_offsets_bytes.extend_from_slice(&(off as u32).to_le_bytes());
    }

    let species = ["Oak", "Pine"];
    let mut species_bytes = Vec::new();
    let mut species_offsets = vec![0usize];
    for s in species {
        species_bytes.extend_from_slice(s.as_bytes());
        species_offsets.push(species_bytes.len());
    }

    let mut species_offsets_bytes = Vec::new();
    for off in species_offsets {
        species_offsets_bytes.extend_from_slice(&(off as u32).to_le_bytes());
    }

    let mut buffer_views = HashMap::new();
    buffer_views.insert("0".to_string(), names_bytes);
    buffer_views.insert("1".to_string(), string_offsets_bytes);
    buffer_views.insert("2".to_string(), heights_bytes);
    buffer_views.insert("3".to_string(), species_bytes);
    buffer_views.insert("4".to_string(), species_offsets_bytes);

    let extension = json!({
        "featureTables": {
            "buildings": {
                "class": "building",
                "count": 3,
                "properties": {
                    "name": {
                        "bufferView": 0,
                        "stringOffsets": 1
                    },
                    "height": {
                        "bufferView": 2
                    }
                }
            },
            "trees": {
                "class": "tree",
                "count": 2,
                "properties": {
                    "species": {
                        "bufferView": 3,
                        "stringOffsets": 4
                    }
                }
            }
        }
    });

    let metadata = parse_feature_metadata_legacy(ParseFeatureMetadataLegacyOptions {
        extension: Some(extension),
        schema: Some(schema),
        buffer_views: Some(buffer_views),
        textures: None,
    })
    .expect("Should parse feature tables");

    let building_class = metadata.schema.classes.get("building").unwrap();
    let tree_class = metadata.schema.classes.get("tree").unwrap();
    assert_eq!(building_class.id, "building");
    assert_eq!(tree_class.id, "tree");

    assert_eq!(metadata.property_tables.len(), 2);

    let buildings_table = &metadata.property_tables[0];
    assert_eq!(buildings_table.id(), Some(&json!("buildings")));
    assert_eq!(buildings_table.count(), 3);
    assert_eq!(buildings_table.get_property(0, "name"), Some(json!("Building A")));
    assert_eq!(buildings_table.get_property(1, "name"), Some(json!("Building B")));
    assert_eq!(buildings_table.get_property(2, "name"), Some(json!("Building C")));
    assert_eq!(buildings_table.get_property(0, "height"), Some(json!(10.0)));
    assert_eq!(buildings_table.get_property(1, "height"), Some(json!(20.0)));
    assert_eq!(buildings_table.get_property(2, "height"), Some(json!(30.0)));

    let trees_table = &metadata.property_tables[1];
    assert_eq!(trees_table.id(), Some(&json!("trees")));
    assert_eq!(trees_table.count(), 2);
    assert_eq!(trees_table.get_property(0, "species"), Some(json!("Oak")));
    assert_eq!(trees_table.get_property(1, "species"), Some(json!("Pine")));
}

#[test]
fn test_parses_extension_with_statistics_extras_extensions() {
    let schema = create_feature_tables_schema();
    let statistics = json!({
        "classes": {
            "tree": {
                "count": 100,
                "properties": {
                    "height": { "min": 10.0, "max": 20.0 }
                }
            }
        }
    });
    let extras = json!({ "description": "Extra" });
    let extensions = json!({ "EXT_other_extension": {} });

    let extension = json!({
        "statistics": statistics.clone(),
        "extras": extras.clone(),
        "extensions": extensions.clone()
    });

    let metadata = parse_feature_metadata_legacy(ParseFeatureMetadataLegacyOptions {
        extension: Some(extension),
        schema: Some(schema),
        buffer_views: None,
        textures: None,
    })
    .expect("Should parse statistics, extras and extensions");

    assert_eq!(metadata.statistics, Some(statistics));
    assert_eq!(metadata.extras, Some(extras));
    assert_eq!(metadata.extensions, Some(extensions));
}
