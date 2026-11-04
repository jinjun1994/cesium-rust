//! Geographic rectangle reference cases from CesiumJS RectangleSpec (Apache-2.0).
use cesium_math::{Cartographic, Ellipsoid, GeoRectangle, Sphere};
use glam::DVec3;
use std::f64::consts::{FRAC_PI_2, PI, TAU};

fn r(w: f64, s: f64, e: f64, n: f64) -> GeoRectangle {
    GeoRectangle::degrees(w, s, e, n).unwrap()
}
fn same(a: GeoRectangle, b: GeoRectangle) {
    for (x, y) in [
        (a.west, b.west),
        (a.south, b.south),
        (a.east, b.east),
        (a.north, b.north),
    ] {
        assert!((x - y).abs() < 1e-14, "{a:?} vs {b:?}");
    }
}

#[test]
fn dimensions_centers_and_corners() {
    assert_eq!(GeoRectangle::default(), r(0.0, 0.0, 0.0, 0.0));
    let a = GeoRectangle::radians(-0.9, 0.5, 1.4, 1.0).unwrap();
    assert_eq!(a.width(), 1.4 - (-0.9));
    assert_eq!(a.height(), 0.5);
    assert_eq!(
        (a.southwest().longitude, a.southwest().latitude),
        (-0.9, 0.5)
    );
    assert_eq!(
        (a.northeast().longitude, a.northeast().latitude),
        (1.4, 1.0)
    );
    for (west, east, expected) in [
        (170.0, -170.0, 180.0_f64),
        (160.0, -170.0, 175.0),
        (170.0, -160.0, -175.0),
        (160.0, 140.0, -30.0),
    ] {
        let rect = r(west, 0.0, east, 0.0);
        assert!((rect.center().longitude - expected.to_radians()).abs() < 1e-14);
    }
    assert_eq!(GeoRectangle::MAXIMUM.width(), TAU);
    assert!(GeoRectangle::radians(PI + 0.1, 0.0, 0.0, 0.0).is_err());
    assert!(GeoRectangle::radians(0.0, -FRAC_PI_2 - 0.1, 0.0, 0.0).is_err());
}

#[test]
fn from_points_and_cartesians_crossing_antimeridian() {
    let points = [(-179.0, -4.0), (-178.0, 3.0), (178.0, 3.0), (179.0, 4.0)]
        .map(|(lon, lat)| Cartographic::degrees(lon, lat, 0.0).unwrap());
    let expected = r(178.0, -4.0, -178.0, 4.0);
    same(GeoRectangle::from_cartographics(&points).unwrap(), expected);
    let positions: Vec<_> = points
        .iter()
        .map(|&p| Ellipsoid::WGS84.cartographic_to_cartesian(p).unwrap())
        .collect();
    same(
        GeoRectangle::from_cartesians(&positions, Ellipsoid::WGS84).unwrap(),
        expected,
    );
    assert!(GeoRectangle::from_cartographics(&[]).is_none());
}

#[test]
fn upstream_intersection_and_degenerate_edges() {
    let crossing = r(170.0, -10.0, -170.0, 10.0);
    for (other, expected) in [
        (r(-175.0, 5.0, -160.0, 15.0), r(-175.0, 5.0, -170.0, 10.0)),
        (r(160.0, 5.0, 175.0, 15.0), r(170.0, 5.0, 175.0, 10.0)),
        (r(175.0, 5.0, -175.0, 15.0), r(175.0, 5.0, -175.0, 10.0)),
        (
            r(-180.0, -45.0, -90.0, 45.0),
            r(-180.0, -10.0, -170.0, 10.0),
        ),
    ] {
        same(crossing.intersection(other).unwrap(), expected);
        same(other.intersection(crossing).unwrap(), expected);
    }
    for other in [
        r(-170.0, -10.0, -160.0, 10.0),
        r(160.0, -10.0, 170.0, 10.0),
        r(175.0, 10.0, -175.0, 15.0),
    ] {
        assert!(crossing.intersection(other).is_none());
        assert!(other.intersection(crossing).is_none());
    }
}

#[test]
fn union_contains_expand_and_subsection() {
    let a = GeoRectangle::radians(0.5, 0.1, -0.5, 0.9).unwrap();
    let b = GeoRectangle::radians(-0.85, 0.0, -0.4, 0.8).unwrap();
    same(
        a.union(b),
        GeoRectangle::radians(0.5, 0.0, -0.4, 0.9).unwrap(),
    );
    let crossing = r(170.0, -10.0, -170.0, 10.0);
    assert!(crossing.contains(Cartographic::degrees(179.0, 0.0, 0.0).unwrap()));
    assert!(crossing.contains(Cartographic::degrees(-179.0, 0.0, 0.0).unwrap()));
    assert!(!crossing.contains(Cartographic::degrees(0.0, 0.0, 0.0).unwrap()));
    same(crossing.subsection(0.0, 0.0, 1.0, 1.0).unwrap(), crossing);
    same(
        crossing.subsection(0.25, 0.25, 0.75, 0.75).unwrap(),
        r(175.0, -5.0, -175.0, 5.0),
    );
    assert!(crossing.subsection(0.9, 0.0, 0.1, 1.0).is_err());
    let expanded = r(-2.0, -3.0, 2.0, 3.0).expand(Cartographic::degrees(4.0, 1.0, 0.0).unwrap());
    same(expanded, r(-2.0, -3.0, 4.0, 3.0));
}

#[test]
fn subsample_corners_equator_and_surface_height() {
    let ellipsoid = Ellipsoid::WGS84;
    let rect = GeoRectangle::radians(0.1, -0.1, 0.2, 0.0).unwrap();
    let samples = rect.subsample(ellipsoid, 100_000.0).unwrap();
    assert_eq!(samples.len(), 6);
    for (sample, mut c) in samples[..4].iter().zip([
        rect.northwest(),
        rect.northeast(),
        rect.southeast(),
        rect.southwest(),
    ]) {
        c.height = 100_000.0;
        assert_eq!(*sample, ellipsoid.cartographic_to_cartesian(c).unwrap());
    }
    for (sample, longitude) in samples[4..].iter().zip([rect.west, rect.east]) {
        let c = ellipsoid.cartesian_to_cartographic(*sample).unwrap();
        assert_eq!(c.latitude, 0.0);
        assert!((c.longitude - longitude).abs() < 1e-14);
        assert!((c.height - 100_000.0).abs() < 1e-8);
    }
    assert!(rect.subsample(ellipsoid, f64::NAN).is_none());
    assert_eq!(
        ellipsoid.surface_area_rectangle(rect).unwrap(),
        ellipsoid
            .surface_area(rect.west, rect.south, rect.east, rect.north)
            .unwrap()
    );
}

#[test]
fn upstream_bounding_sphere_zero_equator_and_pole() {
    let ellipsoid = Ellipsoid::WGS84;
    let zero = Sphere {
        center: DVec3::ZERO,
        radius: 0.0,
    };
    assert_eq!(
        GeoRectangle::from_bounding_sphere(zero, ellipsoid),
        Some(GeoRectangle::MAXIMUM)
    );
    let sphere = Sphere {
        center: DVec3::new(10_000_000.0, 0.0, 0.0),
        radius: 1000.0,
    };
    let result = GeoRectangle::from_bounding_sphere(sphere, ellipsoid).unwrap();
    let expected = GeoRectangle::radians(
        -0.00009999999966666667,
        -0.00010042880729608389,
        0.00009999999966666667,
        0.00010042880729608389,
    )
    .unwrap();
    same(result, expected);
    let pole = Sphere {
        center: DVec3::new(0.0, 0.0, ellipsoid.radii().z),
        radius: 1000.0,
    };
    let result = GeoRectangle::from_bounding_sphere(pole, ellipsoid).unwrap();
    let expected = GeoRectangle::radians(-FRAC_PI_2, 1.5706400668742968, PI, FRAC_PI_2).unwrap();
    same(result, expected);
    assert!(GeoRectangle::from_bounding_sphere(
        Sphere {
            radius: -1.0,
            ..sphere
        },
        ellipsoid
    )
    .is_none());
}
