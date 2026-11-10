//! Tests for ModelClippingPolygonsPipelineStage matching CesiumJS `Scene/Model/ModelClippingPolygonsPipelineStageSpec.js` (Apache-2.0).

use cesium_math::{Cartesian2, Cartesian3, Cartographic, Rectangle, EPSILON7};
use cesium_renderer::clipping_polygon::{ClippingPolygon, ClippingPolygonOptions};
use cesium_renderer::clipping_polygon_collection::{
    ClippingPolygonCollection, ClippingPolygonCollectionOptions,
};
use cesium_renderer::model_clipping_polygons_pipeline_stage::{
    ModelClippingPolygonData, ModelClippingPolygonsFrameState, ModelClippingPolygonsModel,
    ModelClippingPolygonsPipelineStage, ModelClippingPolygonsRenderResources,
};

fn process_with_rectangle(
    rectangle: Rectangle,
) -> (
    ModelClippingPolygonsRenderResources,
    ModelClippingPolygonsFrameState,
    ModelClippingPolygonsModel,
) {
    let positions = vec![
        Cartesian3::new(-1.3194369277314022, 0.6988062530900625, 0.0),
        Cartesian3::new(-1.31941, 0.69879, 0.0),
        Cartesian3::new(-1.3193931220959367, 0.698743632490865, 0.0),
    ];

    let clipping_polygons = ClippingPolygonCollection::new(ClippingPolygonCollectionOptions {
        polygons: Some(vec![ClippingPolygon::new(ClippingPolygonOptions {
            positions: Some(positions),
            ..Default::default()
        })
        .unwrap()]),
        ..Default::default()
    });

    let mut model = ModelClippingPolygonsModel::new(clipping_polygons);
    model.clipping_polygon_data = Some(ModelClippingPolygonData {
        rectangle: Some(rectangle),
        ..Default::default()
    });

    let frame_state = ModelClippingPolygonsFrameState::default();
    let mut render_resources = ModelClippingPolygonsRenderResources::new(model.clone());

    ModelClippingPolygonsPipelineStage::process(&mut render_resources, &model, &frame_state);

    (render_resources, frame_state, model)
}

fn set_eye(
    render_resources: &mut ModelClippingPolygonsRenderResources,
    model: &ModelClippingPolygonsModel,
    frame_state: &mut ModelClippingPolygonsFrameState,
    longitude_deg: f64,
    latitude_deg: f64,
) {
    frame_state.context.uniform_state.eye_cartographic = Cartesian3::new(
        longitude_deg.to_radians(),
        latitude_deg.to_radians(),
        0.0,
    );
    ModelClippingPolygonsPipelineStage::process(render_resources, model, frame_state);
}

#[test]
fn test_maps_eye_position_to_its_uv_within_clipping_rectangle() {
    let rectangle = Rectangle::from_degrees(-10.0, -20.0, 10.0, 20.0);
    let (mut render_resources, mut frame_state, model) = process_with_rectangle(rectangle);

    set_eye(&mut render_resources, &model, &mut frame_state, 5.0, 10.0);
    let uv = render_resources.uniform_map.u_clipping_camera_uv();

    // west=-10, width=20  -> u = (5 - (-10)) / 20 = 0.75
    // south=-20, height=40 -> v = (10 - (-20)) / 40 = 0.75
    let expected = Cartesian2::new(0.75, 0.75);
    assert!(uv.equals_epsilon(&expected, Some(EPSILON7), None));
}

#[test]
fn test_wraps_longitude_for_clipping_rectangles_that_cross_antimeridian() {
    let rectangle = Rectangle::from_degrees(170.0, -10.0, -170.0, 10.0);
    let (mut render_resources, mut frame_state, model) = process_with_rectangle(rectangle);

    // An eye at 185 degrees longitude, expressed in [-180, 180] as -175.
    set_eye(&mut render_resources, &model, &mut frame_state, -175.0, 0.0);
    let uv = render_resources.uniform_map.u_clipping_camera_uv();

    // The rectangle spans 170 -> 190; the eye sits 15 degrees in -> u = 0.75.
    let expected = Cartesian2::new(0.75, 0.5);
    assert!(uv.equals_epsilon(&expected, Some(EPSILON7), None));
}

#[test]
fn test_uses_eye_of_pass_being_rendered_not_scene_camera() {
    let rectangle = Rectangle::from_degrees(-10.0, -20.0, 10.0, 20.0);
    let (mut render_resources, mut frame_state, model) = process_with_rectangle(rectangle);

    frame_state.camera.position_cartographic = Cartographic::from_degrees(5.0, 10.0, None);
    set_eye(&mut render_resources, &model, &mut frame_state, -5.0, -10.0);
    let uv = render_resources.uniform_map.u_clipping_camera_uv();

    // u = (-5 - (-10)) / 20 = 0.25, v = (-10 - (-20)) / 40 = 0.25
    let expected = Cartesian2::new(0.25, 0.25);
    assert!(uv.equals_epsilon(&expected, Some(EPSILON7), None));
}
