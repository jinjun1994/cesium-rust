//! Direct port and verification against CesiumJS `Cesium3DTilesetGraphicsSpec.js` (Apache-2.0).

use cesium_math::Cesium3DTilesetGraphics;

#[test]
fn test_creates_expected_instance_from_construction() {
    let tileset = Cesium3DTilesetGraphics::from_constants(
        Some(false),
        Some("0".to_string()),
        Some(2.0),
    );

    let values = tileset.get_value(None);
    assert_eq!(values.uri.as_deref(), Some("0"));
    assert_eq!(values.show, Some(false));
    assert_eq!(values.maximum_screen_space_error, Some(2.0));
}

#[test]
fn test_merge_assigns_unassigned_properties() {
    let source = Cesium3DTilesetGraphics::from_constants(
        Some(true),
        Some("http://example.com/tileset.json".to_string()),
        Some(2.0),
    );

    let mut target = Cesium3DTilesetGraphics::new();
    target.merge(&source);

    let values = target.get_value(None);
    assert_eq!(
        values.uri.as_deref(),
        Some("http://example.com/tileset.json")
    );
    assert_eq!(values.show, Some(true));
    assert_eq!(values.maximum_screen_space_error, Some(2.0));
}

#[test]
fn test_merge_does_not_assign_assigned_properties() {
    let source = Cesium3DTilesetGraphics::from_constants(
        Some(true),
        Some("source_uri".to_string()),
        Some(2.0),
    );

    let mut target = Cesium3DTilesetGraphics::from_constants(
        Some(false),
        Some("target_uri".to_string()),
        Some(5.0),
    );

    target.merge(&source);

    let values = target.get_value(None);
    assert_eq!(values.uri.as_deref(), Some("target_uri"));
    assert_eq!(values.show, Some(false));
    assert_eq!(values.maximum_screen_space_error, Some(5.0));
}

#[test]
fn test_clone_works() {
    let source = Cesium3DTilesetGraphics::from_constants(
        Some(true),
        Some("tileset.json".to_string()),
        Some(2.0),
    );

    let result = source.clone();
    let values = result.get_value(None);
    assert_eq!(values.uri.as_deref(), Some("tileset.json"));
    assert_eq!(values.show, Some(true));
    assert_eq!(values.maximum_screen_space_error, Some(2.0));

    let mut existing_result = Cesium3DTilesetGraphics::new();
    source.clone_to(&mut existing_result);
    let values2 = existing_result.get_value(None);
    assert_eq!(values2.uri.as_deref(), Some("tileset.json"));
    assert_eq!(values2.show, Some(true));
    assert_eq!(values2.maximum_screen_space_error, Some(2.0));
}

#[test]
fn test_merge_throws_if_source_undefined() {
    let mut target = Cesium3DTilesetGraphics::new();
    assert!(target.try_merge(None).is_err());
}
