use cesium_math::{Cartographic, ClassificationType, Color, GeoRectangle};
use cesium_renderer::commands::Pass;
use cesium_renderer::scene_primitive::{PrimitivePasses, PrimitiveRenderFrameState};
use cesium_renderer::vector_3d_tile_polygons::{
    encode_polygon_positions, Vector3DTilePolygons, Vector3DTilePolygonsOptions,
};

#[test]
fn test_renders_single_polygon() {
    let rectangle = GeoRectangle::from_degrees(-40.0, -40.0, 40.0, 40.0);
    let carto = vec![
        Cartographic::from_degrees(-10.0, -10.0, Some(0.0)),
        Cartographic::from_degrees(10.0, -10.0, Some(0.0)),
        Cartographic::from_degrees(10.0, 10.0, Some(0.0)),
        Cartographic::from_degrees(-10.0, 10.0, Some(0.0)),
    ];
    let encoded = encode_polygon_positions(&rectangle, &carto);
    let counts = vec![4];
    let indices = vec![0, 1, 2, 0, 2, 3];
    let index_counts = vec![6];

    let mut polygons = Vector3DTilePolygons::new(Vector3DTilePolygonsOptions {
        positions: encoded,
        counts,
        indices,
        index_counts,
        batch_ids: vec![0],
        rectangle,
        minimum_height: 0.0,
        maximum_height: 100.0,
        ..Default::default()
    });

    assert!(polygons.ready);
    assert_eq!(polygons.triangles_length, 2);
    assert_eq!(polygons.batches.len(), 1);
    assert_eq!(polygons.batches[0].count, 6);

    let mut frame_state = PrimitiveRenderFrameState {
        passes: PrimitivePasses {
            render: true,
            pick: false,
        },
        ..Default::default()
    };
    polygons.update(&mut frame_state);
    assert_eq!(frame_state.command_list.len(), 2);
    assert_eq!(frame_state.command_list[0].pass, Some(Pass::Globe));
    assert_eq!(
        frame_state.command_list[1].pass,
        Some(Pass::Cesium3dTileClassification)
    );
}

#[test]
fn test_renders_multiple_polygons() {
    let rectangle = GeoRectangle::from_degrees(-40.0, -40.0, 40.0, 40.0);
    let carto = vec![
        // Polygon 0 (triangle)
        Cartographic::from_degrees(-10.0, -10.0, Some(0.0)),
        Cartographic::from_degrees(0.0, -10.0, Some(0.0)),
        Cartographic::from_degrees(-5.0, 0.0, Some(0.0)),
        // Polygon 1 (triangle)
        Cartographic::from_degrees(10.0, 10.0, Some(0.0)),
        Cartographic::from_degrees(20.0, 10.0, Some(0.0)),
        Cartographic::from_degrees(15.0, 20.0, Some(0.0)),
    ];
    let encoded = encode_polygon_positions(&rectangle, &carto);
    let counts = vec![3, 3];
    let indices = vec![0, 1, 2, 3, 4, 5];
    let index_counts = vec![3, 3];

    let polygons = Vector3DTilePolygons::new(Vector3DTilePolygonsOptions {
        positions: encoded,
        counts,
        indices,
        index_counts,
        batch_ids: vec![0, 1],
        rectangle,
        minimum_height: 0.0,
        maximum_height: 100.0,
        ..Default::default()
    });

    assert_eq!(polygons.triangles_length, 2);
    assert_eq!(polygons.batches.len(), 2);
}

#[test]
fn test_renders_multiple_polygons_after_rebatch_and_color_update() {
    let rectangle = GeoRectangle::from_degrees(-40.0, -40.0, 40.0, 40.0);
    let carto = vec![
        Cartographic::from_degrees(-10.0, -10.0, Some(0.0)),
        Cartographic::from_degrees(10.0, -10.0, Some(0.0)),
        Cartographic::from_degrees(10.0, 10.0, Some(0.0)),
    ];
    let encoded = encode_polygon_positions(&rectangle, &carto);

    let mut polygons = Vector3DTilePolygons::new(Vector3DTilePolygonsOptions {
        positions: encoded,
        counts: vec![3],
        indices: vec![0, 1, 2],
        index_counts: vec![3],
        batch_ids: vec![0],
        rectangle,
        minimum_height: 0.0,
        maximum_height: 100.0,
        ..Default::default()
    });

    assert_eq!(polygons.batches[0].color, Color::WHITE);
    polygons.update_commands(0, Color::BLUE);
    assert_eq!(polygons.batches[0].color, Color::BLUE);

    polygons.apply_debug_settings(true, Color::RED);
    assert_eq!(polygons.batches[0].color, Color::RED);
}

#[test]
fn test_renders_multiple_polygons_with_different_heights() {
    let rectangle = GeoRectangle::from_degrees(-40.0, -40.0, 40.0, 40.0);
    let carto = vec![
        Cartographic::from_degrees(-10.0, -10.0, Some(0.0)),
        Cartographic::from_degrees(10.0, -10.0, Some(0.0)),
        Cartographic::from_degrees(10.0, 10.0, Some(0.0)),
    ];
    let encoded = encode_polygon_positions(&rectangle, &carto);

    let polygons = Vector3DTilePolygons::new(Vector3DTilePolygonsOptions {
        positions: encoded,
        counts: vec![3],
        indices: vec![0, 1, 2],
        index_counts: vec![3],
        batch_ids: vec![0],
        rectangle,
        minimum_height: 0.0,
        maximum_height: 100.0,
        polygon_minimum_heights: Some(vec![10.0]),
        polygon_maximum_heights: Some(vec![90.0]),
        ..Default::default()
    });

    assert_eq!(polygons.polygon_minimum_heights, Some(vec![10.0]));
    assert_eq!(polygons.polygon_maximum_heights, Some(vec![90.0]));
}

#[test]
fn test_renders_wireframe_and_based_on_classification_type() {
    let rectangle = GeoRectangle::from_degrees(-40.0, -40.0, 40.0, 40.0);
    let carto = vec![
        Cartographic::from_degrees(-10.0, -10.0, Some(0.0)),
        Cartographic::from_degrees(10.0, -10.0, Some(0.0)),
        Cartographic::from_degrees(10.0, 10.0, Some(0.0)),
    ];
    let encoded = encode_polygon_positions(&rectangle, &carto);

    let mut polygons = Vector3DTilePolygons::new(Vector3DTilePolygonsOptions {
        positions: encoded,
        counts: vec![3],
        indices: vec![0, 1, 2],
        index_counts: vec![3],
        batch_ids: vec![0],
        rectangle,
        minimum_height: 0.0,
        maximum_height: 100.0,
        classification_type: Some(ClassificationType::Terrain),
        ..Default::default()
    });

    polygons.debug_wireframe = true;

    let mut frame_state = PrimitiveRenderFrameState {
        passes: PrimitivePasses {
            render: true,
            pick: false,
        },
        ..Default::default()
    };
    polygons.update(&mut frame_state);
    assert_eq!(frame_state.command_list.len(), 1);
    assert_eq!(frame_state.command_list[0].pass, Some(Pass::Globe));
    assert_eq!(
        frame_state.command_list[0].shader_program,
        Some(cesium_renderer::commands::ShaderProgramHandle::Name(
            "Vector3DTilePolygonsWireframe".to_string()
        ))
    );

    // Switch to Cesium3DTile
    frame_state.command_list.clear();
    polygons.classification_type = ClassificationType::Cesium3DTile;
    polygons.debug_wireframe = false;
    polygons.update(&mut frame_state);
    assert_eq!(frame_state.command_list.len(), 1);
    assert_eq!(
        frame_state.command_list[0].pass,
        Some(Pass::Cesium3dTileClassification)
    );
}

#[test]
fn test_picks_and_destroys() {
    let rectangle = GeoRectangle::from_degrees(-40.0, -40.0, 40.0, 40.0);
    let carto = vec![
        Cartographic::from_degrees(-10.0, -10.0, Some(0.0)),
        Cartographic::from_degrees(10.0, -10.0, Some(0.0)),
        Cartographic::from_degrees(10.0, 10.0, Some(0.0)),
    ];
    let encoded = encode_polygon_positions(&rectangle, &carto);

    let mut polygons = Vector3DTilePolygons::new(Vector3DTilePolygonsOptions {
        positions: encoded,
        counts: vec![3],
        indices: vec![0, 1, 2],
        index_counts: vec![3],
        batch_ids: vec![0],
        rectangle,
        minimum_height: 0.0,
        maximum_height: 100.0,
        classification_type: Some(ClassificationType::Both),
        ..Default::default()
    });

    let mut frame_state = PrimitiveRenderFrameState {
        passes: PrimitivePasses {
            render: false,
            pick: true,
        },
        ..Default::default()
    };
    polygons.update(&mut frame_state);
    assert_eq!(frame_state.command_list.len(), 2);

    // Destroy
    assert!(!polygons.is_destroyed());
    polygons.destroy();
    assert!(polygons.is_destroyed());
    assert!(!polygons.ready);

    frame_state.command_list.clear();
    polygons.update(&mut frame_state);
    assert!(frame_state.command_list.is_empty());
}
