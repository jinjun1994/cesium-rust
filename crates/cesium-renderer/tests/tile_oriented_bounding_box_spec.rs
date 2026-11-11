use cesium_math::{
    Cartesian3, Color, Matrix3, Plane, PlaneIntersection, EPSILON6,
};
use cesium_renderer::tile_oriented_bounding_box::{
    TileOrientedBoundingBox, TileOrientedBoundingBoxCamera, TileOrientedBoundingBoxFrameState,
};

#[test]
fn can_be_instantiated_with_center_and_half_axes() {
    let center = Cartesian3::new(0.0, 0.0, 0.0);
    let half_axes = Matrix3::from_scale(Cartesian3::new(0.5, 0.5, 0.5));
    let tile_bounding_volume = TileOrientedBoundingBox::from_center_half_axes(center, half_axes);

    assert_eq!(tile_bounding_volume.center(), center);
    assert_eq!(tile_bounding_volume.half_axes(), half_axes);
    let bs_center: Cartesian3 = tile_bounding_volume.bounding_sphere().center.into();
    assert_eq!(bs_center, center);
    assert!(tile_bounding_volume.bounding_sphere().radius > 0.5);
    assert!(tile_bounding_volume.bounding_sphere().radius < 1.0);
}

#[test]
fn create_debug_volume_throws_when_color_is_undefined() {
    let center = Cartesian3::new(0.0, 0.0, 0.0);
    let half_axes = Matrix3::from_scale(Cartesian3::new(0.5, 0.5, 0.5));
    let tile_bounding_volume = TileOrientedBoundingBox::from_center_half_axes(center, half_axes);

    assert!(tile_bounding_volume.create_debug_volume(None).is_err());
}

#[test]
fn can_create_debug_volume() {
    let center = Cartesian3::new(0.0, 0.0, 0.0);
    let half_axes = Matrix3::from_scale(Cartesian3::new(0.5, 0.5, 0.5));
    let tile_bounding_volume = TileOrientedBoundingBox::from_center_half_axes(center, half_axes);

    assert!(tile_bounding_volume
        .create_debug_volume(Some(Color::BLUE))
        .is_ok());
}

#[test]
fn distance_to_camera_throws_when_frame_state_is_undefined() {
    let center = Cartesian3::new(0.0, 0.0, 0.0);
    let half_axes = Matrix3::from_scale(Cartesian3::new(0.5, 0.5, 0.5));
    let tile_bounding_volume = TileOrientedBoundingBox::from_center_half_axes(center, half_axes);

    assert!(tile_bounding_volume.distance_to_camera(None).is_err());
}

#[test]
fn has_distance_0_to_camera_if_camera_is_inside() {
    let center = Cartesian3::new(0.0, 0.0, 0.0);
    let half_axes = Matrix3::from_scale(Cartesian3::new(0.5, 0.5, 0.5));
    let tile_bounding_volume = TileOrientedBoundingBox::from_center_half_axes(center, half_axes);

    let mut frame_state = TileOrientedBoundingBoxFrameState::default();

    frame_state.camera.set_position(Cartesian3::new(0.0, 0.0, 0.0));
    assert_eq!(
        tile_bounding_volume
            .distance_to_camera(Some(&frame_state))
            .unwrap(),
        0.0
    );

    frame_state
        .camera
        .set_position(Cartesian3::new(-0.5, -0.5, -0.5));
    assert_eq!(
        tile_bounding_volume
            .distance_to_camera(Some(&frame_state))
            .unwrap(),
        0.0
    );

    frame_state
        .camera
        .set_position(Cartesian3::new(0.5, 0.5, 0.5));
    assert_eq!(
        tile_bounding_volume
            .distance_to_camera(Some(&frame_state))
            .unwrap(),
        0.0
    );
}

#[test]
fn has_correct_distance_to_camera_if_camera_is_slightly_outside_box() {
    let center = Cartesian3::new(0.0, 0.0, 0.0);
    let half_axes = Matrix3::from_scale(Cartesian3::new(0.5, 0.5, 0.5));
    let tile_bounding_volume = TileOrientedBoundingBox::from_center_half_axes(center, half_axes);

    let mut frame_state = TileOrientedBoundingBoxFrameState::default();

    let eps6 = EPSILON6;
    frame_state
        .camera
        .set_position(Cartesian3::new(0.5 + eps6, 0.5, 0.5));
    assert_ne!(
        tile_bounding_volume
            .distance_to_camera(Some(&frame_state))
            .unwrap(),
        0.0
    );

    frame_state
        .camera
        .set_position(Cartesian3::new(-0.5, -0.5, -0.5 - eps6));
    assert_ne!(
        tile_bounding_volume
            .distance_to_camera(Some(&frame_state))
            .unwrap(),
        0.0
    );

    frame_state
        .camera
        .set_position(Cartesian3::new(100.5, 100.5, 100.5));
    let expected = 30000.0_f64.sqrt();
    let actual = tile_bounding_volume
        .distance_to_camera(Some(&frame_state))
        .unwrap();
    assert!((actual - expected).abs() < 1e-10);
}

#[test]
fn has_correct_distance_to_camera_for_large_distances() {
    let center = Cartesian3::new(0.0, 0.0, 0.0);
    let half_axes = Matrix3::from_scale(Cartesian3::new(0.5, 0.5, 0.5));
    let tile_bounding_volume = TileOrientedBoundingBox::from_center_half_axes(center, half_axes);

    let frame_state = TileOrientedBoundingBoxFrameState::new(TileOrientedBoundingBoxCamera::new(
        Cartesian3::new(
            2170456.713380141,
            -36351235.19646463,
            28403328.27058654,
        ),
    ));

    let actual = tile_bounding_volume
        .distance_to_camera(Some(&frame_state))
        .unwrap();
    let expected = 46183029.05370139;
    assert!((actual - expected).abs() < EPSILON6);
}

#[test]
fn intersect_plane_throws_when_plane_is_undefined() {
    let center = Cartesian3::new(0.0, 0.0, 0.0);
    let half_axes = Matrix3::from_scale(Cartesian3::new(0.5, 0.5, 0.5));
    let tile_bounding_volume = TileOrientedBoundingBox::from_center_half_axes(center, half_axes);

    assert!(tile_bounding_volume.intersect_plane(None).is_err());
}

#[test]
fn intersects_plane() {
    let center = Cartesian3::new(0.0, 0.0, 0.0);
    let half_axes = Matrix3::from_scale(Cartesian3::new(0.5, 0.5, 0.5));
    let tile_bounding_volume = TileOrientedBoundingBox::from_center_half_axes(center, half_axes);

    let plane = Plane::new(Cartesian3::UNIT_X.into(), 0.0).unwrap();
    assert_eq!(
        tile_bounding_volume.intersect_plane(Some(plane)).unwrap(),
        PlaneIntersection::Intersecting
    );

    let plane = Plane::new(Cartesian3::UNIT_X.into(), 0.5 - EPSILON6).unwrap();
    assert_eq!(
        tile_bounding_volume.intersect_plane(Some(plane)).unwrap(),
        PlaneIntersection::Intersecting
    );

    let plane = Plane::new(Cartesian3::UNIT_X.into(), -0.5 + EPSILON6).unwrap();
    assert_eq!(
        tile_bounding_volume.intersect_plane(Some(plane)).unwrap(),
        PlaneIntersection::Intersecting
    );
}

#[test]
fn does_not_intersect_plane() {
    let center = Cartesian3::new(0.0, 0.0, 0.0);
    let half_axes = Matrix3::from_scale(Cartesian3::new(0.5, 0.5, 0.5));
    let tile_bounding_volume = TileOrientedBoundingBox::from_center_half_axes(center, half_axes);

    let eps6 = EPSILON6;

    let plane = Plane::new(Cartesian3::UNIT_X.into(), 0.5 + eps6).unwrap();
    assert_eq!(
        tile_bounding_volume.intersect_plane(Some(plane)).unwrap(),
        PlaneIntersection::Inside
    );

    let plane = Plane::new(Cartesian3::UNIT_Y.into(), 0.5 + eps6).unwrap();
    assert_eq!(
        tile_bounding_volume.intersect_plane(Some(plane)).unwrap(),
        PlaneIntersection::Inside
    );

    let plane = Plane::new(Cartesian3::UNIT_Z.into(), 0.5 + eps6).unwrap();
    assert_eq!(
        tile_bounding_volume.intersect_plane(Some(plane)).unwrap(),
        PlaneIntersection::Inside
    );

    let plane = Plane::new(Cartesian3::UNIT_X.into(), -0.5 - eps6).unwrap();
    assert_eq!(
        tile_bounding_volume.intersect_plane(Some(plane)).unwrap(),
        PlaneIntersection::Outside
    );

    let plane = Plane::new(Cartesian3::UNIT_Y.into(), -0.5 - eps6).unwrap();
    assert_eq!(
        tile_bounding_volume.intersect_plane(Some(plane)).unwrap(),
        PlaneIntersection::Outside
    );

    let plane = Plane::new(Cartesian3::UNIT_Z.into(), -0.5 - eps6).unwrap();
    assert_eq!(
        tile_bounding_volume.intersect_plane(Some(plane)).unwrap(),
        PlaneIntersection::Outside
    );
}
