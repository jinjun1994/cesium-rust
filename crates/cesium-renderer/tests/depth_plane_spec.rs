use cesium_math::Ellipsoid;
use cesium_renderer::DepthPlane;
use glam::DVec3;

#[test]
fn test_should_use_default_depth_plane_ellipsoid_offset() {
    let mut depth_plane = DepthPlane::new(None);
    let ellipsoid = Ellipsoid::WGS84;

    let camera_position = Some(DVec3::new(0.0, 0.0, 10000000.0));
    depth_plane.update(&ellipsoid, camera_position, false);

    assert_eq!(depth_plane.bounding_volume.radius, 6378137.0);
}

#[test]
fn test_should_use_provided_depth_plane_ellipsoid_offset() {
    let mut depth_plane = DepthPlane::new(Some(-8137.0));
    let ellipsoid = Ellipsoid::WGS84;

    let camera_position = Some(DVec3::new(0.0, 0.0, 10000000.0));
    depth_plane.update(&ellipsoid, camera_position, false);

    assert_eq!(depth_plane.bounding_volume.radius, 6370000.0);
}
