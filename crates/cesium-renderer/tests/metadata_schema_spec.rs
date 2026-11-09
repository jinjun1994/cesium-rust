use cesium_renderer::MetadataSchema;
use serde_json::json;

#[test]
fn test_creates_schema_with_default_values() {
    let empty_json = json!({});
    let schema = MetadataSchema::from_json(Some(&empty_json)).unwrap();

    assert!(schema.classes.is_empty());
    assert!(schema.enums.is_empty());
    assert_eq!(schema.name, None);
    assert_eq!(schema.description, None);
    assert_eq!(schema.version, None);
    assert_eq!(schema.extras, None);
}

#[test]
fn test_creates_schema() {
    let extras = json!({
        "description": "Extra"
    });
    let extensions = json!({
        "EXT_other_extension": {}
    });

    let schema_json = json!({
        "enums": {
            "color": {
                "values": [
                    { "name": "RED", "value": 0 },
                    { "name": "GREEN", "value": 1 },
                    { "name": "BLUE", "value": 2 }
                ]
            },
            "species": {
                "values": [
                    { "name": "Oak", "value": 0 },
                    { "name": "Pine", "value": 1 },
                    { "name": "Other", "value": -1 }
                ]
            }
        },
        "classes": {
            "city": {
                "properties": {
                    "name": {
                        "type": "STRING"
                    }
                }
            },
            "neighborhood": {
                "properties": {
                    "color": {
                        "type": "ENUM",
                        "enumType": "color"
                    },
                    "coordinates": {
                        "type": "SCALAR",
                        "componentType": "FLOAT64",
                        "array": true,
                        "count": 2
                    }
                }
            },
            "tree": {
                "properties": {
                    "species": {
                        "type": "ENUM",
                        "array": true,
                        "enumType": "species"
                    },
                    "height": {
                        "type": "SCALAR",
                        "componentType": "FLOAT32"
                    }
                }
            }
        },
        "id": "mySchema",
        "name": "My Schema",
        "description": "My Schema Description",
        "version": "3.1.0",
        "extras": extras,
        "extensions": extensions
    });

    let schema = MetadataSchema::from_json(Some(&schema_json)).unwrap();

    let city_class = schema.classes.get("city").unwrap();
    let neighborhood_class = schema.classes.get("neighborhood").unwrap();
    let tree_class = schema.classes.get("tree").unwrap();

    assert_eq!(city_class.id, "city");
    assert_eq!(neighborhood_class.id, "neighborhood");
    assert_eq!(tree_class.id, "tree");

    assert_eq!(city_class.properties.get("name").unwrap().id, "name");
    assert_eq!(
        neighborhood_class
            .properties
            .get("color")
            .unwrap()
            .enum_type
            .as_ref()
            .unwrap()
            .id,
        "color"
    );
    assert_eq!(
        neighborhood_class.properties.get("coordinates").unwrap().id,
        "coordinates"
    );
    assert_eq!(
        tree_class
            .properties
            .get("species")
            .unwrap()
            .enum_type
            .as_ref()
            .unwrap()
            .id,
        "species"
    );
    assert_eq!(tree_class.properties.get("height").unwrap().id, "height");

    assert_eq!(schema.id.as_deref(), Some("mySchema"));
    assert_eq!(schema.name.as_deref(), Some("My Schema"));
    assert_eq!(schema.description.as_deref(), Some("My Schema Description"));
    assert_eq!(schema.version.as_deref(), Some("3.1.0"));

    assert_eq!(schema.extras, Some(extras));
    assert_eq!(schema.extensions, Some(extensions));
}

#[test]
fn test_constructor_throws_without_schema() {
    assert!(MetadataSchema::from_json(None).is_err());
}
