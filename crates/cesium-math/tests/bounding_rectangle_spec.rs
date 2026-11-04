//! Cases adapted from CesiumJS BoundingRectangleSpec (Apache-2.0).
use cesium_math::{
    BoundingRectangle as R, Ellipsoid, GeoRectangle, GeographicProjection, WebMercatorProjection,
};
use glam::DVec2;

#[test]
fn construction_points_equality_and_pack() {
    assert_eq!(R::default(), R::new(0.0, 0.0, 0.0, 0.0));
    assert_eq!(R::from_points(&[]).unwrap(), R::default());
    let points = [
        DVec2::new(3.0, -1.0),
        DVec2::new(2.0, -2.0),
        DVec2::new(1.0, -3.0),
        DVec2::ZERO,
        DVec2::new(-1.0, 1.0),
        DVec2::new(-2.0, 2.0),
        DVec2::new(-3.0, 3.0),
    ];
    assert_eq!(
        R::from_points(&points).unwrap(),
        R::new(-3.0, -3.0, 6.0, 6.0)
    );
    assert!(R::from_points(&[DVec2::splat(f64::NAN)]).is_err());
    let r = R::new(1.0, 2.0, 3.0, 4.0);
    let mut packed = [9.0; 6];
    r.pack(&mut packed, 1).unwrap();
    assert_eq!(packed, [9.0, 1.0, 2.0, 3.0, 4.0, 9.0]);
    assert_eq!(R::unpack(&packed, 1).unwrap(), r);
    assert!(r.pack(&mut packed, 3).is_err());
    assert!(R::unpack(&packed, usize::MAX).is_err());
}

#[test]
fn projection_from_geographic_rectangle() {
    let p = GeographicProjection::new(Ellipsoid::UNIT_SPHERE).unwrap();
    let rectangle = GeoRectangle::MAXIMUM;
    assert_eq!(
        R::from_geographic(rectangle, &p).unwrap(),
        R::new(
            rectangle.west,
            rectangle.south,
            rectangle.east - rectangle.west,
            rectangle.north - rectangle.south
        )
    );
    let crossed = GeoRectangle::degrees(170.0, -10.0, -170.0, 10.0).unwrap();
    assert!(R::from_geographic(crossed, &p).unwrap().width < 0.0);
    let mercator = WebMercatorProjection::new(Ellipsoid::UNIT_SPHERE).unwrap();
    let r = R::from_geographic(
        GeoRectangle::radians(0.1, 0.2, 0.3, 0.4).unwrap(),
        &mercator,
    )
    .unwrap();
    assert!((r.x - 0.1).abs() < 1e-15 && (r.width - 0.2).abs() < 1e-15 && r.height > 0.2);
}

#[test]
fn upstream_union_expand_and_intersections() {
    let base = R::new(2.0, 0.0, 1.0, 1.0);
    assert_eq!(
        base.union(R::new(-2.0, 0.0, 1.0, 2.0)),
        R::new(-2.0, 0.0, 5.0, 2.0)
    );
    for (point, expected) in [
        (DVec2::new(4.0, 0.0), R::new(2.0, 0.0, 2.0, 1.0)),
        (DVec2::new(0.0, 0.0), R::new(0.0, 0.0, 3.0, 1.0)),
        (DVec2::new(2.0, 2.0), R::new(2.0, 0.0, 1.0, 2.0)),
        (DVec2::new(2.0, -1.0), R::new(2.0, -1.0, 1.0, 2.0)),
        (DVec2::new(2.5, 0.6), base),
    ] {
        assert_eq!(base.expand(point), expected);
    }
    let square = R::new(0.0, 0.0, 4.0, 4.0);
    assert!(square.intersects(R::new(2.0, 2.0, 4.0, 4.0)));
    assert!(square.intersects(R::new(4.0, 4.0, 1.0, 1.0)));
    for r in [
        R::new(-6.0, 2.0, 4.0, 4.0),
        R::new(8.0, 2.0, 4.0, 4.0),
        R::new(2.0, -6.0, 4.0, 4.0),
        R::new(2.0, 8.0, 4.0, 4.0),
    ] {
        assert!(!square.intersects(r));
    }
}
