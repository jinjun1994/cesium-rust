use cesium_math::DeveloperError;
use cesium_renderer::Empty3DTileContent;
use serde_json::json;

#[test]
fn test_destroys() {
    let mut content = Empty3DTileContent::default();
    assert!(!content.is_destroyed());
    content.destroy();
    assert!(content.is_destroyed());
}

#[test]
fn test_gets_properties() {
    let content = Empty3DTileContent::new(
        Some("mockTileset".to_string()),
        Some("mockTile".to_string()),
    );

    assert_eq!(content.features_length(), 0);
    assert_eq!(content.points_length(), 0);
    assert_eq!(content.triangles_length(), 0);
    assert_eq!(content.geometry_byte_length(), 0);
    assert_eq!(content.textures_byte_length(), 0);
    assert_eq!(content.batch_table_byte_length(), 0);
    assert!(content.inner_contents().is_none());
    assert_eq!(content.tileset_id.as_deref(), Some("mockTileset"));
    assert_eq!(content.tile_id.as_deref(), Some("mockTile"));
    assert!(content.url().is_none());
}

#[test]
fn test_content_metadata_returns_undefined() {
    let content = Empty3DTileContent::new(
        Some("mockTileset".to_string()),
        Some("mockTile".to_string()),
    );
    assert!(content.metadata().is_none());
}

#[test]
fn test_group_metadata_returns_undefined() {
    let content = Empty3DTileContent::new(
        Some("mockTileset".to_string()),
        Some("mockTile".to_string()),
    );
    assert!(content.group().is_none());
}

#[test]
fn test_assigning_group_throws() {
    let mut content = Empty3DTileContent::new(
        Some("mockTileset".to_string()),
        Some("mockTile".to_string()),
    );
    let err = content.set_group(json!({})).expect_err("should throw");
    assert_eq!(err.name(), DeveloperError::NAME);
}

#[test]
fn test_assigning_content_metadata_throws() {
    let mut content = Empty3DTileContent::new(
        Some("mockTileset".to_string()),
        Some("mockTile".to_string()),
    );
    let err = content.set_metadata(json!({})).expect_err("should throw");
    assert_eq!(err.name(), DeveloperError::NAME);
}
