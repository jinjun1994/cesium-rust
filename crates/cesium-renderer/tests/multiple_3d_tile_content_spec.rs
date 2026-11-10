//! Multiple3DTileContentSpec matching CesiumJS `Specs/Scene/Multiple3DTileContentSpec.js` (Apache-2.0).

use cesium_math::{BoundingSphere, Color};
use cesium_renderer::bounding_volume_semantics::MetadataClass;
use cesium_renderer::group_metadata::{GroupMetadata, GroupMetadataOptions};
use cesium_renderer::{
    Cesium3DContentGroup, ContentMetadata, ContentMetadataOptions, Multiple3DTileContent,
    TilesetStatistics,
};
use glam::DVec3;
use serde_json::json;

fn sample_contents_json() -> serde_json::Value {
    json!({
        "contents": [
            { "uri": "pointcloud.pnts" },
            { "uri": "batched.b3dm" },
            { "uri": "gltfModel.glb" }
        ]
    })
}

fn sample_legacy_content_json() -> serde_json::Value {
    json!({
        "content": [
            { "uri": "pointcloud.pnts" },
            { "uri": "batched.b3dm" },
            { "uri": "gltfModel.glb" }
        ]
    })
}

#[test]
fn test_inner_content_urls_returns_urls_from_contents_array() {
    let content =
        Multiple3DTileContent::new("http://example.com/tileset.json", &sample_contents_json())
            .expect("Should parse contents array");

    assert_eq!(
        content.inner_content_urls(),
        vec![
            "pointcloud.pnts".to_string(),
            "batched.b3dm".to_string(),
            "gltfModel.glb".to_string()
        ]
    );
}

#[test]
fn test_inner_content_urls_returns_urls_from_legacy_content() {
    let content = Multiple3DTileContent::new(
        "http://example.com/tileset.json",
        &sample_legacy_content_json(),
    )
    .expect("Should parse legacy content array");

    assert_eq!(
        content.inner_content_urls(),
        vec![
            "pointcloud.pnts".to_string(),
            "batched.b3dm".to_string(),
            "gltfModel.glb".to_string()
        ]
    );
}

#[test]
fn test_request_inner_contents_resolves_if_successful() {
    let mut content =
        Multiple3DTileContent::new("http://example.com/tileset.json", &sample_contents_json())
            .unwrap();
    let mut statistics = TilesetStatistics::default();

    let scheduled = content.request_inner_contents(
        &mut statistics,
        10, // Max requests per server
        "http://example.com",
        |_uri| Ok(vec![1, 2, 3, 4]),
        |_url, _msg| {},
    );

    assert!(scheduled);
    assert_eq!(statistics.number_of_pending_requests, 0);
    assert_eq!(statistics.number_of_attempted_requests, 0);
    assert_eq!(content.inner_contents.len(), 3);
    assert!(content.ready);
}

#[test]
fn test_request_inner_contents_returns_none_and_updates_statistics_if_cannot_schedule() {
    let mut content =
        Multiple3DTileContent::new("http://example.com/tileset.json", &sample_contents_json())
            .unwrap();
    let mut statistics = TilesetStatistics::default();

    // 3 contents to schedule, but maximumRequestsPerServer is 2
    let scheduled = content.request_inner_contents(
        &mut statistics,
        2,
        "http://example.com",
        |_uri| Ok(vec![1, 2, 3]),
        |_url, _msg| {},
    );

    assert!(!scheduled);
    assert_eq!(statistics.number_of_pending_requests, 0);
    assert_eq!(statistics.number_of_attempted_requests, 3);
}

#[test]
fn test_request_inner_contents_handles_failures() {
    let mut content =
        Multiple3DTileContent::new("http://example.com/tileset.json", &sample_contents_json())
            .unwrap();
    let mut statistics = TilesetStatistics::default();

    let mut failed_urls = Vec::new();
    let scheduled = content.request_inner_contents(
        &mut statistics,
        10,
        "http://example.com",
        |uri| {
            if uri == "batched.b3dm" {
                Err("404 Not Found".to_string())
            } else {
                Ok(vec![0; 8])
            }
        },
        |url, msg| {
            failed_urls.push((url, msg));
        },
    );

    assert!(scheduled);
    assert_eq!(statistics.number_of_pending_requests, 0);
    assert_eq!(failed_urls.len(), 1);
    assert_eq!(failed_urls[0].0, "batched.b3dm");
    assert_eq!(failed_urls[0].1, "404 Not Found");
    // Valid tiles are still created
    assert_eq!(content.inner_contents.len(), 2);
}

#[test]
fn test_cancel_requests_cancels_in_flight() {
    let mut content =
        Multiple3DTileContent::new("http://example.com/tileset.json", &sample_contents_json())
            .unwrap();
    let mut statistics = TilesetStatistics::default();

    content.requests_in_flight = 3;
    statistics.number_of_pending_requests = 3;

    content.cancel_requests(&mut statistics);

    assert_eq!(content.cancel_count, 1);
    assert_eq!(content.requests_in_flight, 0);
    assert_eq!(statistics.number_of_pending_requests, 0);
    assert_eq!(statistics.number_of_attempted_requests, 3);
}

#[test]
fn test_becomes_ready_and_renders_multiple_contents() {
    let mut content =
        Multiple3DTileContent::new("http://example.com/tileset.json", &sample_contents_json())
            .unwrap();
    let mut statistics = TilesetStatistics::default();

    content.request_inner_contents(
        &mut statistics,
        10,
        "http://example.com",
        |_uri| Ok(vec![0; 16]),
        |_u, _m| {},
    );

    assert!(content.ready);
    assert_eq!(content.features_length(), 0);
    assert_eq!(content.points_length(), 0);
    assert_eq!(content.triangles_length(), 0);
    assert_eq!(content.geometry_byte_length(), 0);
    assert_eq!(content.textures_byte_length(), 0);
    assert_eq!(content.batch_table_byte_length(), 0);
    assert_eq!(content.url(), None);

    // Setup bounds and colors to test picking and rendering:
    // Content 0 (building): center at origin, radius 10, initially yellow
    content.inner_contents[0].bounding_volume = Some(BoundingSphere::new(DVec3::ZERO, 10.0));
    content.inner_contents[0].color = Some(Color::YELLOW);

    // Content 1 (instance): center at origin, radius 5, initially green
    content.inner_contents[1].bounding_volume = Some(BoundingSphere::new(DVec3::ZERO, 5.0));
    content.inner_contents[1].color = Some(Color::GREEN);

    let ray_origin = DVec3::new(0.0, 0.0, -20.0);
    let ray_dir = DVec3::Z;

    // Pick building first
    let picked = content.pick(ray_origin, ray_dir);
    assert_eq!(picked, Some(0));

    // Rendered color is yellow
    let rgba = content.simulate_rendered_rgba();
    assert!(rgba[0] > 0);
    assert!(rgba[1] > 0);
    assert_eq!(rgba[2], 0);
    assert_eq!(rgba[3], 255);

    // Hide building, pick instance
    content.inner_contents[0].show = false;
    let picked_instance = content.pick(ray_origin, ray_dir);
    assert_eq!(picked_instance, Some(1));

    // Rendered color is now green
    let rgba_green = content.simulate_rendered_rgba();
    assert_eq!(rgba_green[0], 0);
    assert!(rgba_green[1] > 0);
    assert_eq!(rgba_green[2], 0);
    assert_eq!(rgba_green[3], 255);

    // Hide instance -> blank render
    content.inner_contents[1].show = false;
    content.inner_contents[2].show = false;
    let rgba_blank = content.simulate_rendered_rgba();
    assert_eq!(rgba_blank, [0, 0, 0, 255]);
}

#[test]
fn test_group_metadata_invariants_and_inner_contents() {
    let mut content =
        Multiple3DTileContent::new("http://example.com/tileset.json", &sample_contents_json())
            .unwrap();

    // Group metadata on multiple content itself is always None
    assert!(content.group().is_none());

    let empty_class = MetadataClass::from_json(&json!({
        "id": "test",
        "class": {}
    }))
    .unwrap();

    // Assigning group metadata to multiple content throws DeveloperError
    let dummy_group = Cesium3DContentGroup::new(
        GroupMetadata::new(GroupMetadataOptions {
            id: Some("grp".to_string()),
            group: Some(json!({"properties": {}})),
            class_def: Some(empty_class),
        })
        .unwrap(),
    );
    let err = content.set_group(dummy_group);
    assert!(err.is_err());
    assert!(err
        .unwrap_err()
        .message
        .contains("cannot have group metadata"));

    let meta_class = MetadataClass::from_json(&json!({
        "id": "buildings",
        "class": {
            "properties": {
                "color": {
                    "type": "VEC3",
                    "componentType": "FLOAT32"
                },
                "priority": {
                    "type": "SCALAR",
                    "componentType": "INT32"
                },
                "isInstanced": {
                    "type": "BOOLEAN"
                }
            }
        }
    }))
    .unwrap();

    let group_meta = GroupMetadata::new(GroupMetadataOptions {
        id: Some("buildingGroup".to_string()),
        group: Some(json!({
            "properties": {
                "color": [255.0, 127.0, 0.0],
                "priority": 10,
                "isInstanced": false
            }
        })),
        class_def: Some(meta_class),
    })
    .unwrap();

    let mut statistics = TilesetStatistics::default();
    content.request_inner_contents(
        &mut statistics,
        10,
        "http://example.com",
        |_uri| Ok(vec![0; 4]),
        |_u, _m| {},
    );

    content.inner_contents[0].group = Some(Cesium3DContentGroup::new(group_meta));
    let inner_group = content.inner_contents[0].group.as_ref().unwrap();
    assert_eq!(
        inner_group.metadata.get_property(Some("priority")).unwrap(),
        Some(json!(10))
    );
    assert_eq!(
        inner_group
            .metadata
            .get_property(Some("isInstanced"))
            .unwrap(),
        Some(json!(false))
    );
}

#[test]
fn test_content_metadata_invariants_and_inner_contents() {
    let mut content =
        Multiple3DTileContent::new("http://example.com/tileset.json", &sample_contents_json())
            .unwrap();

    // Metadata on Multiple3DTileContent itself returns None
    assert!(content.metadata().is_none());

    let meta_class = MetadataClass::from_json(&json!({
        "id": "authorClass",
        "class": {
            "properties": {
                "author": {
                    "type": "STRING"
                }
            }
        }
    }))
    .unwrap();

    let dummy_metadata = ContentMetadata::new(ContentMetadataOptions {
        content: Some(json!({
            "properties": {
                "author": "Cesium"
            }
        })),
        class_def: Some(meta_class.clone()),
    })
    .unwrap();

    // Assigning content metadata throws DeveloperError
    let err = content.set_metadata(dummy_metadata);
    assert!(err.is_err());
    assert!(err.unwrap_err().message.contains("cannot have metadata"));

    // Setting content metadata on inner contents works correctly
    let mut statistics = TilesetStatistics::default();
    content.request_inner_contents(
        &mut statistics,
        10,
        "http://example.com",
        |_uri| Ok(vec![0; 4]),
        |_u, _m| {},
    );

    let inner_metadata = ContentMetadata::new(ContentMetadataOptions {
        content: Some(json!({
            "properties": {
                "author": "Sample Author"
            }
        })),
        class_def: Some(meta_class),
    })
    .unwrap();

    content.inner_contents[0].metadata = Some(inner_metadata);
    let meta = content.inner_contents[0].metadata.as_ref().unwrap();
    assert_eq!(meta.get_property("author").unwrap(), json!("Sample Author"));
}

#[test]
fn test_destroys() {
    let mut content =
        Multiple3DTileContent::new("http://example.com/tileset.json", &sample_contents_json())
            .unwrap();
    assert!(!content.is_destroyed());
    content.destroy();
    assert!(content.is_destroyed());
    assert!(content.inner_contents.is_empty());
}
