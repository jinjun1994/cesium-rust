use cesium_math::Color;
use cesium_renderer::composite_3d_tile_content::{Composite3DTileContent, CompositeInnerContent};
use cesium_renderer::content_metadata::{ContentMetadata, ContentMetadataOptions};
use cesium_renderer::group_metadata::{GroupMetadata, GroupMetadataOptions};
use cesium_renderer::metadata_class::MetadataClass;
use cesium_renderer::multiple_3d_tile_content::Cesium3DContentGroup;
use cesium_renderer::Cesium3DTileStyleDocument;
use serde_json::json;

fn make_b3dm_stub() -> Vec<u8> {
    // 28-byte b3dm header: magic(4), version(4), byteLength(4), ftJson(4), ftBin(4), btJson(4), btBin(4)
    let mut buf = Vec::with_capacity(28);
    buf.extend_from_slice(b"b3dm");
    buf.extend_from_slice(&1u32.to_le_bytes());
    buf.extend_from_slice(&28u32.to_le_bytes());
    buf.extend_from_slice(&0u32.to_le_bytes());
    buf.extend_from_slice(&0u32.to_le_bytes());
    buf.extend_from_slice(&0u32.to_le_bytes());
    buf.extend_from_slice(&0u32.to_le_bytes());
    buf
}

fn make_i3dm_stub(gltf_format: u32, uri: Option<&str>) -> Vec<u8> {
    let uri_bytes = uri.map(|u| u.as_bytes()).unwrap_or(&[]);
    let total_len = 32 + uri_bytes.len();
    let mut buf = Vec::with_capacity(total_len);
    buf.extend_from_slice(b"i3dm");
    buf.extend_from_slice(&1u32.to_le_bytes());
    buf.extend_from_slice(&(total_len as u32).to_le_bytes());
    buf.extend_from_slice(&0u32.to_le_bytes());
    buf.extend_from_slice(&0u32.to_le_bytes());
    buf.extend_from_slice(&0u32.to_le_bytes());
    buf.extend_from_slice(&0u32.to_le_bytes());
    buf.extend_from_slice(&gltf_format.to_le_bytes());
    buf.extend_from_slice(uri_bytes);
    buf
}

fn make_cmpt(version: u32, tiles: &[Vec<u8>]) -> Vec<u8> {
    let body_len: usize = tiles.iter().map(Vec::len).sum();
    let total_len = 16 + body_len;
    let mut buf = Vec::with_capacity(total_len);
    buf.extend_from_slice(b"cmpt");
    buf.extend_from_slice(&version.to_le_bytes());
    buf.extend_from_slice(&(total_len as u32).to_le_bytes());
    buf.extend_from_slice(&(tiles.len() as u32).to_le_bytes());
    for t in tiles {
        buf.extend_from_slice(t);
    }
    buf
}

#[test]
fn throws_with_invalid_version() {
    let buf = make_cmpt(2, &[]);
    let err = Composite3DTileContent::from_tile_type(None, None, "tile.cmpt", &buf, 0).unwrap_err();
    assert!(
        err.to_string()
            .contains("Only Composite Tile version 1 is supported. Version 2 is not."),
        "unexpected error: {err}"
    );
}

#[test]
fn throws_with_invalid_inner_tile_content_type() {
    let mut bad_inner = make_i3dm_stub(1, None);
    bad_inner[0..4].copy_from_slice(b"xxxx");
    let buf = make_cmpt(1, &[bad_inner]);
    let err = Composite3DTileContent::from_tile_type(None, None, "tile.cmpt", &buf, 0).unwrap_err();
    assert!(
        err.to_string()
            .contains("Unknown tile content type, xxxx, inside Composite tile"),
        "unexpected error: {err}"
    );
}

#[test]
fn throws_with_invalid_tile_content() {
    let bad_i3dm = make_i3dm_stub(0, Some("invalid"));
    let buf = make_cmpt(1, &[bad_i3dm]);
    assert!(
        Composite3DTileContent::from_tile_type(None, None, "tile.cmpt", &buf, 0).is_err()
    );
}

#[test]
fn becomes_ready_and_parses_nested_composite() {
    let inner_cmpt = make_cmpt(1, &[make_b3dm_stub(), make_i3dm_stub(1, None)]);
    let outer_cmpt = make_cmpt(1, &[inner_cmpt, make_b3dm_stub()]);

    let mut content =
        Composite3DTileContent::from_tile_type(None, None, "https://example.com/root.cmpt", &outer_cmpt, 0)
            .unwrap();

    assert!(content.ready());
    assert!(content.update());
    assert_eq!(content.features_length(), 0);
    assert_eq!(content.points_length(), 0);
    assert_eq!(content.triangles_length(), 0);
    assert_eq!(content.geometry_byte_length(), 0);
    assert_eq!(content.textures_byte_length(), 0);
    assert_eq!(content.batch_table_byte_length(), 0);
    assert!(content.batch_table().is_none());
    assert!(!content.has_property(0, "height"));
    assert!(content.get_feature(0).is_none());

    assert_eq!(content.inner_contents().len(), 2);
    match &content.inner_contents()[0] {
        CompositeInnerContent::Composite(nested) => {
            assert_eq!(
                nested.url(),
                "https://example.com/root.cmpt?compositeIndex=0"
            );
            assert_eq!(nested.inner_contents().len(), 2);
            assert_eq!(
                nested.inner_contents()[0].resource_url(),
                "https://example.com/root.cmpt?compositeIndex=0_0"
            );
            assert_eq!(
                nested.inner_contents()[1].resource_url(),
                "https://example.com/root.cmpt?compositeIndex=0_1"
            );
        }
        other => panic!("expected nested composite, got {other:?}"),
    }
    assert_eq!(
        content.inner_contents()[1].resource_url(),
        "https://example.com/root.cmpt?compositeIndex=1"
    );
}

#[test]
fn propagates_group_and_content_metadata_to_inner_contents() {
    let cmpt = make_cmpt(1, &[make_b3dm_stub(), make_i3dm_stub(1, None)]);
    let mut content =
        Composite3DTileContent::from_tile_type(None, None, "tile.cmpt", &cmpt, 0).unwrap();

    let group_class = MetadataClass::from_json(&json!({
        "id": "test",
        "class": {
            "properties": {
                "name": { "type": "STRING" },
                "height": { "type": "SCALAR", "componentType": "FLOAT32" }
            }
        }
    }))
    .unwrap();
    let group_metadata = GroupMetadata::new(GroupMetadataOptions {
        id: Some("testGroup".to_string()),
        group: Some(json!({
            "properties": { "name": "Test Group", "height": 35.6 }
        })),
        class_def: Some(group_class),
    })
    .unwrap();

    content.set_group(Some(Cesium3DContentGroup::new(group_metadata)));
    assert_eq!(
        content.group().unwrap().metadata.id(),
        Some("testGroup")
    );
    for inner in content.inner_contents() {
        assert_eq!(
            inner.group().unwrap().metadata.id(),
            Some("testGroup")
        );
    }

    let content_class = MetadataClass::from_json(&json!({
        "id": "contentTest",
        "class": {
            "properties": {
                "author": { "type": "STRING" }
            }
        }
    }))
    .unwrap();
    let explicit_metadata = ContentMetadata::new(ContentMetadataOptions {
        content: Some(json!({
            "properties": { "author": "Test Author" }
        })),
        class_def: Some(content_class),
    })
    .unwrap();

    content.set_metadata(Some(explicit_metadata));
    assert_eq!(
        content.metadata().unwrap().get_property("author").unwrap(),
        json!("Test Author")
    );
    for inner in content.inner_contents() {
        assert_eq!(
            inner.metadata().unwrap().get_property("author").unwrap(),
            json!("Test Author")
        );
    }
}

#[test]
fn propagates_dirty_debug_style_and_destroys() {
    let cmpt = make_cmpt(1, &[make_b3dm_stub(), make_i3dm_stub(1, None)]);
    let mut content =
        Composite3DTileContent::from_tile_type(None, None, "tile.cmpt", &cmpt, 0).unwrap();

    assert!(!content.feature_properties_dirty());
    content.set_feature_properties_dirty(true);
    assert!(content.feature_properties_dirty());
    for inner in content.inner_contents() {
        assert!(inner.feature_properties_dirty());
    }

    let style = Cesium3DTileStyleDocument::from_json(&json!({ "color": "color('#ff0000')" }))
        .unwrap();
    content.apply_style(Some(&style));
    content.apply_debug_settings(true, Some(Color::YELLOW));
    for inner in content.inner_contents() {
        if let CompositeInnerContent::Leaf(leaf) = inner {
            assert!(leaf.style_applied);
            assert!(leaf.debug_enabled);
            assert_eq!(leaf.debug_color, Some(Color::YELLOW));
        }
    }

    assert!(!content.is_destroyed());
    content.destroy();
    assert!(content.is_destroyed());
    assert!(content.inner_contents().is_empty());
}
