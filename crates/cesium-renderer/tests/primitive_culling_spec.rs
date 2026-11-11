use cesium_math::{
    BoundingSphere, CullingVolume, Ellipsoid, Occluder, PerspectiveFrustum,
};
use cesium_renderer::primitive_culling::PrimitiveCuller;
use glam::DVec3;

fn create_culling_volume(position: DVec3, direction: DVec3, up: DVec3, far: f64) -> CullingVolume {
    PerspectiveFrustum {
        fov: 60.0_f64.to_radians(),
        aspect_ratio: 1.0,
        near: 1.0,
        far,
        x_offset: 0.0,
        y_offset: 0.0,
    }
    .compute_culling_volume(position, direction, up)
    .unwrap()
}

#[test]
fn test_primitive_culling_frustum_in_3d() {
    let rect_sphere = BoundingSphere::new(DVec3::new(10.0, 0.0, 0.0), 2.0);

    // Camera looking directly along +X
    let camera_pos = DVec3::ZERO;
    let direction = DVec3::X;
    let up = DVec3::Z;
    let volume = create_culling_volume(camera_pos, direction, up, 1000.0);

    assert!(PrimitiveCuller::is_visible(&rect_sphere, &volume, None));

    // Move camera forward past the sphere (now behind frustum near plane)
    let moved_camera = DVec3::new(20.0, 0.0, 0.0);
    let moved_volume = create_culling_volume(moved_camera, direction, up, 1000.0);

    assert!(!PrimitiveCuller::is_visible(&rect_sphere, &moved_volume, None));
}

#[test]
fn test_primitive_culling_frustum_in_columbus_view() {
    let cv_sphere = BoundingSphere::new(DVec3::new(15.0, 5.0, 0.0), 3.0);

    let camera_pos = DVec3::new(0.0, 5.0, 0.0);
    let direction = DVec3::X;
    let up = DVec3::Z;
    let volume = create_culling_volume(camera_pos, direction, up, 1000.0);

    assert!(PrimitiveCuller::is_visible(&cv_sphere, &volume, None));

    let moved_camera = DVec3::new(50.0, 5.0, 0.0);
    let moved_volume = create_culling_volume(moved_camera, direction, up, 1000.0);
    assert!(!PrimitiveCuller::is_visible(&cv_sphere, &moved_volume, None));
}

#[test]
fn test_primitive_culling_frustum_in_2d() {
    let sphere_2d = BoundingSphere::new(DVec3::new(0.0, 0.0, 0.0), 5.0);

    let camera_pos = DVec3::new(0.0, 0.0, 50.0);
    let direction = -DVec3::Z;
    let up = DVec3::Y;
    let volume = create_culling_volume(camera_pos, direction, up, 1000.0);

    assert!(PrimitiveCuller::is_visible(&sphere_2d, &volume, None));

    // In 2D, far outside the view
    let far_sphere = BoundingSphere::new(DVec3::new(2000.0, 2000.0, 0.0), 1.0);
    assert!(!PrimitiveCuller::is_visible(&far_sphere, &volume, None));
}

#[test]
fn test_primitive_culling_polygon_occlusion() {
    let ellipsoid = Ellipsoid::WGS84;
    let camera_pos = DVec3::new(0.0, 0.0, 20_000_000.0);
    let occluder = Occluder::new(
        BoundingSphere::new(DVec3::ZERO, ellipsoid.maximum_radius()),
        camera_pos,
    )
    .unwrap();

    // Front of the globe (facing camera)
    let visible_sphere = BoundingSphere::new(DVec3::new(0.0, 0.0, 6_378_137.0), 100.0);
    let volume = create_culling_volume(camera_pos, -DVec3::Z, DVec3::Y, 30_000_000.0);
    assert!(PrimitiveCuller::is_visible(&visible_sphere, &volume, Some(&occluder)));

    // Back of the globe (occluded by ellipsoid)
    let occluded_sphere = BoundingSphere::new(DVec3::new(0.0, 0.0, -6_378_137.0), 100.0);
    assert!(!PrimitiveCuller::is_visible(&occluded_sphere, &volume, Some(&occluder)));
}

#[test]
fn test_primitive_culling_labels_and_billboards() {
    let center = DVec3::new(100.0, 50.0, 0.0);
    let label_sphere = BoundingSphere::new(center, 10.0);

    let camera_pos = center + DVec3::new(0.0, 0.0, 100.0);
    let volume = create_culling_volume(camera_pos, -DVec3::Z, DVec3::Y, 1000.0);

    assert!(PrimitiveCuller::is_visible(&label_sphere, &volume, None));

    // Behind camera frustum
    let past_camera = center + DVec3::new(0.0, 0.0, -500.0);
    let past_volume = create_culling_volume(past_camera, -DVec3::Z, DVec3::Y, 1000.0);
    assert!(!PrimitiveCuller::is_visible(&label_sphere, &past_volume, None));
}

#[test]
fn test_primitive_culling_polylines() {
    let p1 = DVec3::new(-100.0, 30.0, 0.0);
    let p2 = DVec3::new(-93.0, 37.0, 0.0);
    let polyline_sphere = BoundingSphere::from_points(&[p1, p2]).unwrap();

    let center = polyline_sphere.center;
    let camera_pos = center + DVec3::new(0.0, 0.0, 100.0);
    let volume = create_culling_volume(camera_pos, -DVec3::Z, DVec3::Y, 1000.0);

    assert!(PrimitiveCuller::is_visible(&polyline_sphere, &volume, None));

    let moved_camera = center + DVec3::new(0.0, 0.0, -500.0);
    let moved_volume = create_culling_volume(moved_camera, -DVec3::Z, DVec3::Y, 1000.0);
    assert!(!PrimitiveCuller::is_visible(&polyline_sphere, &moved_volume, None));
}
