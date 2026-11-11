use cesium_math::{BoundingSphere, Cartesian3, Cartographic, Color, GeoRectangle};
use cesium_renderer::commands::Pass;
use cesium_renderer::model_3d_tile_content::Cesium3DTileStyle;
use cesium_renderer::scene_primitive::{PrimitivePasses, PrimitiveRenderFrameState};
use cesium_renderer::vector_3d_tile_points::encode_positions;
use cesium_renderer::vector_3d_tile_polylines::{
    Vector3DTilePolylines, Vector3DTilePolylinesOptions,
};

#[test]
fn test_renders_single_polyline() {
    let rectangle = GeoRectangle::from_degrees(-40.0, -40.0, 40.0, 40.0);
    let min_height = 0.0;
    let max_height = 5.0;
    let carto_positions = vec![
        Cartographic::from_degrees(0.0, 0.0, Some(1.0)),
        Cartographic::from_degrees(1.0, 0.0, Some(2.0)),
    ];
    let positions = encode_positions(&rectangle, min_height, max_height, &carto_positions);

    let mut polylines = Vector3DTilePolylines::new(Vector3DTilePolylinesOptions {
        positions,
        widths: vec![10],
        counts: vec![2],
        batch_ids: vec![0],
        rectangle,
        minimum_height: min_height,
        maximum_height: max_height,
        center: Some(Cartesian3::ZERO),
        bounding_volume: Some(BoundingSphere::new(glam::DVec3::ZERO, 1000000.0)),
        keep_decoded_positions: true,
    });

    assert!(polylines.ready);
    assert_eq!(polylines.triangles_length, 2); // 1 segment -> 2 triangles
    assert_eq!(polylines.counts.len(), 1);

    let decoded_positions = polylines.get_positions(0);
    assert!(decoded_positions.is_some());
    assert_eq!(decoded_positions.unwrap().len(), 2);

    let mut frame_state = PrimitiveRenderFrameState {
        passes: PrimitivePasses {
            render: true,
            pick: false,
        },
        ..Default::default()
    };
    polylines.update(&mut frame_state);
    assert_eq!(frame_state.command_list.len(), 1);
    assert_eq!(frame_state.command_list[0].pass, Some(Pass::Translucent));
}

#[test]
fn test_renders_multiple_polylines() {
    let rectangle = GeoRectangle::from_degrees(-40.0, -40.0, 40.0, 40.0);
    let min_height = 0.0;
    let max_height = 100.0;
    let carto_positions = vec![
        Cartographic::from_degrees(1.0, 0.0, Some(1.0)),
        Cartographic::from_degrees(2.0, 0.0, Some(2.0)),
        Cartographic::from_degrees(-6.0, 0.0, Some(12.0)),
        Cartographic::from_degrees(-5.0, 0.0, Some(15.0)),
        Cartographic::from_degrees(0.0, 10.0, Some(0.0)),
        Cartographic::from_degrees(0.0, 5.0, Some(5.0)),
        Cartographic::from_degrees(0.0, 0.0, Some(10.0)),
        Cartographic::from_degrees(0.0, -5.0, Some(15.0)),
    ];
    let positions = encode_positions(&rectangle, min_height, max_height, &carto_positions);

    let polylines = Vector3DTilePolylines::new(Vector3DTilePolylinesOptions {
        positions,
        widths: vec![10, 10, 10],
        counts: vec![2, 2, 4],
        batch_ids: vec![0, 1, 2],
        rectangle,
        minimum_height: min_height,
        maximum_height: max_height,
        center: Some(Cartesian3::ZERO),
        bounding_volume: None,
        keep_decoded_positions: true,
    });

    assert_eq!(polylines.counts.len(), 3);
    // line 0: 1 segment -> 2 triangles
    // line 1: 1 segment -> 2 triangles
    // line 2: 3 segments -> 6 triangles
    assert_eq!(polylines.triangles_length, 10);

    let pos0 = polylines.get_positions(0).unwrap();
    assert_eq!(pos0.len(), 2);

    let pos1 = polylines.get_positions(1).unwrap();
    assert_eq!(pos1.len(), 2);

    let pos2 = polylines.get_positions(2).unwrap();
    assert_eq!(pos2.len(), 4);

    assert!(polylines.get_positions(99).is_none());
}

#[test]
fn test_picks_a_polyline() {
    let rectangle = GeoRectangle::from_degrees(-40.0, -40.0, 40.0, 40.0);
    let carto_positions = vec![
        Cartographic::from_degrees(0.0, 0.0, Some(1.0)),
        Cartographic::from_degrees(1.0, 0.0, Some(2.0)),
    ];
    let positions = encode_positions(&rectangle, 0.0, 5.0, &carto_positions);

    let mut polylines = Vector3DTilePolylines::new(Vector3DTilePolylinesOptions {
        positions,
        widths: vec![10],
        counts: vec![2],
        batch_ids: vec![0],
        rectangle,
        minimum_height: 0.0,
        maximum_height: 5.0,
        center: None,
        bounding_volume: None,
        keep_decoded_positions: false,
    });

    let mut features = Vec::new();
    polylines.create_features(&mut features);
    assert_eq!(features.len(), 1);
    assert_eq!(features[0].batch_id, 0);

    let mut frame_state = PrimitiveRenderFrameState {
        passes: PrimitivePasses {
            render: false,
            pick: true,
        },
        ..Default::default()
    };
    polylines.update(&mut frame_state);
    assert_eq!(frame_state.command_list.len(), 1);
}

#[test]
fn test_apply_style_and_debug_settings() {
    let rectangle = GeoRectangle::from_degrees(-40.0, -40.0, 40.0, 40.0);
    let carto_positions = vec![
        Cartographic::from_degrees(0.0, 0.0, Some(1.0)),
        Cartographic::from_degrees(1.0, 0.0, Some(2.0)),
    ];
    let positions = encode_positions(&rectangle, 0.0, 5.0, &carto_positions);

    let mut polylines = Vector3DTilePolylines::new(Vector3DTilePolylinesOptions {
        positions,
        widths: vec![10],
        counts: vec![2],
        batch_ids: vec![0],
        rectangle,
        minimum_height: 0.0,
        maximum_height: 5.0,
        center: None,
        bounding_volume: None,
        keep_decoded_positions: false,
    });

    let mut features = Vec::new();
    polylines.create_features(&mut features);

    let style = Cesium3DTileStyle {
        show_expression: Some("true".to_string()),
        color_expression: Some("color('blue')".to_string()),
        ..Default::default()
    };
    polylines.apply_style(Some(&style), &mut features);
    assert_eq!(features[0].color, Color::BLUE);

    polylines.apply_debug_settings(true, Color::RED);
    assert_eq!(polylines.highlight_color, Color::RED);
    polylines.apply_debug_settings(false, Color::RED);
    assert_eq!(polylines.highlight_color, Color::WHITE);
}

#[test]
fn test_is_destroyed_and_destroy() {
    let mut polylines = Vector3DTilePolylines::new(Vector3DTilePolylinesOptions::default());
    assert!(!polylines.is_destroyed());
    polylines.destroy();
    assert!(polylines.is_destroyed());
    assert!(!polylines.ready);
}
