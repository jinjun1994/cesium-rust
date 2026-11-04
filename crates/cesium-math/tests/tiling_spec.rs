//! Cases from CesiumJS WebMercatorProjection and tiling scheme Specs (Apache-2.0).
use cesium_math::{
    Cartographic, Ellipsoid, GeoRectangle, GeographicTilingScheme, MapProjection, TilingScheme,
    WebMercatorProjection, WebMercatorTilingScheme, WGS84_A,
};
use glam::{DVec2, DVec3};
use std::f64::consts::{FRAC_PI_2, FRAC_PI_4, PI};

fn point(lon: f64, lat: f64) -> Cartographic {
    Cartographic::radians(lon, lat, 0.0).unwrap()
}

// Exercise both implementations through the shared upstream interface.
fn check_scheme<S: TilingScheme>(scheme: &S, expected_root: (u32, u32)) {
    assert_eq!(scheme.tile_counts(0), Some(expected_root));
    assert_eq!(scheme.ellipsoid(), scheme.projection().ellipsoid());
    let projected = scheme.projection().project(point(0.1, 0.2)).unwrap();
    let recovered = scheme.projection().unproject(projected).unwrap();
    assert!((recovered.longitude - 0.1).abs() < 1e-14);
    assert!((recovered.latitude - 0.2).abs() < 1e-14);
    assert!(scheme.rectangle_to_native(scheme.rectangle()).is_some());
    assert!(scheme.tile_native_rectangle(0, 0, 0).is_some());
    assert!(scheme.tile_rectangle(0, 0, 0).is_some());
    assert!(scheme.position_to_tile(point(0.0, 0.0), 0).is_some());
}

#[test]
fn tiling_schemes_conform_to_shared_interface() {
    check_scheme(&GeographicTilingScheme::default(), (2, 1));
    check_scheme(&WebMercatorTilingScheme::default(), (1, 1));
}

#[test]
fn upstream_adjacent_tiles_and_outside_bounds() {
    let bounds = GeoRectangle::radians(0.1, 0.2, 0.3, 0.4).unwrap();
    let geo = GeographicTilingScheme::new(Ellipsoid::UNIT_SPHERE, bounds, 2, 2).unwrap();
    let projection = WebMercatorProjection::new(Ellipsoid::UNIT_SPHERE).unwrap();
    let sw = projection.project(bounds.southwest()).unwrap().truncate();
    let ne = projection.project(bounds.northeast()).unwrap().truncate();
    let mercator =
        WebMercatorTilingScheme::new(Ellipsoid::UNIT_SPHERE, Some((sw, ne)), 2, 2).unwrap();
    fn check<S: TilingScheme>(s: &S) {
        let nw = s.tile_rectangle(0, 0, 1).unwrap();
        let ne = s.tile_rectangle(1, 0, 1).unwrap();
        let sw = s.tile_rectangle(0, 1, 1).unwrap();
        assert!((nw.east - ne.west).abs() < 1e-15);
        assert!((nw.south - sw.north).abs() < 1e-15);
        for p in [
            point(0.05, 0.3),
            point(0.2, 0.1),
            point(0.4, 0.3),
            point(0.2, 0.5),
        ] {
            assert_eq!(s.position_to_tile(p, 0), None);
        }
    }
    check(&geo);
    check(&mercator);
}

#[test]
fn web_mercator_upstream_projection_and_pole_clamp() {
    let p = WebMercatorProjection::default();
    assert_eq!(p.ellipsoid(), Ellipsoid::WGS84);
    let c = Cartographic::radians(PI, FRAC_PI_4, 12.0).unwrap();
    let q = p.project(c).unwrap();
    let expected_y = WGS84_A * (PI / 4.0 + c.latitude / 2.0).tan().ln();
    assert!((q.x - PI * WGS84_A).abs() < 1e-8 && (q.y - expected_y).abs() < 1e-8 && q.z == 12.0);
    let restored = p.unproject(q).unwrap();
    assert!(
        (restored.longitude - c.longitude).abs() < 1e-14
            && (restored.latitude - c.latitude).abs() < 1e-14
            && restored.height == 12.0
    );
    for sign in [-1.0, 1.0] {
        let pole = p.project(point(0.0, sign * FRAC_PI_2)).unwrap();
        let limit = p
            .project(point(0.0, sign * WebMercatorProjection::MAXIMUM_LATITUDE))
            .unwrap();
        assert_eq!(pole.y, limit.y);
        assert!((pole.y - sign * 20_037_508.342787).abs() < 1e-3);
    }
    assert!(WebMercatorProjection::new(Ellipsoid::default()).is_err());
    assert!(p.unproject(DVec3::splat(f64::NAN)).is_none());
}

#[test]
fn geographic_tile_boundaries_and_crossing_rectangle() {
    let scheme = GeographicTilingScheme::default();
    assert_eq!(scheme.tile_counts(0), Some((2, 1)));
    assert_eq!(scheme.tile_counts(2), Some((8, 4)));
    assert_eq!(scheme.tile_counts(32), None);
    assert_eq!(
        scheme.position_to_tile(point(-PI, FRAC_PI_2), 1),
        Some((0, 0))
    );
    assert_eq!(
        scheme.position_to_tile(point(PI, -FRAC_PI_2), 1),
        Some((3, 1))
    );
    assert_eq!(scheme.position_to_tile(point(0.0, 0.0), 1), Some((2, 1)));
    let left = scheme.tile_rectangle(0, 0, 1).unwrap();
    let right = scheme.tile_rectangle(1, 0, 1).unwrap();
    assert_eq!(left.east, right.west);
    assert!(scheme.tile_rectangle(4, 0, 1).is_none());
    let native = scheme.tile_native_rectangle(0, 0, 0).unwrap();
    assert_eq!(
        (native.west, native.south, native.east, native.north),
        (-180.0, -90.0, 0.0, 90.0)
    );
    let bounds = GeoRectangle::degrees(170.0, -10.0, -170.0, 10.0).unwrap();
    let custom = GeographicTilingScheme::new(Ellipsoid::WGS84, bounds, 2, 1).unwrap();
    assert_eq!(
        custom.position_to_tile(Cartographic::degrees(175.0, 0.0, 0.0).unwrap(), 0),
        Some((0, 0))
    );
    assert_eq!(
        custom.position_to_tile(Cartographic::degrees(-175.0, 0.0, 0.0).unwrap(), 0),
        Some((1, 0))
    );
    assert!(custom.position_to_tile(point(0.0, 0.0), 0).is_none());
}

#[test]
fn mercator_tiles_northwest_and_boundary_clamping() {
    let s = WebMercatorTilingScheme::default();
    assert_eq!(s.tile_counts(0), Some((1, 1)));
    assert_eq!(s.tile_counts(2), Some((4, 4)));
    let max = WebMercatorProjection::MAXIMUM_LATITUDE;
    assert_eq!(s.position_to_tile(point(-PI, max), 1), Some((0, 0)));
    assert_eq!(s.position_to_tile(point(PI, max), 1), Some((1, 0)));
    assert_eq!(s.position_to_tile(point(-PI, -max), 1), Some((0, 1)));
    assert_eq!(s.position_to_tile(point(PI, -max), 1), Some((1, 1)));
    assert_eq!(s.position_to_tile(point(0.0, 0.0), 2), Some((2, 2)));
    assert!(s.position_to_tile(point(0.0, FRAC_PI_2), 1).is_none());
    let nw = s.tile_native_rectangle(0, 0, 1).unwrap();
    let ne = s.tile_native_rectangle(1, 0, 1).unwrap();
    assert_eq!(nw.east, ne.west);
    let root = s.tile_rectangle(0, 0, 0).unwrap();
    assert!(
        (root.west + PI).abs() < 1e-14
            && (root.east - PI).abs() < 1e-14
            && (root.north - max).abs() < 1e-14
    );
    assert!(s.tile_rectangle(2, 0, 1).is_none());
    assert!(
        WebMercatorTilingScheme::new(Ellipsoid::WGS84, Some((DVec2::ONE, DVec2::ZERO)), 1, 1)
            .is_err()
    );
}

#[test]
fn native_conversion_and_custom_mercator_bounds() {
    let rect = GeoRectangle::radians(0.1, 0.2, 0.3, 0.4).unwrap();
    let geographic = GeographicTilingScheme::default().rectangle_to_native(rect);
    assert!((geographic.west - 0.1_f64.to_degrees()).abs() < 1e-13);
    let scheme = WebMercatorTilingScheme::new(
        Ellipsoid::UNIT_SPHERE,
        Some((DVec2::new(-1.0, -1.0), DVec2::new(1.0, 1.0))),
        2,
        2,
    )
    .unwrap();
    let native = scheme.rectangle_to_native(rect).unwrap();
    let projection = WebMercatorProjection::new(Ellipsoid::UNIT_SPHERE).unwrap();
    let sw = projection.project(rect.southwest()).unwrap();
    let ne = projection.project(rect.northeast()).unwrap();
    assert_eq!(
        (native.west, native.south, native.east, native.north),
        (sw.x, sw.y, ne.x, ne.y)
    );
    assert_eq!(scheme.tile_counts(0), Some((2, 2)));
    let upper_left = scheme.tile_native_rectangle(0, 0, 0).unwrap();
    assert_eq!(
        (
            upper_left.west,
            upper_left.south,
            upper_left.east,
            upper_left.north
        ),
        (-1.0, 0.0, 0.0, 1.0)
    );
    assert_eq!(scheme.position_to_tile(point(0.0, 0.0), 0), Some((1, 1)));
    assert!(scheme.position_to_tile(point(2.0, 0.0), 0).is_none());
}
