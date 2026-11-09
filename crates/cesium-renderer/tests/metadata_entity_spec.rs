use cesium_renderer::{
    MetadataClass, MetadataEntity, MetadataEntityBase, MetadataEntityTrait,
};
use serde_json::json;
use std::collections::HashMap;

fn create_fixtures() -> (MetadataClass, MetadataClass, MetadataClass) {
    let class_with_no_properties = MetadataClass::from_json(&json!({
        "id": "building",
        "class": {}
    }))
    .unwrap();

    let class_definition = MetadataClass::from_json(&json!({
        "id": "building",
        "class": {
            "properties": {
                "name": {
                    "type": "STRING",
                    "semantic": "NAME",
                    "required": true
                },
                "height": {
                    "type": "SCALAR",
                    "componentType": "FLOAT32",
                    "required": false,
                    "default": 10.0
                },
                "position": {
                    "type": "SCALAR",
                    "componentType": "FLOAT32",
                    "array": true,
                    "count": 3,
                    "required": true
                },
                "temperature": {
                    "type": "SCALAR",
                    "componentType": "UINT8",
                    "normalized": true,
                    "offset": 32.0,
                    "scale": 180.0
                }
            }
        }
    }))
    .unwrap();

    let class_with_no_data = MetadataClass::from_json(&json!({
        "id": "noData",
        "class": {
            "properties": {
                "noDefault": {
                    "type": "SCALAR",
                    "componentType": "INT32",
                    "required": false,
                    "noData": -1
                },
                "hasDefault": {
                    "type": "SCALAR",
                    "componentType": "INT32",
                    "required": false,
                    "noData": -1,
                    "default": 100
                }
            }
        }
    }))
    .unwrap();

    (class_with_no_properties, class_definition, class_with_no_data)
}

#[test]
fn test_throws_when_using_metadata_entity_directly() {
    let mut entity = MetadataEntityBase;
    assert!(entity.class_definition().is_err());
    assert!(entity.has_property("prop").is_err());
    assert!(entity.has_property_by_semantic("SEM").is_err());
    assert!(entity.get_property_ids().is_err());
    assert!(entity.get_property("prop").is_err());
    assert!(entity.set_property("prop", json!(1)).is_err());
    assert!(entity.get_property_by_semantic("SEM").is_err());
    assert!(entity.set_property_by_semantic("SEM", json!(1)).is_err());
}

#[test]
fn test_has_property() {
    let (empty_class, class_def, _) = create_fixtures();
    let mut props = HashMap::new();

    assert!(!MetadataEntity::has_property("name", &props, &empty_class));

    // Default value property returns true even if not in props
    assert!(MetadataEntity::has_property("height", &props, &class_def));

    props.insert("name".to_string(), json!("Building A"));
    assert!(MetadataEntity::has_property("name", &props, &class_def));
}

#[test]
fn test_has_property_by_semantic() {
    let (_, class_def, _) = create_fixtures();
    let props = HashMap::new();

    assert!(MetadataEntity::has_property_by_semantic(
        "NAME",
        &props,
        &class_def
    ));
    assert!(!MetadataEntity::has_property_by_semantic(
        "UNKNOWN",
        &props,
        &class_def
    ));
}

#[test]
fn test_get_property_ids() {
    let (_, class_def, _) = create_fixtures();
    let mut props = HashMap::new();
    props.insert("name".to_string(), json!("Building A"));

    let ids = MetadataEntity::get_property_ids(&props, &class_def);
    assert!(ids.contains(&"name".to_string()));
    assert!(ids.contains(&"height".to_string())); // default property included
}

#[test]
fn test_get_and_set_property() {
    let (_, class_def, _) = create_fixtures();
    let mut props = HashMap::new();
    props.insert("name".to_string(), json!("Building A"));

    // Get existing
    let name_val = MetadataEntity::get_property("name", &props, &class_def)
        .unwrap()
        .unwrap();
    assert_eq!(name_val, json!("Building A"));

    // Get default
    let height_val = MetadataEntity::get_property("height", &props, &class_def)
        .unwrap()
        .unwrap();
    assert_eq!(height_val, json!(10.0));

    // Set existing
    assert!(MetadataEntity::set_property("name", json!("Building B"), &mut props, &class_def).unwrap());
    assert_eq!(props["name"], json!("Building B"));

    // Set non-existent returns false
    assert!(!MetadataEntity::set_property("unknown", json!(1), &mut props, &class_def).unwrap());
}

#[test]
fn test_get_and_set_property_by_semantic() {
    let (_, class_def, _) = create_fixtures();
    let mut props = HashMap::new();
    props.insert("name".to_string(), json!("Building A"));

    let val = MetadataEntity::get_property_by_semantic("NAME", &props, &class_def)
        .unwrap()
        .unwrap();
    assert_eq!(val, json!("Building A"));

    assert!(MetadataEntity::set_property_by_semantic(
        "NAME",
        json!("Building C"),
        &mut props,
        &class_def
    )
    .unwrap());
    assert_eq!(props["name"], json!("Building C"));
}

#[test]
fn test_no_data_and_defaults() {
    let (_, _, class_with_no_data) = create_fixtures();
    let mut props = HashMap::new();
    props.insert("noDefault".to_string(), json!(-1)); // matches noData
    props.insert("hasDefault".to_string(), json!(-1)); // matches noData

    let no_def = MetadataEntity::get_property("noDefault", &props, &class_with_no_data).unwrap();
    assert_eq!(no_def, None);

    let has_def =
        MetadataEntity::get_property("hasDefault", &props, &class_with_no_data).unwrap();
    assert_eq!(has_def, Some(json!(100)));
}

#[test]
fn test_normalization_and_value_transforms() {
    let (_, class_def, _) = create_fixtures();
    let mut props = HashMap::new();

    // raw UINT8 value 0 -> normalized = 0.0 -> transformed = 180 * 0.0 + 32 = 32.0
    props.insert("temperature".to_string(), json!(0));
    let t_val = MetadataEntity::get_property("temperature", &props, &class_def)
        .unwrap()
        .unwrap();
    assert_eq!(t_val, json!(32.0));

    // raw UINT8 value 255 -> normalized = 1.0 -> transformed = 180 * 1.0 + 32 = 212.0
    props.insert("temperature".to_string(), json!(255));
    let t_val2 = MetadataEntity::get_property("temperature", &props, &class_def)
        .unwrap()
        .unwrap();
    assert_eq!(t_val2, json!(212.0));

    // set property with value 212.0 -> unapply transform = (212 - 32) / 180 = 1.0 -> unnormalize = 255
    assert!(MetadataEntity::set_property("temperature", json!(212.0), &mut props, &class_def).unwrap());
    assert_eq!(props["temperature"], json!(255.0));
}
