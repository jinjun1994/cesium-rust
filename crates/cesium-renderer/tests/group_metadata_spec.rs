use cesium_renderer::bounding_volume_semantics::MetadataClass;
use cesium_renderer::group_metadata::{GroupMetadata, GroupMetadataOptions};
use serde_json::json;

fn make_building_class_with_no_properties() -> MetadataClass {
    MetadataClass::from_json(&json!({
        "id": "building",
        "class": {}
    }))
}

fn make_building_class() -> MetadataClass {
    MetadataClass::from_json(&json!({
        "id": "building",
        "class": {
            "properties": {
                "position": {
                    "type": "VEC3",
                    "componentType": "FLOAT32"
                },
                "height": {
                    "type": "SCALAR",
                    "componentType": "FLOAT32",
                    "semantic": "_HEIGHT"
                }
            }
        }
    }))
}

#[test]
fn creates_group_metadata_with_default_values() {
    let group_metadata = GroupMetadata::new(GroupMetadataOptions {
        id: Some("building".to_string()),
        group: Some(json!({})),
        class_def: Some(make_building_class_with_no_properties()),
    })
    .unwrap();

    assert_eq!(group_metadata.id(), Some("building"));
    assert!(group_metadata.extras().is_none());
    assert!(group_metadata.extensions().is_none());
}

#[test]
fn creates_group_metadata() {
    let extras = json!({ "other": 0 });
    let extensions = json!({ "EXT_other_extension": {} });
    let properties = json!({
        "position": [0.0, 0.0, 0.0],
        "height": 10.0
    });

    let group_metadata = GroupMetadata::new(GroupMetadataOptions {
        id: Some("building".to_string()),
        group: Some(json!({
            "extras": extras,
            "extensions": extensions,
            "properties": properties
        })),
        class_def: Some(make_building_class()),
    })
    .unwrap();

    assert_eq!(group_metadata.id(), Some("building"));
    assert_eq!(group_metadata.extras().unwrap()["other"], 0);
    assert!(group_metadata.extensions().unwrap().get("EXT_other_extension").is_some());
    assert_eq!(
        group_metadata.get_property(Some("position")).unwrap().unwrap(),
        json!([0.0, 0.0, 0.0])
    );
}

#[test]
fn constructor_throws_without_group() {
    let err = GroupMetadata::new(GroupMetadataOptions {
        id: Some("building".to_string()),
        group: None,
        class_def: Some(make_building_class_with_no_properties()),
    })
    .unwrap_err();

    assert!(err.message.contains("options.group is required"));
}

#[test]
fn constructor_throws_without_class() {
    let err = GroupMetadata::new(GroupMetadataOptions {
        id: Some("building".to_string()),
        group: Some(json!({})),
        class_def: None,
    })
    .unwrap_err();

    assert!(err.message.contains("options.class is required"));
}

#[test]
fn has_property_returns_false_when_no_properties() {
    let group_metadata = GroupMetadata::new(GroupMetadataOptions {
        id: Some("building".to_string()),
        group: Some(json!({})),
        class_def: Some(make_building_class_with_no_properties()),
    })
    .unwrap();

    assert!(!group_metadata.has_property(Some("height")).unwrap());
}

#[test]
fn has_property_returns_false_when_no_property_with_given_id() {
    let group_metadata = GroupMetadata::new(GroupMetadataOptions {
        id: Some("building".to_string()),
        group: Some(json!({
            "properties": {
                "height": 10.0
            }
        })),
        class_def: Some(make_building_class()),
    })
    .unwrap();

    assert!(!group_metadata.has_property(Some("color")).unwrap());
}

#[test]
fn has_property_returns_true_when_property_exists() {
    let group_metadata = GroupMetadata::new(GroupMetadataOptions {
        id: Some("building".to_string()),
        group: Some(json!({
            "properties": {
                "height": 10.0
            }
        })),
        class_def: Some(make_building_class()),
    })
    .unwrap();

    assert!(group_metadata.has_property(Some("height")).unwrap());
}

#[test]
fn has_property_returns_true_when_class_has_default_value() {
    let class_def = MetadataClass::from_json(&json!({
        "id": "building",
        "class": {
            "properties": {
                "height": {
                    "type": "SCALAR",
                    "componentType": "FLOAT32",
                    "default": 10.0
                }
            }
        }
    }));

    let group_metadata = GroupMetadata::new(GroupMetadataOptions {
        id: Some("building".to_string()),
        group: Some(json!({})),
        class_def: Some(class_def),
    })
    .unwrap();

    assert!(group_metadata.has_property(Some("height")).unwrap());
}

#[test]
fn has_property_throws_without_property_id() {
    let group_metadata = GroupMetadata::new(GroupMetadataOptions {
        id: Some("building".to_string()),
        group: Some(json!({})),
        class_def: Some(make_building_class_with_no_properties()),
    })
    .unwrap();

    assert!(group_metadata.has_property(None).is_err());
}

#[test]
fn has_property_by_semantic_returns_false_when_no_properties() {
    let group_metadata = GroupMetadata::new(GroupMetadataOptions {
        id: Some("building".to_string()),
        group: Some(json!({})),
        class_def: Some(make_building_class_with_no_properties()),
    })
    .unwrap();

    assert!(!group_metadata.has_property_by_semantic(Some("_HEIGHT")).unwrap());
}

#[test]
fn has_property_by_semantic_returns_true_when_property_exists() {
    let group_metadata = GroupMetadata::new(GroupMetadataOptions {
        id: Some("building".to_string()),
        group: Some(json!({
            "properties": {
                "height": 10.0
            }
        })),
        class_def: Some(make_building_class()),
    })
    .unwrap();

    assert!(group_metadata.has_property_by_semantic(Some("_HEIGHT")).unwrap());
}

#[test]
fn has_property_by_semantic_returns_true_when_class_has_default() {
    let class_def = MetadataClass::from_json(&json!({
        "id": "building",
        "class": {
            "properties": {
                "height": {
                    "type": "SCALAR",
                    "componentType": "FLOAT32",
                    "semantic": "_HEIGHT",
                    "default": 10.0
                }
            }
        }
    }));

    let group_metadata = GroupMetadata::new(GroupMetadataOptions {
        id: Some("building".to_string()),
        group: Some(json!({})),
        class_def: Some(class_def),
    })
    .unwrap();

    assert!(group_metadata.has_property_by_semantic(Some("_HEIGHT")).unwrap());
}

#[test]
fn has_property_by_semantic_throws_without_semantic() {
    let group_metadata = GroupMetadata::new(GroupMetadataOptions {
        id: Some("building".to_string()),
        group: Some(json!({})),
        class_def: Some(make_building_class_with_no_properties()),
    })
    .unwrap();

    assert!(group_metadata.has_property_by_semantic(None).is_err());
}

#[test]
fn get_property_ids_returns_property_ids_and_defaults() {
    let class_def = MetadataClass::from_json(&json!({
        "id": "building",
        "class": {
            "properties": {
                "height": {
                    "type": "SCALAR",
                    "default": 10.0
                },
                "position": {
                    "type": "VEC3"
                }
            }
        }
    }));

    let group_metadata = GroupMetadata::new(GroupMetadataOptions {
        id: Some("building".to_string()),
        group: Some(json!({
            "properties": {
                "position": [1.0, 2.0, 3.0]
            }
        })),
        class_def: Some(class_def),
    })
    .unwrap();

    let mut ids = Vec::new();
    group_metadata.get_property_ids(&mut ids);
    assert!(ids.contains(&"position".to_string()));
    assert!(ids.contains(&"height".to_string()));
}

#[test]
fn get_property_returns_value_and_defaults() {
    let class_def = MetadataClass::from_json(&json!({
        "id": "building",
        "class": {
            "properties": {
                "height": {
                    "type": "SCALAR",
                    "default": 10.0
                },
                "position": {
                    "type": "VEC3"
                }
            }
        }
    }));

    let group_metadata = GroupMetadata::new(GroupMetadataOptions {
        id: Some("building".to_string()),
        group: Some(json!({
            "properties": {
                "position": [1.0, 2.0, 3.0]
            }
        })),
        class_def: Some(class_def),
    })
    .unwrap();

    assert_eq!(
        group_metadata.get_property(Some("position")).unwrap().unwrap(),
        json!([1.0, 2.0, 3.0])
    );
    assert_eq!(
        group_metadata.get_property(Some("height")).unwrap().unwrap(),
        json!(10.0)
    );
    assert!(group_metadata.get_property(Some("non_existent")).unwrap().is_none());
    assert!(group_metadata.get_property(None).is_err());
}

#[test]
fn set_property_sets_value() {
    let mut group_metadata = GroupMetadata::new(GroupMetadataOptions {
        id: Some("building".to_string()),
        group: Some(json!({
            "properties": {
                "height": 10.0
            }
        })),
        class_def: Some(make_building_class()),
    })
    .unwrap();

    assert!(group_metadata
        .set_property(Some("height"), Some(json!(20.0)))
        .unwrap());
    assert_eq!(
        group_metadata.get_property(Some("height")).unwrap().unwrap(),
        json!(20.0)
    );

    // Cannot set non-existent property
    assert!(!group_metadata
        .set_property(Some("non_existent"), Some(json!(1)))
        .unwrap());
    assert!(group_metadata.set_property(None, Some(json!(1))).is_err());
    assert!(group_metadata.set_property(Some("height"), None).is_err());
}

#[test]
fn get_and_set_property_by_semantic() {
    let mut group_metadata = GroupMetadata::new(GroupMetadataOptions {
        id: Some("building".to_string()),
        group: Some(json!({
            "properties": {
                "height": 10.0
            }
        })),
        class_def: Some(make_building_class()),
    })
    .unwrap();

    assert_eq!(
        group_metadata
            .get_property_by_semantic(Some("_HEIGHT"))
            .unwrap()
            .unwrap(),
        json!(10.0)
    );

    assert!(group_metadata
        .set_property_by_semantic(Some("_HEIGHT"), Some(json!(35.0)))
        .unwrap());
    assert_eq!(
        group_metadata.get_property(Some("height")).unwrap().unwrap(),
        json!(35.0)
    );

    assert!(!group_metadata
        .set_property_by_semantic(Some("_INVALID"), Some(json!(1.0)))
        .unwrap());
    assert!(group_metadata.get_property_by_semantic(None).is_err());
    assert!(group_metadata
        .set_property_by_semantic(None, Some(json!(1.0)))
        .is_err());
    assert!(group_metadata
        .set_property_by_semantic(Some("_HEIGHT"), None)
        .is_err());
}
