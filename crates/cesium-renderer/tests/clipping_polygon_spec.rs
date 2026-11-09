use cesium_math::{Cartesian3, DeveloperError, Ellipsoid, GeoRectangle, EPSILON10};
use cesium_renderer::{ClippingPolygon, ClippingPolygonOptions};

fn sample_positions() -> Vec<Cartesian3> {
    Cartesian3::from_radians_array(
        &[
            -1.3194369277314022,
            0.6988062530900625,
            -1.31941,
            0.69879,
            -1.3193955980204217,
            0.6988091578771254,
            -1.3193931220959367,
            0.698743632490865,
            -1.3194358224045408,
            0.6987471965556998,
        ],
        None,
    )
    .unwrap()
}

fn sample_hole() -> Vec<Cartesian3> {
    Cartesian3::from_radians_array(
        &[
            -1.31942, 0.69879, -1.319405, 0.69879, -1.319412, 0.698763,
        ],
        None,
    )
    .unwrap()
}

#[test]
fn test_constructs() {
    let positions = sample_positions();
    let polygon = ClippingPolygon::new(ClippingPolygonOptions {
        positions: Some(positions.clone()),
        ..Default::default()
    })
    .expect("should construct");

    assert_eq!(polygon.length(), 5);
    assert_eq!(polygon.positions(), positions.as_slice());
    assert!(polygon.holes().is_empty());
    assert_eq!(polygon.ellipsoid(), Ellipsoid::WGS84);
}

#[test]
fn test_constructs_with_holes() {
    let positions = sample_positions();
    let hole = sample_hole();

    let polygon = ClippingPolygon::new(ClippingPolygonOptions {
        positions: Some(positions.clone()),
        holes: Some(vec![hole.clone()]),
        ..Default::default()
    })
    .expect("should construct");

    assert_eq!(polygon.positions(), positions.as_slice());
    assert_eq!(polygon.holes().len(), 1);
    assert_eq!(polygon.holes()[0], hole);
    assert_eq!(polygon.length(), positions.len() + hole.len());
}

#[test]
fn test_clones_holes() {
    let positions = sample_positions();
    let hole = sample_hole();

    let polygon = ClippingPolygon::new(ClippingPolygonOptions {
        positions: Some(positions),
        holes: Some(vec![hole.clone()]),
        ..Default::default()
    })
    .expect("should construct");

    let cloned = ClippingPolygon::clone_polygon(&polygon, None);
    assert_eq!(cloned.holes().len(), 1);
    assert_eq!(cloned.holes()[0], hole);
}

#[test]
fn test_throws_when_fewer_than_3_positions() {
    let err1 = ClippingPolygon::new(ClippingPolygonOptions::default())
        .expect_err("should throw without positions");
    assert_eq!(err1.name(), DeveloperError::NAME);

    let err2 = ClippingPolygon::new(ClippingPolygonOptions {
        positions: Some(
            Cartesian3::from_radians_array(
                &[
                    -1.3194369277314022,
                    0.6988062530900625,
                    -1.31941,
                    0.69879,
                ],
                None,
            )
            .unwrap(),
        ),
        ..Default::default()
    })
    .expect_err("should throw with fewer than 3 positions");
    assert_eq!(err2.name(), DeveloperError::NAME);
}

#[test]
fn test_clones() {
    let positions = sample_positions();
    let polygon = ClippingPolygon::new(ClippingPolygonOptions {
        positions: Some(positions.clone()),
        ..Default::default()
    })
    .expect("should construct");

    let mut cloned = ClippingPolygon::clone_polygon(&polygon, None);
    assert_eq!(polygon.positions(), cloned.positions());
    assert_eq!(polygon.ellipsoid(), cloned.ellipsoid());

    let mut scratch = ClippingPolygon::new(ClippingPolygonOptions {
        positions: Some(positions),
        ..Default::default()
    })
    .expect("should construct");

    cloned = ClippingPolygon::clone_polygon(&polygon, Some(&mut scratch));
    assert_eq!(polygon.positions(), cloned.positions());
    assert_eq!(polygon.ellipsoid(), cloned.ellipsoid());
}

#[test]
fn test_equals_verifies_equality() {
    let positions = sample_positions();
    let polygon_a = ClippingPolygon::new(ClippingPolygonOptions {
        positions: Some(positions.clone()),
        ..Default::default()
    })
    .expect("should construct");

    let polygon_b = ClippingPolygon::new(ClippingPolygonOptions {
        positions: Some(positions),
        ..Default::default()
    })
    .expect("should construct");

    // Independent instances with cloned buffers are not pointer-equal
    assert!(!ClippingPolygon::equals(&polygon_a, &polygon_b));
    // Same instance is equal to itself
    assert!(ClippingPolygon::equals(&polygon_a, &polygon_a));
}

#[test]
fn test_rectangle_getter_returns_enclosing_rectangle() {
    let positions = sample_positions();
    let polygon = ClippingPolygon::new(ClippingPolygonOptions {
        positions: Some(positions),
        ..Default::default()
    })
    .expect("should construct");

    let result = polygon.rectangle();
    assert!((result.west - (-1.3194369277314024)).abs() <= EPSILON10);
    assert!((result.south - 0.6987436324908647).abs() <= EPSILON10);
    assert!((result.east - (-1.3193931220959367)).abs() <= EPSILON10);
    assert!((result.north - 0.6988091578771254).abs() <= EPSILON10);
}

#[test]
fn test_compute_rectangle_deprecated() {
    let positions = sample_positions();
    let polygon = ClippingPolygon::new(ClippingPolygonOptions {
        positions: Some(positions),
        ..Default::default()
    })
    .expect("should construct");

    let mut scratch = GeoRectangle::default();
    let returned = polygon.compute_rectangle(&mut scratch);
    assert_eq!(returned, polygon.rectangle());
}
