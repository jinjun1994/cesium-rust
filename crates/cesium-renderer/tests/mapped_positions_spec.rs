//! MappedPositions spec tests matching CesiumJS `Scene/Model/MappedPositionsSpec.js`.

use cesium_math::{Cartographic, Ellipsoid, GeoRectangle};
use cesium_renderer::mapped_positions::MappedPositions;

#[test]
fn test_constructor_valid() {
    let positions = vec![
        Cartographic::from_degrees(0.0, 0.0, Some(0.0)),
        Cartographic::from_degrees(0.1, 0.0, Some(0.0)),
        Cartographic::from_degrees(0.0, 0.1, Some(0.0)),
        Cartographic::from_degrees(0.1, 0.1, Some(0.0)),
    ];
    let rect = GeoRectangle::from_degrees(0.0, 0.0, 0.1, 0.1);
    let ellipsoid = Ellipsoid::WGS84;

    let mapped = MappedPositions::new(
        Some(positions.clone()),
        Some(4),
        Some(rect),
        Some(ellipsoid),
    )
    .expect("Should construct MappedPositions");

    assert_eq!(mapped.num_positions(), 4);
    assert_eq!(mapped.cartographic_positions().len(), 4);
    assert_eq!(mapped.cartographic_bounding_rectangle(), rect);
    assert_eq!(mapped.ellipsoid(), ellipsoid);
}

#[test]
fn test_constructor_throws_without_cartographic_positions() {
    let rect = GeoRectangle::from_degrees(0.0, 0.0, 0.1, 0.1);
    let res = MappedPositions::new(None, Some(4), Some(rect), Some(Ellipsoid::WGS84));
    assert!(res.is_err());
    assert_eq!(
        res.err().unwrap().to_string(),
        "DeveloperError: cartographicPositions is required."
    );
}

#[test]
fn test_constructor_throws_with_invalid_num_positions() {
    let positions = vec![
        Cartographic::from_degrees(0.0, 0.0, Some(0.0)),
        Cartographic::from_degrees(0.1, 0.0, Some(0.0)),
        Cartographic::from_degrees(0.0, 0.1, Some(0.0)),
        Cartographic::from_degrees(0.1, 0.1, Some(0.0)),
    ];
    let rect = GeoRectangle::from_degrees(0.0, 0.0, 0.1, 0.1);

    // None
    let res_none = MappedPositions::new(
        Some(positions.clone()),
        None,
        Some(rect),
        Some(Ellipsoid::WGS84),
    );
    assert!(res_none.is_err());
    assert_eq!(
        res_none.err().unwrap().to_string(),
        "DeveloperError: numPositions is required."
    );

    // Negative
    let res_neg = MappedPositions::new(
        Some(positions),
        Some(-1),
        Some(rect),
        Some(Ellipsoid::WGS84),
    );
    assert!(res_neg.is_err());
    assert_eq!(
        res_neg.err().unwrap().to_string(),
        "DeveloperError: numPositions must be greater than or equal to 0."
    );
}

#[test]
fn test_constructor_throws_without_cartographic_bounding_rectangle() {
    let positions = vec![
        Cartographic::from_degrees(0.0, 0.0, Some(0.0)),
        Cartographic::from_degrees(0.1, 0.0, Some(0.0)),
        Cartographic::from_degrees(0.0, 0.1, Some(0.0)),
        Cartographic::from_degrees(0.1, 0.1, Some(0.0)),
    ];
    let res = MappedPositions::new(Some(positions), Some(4), None, Some(Ellipsoid::WGS84));
    assert!(res.is_err());
    assert_eq!(
        res.err().unwrap().to_string(),
        "DeveloperError: cartographicBoundingRectangle is required."
    );
}

#[test]
fn test_constructor_throws_without_ellipsoid() {
    let positions = vec![
        Cartographic::from_degrees(0.0, 0.0, Some(0.0)),
        Cartographic::from_degrees(0.1, 0.0, Some(0.0)),
        Cartographic::from_degrees(0.0, 0.1, Some(0.0)),
        Cartographic::from_degrees(0.1, 0.1, Some(0.0)),
    ];
    let rect = GeoRectangle::from_degrees(0.0, 0.0, 0.1, 0.1);
    let res = MappedPositions::new(Some(positions), Some(4), Some(rect), None);
    assert!(res.is_err());
    assert_eq!(
        res.err().unwrap().to_string(),
        "DeveloperError: ellipsoid is required."
    );
}
