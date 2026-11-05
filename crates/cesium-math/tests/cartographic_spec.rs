#![allow(clippy::excessive_precision)]

use std::f64::consts::PI;

use cesium_math::{Cartesian3, Cartographic, Ellipsoid, EPSILON8};

fn surface_cartesian() -> Cartesian3 {
    Cartesian3::new(4094327.7921465295, 1909216.4044747739, 4487348.4088659193)
}

fn surface_cartographic() -> Cartographic {
    Cartographic::new(25.0_f64.to_radians(), 45.0_f64.to_radians(), 0.0)
}

#[test]
fn default_constructor_sets_expected_properties() {
    let c = Cartographic::default();
    assert_eq!(c.longitude, 0.0);
    assert_eq!(c.latitude, 0.0);
    assert_eq!(c.height, 0.0);
}

#[test]
fn constructor_sets_expected_properties_from_parameters() {
    let c = Cartographic::new(1.0, 2.0, 3.0);
    assert_eq!(c.longitude, 1.0);
    assert_eq!(c.latitude, 2.0);
    assert_eq!(c.height, 3.0);
}

#[test]
fn to_cartesian_conversion_from_cartographic_to_cartesian3() {
    let lon = 150.0_f64.to_radians();
    let lat = (-40.0_f64).to_radians();
    let height = 100000.0;
    let c = Cartographic::new(lon, lat, height);
    let actual = c.to_cartesian(Some(&Ellipsoid::WGS84));
    let expected = Cartesian3::from(Ellipsoid::WGS84.cartographic_to_cartesian(c).unwrap());
    assert_eq!(actual, expected);
}

#[test]
fn to_cartesian_uses_default_or_custom_ellipsoid() {
    let expected_position =
        Cartesian3::new(1593514.338295244, 691991.9979835141, 20442.318221152018);
    let cartographic = Cartographic::from_degrees(23.47315, 0.67416, None);
    let position = cartographic.to_cartesian(Some(&Ellipsoid::MOON));

    assert!(position.equals_epsilon(&expected_position, Some(EPSILON8), Some(EPSILON8)));
}

#[test]
fn from_radians_sets_expected_properties() {
    let c = Cartographic::from_radians_with_height(PI / 2.0, PI / 4.0, Some(100.0));
    assert_eq!(c.longitude, PI / 2.0);
    assert_eq!(c.latitude, PI / 4.0);
    assert_eq!(c.height, 100.0);

    let c_default = Cartographic::from_radians(PI / 2.0, PI / 4.0);
    assert_eq!(c_default.longitude, PI / 2.0);
    assert_eq!(c_default.latitude, PI / 4.0);
    assert_eq!(c_default.height, 0.0);
}

#[test]
fn from_degrees_sets_expected_properties() {
    let c = Cartographic::from_degrees(90.0, 45.0, Some(100.0));
    assert_eq!(c.longitude, PI / 2.0);
    assert_eq!(c.latitude, PI / 4.0);
    assert_eq!(c.height, 100.0);

    let c_default = Cartographic::from_degrees(90.0, 45.0, None);
    assert_eq!(c_default.longitude, PI / 2.0);
    assert_eq!(c_default.latitude, PI / 4.0);
    assert_eq!(c_default.height, 0.0);
}

#[test]
fn from_cartesian_works_with_wgs84() {
    let c = Cartographic::from_cartesian(surface_cartesian(), Some(&Ellipsoid::WGS84)).unwrap();
    assert!(c.equals_epsilon(Some(&surface_cartographic()), EPSILON8));

    let c_default = Cartographic::from_cartesian(surface_cartesian(), None).unwrap();
    assert!(c_default.equals_epsilon(Some(&surface_cartographic()), EPSILON8));
}

#[test]
fn from_cartesian_works_with_moon_ellipsoid() {
    let position =
        Cartesian3::new(1593514.338295244, 691991.9979835141, 20442.318221152018);
    let cartographic = Cartographic::from_cartesian(position, Some(&Ellipsoid::MOON)).unwrap();
    let expected = Cartographic::from_degrees(23.47315, 0.67416, None);
    assert!(cartographic.equals_epsilon(Some(&expected), EPSILON8));
}

#[test]
fn from_cartesian_works_with_value_above_ellipsoid_surface() {
    let c1 = Cartographic::from_degrees(35.766989, 33.333602, Some(3000.0));
    let p = c1.to_cartesian(Some(&Ellipsoid::WGS84));
    let c2 = Cartographic::from_cartesian(p, Some(&Ellipsoid::WGS84)).unwrap();
    assert!(c2.equals_epsilon(Some(&c1), EPSILON8));
}

#[test]
fn from_cartesian_works_with_value_below_ellipsoid_surface() {
    let c1 = Cartographic::from_degrees(35.766989, 33.333602, Some(-3000.0));
    let p = c1.to_cartesian(Some(&Ellipsoid::WGS84));
    let c2 = Cartographic::from_cartesian(p, Some(&Ellipsoid::WGS84)).unwrap();
    assert!(c2.equals_epsilon(Some(&c1), EPSILON8));
}

#[test]
fn clone_and_equality() {
    let c1 = Cartographic::new(1.0, 2.0, 3.0);
    let c2 = c1;
    assert!(c1.equals(Some(&c2)));
    assert!(!c1.equals(Some(&Cartographic::new(2.0, 2.0, 3.0))));
    assert!(!c1.equals(Some(&Cartographic::new(2.0, 1.0, 3.0))));
    assert!(!c1.equals(Some(&Cartographic::new(1.0, 2.0, 4.0))));
    assert!(!c1.equals(None));
}

#[test]
fn equals_epsilon() {
    let c = Cartographic::new(1.0, 2.0, 3.0);
    assert!(c.equals_epsilon(Some(&Cartographic::new(1.0, 2.0, 3.0)), 0.0));
    assert!(c.equals_epsilon(Some(&Cartographic::new(1.0, 2.0, 3.0)), 1.0));
    assert!(c.equals_epsilon(Some(&Cartographic::new(2.0, 2.0, 3.0)), 1.0));
    assert!(c.equals_epsilon(Some(&Cartographic::new(1.0, 3.0, 3.0)), 1.0));
    assert!(c.equals_epsilon(Some(&Cartographic::new(1.0, 2.0, 4.0)), 1.0));

    assert!(!c.equals_epsilon(Some(&Cartographic::new(2.0, 2.0, 3.0)), 0.99999));
    assert!(!c.equals_epsilon(Some(&Cartographic::new(1.0, 3.0, 3.0)), 0.99999));
    assert!(!c.equals_epsilon(Some(&Cartographic::new(1.0, 2.0, 4.0)), 0.99999));
    assert!(!c.equals_epsilon(None, 1.0));
}

#[test]
fn to_string_formatting() {
    let c = Cartographic::new(1.123, 2.345, 6.789);
    assert_eq!(format!("{}", c), "(1.123, 2.345, 6.789)");
}
