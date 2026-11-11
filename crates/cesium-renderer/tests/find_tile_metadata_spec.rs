//! Spec matching CesiumJS `Specs/Scene/findTileMetadataSpec.js` (Apache-2.0).

use cesium_renderer::{
    find_tile_metadata, has_find_tile_metadata_warned, reset_find_tile_metadata_warning,
    MetadataClass, MetadataSchema, TilesetMetadataQuery,
};
use serde_json::json;

fn create_mock_tileset() -> TilesetMetadataQuery {
    let tile_class = MetadataClass::from_json(&json!({
        "id": "tile",
        "class": {
            "properties": {
                "height": {
                    "type": "SCALAR",
                    "componentType": "FLOAT32"
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
    schema.classes.insert("tile".to_string(), tile_class);

    TilesetMetadataQuery::new(Some(schema))
}

#[test]
fn test_returns_undefined_if_there_is_no_metadata_or_extension() {
    let mock_tileset = create_mock_tileset();
    let tile_header = json!({
        "boundingVolume": {},
        "geometricError": 64
    });

    let metadata = find_tile_metadata(&mock_tileset, &tile_header);
    assert!(metadata.is_none());
}

#[test]
fn test_logs_warning_and_returns_undefined_if_the_tileset_is_missing_a_schema() {
    reset_find_tile_metadata_warning();
    let tileset_without_schema = TilesetMetadataQuery::new(None);
    let tile_header = json!({
        "boundingVolume": {},
        "geometricError": 64,
        "metadata": {
            "class": "tile",
            "properties": {
                "height": 250.5,
                "color": [255, 255, 0]
            }
        }
    });

    let metadata = find_tile_metadata(&tileset_without_schema, &tile_header);
    assert!(metadata.is_none());
    assert!(has_find_tile_metadata_warned());
}

#[test]
fn test_returns_metadata_if_there_is_metadata() {
    let mock_tileset = create_mock_tileset();
    let tile_header = json!({
        "boundingVolume": {},
        "geometricError": 64,
        "metadata": {
            "class": "tile",
            "properties": {
                "height": 250.5,
                "color": [255, 255, 0]
            }
        }
    });

    let metadata = find_tile_metadata(&mock_tileset, &tile_header);
    assert!(metadata.is_some());
    let metadata = metadata.unwrap();

    assert_eq!(metadata.get_property("height").unwrap(), json!(250.5));
    assert_eq!(metadata.get_property("color").unwrap(), json!([255, 255, 0]));
}

#[test]
fn test_returns_metadata_if_there_is_an_extension_legacy() {
    let mock_tileset = create_mock_tileset();
    let tile_header = json!({
        "boundingVolume": {},
        "geometricError": 64,
        "extensions": {
            "3DTILES_metadata": {
                "class": "tile",
                "properties": {
                    "height": 250.5,
                    "color": [255, 255, 0]
                }
            }
        }
    });

    let metadata = find_tile_metadata(&mock_tileset, &tile_header);
    assert!(metadata.is_some());
    let metadata = metadata.unwrap();

    assert_eq!(metadata.get_property("height").unwrap(), json!(250.5));
    assert_eq!(metadata.get_property("color").unwrap(), json!([255, 255, 0]));
}
