use cesium_math::{Cartesian3, Spherical, EPSILON15};
use std::f64::consts::PI;

#[test]
fn test_spherical_default_construction() {
    let v = Spherical::default();
    assert_eq!(v.clock, 0.0);
    assert_eq!(v.cone, 0.0);
    assert_eq!(v.magnitude, 1.0);
}

#[test]
fn test_spherical_constructor_parameters() {
    let v = Spherical::new(1.0, 2.0, 3.0);
    assert_eq!(v.clock, 1.0);
    assert_eq!(v.cone, 2.0);
    assert_eq!(v.magnitude, 3.0);
}

#[test]
fn test_spherical_from_cartesian3() {
    let forty_five_degrees = PI / 4.0;
    let sixty_degrees = PI / 3.0;
    let cartesian = Cartesian3::new(1.0, 3.0f64.sqrt(), -2.0);
    let expected = Spherical::new(
        sixty_degrees,
        forty_five_degrees + PI / 2.0,
        8.0f64.sqrt(),
    );

    let actual = Spherical::from_cartesian3(cartesian);
    assert!(expected.equals_epsilon(&actual, EPSILON15));
}

#[test]
fn test_spherical_normalize() {
    let v = Spherical::new(0.0, 2.0, 3.0);
    let norm = v.normalize();
    assert_eq!(norm.clock, 0.0);
    assert_eq!(norm.cone, 2.0);
    assert_eq!(norm.magnitude, 1.0);
}

#[test]
fn test_spherical_equals_epsilon() {
    let a = Spherical::new(1.0, 2.0, 1.0);
    let b = Spherical::new(1.0, 2.0, 1.0);
    let c = Spherical::new(1.0, 2.0, 2.0);
    let d = Spherical::new(1.0, 2.0, 3.0);

    assert!(a.equals_epsilon(&b, 0.0));
    assert!(a.equals_epsilon(&c, 1.0));
    assert!(!a.equals_epsilon(&d, 1.0));
}

#[test]
fn test_spherical_to_string() {
    let v = Spherical::new(1.0, 2.0, 3.0);
    assert_eq!(v.to_string(), "(1, 2, 3)");
}
