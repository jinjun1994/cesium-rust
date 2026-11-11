//! SceneTransformsSpec matching CesiumJS `Scene/SceneTransformsSpec.js` (Apache-2.0).

use cesium_math::{
    BoundingRectangle, Cartographic, Ellipsoid, GeographicProjection, SceneMode,
};
use cesium_renderer::{SceneTransforms, WorldToWindowCoordinatesOptions};
use glam::{DMat4, DVec2, DVec3};

#[test]
fn test_actual_ellipsoid_position_to_cartographic_in_3d() {
    let projection = GeographicProjection::default();
    let cartographic = Cartographic::degrees(-75.0, 40.0, 100.0).unwrap();
    let position = Ellipsoid::WGS84.cartographic_to_cartesian(cartographic).unwrap();

    let result = SceneTransforms::actual_ellipsoid_position_to_cartographic(
        SceneMode::Scene3D,
        &projection,
        position,
    )
    .unwrap();

    assert!((result.longitude - cartographic.longitude).abs() < 1e-8);
    assert!((result.latitude - cartographic.latitude).abs() < 1e-8);
    assert!((result.height - cartographic.height).abs() < 1e-4);
}

#[test]
fn test_actual_ellipsoid_position_to_cartographic_in_columbus_view() {
    let projection = GeographicProjection::default();
    let cartographic = Cartographic::degrees(-75.0, 40.0, 100.0).unwrap();
    let projected = projection.project(cartographic).unwrap();

    // The pick result in 2D/CV is laid out as (height, easting, northing).
    let position = DVec3::new(projected.z, projected.x, projected.y);

    let result = SceneTransforms::actual_ellipsoid_position_to_cartographic(
        SceneMode::ColumbusView,
        &projection,
        position,
    )
    .unwrap();

    assert!((result.longitude - cartographic.longitude).abs() < 1e-8);
    assert!((result.latitude - cartographic.latitude).abs() < 1e-8);
    assert!((result.height - cartographic.height).abs() < 1e-4);
}

#[test]
fn test_compute_actual_ellipsoid_position() {
    let projection = GeographicProjection::default();
    let cartographic = Cartographic::degrees(0.0, 0.0, 50.0).unwrap();
    let position = Ellipsoid::WGS84.cartographic_to_cartesian(cartographic).unwrap();

    // In 3D: returns identical position
    let pos_3d = SceneTransforms::compute_actual_ellipsoid_position(
        SceneMode::Scene3D,
        1.0,
        &projection,
        position,
    )
    .unwrap();
    assert_eq!(pos_3d, position);

    // In Columbus View: returns (height, easting, northing)
    let pos_cv = SceneTransforms::compute_actual_ellipsoid_position(
        SceneMode::ColumbusView,
        0.0,
        &projection,
        position,
    )
    .unwrap();
    let projected = projection.project(cartographic).unwrap();
    assert!((pos_cv.x - projected.z).abs() < 1e-6);
    assert!((pos_cv.y - projected.x).abs() < 1e-6);
    assert!((pos_cv.z - projected.y).abs() < 1e-6);

    // In 2D: height flattened to 0.0
    let pos_2d = SceneTransforms::compute_actual_ellipsoid_position(
        SceneMode::Scene2D,
        0.0,
        &projection,
        position,
    )
    .unwrap();
    assert_eq!(pos_2d.x, 0.0);
    assert!((pos_2d.y - projected.x).abs() < 1e-6);
    assert!((pos_2d.z - projected.y).abs() < 1e-6);
}

#[test]
fn test_world_to_window_coordinates_3d() {
    let projection = GeographicProjection::default();

    // Camera at (0, 0, 10_000_000) looking along -Z toward (0, 0, 0), up along +Y
    let eye = DVec3::new(0.0, 0.0, 10_000_000.0);
    let target = DVec3::ZERO;
    let up = DVec3::Y;
    let view_matrix = DMat4::look_at_rh(eye, target, up);

    // Standard perspective projection: 60 deg fov, aspect 1.0, near 1.0, far 20_000_000
    let fovy = 60.0_f64.to_radians();
    let aspect = 1.0;
    let near = 1.0;
    let far = 20_000_000.0;
    let projection_matrix = DMat4::perspective_rh(fovy, aspect, near, far);

    let canvas_width = 800.0;
    let canvas_height = 800.0;

    // Target point in center of screen
    let center_point = DVec3::new(0.0, 0.0, 0.0);
    let coords = SceneTransforms::world_to_window_coordinates(WorldToWindowCoordinatesOptions {
        position: center_point,
        eye_offset: DVec3::ZERO,
        mode: SceneMode::Scene3D,
        morph_time: 1.0,
        map_projection: &projection,
        view_matrix,
        projection_matrix,
        canvas_width,
        canvas_height,
        is_orthographic: false,
    })
    .unwrap()
    .unwrap();

    // In 800x800, center point projects to (400, 400)
    assert!((coords.x - 400.0).abs() < 1.0);
    assert!((coords.y - 400.0).abs() < 1.0);

    // Point behind camera (z = 15_000_000, while camera is at 10_000_000 looking toward -Z)
    let behind_point = DVec3::new(0.0, 0.0, 15_000_000.0);
    let behind_coords = SceneTransforms::world_to_window_coordinates(WorldToWindowCoordinatesOptions {
        position: behind_point,
        eye_offset: DVec3::ZERO,
        mode: SceneMode::Scene3D,
        morph_time: 1.0,
        map_projection: &projection,
        view_matrix,
        projection_matrix,
        canvas_width,
        canvas_height,
        is_orthographic: false,
    })
    .unwrap();

    assert!(behind_coords.is_none());
}

#[test]
fn test_drawing_buffer_coordinates_scale() {
    let window_pos = DVec2::new(100.0, 200.0);
    let canvas_width = 800.0;
    let canvas_height = 600.0;
    let drawing_buffer_width = 1600.0;
    let drawing_buffer_height = 1200.0;

    let drawing_buf_pos = SceneTransforms::transform_window_to_drawing_buffer(
        window_pos,
        canvas_width,
        canvas_height,
        drawing_buffer_width,
        drawing_buffer_height,
    );

    // 2x scaling on high DPI displays
    assert_eq!(drawing_buf_pos.x, 200.0);
    assert_eq!(drawing_buf_pos.y, 400.0);
}

#[test]
fn test_drawing_buffer_to_world_roundtrip() {
    let eye = DVec3::new(0.0, 0.0, 100.0);
    let target = DVec3::ZERO;
    let up = DVec3::Y;
    let view_matrix = DMat4::look_at_rh(eye, target, up);

    let fovy = 60.0_f64.to_radians();
    let aspect = 1.0;
    let near = 1.0;
    let far = 1000.0;
    let projection_matrix = DMat4::perspective_rh(fovy, aspect, near, far);
    let view_projection = projection_matrix * view_matrix;
    let inverse_view_projection = view_projection.inverse();

    let viewport = BoundingRectangle::new(0.0, 0.0, 800.0, 800.0);

    // Center pixel at (400, 400), target at origin
    let target_world = DVec3::new(0.0, 0.0, 0.0);
    let clip = view_projection * glam::DVec4::new(target_world.x, target_world.y, target_world.z, 1.0);
    let ndc_z = clip.z / clip.w;
    let depth = (ndc_z + 1.0) * 0.5;

    let recovered_world = SceneTransforms::drawing_buffer_to_world_coordinates(
        DVec2::new(400.0, 400.0),
        depth,
        viewport,
        &inverse_view_projection,
    );

    assert!((recovered_world.x - target_world.x).abs() < 1e-4);
    assert!((recovered_world.y - target_world.y).abs() < 1e-4);
    assert!((recovered_world.z - target_world.z).abs() < 1e-4);
}
