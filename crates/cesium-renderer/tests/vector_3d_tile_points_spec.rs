use cesium_math::{Cartographic, Color, GeoRectangle, HeightReference};
use cesium_renderer::commands::Pass;
use cesium_renderer::model_3d_tile_content::Cesium3DTileStyle;
use cesium_renderer::scene_primitive::{PrimitivePasses, PrimitiveRenderFrameState};
use cesium_renderer::vector_3d_tile_points::{
    decode_positions, encode_positions, un_zig_zag, zig_zag, Cesium3DTilePointFeature,
    Vector3DTilePoints, Vector3DTilePointsOptions,
};

#[test]
fn test_zig_zag_roundtrip() {
    for val in [-1000, -100, -1, 0, 1, 100, 1000] {
        let enc = zig_zag(val);
        let dec = un_zig_zag(enc);
        assert_eq!(dec, val, "Zig-zag roundtrip mismatch for {}", val);
    }
}

#[test]
fn test_renders_single_point() {
    let rectangle = GeoRectangle::from_degrees(-40.0, -40.0, 40.0, 40.0);
    let min_height = 0.0;
    let max_height = 100.0;
    let carto = vec![Cartographic::from_degrees(0.0, 0.0, Some(10.0))];

    let encoded = encode_positions(&rectangle, min_height, max_height, &carto);
    let decoded = decode_positions(&rectangle, min_height, max_height, &encoded);
    assert_eq!(decoded.len(), 1);

    let mut points = Vector3DTilePoints::new(Vector3DTilePointsOptions {
        positions: encoded,
        batch_ids: vec![0],
        rectangle,
        minimum_height: min_height,
        maximum_height: max_height,
        height_reference: Some(HeightReference::None),
    });

    assert!(points.ready);
    assert_eq!(points.points_length, 1);
    assert_eq!(points.features.len(), 1);

    let mut features = Vec::new();
    points.create_features(&mut features);
    assert_eq!(features.len(), 1);
    assert_eq!(features[0].batch_id, 0);

    let mut frame_state = PrimitiveRenderFrameState {
        passes: PrimitivePasses {
            render: true,
            pick: false,
        },
        ..Default::default()
    };
    points.update(&mut frame_state);
    assert_eq!(frame_state.command_list.len(), 1);
    assert_eq!(frame_state.command_list[0].pass, Some(Pass::Cesium3dTile));
}

#[test]
fn test_renders_multiple_points() {
    let rectangle = GeoRectangle::from_degrees(-40.0, -40.0, 40.0, 40.0);
    let min_height = 0.0;
    let max_height = 100.0;
    let carto = vec![
        Cartographic::from_degrees(0.0, 0.0, Some(10.0)),
        Cartographic::from_degrees(5.0, 0.0, Some(20.0)),
        Cartographic::from_degrees(-5.0, 0.0, Some(1.0)),
        Cartographic::from_degrees(0.0, 6.0, Some(5.0)),
        Cartographic::from_degrees(0.0, -6.0, Some(90.0)),
    ];

    let encoded = encode_positions(&rectangle, min_height, max_height, &carto);
    let points = Vector3DTilePoints::new(Vector3DTilePointsOptions {
        positions: encoded,
        batch_ids: vec![0, 1, 2, 3, 4],
        rectangle,
        minimum_height: min_height,
        maximum_height: max_height,
        height_reference: None,
    });

    assert_eq!(points.points_length, 5);
    assert_eq!(points.features.len(), 5);
}

#[test]
fn test_picks_a_point() {
    let rectangle = GeoRectangle::from_degrees(-40.0, -40.0, 40.0, 40.0);
    let encoded = encode_positions(
        &rectangle,
        0.0,
        100.0,
        &[Cartographic::from_degrees(0.0, 0.0, Some(10.0))],
    );

    let mut points = Vector3DTilePoints::new(Vector3DTilePointsOptions {
        positions: encoded,
        batch_ids: vec![0],
        rectangle,
        minimum_height: 0.0,
        maximum_height: 100.0,
        height_reference: None,
    });

    let mut frame_state = PrimitiveRenderFrameState {
        passes: PrimitivePasses {
            render: false,
            pick: true,
        },
        ..Default::default()
    };
    points.update(&mut frame_state);
    assert_eq!(frame_state.command_list.len(), 1);
}

#[test]
fn test_renders_multiple_points_with_style() {
    let rectangle = GeoRectangle::from_degrees(-40.0, -40.0, 40.0, 40.0);
    let encoded = encode_positions(
        &rectangle,
        0.0,
        100.0,
        &[
            Cartographic::from_degrees(0.0, 0.0, Some(10.0)),
            Cartographic::from_degrees(1.0, 1.0, Some(10.0)),
        ],
    );

    let mut points = Vector3DTilePoints::new(Vector3DTilePointsOptions {
        positions: encoded,
        batch_ids: vec![0, 1],
        rectangle,
        minimum_height: 0.0,
        maximum_height: 100.0,
        height_reference: None,
    });

    let style = Cesium3DTileStyle {
        show_expression: Some("true".to_string()),
        color_expression: Some("color('red')".to_string()),
        ..Default::default()
    };
    points.apply_style(Some(&style));
    assert!(points.features[0].show);
    assert_eq!(points.features[0].color, Color::RED);
    assert_eq!(points.features[1].color, Color::RED);

    // Reset style
    points.apply_style(None);
    assert_eq!(points.features[0].color, Cesium3DTilePointFeature::DEFAULT_COLOR);
}

#[test]
fn test_renders_multiple_points_with_debug_color() {
    let rectangle = GeoRectangle::from_degrees(-40.0, -40.0, 40.0, 40.0);
    let encoded = encode_positions(
        &rectangle,
        0.0,
        100.0,
        &[Cartographic::from_degrees(0.0, 0.0, Some(10.0))],
    );

    let mut points = Vector3DTilePoints::new(Vector3DTilePointsOptions {
        positions: encoded,
        batch_ids: vec![0],
        rectangle,
        minimum_height: 0.0,
        maximum_height: 100.0,
        height_reference: None,
    });

    assert_eq!(points.highlight_color, Color::WHITE);
    points.apply_debug_settings(true, Color::BLUE);
    assert_eq!(points.highlight_color, Color::BLUE);
    points.apply_debug_settings(false, Color::BLUE);
    assert_eq!(points.highlight_color, Color::WHITE);
}

#[test]
fn test_is_destroyed_and_destroy() {
    let rectangle = GeoRectangle::from_degrees(-40.0, -40.0, 40.0, 40.0);
    let encoded = encode_positions(
        &rectangle,
        0.0,
        100.0,
        &[Cartographic::from_degrees(0.0, 0.0, Some(10.0))],
    );

    let mut points = Vector3DTilePoints::new(Vector3DTilePointsOptions {
        positions: encoded,
        batch_ids: vec![0],
        rectangle,
        minimum_height: 0.0,
        maximum_height: 100.0,
        height_reference: None,
    });

    assert!(!points.is_destroyed());
    points.destroy();
    assert!(points.is_destroyed());
    assert!(!points.ready);
}
