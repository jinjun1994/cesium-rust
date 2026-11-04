use cesium_math::{ApproximateTerrainHeights, GeoRectangle};
use glam::DVec3;

#[test]
fn test_approximate_terrain_heights_initializes() {
    assert!(ApproximateTerrainHeights::is_initialized());
}

#[test]
fn test_approximate_terrain_heights_get_min_max_heights() {
    let rect = GeoRectangle::from_degrees(-121.0, 10.0, -120.0, 11.0);
    let result = ApproximateTerrainHeights::get_minimum_maximum_heights(&rect, None);

    assert!(
        (result.minimum_terrain_height - (-5269.86)).abs() <= 1e-2,
        "min terrain height expected -5269.86, got {}",
        result.minimum_terrain_height
    );
    assert!(
        (result.maximum_terrain_height - (-28.53)).abs() <= 1e-2,
        "max terrain height expected -28.53, got {}",
        result.maximum_terrain_height
    );
}

#[test]
fn test_approximate_terrain_heights_get_bounding_sphere() {
    let rect = GeoRectangle::from_degrees(-121.0, 10.0, -120.0, 11.0);
    let sphere = ApproximateTerrainHeights::get_bounding_sphere(&rect, None);

    let expected_center = DVec3::new(
        -3183013.849117281,
        -5403772.559109628,
        1154581.5821590829,
    );
    let expected_radius = 77884.16321007285;

    let center_diff = (sphere.center - expected_center).length();
    assert!(
        center_diff <= 1.0,
        "sphere center distance diff {} is too large (got {:?}, expected {:?})",
        center_diff,
        sphere.center,
        expected_center
    );
    let radius_diff = (sphere.radius - expected_radius).abs();
    assert!(
        radius_diff <= 1.0,
        "sphere radius diff {} is too large (got {}, expected {})",
        radius_diff,
        sphere.radius,
        expected_radius
    );
}
