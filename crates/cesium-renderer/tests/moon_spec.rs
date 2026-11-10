use cesium_math::{
    CelestialFrameTransforms, Ellipsoid, JulianDate, Simon1994PlanetaryPositions,
};
use cesium_renderer::moon::{Moon, MoonOptions};

#[test]
fn test_default_constructs_the_moon() {
    let moon = Moon::default();
    assert!(moon.show);
    assert!(moon.texture_url.contains("Assets/Textures/moonSmall.jpg"));
    assert_eq!(moon.ellipsoid(), Ellipsoid::MOON);
    assert!(moon.only_sun_lighting);
    assert_eq!(
        moon.ellipsoid_primitive().radii,
        Some(Ellipsoid::MOON.radii().into())
    );
}

#[test]
fn test_custom_options() {
    let moon = Moon::new(Some(MoonOptions {
        show: Some(false),
        texture_url: Some("custom/moon.png".to_string()),
        ellipsoid: Some(Ellipsoid::WGS84),
        only_sun_lighting: Some(false),
    }));

    assert!(!moon.show);
    assert_eq!(moon.texture_url, "custom/moon.png");
    assert_eq!(moon.ellipsoid(), Ellipsoid::WGS84);
    assert!(!moon.only_sun_lighting);
}

#[test]
fn test_computes_model_matrix() {
    let mut moon = Moon::default();
    let date = JulianDate::from_iso8601("2026-11-10T12:00:00Z").unwrap();

    let model_matrix = moon.compute_model_matrix(&date);

    // Verify translation matches lunar position computation
    let icrf_to_fixed = CelestialFrameTransforms::compute_icrf_to_central_body_fixed_matrix(&date);
    let moon_inertial =
        Simon1994PlanetaryPositions::compute_moon_position_in_earth_inertial_frame(Some(&date));
    let expected_translation = icrf_to_fixed.multiply_by_vector(moon_inertial);

    let actual_translation = model_matrix.get_translation();
    assert!((actual_translation.x - expected_translation.x).abs() < 1e-4);
    assert!((actual_translation.y - expected_translation.y).abs() < 1e-4);
    assert!((actual_translation.z - expected_translation.z).abs() < 1e-4);

    // Verify distance from Earth center is within lunar orbit range (350,000 km to 410,000 km)
    let dist = actual_translation.magnitude();
    assert!(dist > 350_000_000.0 && dist < 415_000_000.0);

    // Verify rotation matrix is valid rotation (determinant near 1.0)
    let rot = model_matrix.get_matrix3();
    let r_cols = rot.values;
    let det = r_cols[0] * (r_cols[4] * r_cols[8] - r_cols[5] * r_cols[7])
        - r_cols[1] * (r_cols[3] * r_cols[8] - r_cols[5] * r_cols[6])
        + r_cols[2] * (r_cols[3] * r_cols[7] - r_cols[4] * r_cols[6]);
    assert!((det.abs() - 1.0).abs() < 1e-5);
}

#[test]
fn test_does_not_render_when_show_is_false() {
    let mut moon = Moon::default();
    let date = JulianDate::from_iso8601("2026-11-10T12:00:00Z").unwrap();

    moon.compute_model_matrix(&date);
    assert!(moon.ellipsoid_primitive().show);

    moon.show = false;
    moon.compute_model_matrix(&date);
    assert!(!moon.ellipsoid_primitive().show);
}

#[test]
fn test_is_destroyed() {
    let mut moon = Moon::default();
    assert!(!moon.is_destroyed());
    moon.destroy();
    assert!(moon.is_destroyed());
}
