//! Tests adapted from CesiumJS WebMercatorProjectionSpec (Apache-2.0).
use cesium_math::{Cartographic, Ellipsoid, WebMercatorProjection};
use glam::{DVec2, DVec3};
use std::f64::consts::{FRAC_PI_2, FRAC_PI_4, PI};

#[test]
fn construct0() {
    let projection = WebMercatorProjection::default();
    assert_eq!(projection.ellipsoid(), Ellipsoid::WGS84);
}

#[test]
fn construct1() {
    let ellipsoid = Ellipsoid::UNIT_SPHERE;
    let projection = WebMercatorProjection::new(ellipsoid).unwrap();
    assert_eq!(projection.ellipsoid(), ellipsoid);
}

#[test]
fn project0() {
    let height = 10.0;
    let cartographic = Cartographic::new(0.0, 0.0, height);
    let projection = WebMercatorProjection::default();
    assert_eq!(
        projection.project(cartographic).unwrap(),
        DVec3::new(0.0, 0.0, height)
    );
}

#[test]
fn project1_wgs84() {
    let ellipsoid = Ellipsoid::WGS84;
    let cartographic = Cartographic::new(PI, FRAC_PI_4, 0.0);

    let max_radius = ellipsoid.maximum_radius();
    let expected = DVec3::new(
        max_radius * cartographic.longitude,
        max_radius * (FRAC_PI_4 + cartographic.latitude / 2.0).tan().ln(),
        0.0,
    );

    let projection = WebMercatorProjection::new(ellipsoid).unwrap();
    let projected = projection.project(cartographic).unwrap();
    assert!((projected.x - expected.x).abs() < 1e-8);
    assert!((projected.y - expected.y).abs() < 1e-8);
    assert!((projected.z - expected.z).abs() < 1e-8);
}

#[test]
fn project2_unit_sphere() {
    let ellipsoid = Ellipsoid::UNIT_SPHERE;
    let cartographic = Cartographic::new(-PI, FRAC_PI_4, 0.0);

    let max_radius = ellipsoid.maximum_radius();
    let expected = DVec3::new(
        max_radius * cartographic.longitude,
        max_radius * (FRAC_PI_4 + cartographic.latitude / 2.0).tan().ln(),
        0.0,
    );

    let projection = WebMercatorProjection::new(ellipsoid).unwrap();
    let projected = projection.project(cartographic).unwrap();
    assert!((projected.x - expected.x).abs() < 1e-14);
    assert!((projected.y - expected.y).abs() < 1e-14);
    assert!((projected.z - expected.z).abs() < 1e-14);
}

#[test]
fn unproject0() {
    let cartographic = Cartographic::new(FRAC_PI_2, FRAC_PI_4, 12.0);
    let projection = WebMercatorProjection::default();
    let projected = projection.project(cartographic).unwrap();
    let unprojected = projection.unproject(projected).unwrap();
    assert!((unprojected.longitude - cartographic.longitude).abs() < 1e-14);
    assert!((unprojected.latitude - cartographic.latitude).abs() < 1e-14);
    assert!((unprojected.height - cartographic.height).abs() < 1e-14);
}

#[test]
fn unproject_is_correct_at_corners() {
    let projection = WebMercatorProjection::default();
    let max_lat_deg = 85.05112878_f64.to_radians();

    let southwest = projection
        .unproject_2d(DVec2::new(-20037508.342787, -20037508.342787))
        .unwrap();
    assert!((southwest.longitude - (-PI)).abs() < 1e-12);
    assert!((southwest.latitude - (-max_lat_deg)).abs() < 1e-11);

    let southeast = projection
        .unproject_2d(DVec2::new(20037508.342787, -20037508.342787))
        .unwrap();
    assert!((southeast.longitude - PI).abs() < 1e-12);
    assert!((southeast.latitude - (-max_lat_deg)).abs() < 1e-11);

    let northeast = projection
        .unproject_2d(DVec2::new(20037508.342787, 20037508.342787))
        .unwrap();
    assert!((northeast.longitude - PI).abs() < 1e-12);
    assert!((northeast.latitude - max_lat_deg).abs() < 1e-11);

    let northwest = projection
        .unproject_2d(DVec2::new(-20037508.342787, 20037508.342787))
        .unwrap();
    assert!((northwest.longitude - (-PI)).abs() < 1e-12);
    assert!((northwest.latitude - max_lat_deg).abs() < 1e-11);
}

#[test]
fn project_is_correct_at_corners() {
    let max_latitude = WebMercatorProjection::MAXIMUM_LATITUDE;
    let projection = WebMercatorProjection::default();

    let southwest = projection
        .project(Cartographic::new(-PI, -max_latitude, 0.0))
        .unwrap();
    assert!((southwest.x - (-20037508.342787)).abs() < 1e-3);
    assert!((southwest.y - (-20037508.342787)).abs() < 1e-3);

    let southeast = projection
        .project(Cartographic::new(PI, -max_latitude, 0.0))
        .unwrap();
    assert!((southeast.x - 20037508.342787).abs() < 1e-3);
    assert!((southeast.y - (-20037508.342787)).abs() < 1e-3);

    let northeast = projection
        .project(Cartographic::new(PI, max_latitude, 0.0))
        .unwrap();
    assert!((northeast.x - 20037508.342787).abs() < 1e-3);
    assert!((northeast.y - 20037508.342787).abs() < 1e-3);

    let northwest = projection
        .project(Cartographic::new(-PI, max_latitude, 0.0))
        .unwrap();
    assert!((northwest.x - (-20037508.342787)).abs() < 1e-3);
    assert!((northwest.y - 20037508.342787).abs() < 1e-3);
}

#[test]
fn projected_y_is_clamped_to_valid_latitude_range() {
    let projection = WebMercatorProjection::default();
    let south_pole = projection
        .project(Cartographic::new(0.0, -FRAC_PI_2, 0.0))
        .unwrap();
    let south_limit = projection
        .project(Cartographic::new(
            0.0,
            -WebMercatorProjection::MAXIMUM_LATITUDE,
            0.0,
        ))
        .unwrap();
    assert_eq!(south_pole.y, south_limit.y);

    let north_pole = projection
        .project(Cartographic::new(0.0, FRAC_PI_2, 0.0))
        .unwrap();
    let north_limit = projection
        .project(Cartographic::new(
            0.0,
            WebMercatorProjection::MAXIMUM_LATITUDE,
            0.0,
        ))
        .unwrap();
    assert_eq!(north_pole.y, north_limit.y);
}

#[test]
fn rejects_invalid_inputs() {
    let projection = WebMercatorProjection::default();
    assert!(projection.unproject(DVec3::splat(f64::NAN)).is_none());
    assert!(WebMercatorProjection::new(Ellipsoid::default()).is_err());
}
