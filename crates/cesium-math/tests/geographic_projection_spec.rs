//! Tests adapted from CesiumJS GeographicProjectionSpec (Apache-2.0).
use cesium_math::{Cartographic, Ellipsoid, GeographicProjection};
use glam::DVec3;
use std::f64::consts::{FRAC_PI_2, FRAC_PI_4, PI};

#[test]
fn construct0() {
    let projection = GeographicProjection::default();
    assert_eq!(projection.ellipsoid(), Ellipsoid::WGS84);
}

#[test]
fn construct1() {
    let ellipsoid = Ellipsoid::UNIT_SPHERE;
    let projection = GeographicProjection::new(ellipsoid).unwrap();
    assert_eq!(projection.ellipsoid(), ellipsoid);
}

#[test]
fn project0() {
    let height = 10.0;
    let cartographic = Cartographic::new(0.0, 0.0, height);
    let projection = GeographicProjection::default();
    assert_eq!(
        projection.project(cartographic).unwrap(),
        DVec3::new(0.0, 0.0, height)
    );
}

#[test]
fn project1() {
    let ellipsoid = Ellipsoid::WGS84;
    let cartographic = Cartographic::new(PI, FRAC_PI_2, 0.0);
    let expected = DVec3::new(
        PI * ellipsoid.radii().x,
        FRAC_PI_2 * ellipsoid.radii().x,
        0.0,
    );
    let projection = GeographicProjection::new(ellipsoid).unwrap();
    assert_eq!(projection.project(cartographic).unwrap(), expected);
}

#[test]
fn project2() {
    let ellipsoid = Ellipsoid::UNIT_SPHERE;
    let cartographic = Cartographic::new(-PI, FRAC_PI_2, 0.0);
    let expected = DVec3::new(-PI, FRAC_PI_2, 0.0);
    let projection = GeographicProjection::new(ellipsoid).unwrap();
    assert_eq!(projection.project(cartographic).unwrap(), expected);
}

#[test]
fn unproject0() {
    let cartographic = Cartographic::new(FRAC_PI_2, FRAC_PI_4, 12.0);
    let projection = GeographicProjection::default();
    let projected = projection.project(cartographic).unwrap();
    let unprojected = projection.unproject(projected).unwrap();
    assert!((unprojected.longitude - cartographic.longitude).abs() < 1e-15);
    assert!((unprojected.latitude - cartographic.latitude).abs() < 1e-15);
    assert_eq!(unprojected.height, cartographic.height);
}

#[test]
fn rejects_invalid_inputs() {
    let projection = GeographicProjection::default();
    assert!(projection.unproject(DVec3::splat(f64::NAN)).is_none());
    assert!(GeographicProjection::new(Ellipsoid::default()).is_err());
}
