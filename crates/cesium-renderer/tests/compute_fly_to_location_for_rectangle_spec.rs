use cesium_math::{Cartesian3, Ellipsoid, GeoRectangle, SceneMode};
use cesium_renderer::{
    compute_fly_to_location_for_rectangle, sample_cartographics_for_rectangle,
};

#[test]
fn test_sample_cartographics_for_rectangle() {
    let rectangle = GeoRectangle::from_degrees(10.0, 20.0, 30.0, 40.0);
    let cartos = sample_cartographics_for_rectangle(&rectangle);
    assert_eq!(cartos.len(), 5);

    // center
    let center = rectangle.center();
    assert_eq!(cartos[0].longitude, center.longitude);
    assert_eq!(cartos[0].latitude, center.latitude);

    // southeast (east, south)
    assert_eq!(cartos[1].longitude, rectangle.east);
    assert_eq!(cartos[1].latitude, rectangle.south);

    // southwest (west, south)
    assert_eq!(cartos[2].longitude, rectangle.west);
    assert_eq!(cartos[2].latitude, rectangle.south);

    // northeast (east, north)
    assert_eq!(cartos[3].longitude, rectangle.east);
    assert_eq!(cartos[3].latitude, rectangle.north);

    // northwest (west, north)
    assert_eq!(cartos[4].longitude, rectangle.west);
    assert_eq!(cartos[4].latitude, rectangle.north);
}

#[test]
fn test_samples_terrain_and_returns_expected_result_in_3d() {
    let rectangle = GeoRectangle::new(0.2, 0.4, 0.6, 0.8);
    let ellipsoid = Ellipsoid::WGS84;

    // Synthetic camera coordinates in 3D
    let camera_coords = Cartesian3::new(6000000.0, 2000000.0, 3000000.0);

    let max_height = 1234.0;
    let sampled_heights = [
        Some(145.0),
        Some(1211.0),
        Some(-123.0),
        Some(max_height),
        None,
    ];

    let result = compute_fly_to_location_for_rectangle(
        rectangle,
        SceneMode::Scene3D,
        camera_coords,
        ellipsoid,
        Some(&sampled_heights),
    );

    let baseline = compute_fly_to_location_for_rectangle(
        rectangle,
        SceneMode::Scene3D,
        camera_coords,
        ellipsoid,
        None,
    );

    assert_eq!(result.longitude, baseline.longitude);
    assert_eq!(result.latitude, baseline.latitude);
    assert!((result.height - (baseline.height + max_height)).abs() < 1e-7);
}

#[test]
fn test_samples_terrain_and_returns_expected_result_in_cv() {
    let rectangle = GeoRectangle::new(0.2, 0.4, 0.6, 0.8);
    let ellipsoid = Ellipsoid::WGS84;
    let camera_coords = Cartesian3::new(1000000.0, 2000000.0, 500000.0);

    let max_height = 1234.0;
    let sampled_heights = [
        Some(145.0),
        Some(1211.0),
        Some(-123.0),
        Some(max_height),
        None,
    ];

    let result = compute_fly_to_location_for_rectangle(
        rectangle,
        SceneMode::ColumbusView,
        camera_coords,
        ellipsoid,
        Some(&sampled_heights),
    );

    let baseline = compute_fly_to_location_for_rectangle(
        rectangle,
        SceneMode::ColumbusView,
        camera_coords,
        ellipsoid,
        None,
    );

    assert_eq!(result.longitude, baseline.longitude);
    assert_eq!(result.latitude, baseline.latitude);
    assert!((result.height - (baseline.height + max_height)).abs() < 1e-7);
}

#[test]
fn test_returns_height_above_ellipsoid_when_in_2d() {
    let rectangle = GeoRectangle::new(0.2, 0.4, 0.6, 0.8);
    let ellipsoid = Ellipsoid::WGS84;
    let camera_coords = Cartesian3::new(1000000.0, 2000000.0, 500000.0);

    let max_height = 1234.0;
    let sampled_heights = [Some(max_height)];

    let result = compute_fly_to_location_for_rectangle(
        rectangle,
        SceneMode::Scene2D,
        camera_coords,
        ellipsoid,
        Some(&sampled_heights),
    );

    let baseline = compute_fly_to_location_for_rectangle(
        rectangle,
        SceneMode::Scene2D,
        camera_coords,
        ellipsoid,
        None,
    );

    // In 2D, terrain height adjustment is skipped
    assert_eq!(result.height, baseline.height);
}

#[test]
fn test_returns_height_above_ellipsoid_when_terrain_not_available() {
    let rectangle = GeoRectangle::new(0.2, 0.4, 0.6, 0.8);
    let ellipsoid = Ellipsoid::WGS84;
    let camera_coords = Cartesian3::new(6000000.0, 2000000.0, 3000000.0);

    let result = compute_fly_to_location_for_rectangle(
        rectangle,
        SceneMode::Scene3D,
        camera_coords,
        ellipsoid,
        None,
    );

    assert!(result.height.is_finite());
}
