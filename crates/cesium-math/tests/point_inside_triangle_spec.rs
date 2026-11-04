use cesium_math::{point_inside_triangle, Cartesian2, Cartesian3};

#[test]
fn test_point_inside_triangle_has_point_inside() {
    assert!(point_inside_triangle(
        &Cartesian2::new(0.25, 0.25),
        &Cartesian2::ZERO,
        &Cartesian2::new(1.0, 0.0),
        &Cartesian2::new(0.0, 1.0),
    ));
}

#[test]
fn test_point_inside_triangle_has_point_outside() {
    assert!(!point_inside_triangle(
        &Cartesian2::new(1.0, 1.0),
        &Cartesian2::ZERO,
        &Cartesian2::new(1.0, 0.0),
        &Cartesian2::new(0.0, 1.0),
    ));
}

#[test]
fn test_point_inside_triangle_has_point_outside_2() {
    assert!(!point_inside_triangle(
        &Cartesian2::new(0.5, -0.5),
        &Cartesian2::ZERO,
        &Cartesian2::new(1.0, 0.0),
        &Cartesian2::new(0.0, 1.0),
    ));
}

#[test]
fn test_point_inside_triangle_has_point_outside_3() {
    assert!(!point_inside_triangle(
        &Cartesian2::new(-0.5, 0.5),
        &Cartesian2::ZERO,
        &Cartesian2::new(1.0, 0.0),
        &Cartesian2::new(0.0, 1.0),
    ));
}

#[test]
fn test_point_inside_triangle_has_point_on_corner() {
    assert!(!point_inside_triangle(
        &Cartesian2::ZERO,
        &Cartesian2::ZERO,
        &Cartesian2::new(1.0, 0.0),
        &Cartesian2::new(0.0, 1.0),
    ));
}

#[test]
fn test_point_inside_triangle_has_point_on_edge() {
    assert!(!point_inside_triangle(
        &Cartesian2::new(0.5, 0.0),
        &Cartesian2::ZERO,
        &Cartesian2::new(1.0, 0.0),
        &Cartesian2::new(0.0, 1.0),
    ));
}

#[test]
fn test_point_inside_triangle_3d() {
    let p0 = Cartesian3::new(0.0, 0.0, 0.0);
    let p1 = Cartesian3::new(1.0, 0.0, 0.0);
    let p2 = Cartesian3::new(0.0, 1.0, 1.0);

    let inside = (p0 + p1 + p2) * (1.0 / 3.0);
    assert!(point_inside_triangle(&inside, &p0, &p1, &p2));

    let outside = Cartesian3::new(2.0, 2.0, 2.0);
    assert!(!point_inside_triangle(&outside, &p0, &p1, &p2));
}
