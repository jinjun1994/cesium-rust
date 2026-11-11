use cesium_renderer::metadata_class::MetadataClass;
use cesium_renderer::tileset_metadata::{TilesetMetadata, TilesetMetadataOptions};
use serde_json::json;

#[test]
fn creates_tileset_metadata_with_default_values() {
    let metadata = TilesetMetadata::from_options(TilesetMetadataOptions {
        tileset: Some(json!({})),
        class_json: Some(json!({})),
        class_def: None,
    })
    .unwrap();

    assert!(metadata.extras().is_none());
}

#[test]
fn creates_tileset_metadata() {
    let city_class = MetadataClass::from_json(&json!({
        "id": "city",
        "class": {
            "properties": {
                "neighborhoods": {
                    "type": "STRING",
                    "array": true
                }
            }
        }
    }))
    .unwrap();

    let extras = json!({ "other": 0 });
    let extensions = json!({ "EXT_other_extension": {} });
    let properties = json!({ "neighborhoods": ["A", "B", "C"] });

    let tileset = json!({
        "extras": extras,
        "extensions": extensions,
        "properties": properties
    });

    let metadata = TilesetMetadata::from_options(TilesetMetadataOptions {
        tileset: Some(tileset),
        class_def: Some(city_class.clone()),
        class_json: None,
    })
    .unwrap();

    assert_eq!(metadata.class_definition(), Some(&city_class));
    assert_eq!(metadata.extras(), Some(&extras));
    assert_eq!(metadata.extensions(), Some(&extensions));
    assert_eq!(
        metadata.get_property("neighborhoods").unwrap(),
        json!(["A", "B", "C"])
    );
}

#[test]
fn constructor_throws_without_tileset() {
    let res = TilesetMetadata::from_options(TilesetMetadataOptions {
        tileset: None,
        class_json: Some(json!({})),
        class_def: None,
    });
    assert!(res.is_err());
}

#[test]
fn constructor_throws_without_class() {
    let res = TilesetMetadata::from_options(TilesetMetadataOptions {
        tileset: Some(json!({})),
        class_json: None,
        class_def: None,
    });
    assert!(res.is_err());
}

#[test]
fn has_property_tests() {
    let empty_metadata = TilesetMetadata::from_options(TilesetMetadataOptions {
        tileset: Some(json!({})),
        class_json: Some(json!({})),
        class_def: None,
    })
    .unwrap();
    assert!(!empty_metadata.has_property("height"));

    let building_class = MetadataClass::from_json(&json!({
        "id": "building",
        "class": {
            "properties": {
                "height": {
                    "type": "SCALAR",
                    "componentType": "FLOAT32"
                }
            }
        }
    }))
    .unwrap();

    let metadata = TilesetMetadata::from_options(TilesetMetadataOptions {
        tileset: Some(json!({
            "properties": { "height": 10.0 }
        })),
        class_def: Some(building_class),
        class_json: None,
    })
    .unwrap();

    assert!(!metadata.has_property("color"));
    assert!(metadata.has_property("height"));

    let default_class = MetadataClass::from_json(&json!({
        "id": "building",
        "class": {
            "properties": {
                "height": {
                    "type": "SCALAR",
                    "componentType": "FLOAT32",
                    "required": false,
                    "default": 10.0
                }
            }
        }
    }))
    .unwrap();

    let default_metadata = TilesetMetadata::from_options(TilesetMetadataOptions {
        tileset: Some(json!({})),
        class_def: Some(default_class),
        class_json: None,
    })
    .unwrap();
    assert!(default_metadata.has_property("height"));
}

#[test]
fn has_property_by_semantic_tests() {
    let empty_metadata = TilesetMetadata::from_options(TilesetMetadataOptions {
        tileset: Some(json!({})),
        class_json: Some(json!({})),
        class_def: None,
    })
    .unwrap();
    assert!(!empty_metadata.has_property_by_semantic("HEIGHT"));

    let building_class = MetadataClass::from_json(&json!({
        "id": "building",
        "class": {
            "properties": {
                "height": {
                    "type": "SCALAR",
                    "componentType": "FLOAT32"
                }
            }
        }
    }))
    .unwrap();

    let metadata = TilesetMetadata::from_options(TilesetMetadataOptions {
        tileset: Some(json!({
            "properties": { "height": 10.0 }
        })),
        class_def: Some(building_class),
        class_json: None,
    })
    .unwrap();
    assert!(!metadata.has_property_by_semantic("HEIGHT"));

    let semantic_class = MetadataClass::from_json(&json!({
        "id": "building",
        "class": {
            "properties": {
                "height": {
                    "type": "SCALAR",
                    "componentType": "FLOAT32",
                    "semantic": "HEIGHT"
                }
            }
        }
    }))
    .unwrap();

    let semantic_metadata = TilesetMetadata::from_options(TilesetMetadataOptions {
        tileset: Some(json!({
            "properties": { "height": 10.0 }
        })),
        class_def: Some(semantic_class),
        class_json: None,
    })
    .unwrap();
    assert!(semantic_metadata.has_property_by_semantic("HEIGHT"));

    let default_semantic_class = MetadataClass::from_json(&json!({
        "id": "building",
        "class": {
            "properties": {
                "height": {
                    "type": "SCALAR",
                    "componentType": "FLOAT32",
                    "semantic": "HEIGHT",
                    "required": false,
                    "default": 10.0
                }
            }
        }
    }))
    .unwrap();

    let default_semantic_metadata = TilesetMetadata::from_options(TilesetMetadataOptions {
        tileset: Some(json!({})),
        class_def: Some(default_semantic_class),
        class_json: None,
    })
    .unwrap();
    assert!(default_semantic_metadata.has_property_by_semantic("HEIGHT"));
}

#[test]
fn get_property_ids_tests() {
    let empty_metadata = TilesetMetadata::from_options(TilesetMetadataOptions {
        tileset: Some(json!({})),
        class_json: Some(json!({})),
        class_def: None,
    })
    .unwrap();
    assert_eq!(empty_metadata.get_property_ids(None).len(), 0);

    let building_class = MetadataClass::from_json(&json!({
        "id": "building",
        "class": {
            "properties": {
                "height": {
                    "type": "SCALAR",
                    "componentType": "FLOAT32"
                },
                "color": {
                    "type": "STRING"
                }
            }
        }
    }))
    .unwrap();

    let metadata = TilesetMetadata::from_options(TilesetMetadataOptions {
        tileset: Some(json!({
            "properties": {
                "height": 10.0,
                "color": "RED"
            }
        })),
        class_def: Some(building_class),
        class_json: None,
    })
    .unwrap();

    let mut ids = metadata.get_property_ids(None);
    ids.sort();
    assert_eq!(ids, vec!["color", "height"]);

    let mut results = vec!["existing".to_string()];
    metadata.get_property_ids(Some(&mut results));
    assert!(results.contains(&"color".to_string()));
    assert!(results.contains(&"height".to_string()));
    assert!(results.contains(&"existing".to_string()));
}

#[test]
fn get_and_set_property_tests() {
    let building_class = MetadataClass::from_json(&json!({
        "id": "building",
        "class": {
            "properties": {
                "position": {
                    "type": "VEC3",
                    "componentType": "FLOAT32"
                }
            }
        }
    }))
    .unwrap();

    let position = json!([0.0, 0.0, 0.0]);

    let mut metadata = TilesetMetadata::from_options(TilesetMetadataOptions {
        tileset: Some(json!({
            "properties": {
                "position": position
            }
        })),
        class_def: Some(building_class),
        class_json: None,
    })
    .unwrap();

    // getProperty returns throws if property does not exist in class
    assert!(metadata.get_property("color").is_err());

    // getProperty returns value
    assert_eq!(metadata.get_property("position").unwrap(), position);

    // setProperty returns false if property doesn't exist
    assert!(!metadata.set_property("unknown", json!(123)));

    // setProperty sets property value
    let new_pos = json!([1.0, 1.0, 1.0]);
    assert!(metadata.set_property("position", new_pos.clone()));
    assert_eq!(metadata.get_property("position").unwrap(), new_pos);
}

#[test]
fn semantic_get_and_set_tests() {
    let building_class = MetadataClass::from_json(&json!({
        "id": "building",
        "class": {
            "properties": {
                "height": {
                    "type": "SCALAR",
                    "componentType": "FLOAT32",
                    "semantic": "_HEIGHT"
                }
            }
        }
    }))
    .unwrap();

    let mut metadata = TilesetMetadata::from_options(TilesetMetadataOptions {
        tileset: Some(json!({
            "properties": {
                "height": 10.0
            }
        })),
        class_def: Some(building_class),
        class_json: None,
    })
    .unwrap();

    assert_eq!(
        metadata.get_property_by_semantic("_HEIGHT"),
        Some(json!(10.0))
    );
    assert!(metadata.get_property_by_semantic("_NON_EXISTENT").is_none());

    assert!(metadata.set_property_by_semantic("_HEIGHT", json!(20.0)));
    assert_eq!(
        metadata.get_property_by_semantic("_HEIGHT"),
        Some(json!(20.0))
    );
    assert_eq!(metadata.get_property("height").unwrap(), json!(20.0));

    assert!(!metadata.set_property_by_semantic("_NON_EXISTENT", json!(20.0)));
}
