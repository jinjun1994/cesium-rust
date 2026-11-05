//! Cases adapted from CesiumJS BoundingRectangleSpec (Apache-2.0).
use cesium_math::{
    BoundingRectangle, Ellipsoid, GeoRectangle, GeographicProjection, Intersect,
    WebMercatorProjection,
};
use glam::DVec2;

#[test]
fn default_constructor_sets_expected_values() {
    let rectangle = BoundingRectangle::default();
    assert_eq!(rectangle.x, 0.0);
    assert_eq!(rectangle.y, 0.0);
    assert_eq!(rectangle.width, 0.0);
    assert_eq!(rectangle.height, 0.0);
}

#[test]
fn constructor_sets_expected_parameters() {
    let rectangle = BoundingRectangle::new(1.0, 2.0, 3.0, 4.0);
    assert_eq!(rectangle.x, 1.0);
    assert_eq!(rectangle.y, 2.0);
    assert_eq!(rectangle.width, 3.0);
    assert_eq!(rectangle.height, 4.0);
}

#[test]
fn clone_and_equality() {
    let rectangle = BoundingRectangle::new(1.0, 2.0, 3.0, 4.0);
    let cloned = rectangle;
    assert_eq!(rectangle, cloned);

    assert!(rectangle.equals(Some(&BoundingRectangle::new(1.0, 2.0, 3.0, 4.0))));
    assert!(!rectangle.equals(Some(&BoundingRectangle::new(5.0, 2.0, 3.0, 4.0))));
    assert!(!rectangle.equals(Some(&BoundingRectangle::new(1.0, 6.0, 3.0, 4.0))));
    assert!(!rectangle.equals(Some(&BoundingRectangle::new(1.0, 2.0, 7.0, 4.0))));
    assert!(!rectangle.equals(Some(&BoundingRectangle::new(1.0, 2.0, 3.0, 8.0))));
    assert!(!rectangle.equals(None));
}

#[test]
fn from_points_computes_axis_aligned_bounding_rectangle() {
    let positions = [
        DVec2::new(3.0, -1.0),
        DVec2::new(2.0, -2.0),
        DVec2::new(1.0, -3.0),
        DVec2::ZERO,
        DVec2::new(-1.0, 1.0),
        DVec2::new(-2.0, 2.0),
        DVec2::new(-3.0, 3.0),
    ];
    let rectangle = BoundingRectangle::from_points(&positions).unwrap();
    assert_eq!(rectangle.x, -3.0);
    assert_eq!(rectangle.y, -3.0);
    assert_eq!(rectangle.width, 6.0);
    assert_eq!(rectangle.height, 6.0);
}

#[test]
fn from_points_empty_and_invalid() {
    let empty = BoundingRectangle::from_points(&[]).unwrap();
    assert_eq!(empty, BoundingRectangle::default());
    assert!(BoundingRectangle::from_points(&[DVec2::splat(f64::NAN)]).is_err());
}

#[test]
fn from_rectangle_creates_empty_rectangle_with_no_rectangle() {
    let rectangle = BoundingRectangle::from_rectangle(None, None::<&GeographicProjection>);
    assert_eq!(rectangle.x, 0.0);
    assert_eq!(rectangle.y, 0.0);
    assert_eq!(rectangle.width, 0.0);
    assert_eq!(rectangle.height, 0.0);
}

#[test]
fn from_rectangle_with_geographic_projection() {
    let rectangle = GeoRectangle::MAXIMUM;
    let projection = GeographicProjection::new(Ellipsoid::UNIT_SPHERE).unwrap();
    let expected = BoundingRectangle::new(
        rectangle.west,
        rectangle.south,
        rectangle.east - rectangle.west,
        rectangle.north - rectangle.south,
    );
    let result = BoundingRectangle::from_rectangle(Some(&rectangle), Some(&projection));
    assert_eq!(result, expected);

    // Also verify from_geographic directly
    let direct = BoundingRectangle::from_geographic(rectangle, &projection).unwrap();
    assert_eq!(direct, expected);

    // Antimeridian crossing rectangle
    let crossed = GeoRectangle::degrees(170.0, -10.0, -170.0, 10.0).unwrap();
    assert!(BoundingRectangle::from_geographic(crossed, &projection).unwrap().width < 0.0);

    // WebMercator projection
    let mercator = WebMercatorProjection::new(Ellipsoid::UNIT_SPHERE).unwrap();
    let r = BoundingRectangle::from_geographic(
        GeoRectangle::radians(0.1, 0.2, 0.3, 0.4).unwrap(),
        &mercator,
    )
    .unwrap();
    assert!((r.x - 0.1).abs() < 1e-15 && (r.width - 0.2).abs() < 1e-15 && r.height > 0.2);
}

#[test]
fn intersect_works() {
    let rectangle1 = BoundingRectangle::new(0.0, 0.0, 4.0, 4.0);
    let rectangle2 = BoundingRectangle::new(2.0, 2.0, 4.0, 4.0);
    let rectangle3 = BoundingRectangle::new(-6.0, 2.0, 4.0, 4.0);
    let rectangle4 = BoundingRectangle::new(8.0, 2.0, 4.0, 4.0);
    let rectangle5 = BoundingRectangle::new(2.0, -6.0, 4.0, 4.0);
    let rectangle6 = BoundingRectangle::new(2.0, 8.0, 4.0, 4.0);

    assert_eq!(rectangle1.intersect(&rectangle2), Intersect::INTERSECTING);
    assert_eq!(rectangle1.intersect(&rectangle3), Intersect::OUTSIDE);
    assert_eq!(rectangle1.intersect(&rectangle4), Intersect::OUTSIDE);
    assert_eq!(rectangle1.intersect(&rectangle5), Intersect::OUTSIDE);
    assert_eq!(rectangle1.intersect(&rectangle6), Intersect::OUTSIDE);

    assert_eq!(
        BoundingRectangle::intersect_rectangles(&rectangle1, &rectangle2),
        Intersect::INTERSECTING
    );
    assert_eq!(
        BoundingRectangle::intersect_rectangles(&rectangle1, &rectangle3),
        Intersect::OUTSIDE
    );
}

#[test]
fn union_works() {
    let rectangle1 = BoundingRectangle::new(2.0, 0.0, 1.0, 1.0);
    let rectangle2 = BoundingRectangle::new(-2.0, 0.0, 1.0, 2.0);
    let expected = BoundingRectangle::new(-2.0, 0.0, 5.0, 2.0);
    assert_eq!(rectangle1.union(rectangle2), expected);
}

#[test]
fn expand_works() {
    let rectangle = BoundingRectangle::new(2.0, 0.0, 1.0, 1.0);

    // Grow right
    let point = DVec2::new(4.0, 0.0);
    assert_eq!(
        rectangle.expand(point),
        BoundingRectangle::new(2.0, 0.0, 2.0, 1.0)
    );

    // Grow left
    let point = DVec2::new(0.0, 0.0);
    assert_eq!(
        rectangle.expand(point),
        BoundingRectangle::new(0.0, 0.0, 3.0, 1.0)
    );

    // Grow up
    let point = DVec2::new(2.0, 2.0);
    assert_eq!(
        rectangle.expand(point),
        BoundingRectangle::new(2.0, 0.0, 1.0, 2.0)
    );

    // Grow down
    let point = DVec2::new(2.0, -1.0);
    assert_eq!(
        rectangle.expand(point),
        BoundingRectangle::new(2.0, -1.0, 1.0, 2.0)
    );

    // No grow
    let point = DVec2::new(2.5, 0.6);
    assert_eq!(rectangle.expand(point), rectangle);
}

#[test]
fn pack_unpack_works() {
    let r = BoundingRectangle::new(1.0, 2.0, 3.0, 4.0);
    let mut packed = [9.0; 6];
    r.pack(&mut packed, 1).unwrap();
    assert_eq!(packed, [9.0, 1.0, 2.0, 3.0, 4.0, 9.0]);
    assert_eq!(BoundingRectangle::unpack(&packed, 1).unwrap(), r);
    assert!(r.pack(&mut packed, 3).is_err());
    assert!(BoundingRectangle::unpack(&packed, usize::MAX).is_err());
}
