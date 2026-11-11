use cesium_math::{ClassificationType, GeoRectangle};
use cesium_renderer::commands::Pass;
use cesium_renderer::scene_primitive::{PrimitivePasses, PrimitiveRenderFrameState};
use cesium_renderer::vector_3d_tile_clamped_polylines::{
    Vector3DTileClampedPolylines, Vector3DTileClampedPolylinesOptions,
};

#[test]
fn test_is_destroyed() {
    let mut polylines =
        Vector3DTileClampedPolylines::new(Vector3DTileClampedPolylinesOptions {
            rectangle: GeoRectangle::default(),
            ..Default::default()
        });

    assert!(!polylines.is_destroyed());
    polylines.destroy();
    assert!(polylines.is_destroyed());
}

#[test]
fn test_renders_clamped_polylines() {
    let mut polylines =
        Vector3DTileClampedPolylines::new(Vector3DTileClampedPolylinesOptions {
            rectangle: GeoRectangle::from_degrees(-40.0, -40.0, 40.0, 40.0),
            classification_type: Some(ClassificationType::Terrain),
            positions: Some(vec![0, 0, 0, 10, 10, 10]),
            ..Default::default()
        });

    assert_eq!(polylines.triangles_length, 24);

    let mut frame_state = PrimitiveRenderFrameState {
        passes: PrimitivePasses {
            render: true,
            pick: false,
        },
        ..Default::default()
    };

    polylines.show = false;
    polylines.update(&mut frame_state);
    assert_eq!(frame_state.command_list.len(), 0);

    polylines.show = true;
    polylines.update(&mut frame_state);
    assert_eq!(frame_state.command_list.len(), 1);
    assert_eq!(frame_state.command_list[0].pass, Some(Pass::Globe));
    assert!(frame_state.command_list[0].depth_for_translucent_classification);
}

#[test]
fn test_picks_clamped_polyline() {
    let mut polylines =
        Vector3DTileClampedPolylines::new(Vector3DTileClampedPolylinesOptions {
            rectangle: GeoRectangle::default(),
            classification_type: Some(ClassificationType::Cesium3DTile),
            ..Default::default()
        });

    let mut frame_state = PrimitiveRenderFrameState {
        passes: PrimitivePasses {
            render: false,
            pick: true,
        },
        ..Default::default()
    };

    polylines.update(&mut frame_state);
    assert_eq!(frame_state.command_list.len(), 1);
    assert_eq!(
        frame_state.command_list[0].pass,
        Some(Pass::Cesium3dTileClassification)
    );
}
