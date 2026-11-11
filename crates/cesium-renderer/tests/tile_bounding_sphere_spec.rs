use cesium_math::{Cartesian3, Color, Plane, PlaneIntersection, EPSILON6};
use cesium_renderer::{
    TileBoundingSphere, TileBoundingSphereCamera, TileBoundingSphereFrameState,
};

#[test]
fn test_can_be_instantiated_with_center_and_radius() {
    let center = Cartesian3::new(0.0, 0.0, 0.0);
    let radius = 1.0;
    let tbs = TileBoundingSphere::new(Some(center), Some(radius));

    assert_eq!(tbs.center(), center);
    assert_eq!(tbs.radius(), radius);
    assert_eq!(tbs.bounding_volume().radius, radius);
    assert_eq!(tbs.bounding_sphere().radius, radius);
}

#[test]
fn test_create_debug_volume() {
    let tbs = TileBoundingSphere::new(Some(Cartesian3::ZERO), Some(1.0));
    let debug_volume = tbs.create_debug_volume(Color::BLUE).unwrap();
    assert!(debug_volume.geometry_instances.is_some());
    assert_eq!(
        debug_volume.geometry_instances.as_ref().unwrap().len(),
        1
    );
}

#[test]
fn test_distance_to_camera_inside_bounding_sphere() {
    let tbs = TileBoundingSphere::new(Some(Cartesian3::ZERO), Some(1.0));
    let fs = TileBoundingSphereFrameState {
        camera: TileBoundingSphereCamera::new(Cartesian3::ZERO),
    };
    assert_eq!(tbs.distance_to_camera(&fs), 0.0);
}

#[test]
fn test_distance_to_camera_outside_bounding_sphere() {
    let tbs = TileBoundingSphere::new(Some(Cartesian3::ZERO), Some(1.0));
    let fs = TileBoundingSphereFrameState {
        camera: TileBoundingSphereCamera::new(Cartesian3::new(0.0, 2.0, 0.0)),
    };
    assert_eq!(tbs.distance_to_camera(&fs), 1.0);
}

#[test]
fn test_intersects_plane() {
    let tbs = TileBoundingSphere::new(Some(Cartesian3::ZERO), Some(1.0));
    let normal = Cartesian3::new(0.0, 0.0, 1.0);
    let plane = Plane::new(normal.into(), EPSILON6).unwrap();
    assert_eq!(tbs.intersect_plane(plane), PlaneIntersection::Intersecting);
}

#[test]
fn test_update() {
    let mut tbs = TileBoundingSphere::new(Some(Cartesian3::ZERO), Some(1.0));
    let new_center = Cartesian3::new(10.0, 20.0, 30.0);
    let new_radius = 5.0;
    tbs.update(new_center, new_radius);

    assert_eq!(tbs.center(), new_center);
    assert_eq!(tbs.radius(), new_radius);
}
