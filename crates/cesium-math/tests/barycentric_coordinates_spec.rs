use cesium_math::{
    barycentric_coordinates, barycentric_coordinates_2d, Cartesian2, Cartesian3, EPSILON14,
};

#[test]
fn test_evaluates_to_p0() {
    let p0 = Cartesian3::new(-1.0, 0.0, 0.0);
    let p1 = Cartesian3::new(1.0, 0.0, 0.0);
    let p2 = Cartesian3::new(0.0, 1.0, 1.0);

    let point = p0;
    let result = barycentric_coordinates(&point, &p0, &p1, &p2).unwrap();
    assert_eq!(result, Cartesian3::UNIT_X);
}

#[test]
fn test_evaluates_to_p1() {
    let p0 = Cartesian3::new(-1.0, 0.0, 0.0);
    let p1 = Cartesian3::new(1.0, 0.0, 0.0);
    let p2 = Cartesian3::new(0.0, 1.0, 1.0);

    let point = p1;
    let result = barycentric_coordinates(&point, &p0, &p1, &p2).unwrap();
    assert_eq!(result, Cartesian3::UNIT_Y);
}

#[test]
fn test_evaluates_to_p2() {
    let p0 = Cartesian3::new(-1.0, 0.0, 0.0);
    let p1 = Cartesian3::new(1.0, 0.0, 0.0);
    let p2 = Cartesian3::new(0.0, 1.0, 1.0);

    let point = p2;
    let result = barycentric_coordinates(&point, &p0, &p1, &p2).unwrap();
    assert_eq!(result, Cartesian3::UNIT_Z);
}

#[test]
fn test_evaluates_on_p0_p1_edge() {
    let p0 = Cartesian3::new(-1.0, 0.0, 0.0);
    let p1 = Cartesian3::new(1.0, 0.0, 0.0);
    let p2 = Cartesian3::new(0.0, 1.0, 1.0);

    let point = (p1 + p0) * 0.5;
    let result = barycentric_coordinates(&point, &p0, &p1, &p2).unwrap();
    assert_eq!(result, Cartesian3::new(0.5, 0.5, 0.0));
}

#[test]
fn test_evaluates_on_p0_p2_edge() {
    let p0 = Cartesian3::new(-1.0, 0.0, 0.0);
    let p1 = Cartesian3::new(1.0, 0.0, 0.0);
    let p2 = Cartesian3::new(0.0, 1.0, 1.0);

    let point = (p2 + p0) * 0.5;
    let result = barycentric_coordinates(&point, &p0, &p1, &p2).unwrap();
    assert_eq!(result, Cartesian3::new(0.5, 0.0, 0.5));
}

#[test]
fn test_evaluates_on_p1_p2_edge() {
    let p0 = Cartesian3::new(-1.0, 0.0, 0.0);
    let p1 = Cartesian3::new(1.0, 0.0, 0.0);
    let p2 = Cartesian3::new(0.0, 1.0, 1.0);

    let point = (p2 + p1) * 0.5;
    let result = barycentric_coordinates(&point, &p0, &p1, &p2).unwrap();
    assert_eq!(result, Cartesian3::new(0.0, 0.5, 0.5));
}

#[test]
fn test_evaluates_on_interior() {
    let p0 = Cartesian3::new(-1.0, 0.0, 0.0);
    let p1 = Cartesian3::new(1.0, 0.0, 0.0);
    let p2 = Cartesian3::new(0.0, 1.0, 1.0);

    let scalar = 1.0 / 3.0;
    let point = (p0 + p1 + p2) * scalar;
    let result = barycentric_coordinates(&point, &p0, &p1, &p2).unwrap();
    let expected = Cartesian3::new(scalar, scalar, scalar);
    assert!(result.equals_epsilon(&expected, Some(EPSILON14), None));
}

#[test]
fn test_returns_none_for_colinear_points() {
    let p0 = Cartesian3::new(-1.0, -1.0, 0.0);
    let p1 = Cartesian3::new(0.0, 0.0, 0.0);
    let p2 = Cartesian3::new(1.0, 1.0, 0.0);
    let point = Cartesian3::new(0.5, 0.5, 0.0);
    let coord = barycentric_coordinates(&point, &p0, &p1, &p2);
    assert!(coord.is_none());
}

#[test]
fn test_evaluates_with_equal_length_sides() {
    let p0 = Cartesian3::new(
        9635312487071484.0,
        13827945400273020.0,
        -16479219993905144.0,
    );
    let p1 = Cartesian3::new(
        12832234.180639317,
        -10455085.701705107,
        750010.7274386138,
    );
    let p2 = Cartesian3::new(
        -9689011.10628853,
        -13420063.892507521,
        750010.7274386119,
    );
    assert_eq!(
        barycentric_coordinates(&p0, &p0, &p1, &p2).unwrap(),
        Cartesian3::UNIT_X
    );
    assert_eq!(
        barycentric_coordinates(&p1, &p0, &p1, &p2).unwrap(),
        Cartesian3::UNIT_Y
    );
    assert_eq!(
        barycentric_coordinates(&p2, &p0, &p1, &p2).unwrap(),
        Cartesian3::UNIT_Z
    );
}

#[test]
fn test_evaluates_2d() {
    let p0 = Cartesian2::new(0.0, 0.0);
    let p1 = Cartesian2::new(1.0, 0.0);
    let p2 = Cartesian2::new(0.0, 1.0);

    assert_eq!(
        barycentric_coordinates_2d(&p0, &p0, &p1, &p2).unwrap(),
        Cartesian3::UNIT_X
    );
    assert_eq!(
        barycentric_coordinates_2d(&p1, &p0, &p1, &p2).unwrap(),
        Cartesian3::UNIT_Y
    );
    assert_eq!(
        barycentric_coordinates_2d(&p2, &p0, &p1, &p2).unwrap(),
        Cartesian3::UNIT_Z
    );

    let mid = Cartesian2::new(0.5, 0.0);
    let res = barycentric_coordinates_2d(&mid, &p0, &p1, &p2).unwrap();
    assert_eq!(res, Cartesian3::new(0.5, 0.5, 0.0));
}
