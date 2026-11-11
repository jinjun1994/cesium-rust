use cesium_math::Color;
use cesium_renderer::commands::Pass;
use cesium_renderer::model_3d_tile_content::Cesium3DTileStyle;
use cesium_renderer::scene_primitive::{PrimitivePasses, PrimitiveRenderFrameState};
use cesium_renderer::vector_3d_tile_content::{
    generate_vector_tile_buffer, Vector3DTileContent, VectorTileBufferOptions,
};

#[test]
fn test_throws_with_invalid_version() {
    let buf = generate_vector_tile_buffer(&VectorTileBufferOptions {
        version: Some(2),
        ..Default::default()
    });
    let result = Vector3DTileContent::from_bytes(&buf, None);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        err.message
            .contains("Only Vector tile version 1 is supported"),
        "Unexpected error: {:?}",
        err
    );
}

#[test]
fn test_throws_with_empty_feature_table() {
    let buf = generate_vector_tile_buffer(&VectorTileBufferOptions {
        define_feature_table: Some(false),
        ..Default::default()
    });
    let result = Vector3DTileContent::from_bytes(&buf, None);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        err.message
            .contains("Feature table must have a byte length greater than zero"),
        "Unexpected error: {:?}",
        err
    );
}

#[test]
fn test_throws_without_region() {
    let buf = generate_vector_tile_buffer(&VectorTileBufferOptions {
        define_region: Some(false),
        polygons_length: Some(1),
        ..Default::default()
    });
    let result = Vector3DTileContent::from_bytes(&buf, None);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        err.message
            .contains("Feature table global property: REGION must be defined"),
        "Unexpected error: {:?}",
        err
    );
}

#[test]
fn test_throws_without_all_batch_ids() {
    let buf = generate_vector_tile_buffer(&VectorTileBufferOptions {
        polygons_length: Some(1),
        points_length: Some(1),
        polylines_length: Some(1),
        polygon_batch_ids: Some(vec![1]),
        point_batch_ids: Some(vec![0]),
        polyline_batch_ids: None, // Missing one group!
        ..Default::default()
    });
    let result = Vector3DTileContent::from_bytes(&buf, None);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        err.message
            .contains("If one group of batch ids is defined, then all batch ids must be defined"),
        "Unexpected error: {:?}",
        err
    );
}

#[test]
fn test_throws_when_calling_get_feature_with_invalid_index() {
    let buf = generate_vector_tile_buffer(&VectorTileBufferOptions {
        polygons_length: Some(2),
        ..Default::default()
    });
    let content = Vector3DTileContent::from_bytes(&buf, Some("http://localhost/tile.vctr")).unwrap();
    assert_eq!(content.features_length, 2);

    assert!(content.get_feature(0).is_ok());
    assert!(content.get_feature(1).is_ok());

    let err = content.get_feature(2).unwrap_err();
    assert!(err.message.contains("batchId is required and must be greater than or equal to zero and less than the number of features: 2"));

    let err2 = content.get_feature(1000).unwrap_err();
    assert!(err2.message.contains("batchId is required"));
}

#[test]
fn test_parse_and_access_properties() {
    let mut bt_map = serde_json::Map::new();
    bt_map.insert(
        "name".to_string(),
        serde_json::json!(["Building A", "Building B"]),
    );
    bt_map.insert("height".to_string(), serde_json::json!([12.5, 45.0]));

    let buf = generate_vector_tile_buffer(&VectorTileBufferOptions {
        polygons_length: Some(2),
        batch_table_json: Some(serde_json::Value::Object(bt_map)),
        ..Default::default()
    });

    let content = Vector3DTileContent::from_bytes(&buf, Some("http://localhost/tile.vctr")).unwrap();
    assert_eq!(content.features_length, 2);
    assert_eq!(content.polygons_length, 2);
    assert_eq!(content.triangles_length, 4);
    assert_eq!(content.url, "http://localhost/tile.vctr");
    assert!(!content.is_destroyed());

    let f0 = content.get_feature(0).unwrap();
    assert_eq!(
        f0.get_property("name"),
        Some(&serde_json::Value::String("Building A".to_string()))
    );
    assert_eq!(
        f0.get_property("height"),
        Some(&serde_json::Value::from(12.5))
    );
    assert!(content.has_property(0, "name"));
    assert!(!content.has_property(0, "unknown"));

    let f1 = content.get_feature(1).unwrap();
    assert_eq!(
        f1.get_property("name"),
        Some(&serde_json::Value::String("Building B".to_string()))
    );
}

#[test]
fn test_styles_for_show_and_color() {
    let buf = generate_vector_tile_buffer(&VectorTileBufferOptions {
        points_length: Some(2),
        polygons_length: Some(1),
        ..Default::default()
    });
    let mut content = Vector3DTileContent::from_bytes(&buf, None).unwrap();
    assert_eq!(content.features_length, 3);

    // Initial state
    assert!(content.get_feature(0).unwrap().show);
    assert_eq!(content.get_feature(0).unwrap().color, Color::WHITE);

    // Apply hide style
    let hide_style = Cesium3DTileStyle {
        show_expression: Some("false".to_string()),
        ..Default::default()
    };
    content.apply_style(Some(&hide_style));
    assert!(!content.get_feature(0).unwrap().show);
    assert!(!content.get_feature(2).unwrap().show);

    // Apply color style
    let color_style = Cesium3DTileStyle {
        show_expression: Some("true".to_string()),
        color_expression: Some("color('red')".to_string()),
        ..Default::default()
    };
    content.apply_style(Some(&color_style));
    assert!(content.get_feature(0).unwrap().show);
    assert_eq!(content.get_feature(0).unwrap().color, Color::RED);

    // Reset style to None
    content.apply_style(None);
    assert!(content.get_feature(0).unwrap().show);
    assert_eq!(content.get_feature(0).unwrap().color, Color::WHITE);
}

#[test]
fn test_update_and_destroy() {
    let buf = generate_vector_tile_buffer(&VectorTileBufferOptions {
        points_length: Some(1),
        ..Default::default()
    });
    let mut content = Vector3DTileContent::from_bytes(&buf, None).unwrap();

    let mut frame_state = PrimitiveRenderFrameState {
        passes: PrimitivePasses {
            render: true,
            pick: false,
        },
        ..Default::default()
    };

    content.update(&mut frame_state);
    assert_eq!(frame_state.command_list.len(), 1);
    assert_eq!(frame_state.command_list[0].pass, Some(Pass::Cesium3dTile));

    // Destroy
    assert!(!content.is_destroyed());
    content.destroy();
    assert!(content.is_destroyed());

    // Update after destroy produces no commands
    frame_state.command_list.clear();
    content.update(&mut frame_state);
    assert!(frame_state.command_list.is_empty());
}
