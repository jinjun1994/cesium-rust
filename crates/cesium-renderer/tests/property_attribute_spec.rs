use cesium_renderer::metadata_class::MetadataClass;
use cesium_renderer::property_attribute::PropertyAttribute;
use serde_json::json;

#[test]
fn test_creates_property_attribute() {
    let class_json = json!({
        "id": "points",
        "class": {
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
    });
    let class_def = MetadataClass::from_json(&class_json).unwrap();

    let extras = json!({
        "description": "Extra"
    });
    let extensions = json!({
        "EXT_other_extension": {}
    });

    let property_attribute_json = json!({
        "class": "points",
        "extras": extras,
        "extensions": extensions,
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
    });

    let pa = PropertyAttribute::new(Some(&class_def), Some(&property_attribute_json)).unwrap();

    assert_eq!(pa.class, class_def);
    assert_eq!(pa.extras, Some(extras));
    assert_eq!(pa.extensions, Some(extensions));
}

#[test]
fn test_constructor_throws_without_class() {
    let property_attribute_json = json!({});
    let err = PropertyAttribute::new(None, Some(&property_attribute_json));
    assert!(err.is_err());
}

#[test]
fn test_constructor_throws_without_property_attribute() {
    let class_json = json!({
        "id": "points",
        "class": {
            "properties": {}
        }
    });
    let class_def = MetadataClass::from_json(&class_json).unwrap();
    let err = PropertyAttribute::new(Some(&class_def), None);
    assert!(err.is_err());
}

#[test]
fn test_get_property_returns_property_attribute_property() {
    let class_json = json!({
        "id": "points",
        "class": {
            "properties": {
                "color": {
                    "type": "VEC3",
                    "componentType": "UINT8"
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
    });
    let class_def = MetadataClass::from_json(&class_json).unwrap();

    let property_attribute_json = json!({
        "class": "points",
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
    });

    let pa = PropertyAttribute::new(Some(&class_def), Some(&property_attribute_json)).unwrap();

    assert_eq!(
        pa.get_property(Some("color")).unwrap().unwrap().attribute,
        "_COLOR"
    );
    assert_eq!(
        pa.get_property(Some("intensity")).unwrap().unwrap().attribute,
        "_INTENSITY"
    );
    assert_eq!(
        pa.get_property(Some("pointSize")).unwrap().unwrap().attribute,
        "_POINT_SIZE"
    );

    assert!(pa.get_property(Some("other")).unwrap().is_none());
    assert!(pa.get_property(None).is_err());
}
