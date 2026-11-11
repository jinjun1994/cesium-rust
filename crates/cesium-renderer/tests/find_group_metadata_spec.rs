//! Spec matching CesiumJS `Specs/Scene/findGroupMetadataSpec.js` (Apache-2.0).

use cesium_renderer::{
    find_group_metadata, GroupMetadata, GroupMetadataOptions, MetadataClass,
    TilesetGroupMetadataQuery,
};
use serde_json::json;

fn create_mock_tileset() -> TilesetGroupMetadataQuery {
    let layer_class = MetadataClass::from_json(&json!({
        "id": "layer",
        "class": {
            "properties": {
                "name": {
                    "type": "STRING"
                },
                "elevation": {
                    "type": "SCALAR",
                    "componentType": "FLOAT32"
                }
            }
        }
    }))
    .expect("Failed to create MetadataClass");

    let group = GroupMetadata::new(GroupMetadataOptions {
        id: Some("testGroup".to_string()),
        class_def: Some(layer_class),
        group: Some(json!({
            "properties": {
                "name": "Test Layer testGroup",
                "elevation": 150.0
            }
        })),
    })
    .expect("Failed to create GroupMetadata");

    TilesetGroupMetadataQuery::new(vec![group], vec!["testGroup".to_string()])
}

#[test]
fn test_returns_undefined_if_there_is_no_group_or_extension() {
    let mock_tileset = create_mock_tileset();
    let content_header = json!({
        "uri": "https://example.com/model.b3dm"
    });

    let group = find_group_metadata(&mock_tileset, &content_header);
    assert!(group.is_none());
}

#[test]
fn test_returns_the_group_metadata_if_there_is_a_group_index() {
    let mock_tileset = create_mock_tileset();
    let content_header = json!({
        "uri": "https://example.com/model.b3dm",
        "group": 0
    });

    let group = find_group_metadata(&mock_tileset, &content_header);
    assert!(group.is_some());
    let group = group.unwrap();

    assert_eq!(group.get_property(Some("name")).unwrap(), Some(json!("Test Layer testGroup")));
    assert_eq!(group.get_property(Some("elevation")).unwrap(), Some(json!(150.0)));
}

#[test]
fn test_returns_the_group_metadata_if_there_is_a_group_with_the_same_id_legacy() {
    let mock_tileset = create_mock_tileset();
    let content_header = json!({
        "uri": "https://example.com/model.b3dm",
        "group": "testGroup"
    });

    let group = find_group_metadata(&mock_tileset, &content_header);
    assert!(group.is_some());
    let group = group.unwrap();

    assert_eq!(group.get_property(Some("name")).unwrap(), Some(json!("Test Layer testGroup")));
    assert_eq!(group.get_property(Some("elevation")).unwrap(), Some(json!(150.0)));
}

#[test]
fn test_returns_the_group_metadata_if_there_is_an_extension_legacy() {
    let mock_tileset = create_mock_tileset();
    let content_header = json!({
        "uri": "https://example.com/model.b3dm",
        "extensions": {
            "3DTILES_metadata": {
                "group": "testGroup"
            }
        }
    });

    let group = find_group_metadata(&mock_tileset, &content_header);
    assert!(group.is_some());
    let group = group.unwrap();

    assert_eq!(group.get_property(Some("name")).unwrap(), Some(json!("Test Layer testGroup")));
    assert_eq!(group.get_property(Some("elevation")).unwrap(), Some(json!(150.0)));
}
