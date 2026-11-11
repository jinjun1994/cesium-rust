use std::collections::HashMap;
use std::rc::Rc;

use cesium_renderer::metadata_schema::MetadataSchema;
use cesium_renderer::parse_structural_metadata::{
    parse_structural_metadata, ParseStructuralMetadataOptions,
};
use cesium_renderer::texture::{Texture, TextureOptions};
use serde_json::json;

#[test]
fn throws_without_extension() {
    let schema_json = json!({
        "classes": {
            "building": {
                "properties": {
                    "name": { "type": "STRING" }
                }
            }
        }
    });
    let schema = MetadataSchema::from_json(Some(&schema_json)).unwrap();
    let result = parse_structural_metadata(ParseStructuralMetadataOptions {
        extension: None,
        schema: Some(schema),
        ..Default::default()
    });
    assert!(result.is_err());
}

#[test]
fn throws_without_schema() {
    let result = parse_structural_metadata(ParseStructuralMetadataOptions {
        extension: Some(json!({})),
        schema: None,
        ..Default::default()
    });
    assert!(result.is_err());
}

#[test]
fn parses_extension_with_default_values() {
    let schema_json = json!({
        "classes": {
            "building": {
                "properties": {
                    "name": { "type": "STRING" },
                    "height": { "type": "SCALAR", "componentType": "FLOAT64" }
                }
            }
        }
    });
    let schema = MetadataSchema::from_json(Some(&schema_json)).unwrap();
    let metadata = parse_structural_metadata(ParseStructuralMetadataOptions {
        extension: Some(json!({})),
        schema: Some(schema),
        ..Default::default()
    })
    .unwrap();

    assert!(metadata.statistics().is_none());
    assert!(metadata.extras().is_none());
    assert!(metadata.extensions().is_none());
    assert_eq!(metadata.property_table_count(), 0);
    assert_eq!(metadata.property_textures().len(), 0);
    assert_eq!(metadata.property_attributes().len(), 0);
}

#[test]
fn parses_extension_with_property_tables() {
    let schema_json = json!({
        "classes": {
            "building": {
                "properties": {
                    "name": { "type": "STRING" },
                    "height": { "type": "SCALAR", "componentType": "FLOAT64" }
                }
            },
            "tree": {
                "properties": {
                    "species": { "type": "STRING" }
                }
            }
        }
    });
    let schema = MetadataSchema::from_json(Some(&schema_json)).unwrap();

    // Setup binary buffer views for building (height: f64, name: string offsets + data)
    // Building heights: [10.0, 20.0, 30.0] -> 24 bytes
    let mut heights_data = Vec::new();
    for h in [10.0f64, 20.0f64, 30.0f64] {
        heights_data.extend_from_slice(&h.to_le_bytes());
    }

    // Building names: "Building A", "Building B", "Building C"
    // stringOffsets: uint32 offsets [0, 10, 20, 30]
    let name_str = "Building ABuilding BBuilding C";
    let mut offsets_data = Vec::new();
    for off in [0u32, 10u32, 20u32, 30u32] {
        offsets_data.extend_from_slice(&off.to_le_bytes());
    }

    // Tree species: "Oak", "Pine"
    // speciesOffsets: [0, 3, 7]
    let tree_str = "OakPine";
    let mut tree_offsets = Vec::new();
    for off in [0u32, 3u32, 7u32] {
        tree_offsets.extend_from_slice(&off.to_le_bytes());
    }

    let mut buffer_views = HashMap::new();
    buffer_views.insert("0".to_string(), heights_data);
    buffer_views.insert("1".to_string(), name_str.as_bytes().to_vec());
    buffer_views.insert("2".to_string(), offsets_data);
    buffer_views.insert("3".to_string(), tree_str.as_bytes().to_vec());
    buffer_views.insert("4".to_string(), tree_offsets);

    let extension_json = json!({
        "propertyTables": [
            {
                "name": "Buildings",
                "class": "building",
                "count": 3,
                "properties": {
                    "height": {
                        "values": 0
                    },
                    "name": {
                        "values": 1,
                        "stringOffsets": 2
                    }
                }
            },
            {
                "name": "Trees",
                "class": "tree",
                "count": 2,
                "properties": {
                    "species": {
                        "values": 3,
                        "stringOffsets": 4
                    }
                }
            }
        ]
    });

    let metadata = parse_structural_metadata(ParseStructuralMetadataOptions {
        extension: Some(extension_json),
        schema: Some(schema),
        buffer_views: Some(buffer_views),
        ..Default::default()
    })
    .unwrap();

    let building_class = metadata.schema().classes.get("building").unwrap();
    let tree_class = metadata.schema().classes.get("tree").unwrap();
    assert_eq!(building_class.id, "building");
    assert_eq!(tree_class.id, "tree");

    let buildings_table = metadata.get_property_table(0).unwrap();
    assert_eq!(buildings_table.count(), 3);
    assert_eq!(buildings_table.id(), Some(&json!(0)));
    assert_eq!(buildings_table.name(), Some("Buildings"));

    assert_eq!(
        buildings_table.get_property(0, "name"),
        Some(json!("Building A"))
    );
    assert_eq!(
        buildings_table.get_property(1, "name"),
        Some(json!("Building B"))
    );
    assert_eq!(
        buildings_table.get_property(2, "name"),
        Some(json!("Building C"))
    );

    assert_eq!(
        buildings_table.get_property(0, "height"),
        Some(json!(10.0))
    );
    assert_eq!(
        buildings_table.get_property(1, "height"),
        Some(json!(20.0))
    );
    assert_eq!(
        buildings_table.get_property(2, "height"),
        Some(json!(30.0))
    );

    let trees_table = metadata.get_property_table(1).unwrap();
    assert_eq!(trees_table.count(), 2);
    assert_eq!(trees_table.id(), Some(&json!(1)));
    assert_eq!(trees_table.name(), Some("Trees"));
    assert_eq!(trees_table.get_property(0, "species"), Some(json!("Oak")));
    assert_eq!(trees_table.get_property(1, "species"), Some(json!("Pine")));
}

#[test]
fn parses_extension_with_property_textures() {
    let schema_json = json!({
        "classes": {
            "map": {
                "properties": {
                    "color": {
                        "type": "SCALAR",
                        "componentType": "UINT8",
                        "array": true,
                        "count": 3
                    },
                    "intensity": {
                        "type": "SCALAR",
                        "componentType": "UINT8"
                    }
                }
            },
            "ortho": {
                "properties": {
                    "vegetation": {
                        "type": "SCALAR",
                        "componentType": "UINT8",
                        "normalized": true
                    }
                }
            }
        }
    });
    let schema = MetadataSchema::from_json(Some(&schema_json)).unwrap();

    let mut textures = HashMap::new();
    let tex0 = Rc::new(Texture::new(TextureOptions::default()));
    let tex1 = Rc::new(Texture::new(TextureOptions::default()));
    textures.insert(0, tex0);
    textures.insert(1, tex1);

    let extension_json = json!({
        "propertyTextures": [
            {
                "name": "Map",
                "class": "map",
                "properties": {
                    "color": {
                        "index": 0,
                        "texCoord": 0,
                        "channels": [0, 1, 2]
                    },
                    "intensity": {
                        "index": 0,
                        "texCoord": 0,
                        "channels": [3]
                    }
                }
            },
            {
                "name": "Ortho",
                "class": "ortho",
                "properties": {
                    "vegetation": {
                        "index": 1,
                        "texCoord": 1,
                        "channels": [0]
                    }
                }
            }
        ]
    });

    let metadata = parse_structural_metadata(ParseStructuralMetadataOptions {
        extension: Some(extension_json),
        schema: Some(schema),
        textures: Some(&textures),
        ..Default::default()
    })
    .unwrap();

    let map_class = metadata.schema().classes.get("map").unwrap();
    let ortho_class = metadata.schema().classes.get("ortho").unwrap();
    assert_eq!(map_class.id, "map");
    assert_eq!(ortho_class.id, "ortho");

    let map_texture = metadata.get_property_texture(0).unwrap();
    let ortho_texture = metadata.get_property_texture(1).unwrap();

    assert_eq!(map_texture.id.as_ref(), Some(&json!(0)));
    assert_eq!(map_texture.name.as_deref(), Some("Map"));
    assert_eq!(ortho_texture.id.as_ref(), Some(&json!(1)));
    assert_eq!(ortho_texture.name.as_deref(), Some("Ortho"));
}

#[test]
fn parses_extension_with_property_attributes() {
    let schema_json = json!({
        "classes": {
            "points": {
                "properties": {
                    "color": {
                        "type": "VEC3",
                        "componentType": "UINT8",
                        "array": true,
                        "count": 3
                    },
                    "intensity": {
                        "type": "SCALAR",
                        "componentType": "UINT8"
                    },
                    "pointSize": {
                        "type": "SCALAR",
                        "componentType": "FLOAT32"
                    }
                }
            }
        }
    });
    let schema = MetadataSchema::from_json(Some(&schema_json)).unwrap();

    let extension_json = json!({
        "propertyAttributes": [
            {
                "class": "points",
                "name": "Points",
                "properties": {
                    "color": {
                        "attribute": "_COLOR"
                    },
                    "intensity": {
                        "attribute": "_INTENSITY"
                    },
                    "pointSize": {
                        "attribute": "_POINT_SIZE"
                    }
                }
            }
        ]
    });

    let metadata = parse_structural_metadata(ParseStructuralMetadataOptions {
        extension: Some(extension_json),
        schema: Some(schema),
        ..Default::default()
    })
    .unwrap();

    let points_class = metadata.schema().classes.get("points").unwrap();
    assert_eq!(points_class.id, "points");

    let prop_attr = metadata.get_property_attribute(0).unwrap();
    assert_eq!(prop_attr.id.as_ref(), Some(&json!(0)));
    assert_eq!(prop_attr.name.as_deref(), Some("Points"));
    assert_eq!(
        prop_attr
            .get_property(Some("color"))
            .unwrap()
            .unwrap()
            .attribute,
        "_COLOR"
    );
    assert_eq!(
        prop_attr
            .get_property(Some("intensity"))
            .unwrap()
            .unwrap()
            .attribute,
        "_INTENSITY"
    );
    assert_eq!(
        prop_attr
            .get_property(Some("pointSize"))
            .unwrap()
            .unwrap()
            .attribute,
        "_POINT_SIZE"
    );
}

#[test]
fn parses_extension_with_statistics_extras_and_extensions() {
    let schema_json = json!({
        "classes": {
            "tree": {
                "properties": {
                    "height": { "type": "SCALAR", "componentType": "FLOAT32" }
                }
            }
        }
    });
    let schema = MetadataSchema::from_json(Some(&schema_json)).unwrap();

    let statistics = json!({
        "classes": {
            "tree": {
                "count": 100,
                "properties": {
                    "height": {
                        "min": 10.0,
                        "max": 20.0
                    }
                }
            }
        }
    });
    let extras = json!({ "description": "Extra metadata" });
    let extensions = json!({ "EXT_other_extension": {} });

    let extension_json = json!({
        "statistics": statistics.clone(),
        "extras": extras.clone(),
        "extensions": extensions.clone()
    });

    let metadata = parse_structural_metadata(ParseStructuralMetadataOptions {
        extension: Some(extension_json),
        schema: Some(schema),
        ..Default::default()
    })
    .unwrap();

    assert_eq!(metadata.statistics(), Some(&statistics));
    assert_eq!(metadata.extras(), Some(&extras));
    assert_eq!(metadata.extensions(), Some(&extensions));
}
