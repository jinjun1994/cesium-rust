use cesium_math::DeveloperError;
use cesium_renderer::{ContentMetadata, ContentMetadataOptions, MetadataClass};
use serde_json::json;

fn make_content_class_with_no_properties() -> MetadataClass {
    MetadataClass::from_json(&json!({
        "id": "content",
        "class": {}
    }))
    .unwrap()
}

fn make_content_class() -> MetadataClass {
    MetadataClass::from_json(&json!({
        "id": "content",
        "class": {
            "properties": {
                "color": {
                    "type": "SCALAR",
                    "componentType": "UINT8",
                    "array": true,
                    "count": 3,
                    "semantic": "COLOR"
                },
                "author": {
                    "type": "STRING"
                }
            }
        }
    }))
    .unwrap()
}

fn make_default_content_metadata() -> ContentMetadata {
    ContentMetadata::new(ContentMetadataOptions {
        content: Some(json!({
            "class": "content",
            "properties": {
                "color": [255, 125, 0],
                "author": "Cesium"
            }
        })),
        class_def: Some(make_content_class()),
    })
    .expect("should construct ContentMetadata")
}

#[test]
fn test_throws_without_content() {
    let err = ContentMetadata::new(ContentMetadataOptions {
        content: None,
        class_def: Some(make_content_class()),
    })
    .expect_err("should throw without content");
    assert_eq!(err.name(), DeveloperError::NAME);
}

#[test]
fn test_throws_without_class() {
    let err = ContentMetadata::new(ContentMetadataOptions {
        content: Some(json!({})),
        class_def: None,
    })
    .expect_err("should throw without class");
    assert_eq!(err.name(), DeveloperError::NAME);
}

#[test]
fn test_creates_content_metadata_with_default_values() {
    let metadata = ContentMetadata::new(ContentMetadataOptions {
        content: Some(json!({})),
        class_def: Some(make_content_class_with_no_properties()),
    })
    .expect("should construct with default values");

    assert_eq!(metadata.extras, None);
    assert_eq!(metadata.extensions, None);
}

#[test]
fn test_creates_content_metadata() {
    let properties = json!({
        "color": [0, 0, 255],
        "author": "Sample Author"
    });
    let extras = json!({
        "version": "0.0"
    });
    let extensions = json!({
        "3DTILES_extension": {}
    });

    let content_metadata = ContentMetadata::new(ContentMetadataOptions {
        content: Some(json!({
            "class": "content",
            "properties": properties,
            "extras": extras,
            "extensions": extensions
        })),
        class_def: Some(make_content_class()),
    })
    .expect("should construct");

    assert_eq!(content_metadata.get_property("color").unwrap(), json!([0, 0, 255]));
    assert_eq!(content_metadata.get_property("author").unwrap(), json!("Sample Author"));
    assert_eq!(content_metadata.extras, Some(json!({"version": "0.0"})));
    assert_eq!(content_metadata.extensions, Some(json!({"3DTILES_extension": {}})));
}

#[test]
fn test_has_property() {
    let metadata = make_default_content_metadata();
    assert!(metadata.has_property("color"));
    assert!(!metadata.has_property("numberOfPoints"));
}

#[test]
fn test_has_property_by_semantic() {
    let metadata = make_default_content_metadata();
    assert!(metadata.has_property_by_semantic("COLOR"));
    assert!(!metadata.has_property_by_semantic("NUMBER_OF_POINTS"));
}

#[test]
fn test_get_property_ids() {
    let metadata = make_default_content_metadata();
    let mut ids = Vec::new();
    let mut result = metadata.get_property_ids(&mut ids);
    result.sort();
    assert_eq!(result, vec!["author", "color"]);
}

#[test]
fn test_get_property_throws_if_property_does_not_exist() {
    let metadata = make_default_content_metadata();
    let err = metadata.get_property("numberOfPoints").expect_err("should throw");
    assert_eq!(err.name(), DeveloperError::NAME);
}

#[test]
fn test_get_property_returns_value() {
    let metadata = make_default_content_metadata();
    assert_eq!(metadata.get_property("color").unwrap(), json!([255, 125, 0]));
    assert_eq!(metadata.get_property("author").unwrap(), json!("Cesium"));
}

#[test]
fn test_set_property() {
    let mut metadata = make_default_content_metadata();
    assert!(!metadata.set_property("numberOfPoints", json!(10)));

    assert_eq!(metadata.get_property("author").unwrap(), json!("Cesium"));
    assert!(metadata.set_property("author", json!("Sample Author")));
    assert_eq!(metadata.get_property("author").unwrap(), json!("Sample Author"));
}

#[test]
fn test_get_property_by_semantic() {
    let metadata = make_default_content_metadata();
    assert_eq!(metadata.get_property_by_semantic("AUTHOR"), None);
    assert_eq!(
        metadata.get_property_by_semantic("COLOR"),
        Some(json!([255, 125, 0]))
    );
}

#[test]
fn test_set_property_by_semantic() {
    let mut metadata = make_default_content_metadata();
    assert_eq!(
        metadata.get_property_by_semantic("COLOR"),
        Some(json!([255, 125, 0]))
    );
    assert!(metadata.set_property_by_semantic("COLOR", json!([0, 0, 0])));
    assert_eq!(
        metadata.get_property_by_semantic("COLOR"),
        Some(json!([0, 0, 0]))
    );

    assert!(!metadata.set_property_by_semantic("AUTHOR", json!("Test Author")));
}
