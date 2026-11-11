//! Spec matching CesiumJS `Specs/Scene/findContentMetadataSpec.js` (Apache-2.0).

use cesium_renderer::{
    find_content_metadata, has_find_content_metadata_warned, reset_find_content_metadata_warning,
    MetadataClass, MetadataSchema, TilesetMetadataQuery,
};
use serde_json::json;

fn create_mock_tileset() -> TilesetMetadataQuery {
    let content_class = MetadataClass::from_json(&json!({
        "id": "content",
        "class": {
            "properties": {
                "name": {
                    "type": "STRING"
                },
                "color": {
                    "type": "VEC3",
                    "componentType": "UINT8"
                }
            }
        }
    }))
    .expect("Failed to create MetadataClass");

    let mut schema = MetadataSchema::default();
    schema.classes.insert("content".to_string(), content_class);

    TilesetMetadataQuery::new(Some(schema))
}

#[test]
fn test_returns_undefined_if_there_is_no_metadata_or_extension() {
    let mock_tileset = create_mock_tileset();
    let content_header = json!({
        "uri": "https://example.com/model.b3dm"
    });

    let metadata = find_content_metadata(&mock_tileset, &content_header);
    assert!(metadata.is_none());
}

#[test]
fn test_logs_warning_and_returns_undefined_if_the_tileset_is_missing_a_schema() {
    reset_find_content_metadata_warning();
    let tileset_without_schema = TilesetMetadataQuery::new(None);
    let content_header = json!({
        "uri": "https://example.com/model.b3dm",
        "metadata": {
            "class": "content",
            "properties": {
                "name": "Sample Content",
                "color": [255, 255, 0]
            }
        }
    });

    let metadata = find_content_metadata(&tileset_without_schema, &content_header);
    assert!(metadata.is_none());
    assert!(has_find_content_metadata_warned());
}

#[test]
fn test_returns_metadata_if_there_is_metadata() {
    let mock_tileset = create_mock_tileset();
    let content_header = json!({
        "uri": "https://example.com/model.b3dm",
        "metadata": {
            "class": "content",
            "properties": {
                "name": "Sample Content",
                "color": [255, 255, 0]
            }
        }
    });

    let metadata = find_content_metadata(&mock_tileset, &content_header);
    assert!(metadata.is_some());
    let metadata = metadata.unwrap();

    assert_eq!(metadata.get_property("name").unwrap(), json!("Sample Content"));
    assert_eq!(metadata.get_property("color").unwrap(), json!([255, 255, 0]));
}

#[test]
fn test_returns_metadata_if_there_is_an_extension_legacy() {
    let mock_tileset = create_mock_tileset();
    let content_header = json!({
        "uri": "https://example.com/model.b3dm",
        "extensions": {
            "3DTILES_metadata": {
                "class": "content",
                "properties": {
                    "name": "Sample Content",
                    "color": [255, 255, 0]
                }
            }
        }
    });

    let metadata = find_content_metadata(&mock_tileset, &content_header);
    assert!(metadata.is_some());
    let metadata = metadata.unwrap();

    assert_eq!(metadata.get_property("name").unwrap(), json!("Sample Content"));
    assert_eq!(metadata.get_property("color").unwrap(), json!([255, 255, 0]));
}
