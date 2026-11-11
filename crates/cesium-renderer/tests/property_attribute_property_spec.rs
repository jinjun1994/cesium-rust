use cesium_renderer::metadata_class_property::MetadataClassProperty;
use cesium_renderer::property_attribute_property::PropertyAttributeProperty;
use serde_json::json;

#[test]
fn test_creates_property_attribute_property() {
    let prop_json = json!({
        "type": "SCALAR",
        "componentType": "FLOAT32"
    });
    let class_property = MetadataClassProperty::from_json("intensity".to_string(), &prop_json, None).unwrap();

    let extras = json!({
        "description": "Extra"
    });
    let extensions = json!({
        "EXT_other_extension": {}
    });

    let property_json = json!({
        "attribute": "_INTENSITY",
        "extensions": extensions,
        "extras": extras
    });

    let pap = PropertyAttributeProperty::new(Some(&property_json), Some(&class_property)).unwrap();

    assert_eq!(pap.attribute, "_INTENSITY");
    assert!(!pap.has_value_transform);
    assert_eq!(pap.offset, json!(0.0));
    assert_eq!(pap.scale, json!(1.0));
    assert_eq!(pap.extras, Some(extras));
    assert_eq!(pap.extensions, Some(extensions));
    assert_eq!(pap.class_property, class_property);
}

#[test]
fn test_constructor_throws_without_property() {
    let prop_json = json!({
        "type": "SCALAR",
        "componentType": "FLOAT32"
    });
    let class_property = MetadataClassProperty::from_json("intensity".to_string(), &prop_json, None).unwrap();
    let err = PropertyAttributeProperty::new(None, Some(&class_property));
    assert!(err.is_err());
}

#[test]
fn test_constructor_throws_without_class_property() {
    let property_json = json!({
        "attribute": "_INTENSITY"
    });
    let err = PropertyAttributeProperty::new(Some(&property_json), None);
    assert!(err.is_err());
}

#[test]
fn test_creates_property_with_value_transform_from_class_definition() {
    let prop_json = json!({
        "type": "SCALAR",
        "componentType": "UINT8",
        "normalized": true,
        "offset": 1,
        "scale": 2
    });
    let class_property = MetadataClassProperty::from_json("transformed".to_string(), &prop_json, None).unwrap();

    let property_json = json!({
        "attribute": "_TRANSFORMED"
    });

    let pap = PropertyAttributeProperty::new(Some(&property_json), Some(&class_property)).unwrap();

    assert_eq!(pap.attribute, "_TRANSFORMED");
    assert!(pap.has_value_transform);
    assert_eq!(pap.offset, json!(1));
    assert_eq!(pap.scale, json!(2));
}

#[test]
fn test_creates_property_with_value_transform_override() {
    let prop_json = json!({
        "type": "SCALAR",
        "componentType": "UINT8",
        "normalized": true,
        "offset": 1,
        "scale": 2
    });
    let class_property = MetadataClassProperty::from_json("transformed".to_string(), &prop_json, None).unwrap();

    let property_json = json!({
        "attribute": "_TRANSFORMED",
        "offset": 2,
        "scale": 4
    });

    let pap = PropertyAttributeProperty::new(Some(&property_json), Some(&class_property)).unwrap();

    assert_eq!(pap.attribute, "_TRANSFORMED");
    assert!(pap.has_value_transform);
    assert_eq!(pap.offset, json!(2));
    assert_eq!(pap.scale, json!(4));
}

#[test]
fn test_unpacks_property_and_scale_for_vectors_and_matrices() {
    let prop_json = json!({
        "type": "VEC2",
        "componentType": "UINT8",
        "normalized": true,
        "offset": [1, 2],
        "scale": [2, 4]
    });
    let class_property = MetadataClassProperty::from_json("transformed".to_string(), &prop_json, None).unwrap();

    let property_json = json!({
        "attribute": "_TRANSFORMED"
    });

    let pap = PropertyAttributeProperty::new(Some(&property_json), Some(&class_property)).unwrap();

    assert_eq!(pap.attribute, "_TRANSFORMED");
    assert!(pap.has_value_transform);
    assert_eq!(pap.offset, json!([1, 2]));
    assert_eq!(pap.scale, json!([2, 4]));

    let prop_mat_json = json!({
        "type": "MAT2",
        "componentType": "UINT8",
        "normalized": true,
        "offset": [1, 2, 2, 1],
        "scale": [2, 4, 4, 1]
    });
    let class_property_mat = MetadataClassProperty::from_json("transformed".to_string(), &prop_mat_json, None).unwrap();

    let pap_mat = PropertyAttributeProperty::new(Some(&property_json), Some(&class_property_mat)).unwrap();

    assert_eq!(pap_mat.attribute, "_TRANSFORMED");
    assert!(pap_mat.has_value_transform);
    assert_eq!(pap_mat.offset, json!([1, 2, 2, 1]));
    assert_eq!(pap_mat.scale, json!([2, 4, 4, 1]));
}
