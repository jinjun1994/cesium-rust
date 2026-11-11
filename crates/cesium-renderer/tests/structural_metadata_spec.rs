use std::collections::HashMap;

use cesium_renderer::metadata_schema::MetadataSchema;
use cesium_renderer::property_attribute::PropertyAttribute;
use cesium_renderer::property_table::{PropertyTable, PropertyTableOptions};
use cesium_renderer::property_texture::PropertyTexture;
use cesium_renderer::structural_metadata::{StructuralMetadata, StructuralMetadataOptions};
use serde_json::json;

fn get_property_tables_schema() -> MetadataSchema {
    let schema_json = json!({
        "id": "PropertyTablesSchema",
        "classes": {
            "building": {
                "properties": {
                    "name": {
                        "type": "STRING"
                    },
                    "height": {
                        "type": "SCALAR",
                        "componentType": "FLOAT64"
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
    MetadataSchema::from_json(Some(&schema_json)).unwrap()
}

fn get_property_textures_schema() -> MetadataSchema {
    let schema_json = json!({
        "id": "PropertyTexturesSchema",
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
    MetadataSchema::from_json(Some(&schema_json)).unwrap()
}

fn get_property_attributes_schema() -> MetadataSchema {
    let schema_json = json!({
        "id": "PropertyAttributesSchema",
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
    MetadataSchema::from_json(Some(&schema_json)).unwrap()
}

#[test]
fn test_creates_structural_metadata_with_default_values() {
    let schema = get_property_tables_schema();
    let metadata = StructuralMetadata::new(StructuralMetadataOptions {
        schema,
        property_tables: None,
        property_textures: None,
        property_attributes: None,
        statistics: None,
        extras: None,
        extensions: None,
    })
    .unwrap();

    assert_eq!(metadata.property_table_count(), 0);
    assert!(metadata.statistics().is_none());
    assert!(metadata.extras().is_none());
    assert!(metadata.extensions().is_none());
    assert_eq!(metadata.property_tables_byte_length(), 0);
}

#[test]
fn test_creates_structural_metadata() {
    let schema = get_property_tables_schema();

    let table0 = PropertyTable::new(PropertyTableOptions {
        name: Some("Buildings".to_string()),
        id: Some(json!(0)),
        count: 2,
        ..Default::default()
    });
    let table1 = PropertyTable::new(PropertyTableOptions {
        name: Some("Trees".to_string()),
        id: Some(json!(1)),
        count: 5,
        ..Default::default()
    });

    let mock_property_tables = vec![table0, table1];

    let metadata = StructuralMetadata::new(StructuralMetadataOptions {
        schema,
        property_tables: Some(mock_property_tables),
        property_textures: None,
        property_attributes: None,
        statistics: None,
        extras: None,
        extensions: None,
    })
    .unwrap();

    assert_eq!(metadata.property_table_count(), 2);

    let building_class = metadata.schema().classes.get("building").unwrap();
    let tree_class = metadata.schema().classes.get("tree").unwrap();

    assert_eq!(building_class.id, "building");
    assert_eq!(tree_class.id, "tree");

    let buildings_table = metadata.get_property_table(0).unwrap();
    let trees_table = metadata.get_property_table(1).unwrap();

    assert_eq!(buildings_table.name(), Some("Buildings"));
    assert_eq!(trees_table.name(), Some("Trees"));
    assert_eq!(buildings_table.count(), 2);
    assert_eq!(trees_table.count(), 5);
}

#[test]
fn test_creates_structural_metadata_with_property_textures() {
    let schema = get_property_textures_schema();
    let map_class = schema.classes.get("map").unwrap();
    let ortho_class = schema.classes.get("ortho").unwrap();

    let texture_json0 = json!({
        "class": "map",
        "properties": {
            "color": { "channels": [0, 1, 2], "index": 0 },
            "intensity": { "channels": [3], "index": 0 }
        }
    });
    let texture_json1 = json!({
        "class": "ortho",
        "properties": {
            "vegetation": { "channels": [0], "index": 1 }
        }
    });

    let empty_textures = HashMap::new();
    let tex0 = PropertyTexture::new(
        Some(&texture_json0),
        Some(map_class),
        Some(&empty_textures),
        Some("Map Texture".to_string()),
        Some(json!(0)),
    )
    .unwrap();

    let tex1 = PropertyTexture::new(
        Some(&texture_json1),
        Some(ortho_class),
        Some(&empty_textures),
        Some("Ortho Texture".to_string()),
        Some(json!(1)),
    )
    .unwrap();

    let metadata = StructuralMetadata::new(StructuralMetadataOptions {
        schema,
        property_tables: None,
        property_textures: Some(vec![tex0, tex1]),
        property_attributes: None,
        statistics: None,
        extras: None,
        extensions: None,
    })
    .unwrap();

    assert_eq!(metadata.property_textures().len(), 2);

    let map_texture = metadata.get_property_texture(0).unwrap();
    let ortho_texture = metadata.get_property_texture(1).unwrap();

    assert_eq!(map_texture.id, Some(json!(0)));
    assert_eq!(map_texture.name.as_deref(), Some("Map Texture"));
    assert_eq!(map_texture.class.id, "map");

    assert_eq!(ortho_texture.id, Some(json!(1)));
    assert_eq!(ortho_texture.name.as_deref(), Some("Ortho Texture"));
    assert_eq!(ortho_texture.class.id, "ortho");

    assert_eq!(metadata.property_tables_byte_length(), 0);
}

#[test]
fn test_creates_structural_metadata_with_property_attributes() {
    let schema = get_property_attributes_schema();
    let points_class = schema.classes.get("points").unwrap();

    let prop_attr_json = json!({
        "id": 0,
        "name": "Points",
        "class": "points",
        "properties": {
            "color": { "attribute": "_COLOR" },
            "intensity": { "attribute": "_INTENSITY" },
            "pointSize": { "attribute": "_POINT_SIZE" }
        }
    });

    let property_attribute =
        PropertyAttribute::new(Some(points_class), Some(&prop_attr_json)).unwrap();

    let metadata = StructuralMetadata::new(StructuralMetadataOptions {
        schema,
        property_tables: None,
        property_textures: None,
        property_attributes: Some(vec![property_attribute]),
        statistics: None,
        extras: None,
        extensions: None,
    })
    .unwrap();

    let attr = metadata.get_property_attribute(0).unwrap();
    assert_eq!(attr.id, Some(json!(0)));
    assert_eq!(attr.name.as_deref(), Some("Points"));
    assert_eq!(attr.class.id, "points");

    assert_eq!(
        attr.get_property(Some("color")).unwrap().unwrap().attribute,
        "_COLOR"
    );
    assert_eq!(
        attr.get_property(Some("intensity")).unwrap().unwrap().attribute,
        "_INTENSITY"
    );
    assert_eq!(
        attr.get_property(Some("pointSize")).unwrap().unwrap().attribute,
        "_POINT_SIZE"
    );

    assert_eq!(metadata.property_tables_byte_length(), 0);
}

#[test]
fn test_creates_structural_metadata_with_extras_extensions_statistics() {
    let schema = get_property_tables_schema();
    let extras = json!({ "date": "2021-04-14" });
    let extensions = json!({ "3DTILES_extension": {} });
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

    let mut metadata = StructuralMetadata::new(StructuralMetadataOptions {
        schema,
        property_tables: None,
        property_textures: None,
        property_attributes: None,
        statistics: Some(statistics.clone()),
        extras: Some(extras.clone()),
        extensions: Some(extensions.clone()),
    })
    .unwrap();

    assert_eq!(metadata.extras(), Some(&extras));
    assert_eq!(metadata.extensions(), Some(&extensions));
    assert_eq!(metadata.statistics(), Some(&statistics));

    metadata.destroy();
    assert!(metadata.is_destroyed);
}

#[test]
fn test_get_property_out_of_bounds_errors() {
    let schema = get_property_tables_schema();
    let metadata = StructuralMetadata::new(StructuralMetadataOptions {
        schema,
        property_tables: None,
        property_textures: None,
        property_attributes: None,
        statistics: None,
        extras: None,
        extensions: None,
    })
    .unwrap();

    assert!(metadata.get_property_table(0).is_err());
    assert!(metadata.get_property_texture(0).is_err());
    assert!(metadata.get_property_attribute(0).is_err());
}
