use cesium_renderer::content_metadata::{ContentMetadata, ContentMetadataOptions};
use cesium_renderer::group_metadata::{GroupMetadata, GroupMetadataOptions};
use cesium_renderer::metadata_class::MetadataClass;
use cesium_renderer::multiple_3d_tile_content::Cesium3DContentGroup;
use cesium_renderer::tileset_3d_tile_content::Tileset3DTileContent;
use serde_json::json;

#[test]
fn becomes_ready() {
    let content = Tileset3DTileContent::from_json(
        Some("tileset_root".to_string()),
        Some("tile_root".to_string()),
        Some("./Data/Cesium3DTiles/Tilesets/TilesetOfTilesets/tileset.json".to_string()),
        &json!({
            "asset": { "version": "1.0" },
            "geometricError": 100.0,
            "root": {}
        }),
    );

    assert!(content.ready());
    assert_eq!(content.tileset_id(), Some("tileset_root"));
    assert_eq!(content.tile_id(), Some("tile_root"));
}

#[test]
fn destroys() {
    let mut content = Tileset3DTileContent::new(
        Some("tileset".to_string()),
        Some("tile".to_string()),
        Some("tileset.json".to_string()),
    );
    assert!(!content.is_destroyed());
    content.destroy();
    assert!(content.is_destroyed());
}

#[test]
fn gets_properties() {
    let content = Tileset3DTileContent::new(
        Some("tileset_id".to_string()),
        Some("tile_id".to_string()),
        Some("http://example.com/tileset.json".to_string()),
    );

    assert_eq!(content.features_length(), 0);
    assert_eq!(content.points_length(), 0);
    assert_eq!(content.triangles_length(), 0);
    assert_eq!(content.geometry_byte_length(), 0);
    assert_eq!(content.textures_byte_length(), 0);
    assert_eq!(content.batch_table_byte_length(), 0);
    assert!(content.inner_contents().is_none());
    assert_eq!(content.tileset_id(), Some("tileset_id"));
    assert_eq!(content.tile_id(), Some("tile_id"));
    assert_eq!(content.url(), Some("http://example.com/tileset.json"));
    assert!(content.batch_table().is_none());
    assert!(!content.has_property(0, "name"));
    assert!(content.get_feature(0).is_none());
}

#[test]
fn assigns_group_metadata() {
    let group_class = MetadataClass::from_json(&json!({
        "id": "test",
        "class": {
            "properties": {
                "name": {
                    "type": "STRING"
                },
                "height": {
                    "type": "SCALAR",
                    "componentType": "FLOAT32"
                }
            }
        }
    }))
    .unwrap();

    let group_metadata = GroupMetadata::new(GroupMetadataOptions {
        id: Some("testGroup".to_string()),
        group: Some(json!({
            "properties": {
                "name": "Test Group",
                "height": 35.6
            }
        })),
        class_def: Some(group_class),
    })
    .unwrap();

    let mut content = Tileset3DTileContent::new(
        Some("tileset".to_string()),
        Some("tile".to_string()),
        Some("tileset.json".to_string()),
    );

    content.set_group(Cesium3DContentGroup::new(group_metadata.clone()));
    assert_eq!(
        content.group().unwrap().metadata.id(),
        Some("testGroup")
    );
}

#[test]
fn assigns_metadata() {
    let content_metadata_class = MetadataClass::from_json(&json!({
        "id": "contentTest",
        "class": {
            "properties": {
                "author": {
                    "type": "STRING"
                },
                "color": {
                    "type": "VEC3",
                    "componentType": "UINT8"
                }
            }
        }
    }))
    .unwrap();

    let content_metadata = ContentMetadata::new(ContentMetadataOptions {
        content: Some(json!({
            "properties": {
                "author": "Test Author",
                "color": [255, 0, 0]
            }
        })),
        class_def: Some(content_metadata_class),
    })
    .unwrap();

    let mut content = Tileset3DTileContent::new(
        Some("tileset".to_string()),
        Some("tile".to_string()),
        Some("tileset.json".to_string()),
    );

    content.set_metadata(content_metadata);
    assert_eq!(
        content
            .metadata()
            .unwrap()
            .get_property("author")
            .unwrap()
            .as_str()
            .unwrap(),
        "Test Author"
    );
}
