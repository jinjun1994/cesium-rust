use cesium_math::{
    Cartographic, Ellipsoid, GeoRectangle, GeographicTilingScheme,
};
use std::f64::consts::{FRAC_PI_2, PI};

#[test]
fn test_geographic_tiling_scheme_default_and_ellipsoid() {
    let scheme = GeographicTilingScheme::default();
    assert_eq!(scheme.ellipsoid(), Ellipsoid::WGS84);
    assert_eq!(scheme.tile_counts(0), Some((2, 1)));

    let custom = GeographicTilingScheme::new(
        Ellipsoid::UNIT_SPHERE,
        GeoRectangle::MAXIMUM,
        1,
        1,
    )
    .unwrap();
    assert_eq!(custom.ellipsoid(), Ellipsoid::UNIT_SPHERE);
    assert_eq!(custom.tile_counts(0), Some((1, 1)));
}

#[test]
fn test_geographic_tiling_scheme_tile_xy_to_rectangle_single_root() {
    let scheme = GeographicTilingScheme::new(
        Ellipsoid::WGS84,
        GeoRectangle::MAXIMUM,
        1,
        1,
    )
    .unwrap();
    let rect = scheme.tile_rectangle(0, 0, 0).unwrap();
    let scheme_rect = scheme.rectangle();

    assert!((rect.west - scheme_rect.west).abs() < 1e-10);
    assert!((rect.south - scheme_rect.south).abs() < 1e-10);
    assert!((rect.east - scheme_rect.east).abs() < 1e-10);
    assert!((rect.north - scheme_rect.north).abs() < 1e-10);
}

#[test]
fn test_geographic_tiling_scheme_tiles_numbered_from_northwest() {
    let scheme = GeographicTilingScheme::new(
        Ellipsoid::WGS84,
        GeoRectangle::MAXIMUM,
        2,
        2,
    )
    .unwrap();
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
fn test_geographic_tiling_scheme_adjacent_tiles_overlapping() {
    let scheme = GeographicTilingScheme::new(
        Ellipsoid::WGS84,
        GeoRectangle::MAXIMUM,
        2,
        2,
    )
    .unwrap();
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
fn test_geographic_tiling_scheme_rectangle_to_native() {
    let scheme = GeographicTilingScheme::default();
    let rect_rad = GeoRectangle::radians(0.1, 0.2, 0.3, 0.4).unwrap();
    let native = scheme.rectangle_to_native(rect_rad);

    assert!((native.west - 0.1_f64.to_degrees()).abs() < 1e-12);
    assert!((native.south - 0.2_f64.to_degrees()).abs() < 1e-12);
    assert!((native.east - 0.3_f64.to_degrees()).abs() < 1e-12);
    assert!((native.north - 0.4_f64.to_degrees()).abs() < 1e-12);
}

#[test]
fn test_geographic_tiling_scheme_position_to_tile_xy() {
    let bounds = GeoRectangle::radians(0.1, 0.2, 0.3, 0.4).unwrap();
    let scheme = GeographicTilingScheme::new(Ellipsoid::WGS84, bounds, 1, 1).unwrap();

    let too_far_west = Cartographic::radians(0.05, 0.3, 0.0).unwrap();
    assert!(scheme.position_to_tile(too_far_west, 0).is_none());
    let too_far_south = Cartographic::radians(0.2, 0.1, 0.0).unwrap();
    assert!(scheme.position_to_tile(too_far_south, 0).is_none());
    let too_far_east = Cartographic::radians(0.4, 0.3, 0.0).unwrap();
    assert!(scheme.position_to_tile(too_far_east, 0).is_none());
    let too_far_north = Cartographic::radians(0.2, 0.5, 0.0).unwrap();
    assert!(scheme.position_to_tile(too_far_north, 0).is_none());

    // Default scheme corners at level 0
    let def_scheme = GeographicTilingScheme::default();
    let sw = Cartographic::radians(-PI, -FRAC_PI_2, 0.0).unwrap();
    let nw = Cartographic::radians(-PI, FRAC_PI_2, 0.0).unwrap();
    let ne = Cartographic::radians(PI, FRAC_PI_2, 0.0).unwrap();
    let se = Cartographic::radians(PI, -FRAC_PI_2, 0.0).unwrap();

    assert_eq!(def_scheme.position_to_tile(sw, 0), Some((0, 0)));
    assert_eq!(def_scheme.position_to_tile(nw, 0), Some((0, 0)));
    assert_eq!(def_scheme.position_to_tile(ne, 0), Some((1, 0)));
    assert_eq!(def_scheme.position_to_tile(se, 0), Some((1, 0)));

    // Level 1
    assert_eq!(def_scheme.position_to_tile(sw, 1), Some((0, 1)));
    assert_eq!(def_scheme.position_to_tile(nw, 1), Some((0, 0)));
    assert_eq!(def_scheme.position_to_tile(ne, 1), Some((3, 0)));
    assert_eq!(def_scheme.position_to_tile(se, 1), Some((3, 1)));

    // Center
    let center = Cartographic::radians(0.0, 0.0, 0.0).unwrap();
    assert_eq!(def_scheme.position_to_tile(center, 1), Some((2, 1)));
}
