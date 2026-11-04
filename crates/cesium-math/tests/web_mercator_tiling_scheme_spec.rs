use cesium_math::{
    Cartographic, Ellipsoid, GeoRectangle, WebMercatorProjection, WebMercatorTilingScheme,
};
use glam::DVec2;
use std::f64::consts::PI;

#[test]
fn test_web_mercator_tiling_scheme_default_and_ellipsoid() {
    let scheme = WebMercatorTilingScheme::default();
    assert_eq!(scheme.ellipsoid(), Ellipsoid::WGS84);
    assert_eq!(scheme.tile_counts(0), Some((1, 1)));

    let custom = WebMercatorTilingScheme::new(
        Ellipsoid::UNIT_SPHERE,
        None,
        1,
        1,
    )
    .unwrap();
    assert_eq!(custom.ellipsoid(), Ellipsoid::UNIT_SPHERE);
}

#[test]
fn test_web_mercator_tiling_scheme_tile_xy_to_rectangle_single_root() {
    let scheme = WebMercatorTilingScheme::default();
    let rect = scheme.tile_rectangle(0, 0, 0).unwrap();
    let scheme_rect = scheme.rectangle();

    assert!((rect.west - scheme_rect.west).abs() < 1e-10);
    assert!((rect.south - scheme_rect.south).abs() < 1e-10);
    assert!((rect.east - scheme_rect.east).abs() < 1e-10);
    assert!((rect.north - scheme_rect.north).abs() < 1e-10);
}

#[test]
fn test_web_mercator_tiling_scheme_tiles_numbered_from_northwest() {
    let scheme = WebMercatorTilingScheme::default();
    let northwest = scheme.tile_rectangle(0, 0, 1).unwrap();
    let northeast = scheme.tile_rectangle(1, 0, 1).unwrap();
    let southeast = scheme.tile_rectangle(1, 1, 1).unwrap();
    let southwest = scheme.tile_rectangle(0, 1, 1).unwrap();

    assert_eq!(northeast.north, northwest.north);
    assert_eq!(northeast.south, northwest.south);
    assert_eq!(southeast.north, southwest.north);
    assert_eq!(southeast.south, southwest.south);

    assert_eq!(northwest.west, southwest.west);
    assert_eq!(northwest.east, southwest.east);
    assert_eq!(northeast.west, southeast.west);
    assert_eq!(northeast.east, southeast.east);

    assert!(northeast.north > southeast.north);
    assert!(northeast.south > southeast.south);
    assert!(northwest.north > southwest.north);
    assert!(northwest.south > southwest.south);

    assert!(northeast.east > northwest.east);
    assert!(northeast.west > northwest.west);
    assert!(southeast.east > southwest.east);
    assert!(southeast.west > southwest.west);
}

#[test]
fn test_web_mercator_tiling_scheme_adjacent_tiles_overlapping() {
    let scheme = WebMercatorTilingScheme::default();
    let northwest = scheme.tile_rectangle(0, 0, 1).unwrap();
    let northeast = scheme.tile_rectangle(1, 0, 1).unwrap();
    let southeast = scheme.tile_rectangle(1, 1, 1).unwrap();
    let southwest = scheme.tile_rectangle(0, 1, 1).unwrap();

    assert!((northeast.south - southeast.north).abs() < 1e-14);
    assert!((northwest.south - southwest.north).abs() < 1e-14);
    assert!((northeast.west - northwest.east).abs() < 1e-14);
    assert!((southeast.west - southwest.east).abs() < 1e-14);
}

#[test]
fn test_web_mercator_tiling_scheme_level0_and_level1_corners() {
    let scheme = WebMercatorTilingScheme::default();
    let max_lat = WebMercatorProjection::MAXIMUM_LATITUDE;

    let sw = Cartographic::radians(-PI, -max_lat, 0.0).unwrap();
    let nw = Cartographic::radians(-PI, max_lat, 0.0).unwrap();
    let ne = Cartographic::radians(PI, max_lat, 0.0).unwrap();
    let se = Cartographic::radians(PI, -max_lat, 0.0).unwrap();

    // Level 0
    assert_eq!(scheme.position_to_tile(sw, 0), Some((0, 0)));
    assert_eq!(scheme.position_to_tile(nw, 0), Some((0, 0)));
    assert_eq!(scheme.position_to_tile(ne, 0), Some((0, 0)));
    assert_eq!(scheme.position_to_tile(se, 0), Some((0, 0)));

    // Level 1
    assert_eq!(scheme.position_to_tile(sw, 1), Some((0, 1)));
    assert_eq!(scheme.position_to_tile(nw, 1), Some((0, 0)));
    assert_eq!(scheme.position_to_tile(ne, 1), Some((1, 0)));
    assert_eq!(scheme.position_to_tile(se, 1), Some((1, 1)));

    // Center at Level 1
    let center = Cartographic::radians(0.0, 0.0, 0.0).unwrap();
    assert_eq!(scheme.position_to_tile(center, 1), Some((1, 1)));
}

#[test]
fn test_web_mercator_tiling_scheme_rectangle_to_native() {
    let scheme = WebMercatorTilingScheme::default();
    let rect = GeoRectangle::radians(0.1, 0.2, 0.3, 0.4).unwrap();
    let native = scheme.rectangle_to_native(rect).expect("native rectangle");

    let proj = scheme.projection();
    let sw = proj.project(rect.southwest()).unwrap();
    let ne = proj.project(rect.northeast()).unwrap();

    assert!((native.west - sw.x).abs() < 1e-10);
    assert!((native.south - sw.y).abs() < 1e-10);
    assert!((native.east - ne.x).abs() < 1e-10);
    assert!((native.north - ne.y).abs() < 1e-10);
}

#[test]
fn test_web_mercator_tiling_scheme_custom_bounds() {
    let scheme = WebMercatorTilingScheme::new(
        Ellipsoid::UNIT_SPHERE,
        Some((DVec2::new(-1.0, -1.0), DVec2::new(1.0, 1.0))),
        2,
        2,
    )
    .unwrap();

    assert_eq!(scheme.tile_counts(0), Some((2, 2)));
    let nw = scheme.tile_native_rectangle(0, 0, 0).unwrap();
    assert_eq!(nw.west, -1.0);
    assert_eq!(nw.north, 1.0);
    assert_eq!(nw.east, 0.0);
    assert_eq!(nw.south, 0.0);
}
